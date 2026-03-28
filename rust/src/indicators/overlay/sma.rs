use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "sma",
    full_name: "Simple Moving Average",
    category: IndicatorCategory::Overlay,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["sma"],
};

#[derive(Debug, Clone, Copy)]
pub struct Sma;

impl Indicator for Sma {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options, METADATA.name)?;
        Ok(period - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let period = parse_period(options, METADATA.name)?;
        let lookback = period - 1;
        let mut output = Vec::with_capacity(input.len().saturating_sub(lookback));

        if input.len() <= lookback {
            return Ok(vec![output]);
        }

        let scale = 1.0 / period as Real;
        let mut sum: Real = input.iter().take(period).sum();
        output.push(sum * scale);

        for index in period..input.len() {
            sum += input[index];
            sum -= input[index - period];
            output.push(sum * scale);
        }

        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let period = parse_period(options, METADATA.name)?;
        validate_output_slices(&METADATA, outputs, 1)?;
        let output_len = input.len().saturating_sub(period - 1);
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;

        if output_len == 0 {
            return Ok(0);
        }

        let scale = 1.0 / period as Real;
        let mut sum: Real = input.iter().take(period).sum();
        outputs[0][0] = sum * scale;

        for index in period..input.len() {
            sum += input[index];
            sum -= input[index - period];
            outputs[0][index - period + 1] = sum * scale;
        }

        Ok(output_len)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(SmaStream::new(options)?)))
    }
}

struct SmaStream {
    period: usize,
    scale: Real,
    sum: Real,
    buffer: Vec<Real>,
    cursor: usize,
    progress: usize,
}

impl SmaStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(options, METADATA.name)?;
        Ok(Self {
            period,
            scale: 1.0 / period as Real,
            sum: 0.0,
            buffer: Vec::with_capacity(period),
            cursor: 0,
            progress: 0,
        })
    }
}

impl IndicatorStream for SmaStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(input.len());

        for sample in input {
            if self.buffer.len() < self.period {
                self.buffer.push(*sample);
                self.sum += *sample;
                if self.buffer.len() == self.period {
                    output.push(self.sum * self.scale);
                }
            } else {
                self.sum -= self.buffer[self.cursor];
                self.buffer[self.cursor] = *sample;
                self.sum += *sample;
                self.cursor = (self.cursor + 1) % self.period;
                output.push(self.sum * self.scale);
            }

            self.progress += 1;
        }

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
        let buffer = &mut self.buffer;
        let period = self.period;
        let scale = self.scale;
        let mut sum = self.sum;
        let mut cursor = self.cursor;

        for &sample in input {
            if buffer.len() < period {
                buffer.push(sample);
                sum += sample;
                if buffer.len() == period {
                    outputs[0][out_index] = sum * scale;
                    out_index += 1;
                }
            } else {
                sum -= buffer[cursor];
                buffer[cursor] = sample;
                sum += sample;
                cursor += 1;
                if cursor == period {
                    cursor = 0;
                }
                outputs[0][out_index] = sum * scale;
                out_index += 1;
            }

            self.progress += 1;
        }

        self.sum = sum;
        self.cursor = cursor;
        Ok(out_index)
    }
}

fn parse_period(options: &[Real], indicator: &'static str) -> Result<usize, IndicatorError> {
    expect_option_count(indicator, options, 1)?;
    parse_usize_option(indicator, options, 0, "period", 1)
}
