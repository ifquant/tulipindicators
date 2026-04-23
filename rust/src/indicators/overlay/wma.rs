//! Weighted Moving Average (`wma`) gives newer samples larger linear weights.
//!
//! The batch kernel updates the weighted sum and plain sum together, avoiding a full weighted-window
//! scan for every output.
use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};
use crate::indicators::shared::WmaState;

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "wma",
    full_name: "Weighted Moving Average",
    category: IndicatorCategory::Overlay,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["wma"],
};

#[derive(Debug, Clone, Copy)]
pub struct Wma;

impl Indicator for Wma {
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
        if input.len() < period {
            return Ok(vec![Vec::new()]);
        }

        let weight_total = (period * (period + 1) / 2) as Real;
        let mut sum = 0.0;
        let mut weighted_sum = 0.0;

        for (index, &sample) in input.iter().take(period - 1).enumerate() {
            weighted_sum += sample * (index + 1) as Real;
            sum += sample;
        }

        let mut output = Vec::with_capacity(input.len() - period + 1);
        for index in (period - 1)..input.len() {
            let sample = input[index];
            weighted_sum += sample * period as Real;
            sum += sample;

            output.push(weighted_sum / weight_total);

            if index + 1 < input.len() {
                weighted_sum -= sum;
                sum -= input[index + 1 - period];
            }
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
        let output_len = input.len().saturating_sub(period - 1);
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;

        if input.len() < period {
            return Ok(0);
        }

        let weight_total = (period * (period + 1) / 2) as Real;
        let mut sum = 0.0;
        let mut weighted_sum = 0.0;

        for (index, &sample) in input.iter().take(period - 1).enumerate() {
            weighted_sum += sample * (index + 1) as Real;
            sum += sample;
        }

        let mut out_index = 0usize;
        for index in (period - 1)..input.len() {
            let sample = input[index];
            weighted_sum += sample * period as Real;
            sum += sample;

            outputs[0][out_index] = weighted_sum / weight_total;
            out_index += 1;

            if index + 1 < input.len() {
                weighted_sum -= sum;
                sum -= input[index + 1 - period];
            }
        }

        Ok(out_index)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(WmaStream::new(options)?)))
    }
}

struct WmaStream {
    progress: usize,
    state: WmaState,
}

impl WmaStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        Ok(Self {
            progress: 0,
            state: WmaState::new(parse_period(options)?),
        })
    }
}

impl IndicatorStream for WmaStream {
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
            if let Some(value) = self.state.feed(*sample) {
                output.push(value);
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
