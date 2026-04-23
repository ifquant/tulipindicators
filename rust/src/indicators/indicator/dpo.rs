//! Detrended Price Oscillator (`dpo`) subtracts a displaced moving average from price.
//!
//! The lookback includes the SMA period and displacement so each output has the centered average it
//! needs without revisiting earlier output slots.
use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};
use std::collections::VecDeque;

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "dpo",
    full_name: "Detrended Price Oscillator",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["dpo"],
};

#[derive(Debug, Clone, Copy)]
pub struct Dpo;

impl Indicator for Dpo {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        Ok(period - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let output_len = input.len().saturating_sub(period - 1);
        let mut output = vec![0.0; output_len];
        let produced = run_dpo_batch(input, period, &mut output);
        debug_assert_eq!(produced, output.len());
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let output_len = input.len().saturating_sub(period - 1);
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_dpo_batch(input, period, &mut outputs[0][..output_len]))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(DpoStream::new(options)?)))
    }
}

struct DpoStream {
    period: usize,
    back: usize,
    progress: usize,
    window: VecDeque<Real>,
    sum: Real,
}

impl DpoStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        Ok(Self {
            period,
            back: period / 2 + 1,
            progress: 0,
            window: VecDeque::with_capacity(period),
            sum: 0.0,
        })
    }
}

impl IndicatorStream for DpoStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let mut output = vec![0.0; input.len()];
        let mut outputs = [&mut output[..]];
        let produced = self.feed_in_place(inputs, &mut outputs)?;
        output.truncate(produced);

        Ok(vec![output])
    }

    fn feed_in_place(
        &mut self,
        inputs: &[&[Real]],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), input.len(), 0)?;

        let mut out_index = 0usize;
        for &sample in input {
            self.window.push_back(sample);
            self.sum += sample;

            if self.window.len() > self.period {
                let removed = self
                    .window
                    .pop_front()
                    .expect("dpo window should not be empty");
                self.sum -= removed;
            }

            if self.window.len() == self.period {
                let lag_index = self.period - 1 - self.back;
                outputs[0][out_index] = self.window[lag_index] - self.sum / self.period as Real;
                out_index += 1;
            }

            self.progress += 1;
        }

        Ok(out_index)
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 2)
}

fn run_dpo_batch(input: &[Real], period: usize, output: &mut [Real]) -> usize {
    if input.len() < period {
        return 0;
    }

    let back = period / 2 + 1;
    let scale = 1.0 / period as Real;
    let mut sum = 0.0;
    for &sample in &input[..period] {
        sum += sample;
    }

    let mut out_index = 0usize;
    output[out_index] = input[period - 1 - back] - sum * scale;
    out_index += 1;

    for index in period..input.len() {
        sum += input[index] - input[index - period];
        output[out_index] = input[index - back] - sum * scale;
        out_index += 1;
    }

    out_index
}
