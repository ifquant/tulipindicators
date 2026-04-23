//! Bollinger Bands (`bbands`) produce lower, middle, and upper bands around a rolling mean.
//!
//! The batch path tracks rolling sums and squared sums so standard deviation can be updated without
//! rescanning each period window.
use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "bbands",
    full_name: "Bollinger Bands",
    category: IndicatorCategory::Overlay,
    input_names: &["real"],
    option_names: &["period", "stddev"],
    output_names: &["bbands_lower", "bbands_middle", "bbands_upper"],
};

#[derive(Debug, Clone, Copy)]
pub struct Bbands;

impl Indicator for Bbands {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let (period, _) = parse_options(options)?;
        Ok(period - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let (period, stddev) = parse_options(options)?;
        let lookback = period - 1;
        let mut lower = Vec::with_capacity(input.len().saturating_sub(lookback));
        let mut middle = Vec::with_capacity(input.len().saturating_sub(lookback));
        let mut upper = Vec::with_capacity(input.len().saturating_sub(lookback));

        if input.len() <= lookback {
            return Ok(vec![lower, middle, upper]);
        }

        let scale = 1.0 / period as Real;
        let mut sum = 0.0;
        let mut sum2 = 0.0;

        for value in input.iter().take(period) {
            sum += *value;
            sum2 += value * value;
        }

        push_band_values(
            &mut lower,
            &mut middle,
            &mut upper,
            sum,
            sum2,
            scale,
            stddev,
        );

        for index in period..input.len() {
            sum += input[index];
            sum2 += input[index] * input[index];
            sum -= input[index - period];
            sum2 -= input[index - period] * input[index - period];

            push_band_values(
                &mut lower,
                &mut middle,
                &mut upper,
                sum,
                sum2,
                scale,
                stddev,
            );
        }

        Ok(vec![lower, middle, upper])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let (period, stddev) = parse_options(options)?;
        validate_output_slices(&METADATA, outputs, 3)?;
        let output_len = input.len().saturating_sub(period - 1);
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;
        ensure_output_len(&METADATA, outputs[1].len(), output_len, 1)?;
        ensure_output_len(&METADATA, outputs[2].len(), output_len, 2)?;

        if output_len == 0 {
            return Ok(0);
        }

        let scale = 1.0 / period as Real;
        let mut sum = 0.0;
        let mut sum2 = 0.0;

        for value in input.iter().take(period) {
            sum += *value;
            sum2 += value * value;
        }

        write_band_values(outputs, 0, sum, sum2, scale, stddev);

        for index in period..input.len() {
            sum += input[index];
            sum2 += input[index] * input[index];
            sum -= input[index - period];
            sum2 -= input[index - period] * input[index - period];
            write_band_values(outputs, index - period + 1, sum, sum2, scale, stddev);
        }

        Ok(output_len)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(BbandsStream::new(options)?)))
    }
}

struct BbandsStream {
    period: usize,
    stddev: Real,
    scale: Real,
    progress: usize,
    sum: Real,
    sum2: Real,
    window: Vec<Real>,
    index: usize,
}

impl BbandsStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let (period, stddev) = parse_options(options)?;
        Ok(Self {
            period,
            stddev,
            scale: 1.0 / period as Real,
            progress: 0,
            sum: 0.0,
            sum2: 0.0,
            window: Vec::with_capacity(period),
            index: 0,
        })
    }
}

impl IndicatorStream for BbandsStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let mut lower = Vec::new();
        let mut middle = Vec::new();
        let mut upper = Vec::new();

        for sample in input {
            if self.window.len() < self.period {
                self.window.push(*sample);
                self.sum += *sample;
                self.sum2 += sample * sample;
            } else {
                let replaced = self.window[self.index];
                self.sum -= replaced;
                self.sum2 -= replaced * replaced;
                self.window[self.index] = *sample;
                self.sum += *sample;
                self.sum2 += sample * sample;
                self.index = (self.index + 1) % self.period;
            }

            if self.window.len() == self.period {
                push_band_values(
                    &mut lower,
                    &mut middle,
                    &mut upper,
                    self.sum,
                    self.sum2,
                    self.scale,
                    self.stddev,
                );
            }

            self.progress += 1;
        }

        Ok(vec![lower, middle, upper])
    }
}

fn push_band_values(
    lower: &mut Vec<Real>,
    middle: &mut Vec<Real>,
    upper: &mut Vec<Real>,
    sum: Real,
    sum2: Real,
    scale: Real,
    stddev: Real,
) {
    let mean = sum * scale;
    let variance = (sum2 * scale - mean * mean).max(0.0);
    let deviation = variance.sqrt();

    middle.push(mean);
    lower.push(mean - stddev * deviation);
    upper.push(mean + stddev * deviation);
}

fn write_band_values(
    outputs: &mut [&mut [Real]],
    index: usize,
    sum: Real,
    sum2: Real,
    scale: Real,
    stddev: Real,
) {
    let mean = sum * scale;
    let variance = (sum2 * scale - mean * mean).max(0.0);
    let deviation = variance.sqrt();

    outputs[0][index] = mean - stddev * deviation;
    outputs[1][index] = mean;
    outputs[2][index] = mean + stddev * deviation;
}

fn parse_options(options: &[Real]) -> Result<(usize, Real), IndicatorError> {
    expect_option_count(METADATA.name, options, 2)?;
    let period = parse_usize_option(METADATA.name, options, 0, "period", 1)?;
    Ok((period, options[1]))
}
