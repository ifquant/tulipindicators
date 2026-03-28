use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "tema",
    full_name: "Triple Exponential Moving Average",
    category: IndicatorCategory::Overlay,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["tema"],
};

#[derive(Debug, Clone, Copy)]
pub struct Tema;

impl Indicator for Tema {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        Ok((period - 1) * 3)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let lookback = (period - 1) * 3;
        let mut output = Vec::with_capacity(input.len().saturating_sub(lookback));

        if input.len() <= lookback {
            return Ok(vec![output]);
        }

        let per = ema_multiplier(period);
        let per1 = 1.0 - per;

        let mut ema = input[0];
        let mut ema2 = 0.0;
        let mut ema3 = 0.0;

        for (index, sample) in input.iter().enumerate() {
            ema = ema * per1 + sample * per;
            if index == period - 1 {
                ema2 = ema;
            }
            if index >= period - 1 {
                ema2 = ema2 * per1 + ema * per;
                if index == (period - 1) * 2 {
                    ema3 = ema2;
                }
                if index >= (period - 1) * 2 {
                    ema3 = ema3 * per1 + ema2 * per;
                    if index >= lookback {
                        output.push(3.0 * ema - 3.0 * ema2 + ema3);
                    }
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
