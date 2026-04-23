//! Triangular Moving Average (`trima`) applies a double-smoothed moving-average window.
//!
//! The implementation follows Tulip's period-dependent window shape so odd and even periods produce
//! the expected centered smoothing behavior.
use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};
use crate::indicators::shared::RingSum;

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "trima",
    full_name: "Triangular Moving Average",
    category: IndicatorCategory::Overlay,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["trima"],
};

#[derive(Debug, Clone, Copy)]
pub struct Trima;

impl Indicator for Trima {
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
        let (first_period, second_period) = trima_periods(period);
        let lookback = period - 1;
        let mut output = Vec::with_capacity(input.len().saturating_sub(lookback));

        if input.len() <= lookback {
            return Ok(vec![output]);
        }

        let mut first_sum = RingSum::new(first_period);
        let mut second_sum = RingSum::new(second_period);
        let first_scale = 1.0 / first_period as Real;
        let second_scale = 1.0 / second_period as Real;

        for &sample in input {
            first_sum.push(sample);
            if first_sum.is_full() {
                let sma1 = first_sum.sum * first_scale;
                second_sum.push(sma1);
                if second_sum.is_full() {
                    output.push(second_sum.sum * second_scale);
                }
            }
        }

        Ok(vec![output])
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(TrimaStream::new(options)?)))
    }
}

struct TrimaStream {
    progress: usize,
    first_sum: RingSum,
    second_sum: RingSum,
    first_scale: Real,
    second_scale: Real,
}

impl TrimaStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        let (first_period, second_period) = trima_periods(period);
        Ok(Self {
            progress: 0,
            first_sum: RingSum::new(first_period),
            second_sum: RingSum::new(second_period),
            first_scale: 1.0 / first_period as Real,
            second_scale: 1.0 / second_period as Real,
        })
    }
}

impl IndicatorStream for TrimaStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(input.len());

        for &sample in input {
            self.first_sum.push(sample);
            if self.first_sum.is_full() {
                let sma1 = self.first_sum.sum * self.first_scale;
                self.second_sum.push(sma1);
                if self.second_sum.is_full() {
                    output.push(self.second_sum.sum * self.second_scale);
                }
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

fn trima_periods(period: usize) -> (usize, usize) {
    if period.is_multiple_of(2) {
        (period / 2, period / 2 + 1)
    } else {
        let half = period / 2 + 1;
        (half, half)
    }
}
