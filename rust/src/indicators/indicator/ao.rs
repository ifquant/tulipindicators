//! Awesome Oscillator (`ao`) computes the spread between fast and slow median-price sums.
//!
//! The batch and stream paths both keep rolling sums over 5 and 34 median prices so the
//! oscillator can be updated without copying input windows.
use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::double_input;
use crate::indicators::shared::RingSum;

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "ao",
    full_name: "Awesome Oscillator",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low"],
    option_names: &[],
    output_names: &["ao"],
};

#[derive(Debug, Clone, Copy)]
pub struct Ao;

impl Indicator for Ao {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, _options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(33)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low) = double_input(METADATA.name, inputs)?;
        let mut output = vec![0.0; high.len().saturating_sub(33)];
        let output_len = run_ao_batch(high, low, options, &mut output)?;
        output.truncate(output_len);
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let (high, low) = double_input(METADATA.name, inputs)?;
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(
            &METADATA,
            outputs[0].len(),
            high.len().saturating_sub(33),
            0,
        )?;
        run_ao_batch(high, low, options, outputs[0])
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(AoStream::new(options)?)))
    }
}

struct AoStream {
    progress: usize,
    sum5: RingSum,
    sum34: RingSum,
}

impl AoStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        if !options.is_empty() {
            return Err(IndicatorError::WrongOptionCount {
                indicator: METADATA.name,
                expected: 0,
                actual: options.len(),
            });
        }

        Ok(Self {
            progress: 0,
            sum5: RingSum::new(5),
            sum34: RingSum::new(34),
        })
    }
}

impl IndicatorStream for AoStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low) = double_input(METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(high.len().saturating_sub(33));

        for (&high, &low) in high.iter().zip(low.iter()) {
            let hl = 0.5 * (high + low);
            self.sum5.push(hl);
            self.sum34.push(hl);

            if self.sum5.is_full() && self.sum34.is_full() {
                output.push(self.sum5.sum / 5.0 - self.sum34.sum / 34.0);
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn run_ao_batch(
    high: &[Real],
    low: &[Real],
    options: &[Real],
    output: &mut [Real],
) -> Result<usize, IndicatorError> {
    if !options.is_empty() {
        return Err(IndicatorError::WrongOptionCount {
            indicator: METADATA.name,
            expected: 0,
            actual: options.len(),
        });
    }

    if high.len() <= 33 {
        return Ok(0);
    }

    let mut sum34 = 0.0;
    let mut sum5 = 0.0;

    for index in 0..34 {
        let hl = 0.5 * (high[index] + low[index]);
        sum34 += hl;
        if index >= 29 {
            sum5 += hl;
        }
    }

    output[0] = sum5 / 5.0 - sum34 / 34.0;
    let mut out_index = 1;

    for index in 34..high.len() {
        let hl = 0.5 * (high[index] + low[index]);
        sum34 += hl - 0.5 * (high[index - 34] + low[index - 34]);
        sum5 += hl - 0.5 * (high[index - 5] + low[index - 5]);
        output[out_index] = sum5 / 5.0 - sum34 / 34.0;
        out_index += 1;
    }

    Ok(out_index)
}
