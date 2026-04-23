//! Rate of Change Ratio (`rocr`) divides the current value by the value `period` samples ago.
//!
//! This is the ratio form of ROC and therefore preserves the same lookback and direct indexing
//! shape in batch mode.
use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, output_len_for_input, validate_output_slices, Indicator, IndicatorMetadata,
    IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "rocr",
    full_name: "Rate of Change Ratio",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["rocr"],
};

const ROCR100_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "rocr100",
    full_name: "Rate of Change Ratio 100 Scale",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["rocr100"],
};

#[derive(Debug, Clone, Copy)]
pub struct Rocr;
#[derive(Debug, Clone, Copy)]
pub struct Rocr100;

impl Indicator for Rocr {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        parse_period(options)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let output_len = output_len_for_input(input.len(), period);
        let mut output = Vec::with_capacity(output_len);

        for index in period..input.len() {
            output.push(input[index] / input[index - period]);
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
        let period = parse_period(options)?;
        let output_len = output_len_for_input(input.len(), period);
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;

        for (dst, index) in outputs[0][..output_len].iter_mut().zip(period..input.len()) {
            *dst = input[index] / input[index - period];
        }

        Ok(output_len)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(RocrStream::new(options)?)))
    }
}

impl Indicator for Rocr100 {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &ROCR100_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        parse_period(options)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(ROCR100_METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let output_len = output_len_for_input(input.len(), period);
        let mut output = Vec::with_capacity(output_len);

        for index in period..input.len() {
            output.push(100.0 * input[index] / input[index - period]);
        }

        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(ROCR100_METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let output_len = output_len_for_input(input.len(), period);
        validate_output_slices(&ROCR100_METADATA, outputs, 1)?;
        ensure_output_len(&ROCR100_METADATA, outputs[0].len(), output_len, 0)?;

        for (dst, index) in outputs[0][..output_len].iter_mut().zip(period..input.len()) {
            *dst = 100.0 * input[index] / input[index - period];
        }

        Ok(output_len)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(Rocr100Stream::new(options)?)))
    }
}

struct RocrStream {
    period: usize,
    buffer: Vec<Real>,
    len: usize,
    cursor: usize,
    progress: usize,
}

struct Rocr100Stream {
    period: usize,
    buffer: Vec<Real>,
    len: usize,
    cursor: usize,
    progress: usize,
}

impl RocrStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        Ok(Self {
            period,
            buffer: vec![0.0; period],
            len: 0,
            cursor: 0,
            progress: 0,
        })
    }
}

impl Rocr100Stream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        Ok(Self {
            period,
            buffer: vec![0.0; period],
            len: 0,
            cursor: 0,
            progress: 0,
        })
    }
}

impl IndicatorStream for RocrStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(input.len().saturating_sub(self.period));

        for &sample in input {
            if self.len < self.period {
                self.buffer[self.len] = sample;
                self.len += 1;
            } else {
                let lagged = self.buffer[self.cursor];
                self.buffer[self.cursor] = sample;
                self.cursor = (self.cursor + 1) % self.period;
                output.push(sample / lagged);
            }
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

impl IndicatorStream for Rocr100Stream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &ROCR100_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(ROCR100_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(input.len().saturating_sub(self.period));

        for &sample in input {
            if self.len < self.period {
                self.buffer[self.len] = sample;
                self.len += 1;
            } else {
                let lagged = self.buffer[self.cursor];
                self.buffer[self.cursor] = sample;
                self.cursor = (self.cursor + 1) % self.period;
                output.push(100.0 * sample / lagged);
            }
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 1)
}
