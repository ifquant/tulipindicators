use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "dema",
    full_name: "Double Exponential Moving Average",
    category: IndicatorCategory::Overlay,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["dema"],
};

#[derive(Debug, Clone, Copy)]
pub struct Dema;

impl Indicator for Dema {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        Ok((period - 1) * 2)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let lookback = (period - 1) * 2;
        let mut output = Vec::with_capacity(input.len().saturating_sub(lookback));

        if input.len() <= lookback {
            return Ok(vec![output]);
        }

        let per = ema_multiplier(period);
        let per1 = 1.0 - per;

        let mut ema = input[0];
        let mut ema2 = ema;

        for (index, sample) in input.iter().enumerate() {
            ema = ema * per1 + sample * per;
            if index == period - 1 {
                ema2 = ema;
            }
            if index >= period - 1 {
                ema2 = ema2 * per1 + ema * per;
                if index >= lookback {
                    output.push(ema * 2.0 - ema2);
                }
            }
        }

        Ok(vec![output])
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 1)
}

fn ema_multiplier(period: usize) -> Real {
    2.0 / (period as Real + 1.0)
}
