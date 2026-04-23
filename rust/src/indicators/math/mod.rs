//! Vector math and rolling utility indicators.
//!
//! This family holds helpers that operate on one or two input series without
//! encoding a market-specific interpretation. It includes comparison-style
//! transforms, rolling extrema, decay helpers, and related batch/stream logic.

mod correlation;
mod cross;
mod decay;
mod extrema;

use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};
use crate::indicators::shared::RingSum;

pub use correlation::{Beta, Correl};
pub use cross::{CrossAny, Crossover};
pub use decay::{Decay, EDecay, Lag};
pub use extrema::{Max, MaxIndex, MidPoint, Min, MinIndex, MinMax, MinMaxIndex};

fn parse_positive_period(
    indicator: &'static str,
    options: &[Real],
) -> Result<usize, IndicatorError> {
    expect_option_count(indicator, options, 1)?;
    parse_usize_option(indicator, options, 0, "period", 1)
}

fn parse_nonnegative_period(
    indicator: &'static str,
    options: &[Real],
) -> Result<usize, IndicatorError> {
    expect_option_count(indicator, options, 1)?;
    parse_usize_option(indicator, options, 0, "period", 0)
}

fn bool_to_real(value: bool) -> Real {
    if value {
        1.0
    } else {
        0.0
    }
}

const SUM_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "sum",
    full_name: "Sum Over Period",
    category: IndicatorCategory::Math,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["sum"],
};

#[derive(Debug, Clone, Copy)]
pub struct Sum;

impl Indicator for Sum {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &SUM_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_positive_period(SUM_METADATA.name, options)?;
        Ok(period - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(SUM_METADATA.name, inputs)?;
        let period = parse_positive_period(SUM_METADATA.name, options)?;
        let lookback = period - 1;
        let mut output = Vec::with_capacity(input.len().saturating_sub(lookback));

        if input.len() <= lookback {
            return Ok(vec![output]);
        }

        let mut state = RingSum::new(period);
        for &sample in input {
            state.push(sample);
            if state.is_full() {
                output.push(state.sum);
            }
        }

        Ok(vec![output])
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        let period = parse_positive_period(SUM_METADATA.name, options)?;
        Ok(Some(Box::new(SumStream {
            state: RingSum::new(period),
            progress: 0,
        })))
    }
}

struct SumStream {
    state: RingSum,
    progress: usize,
}

impl IndicatorStream for SumStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &SUM_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(SUM_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(input.len());

        for &sample in input {
            self.state.push(sample);
            if self.state.is_full() {
                output.push(self.state.sum);
            }
            self.progress += 1;
        }

        Ok(vec![output])
    }
}
