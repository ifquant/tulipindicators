use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "macd",
    full_name: "Moving Average Convergence/Divergence",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["short_period", "long_period", "signal_period"],
    output_names: &["macd", "macd_signal", "macd_histogram"],
};

#[derive(Debug, Clone, Copy)]
pub struct Macd;

impl Indicator for Macd {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let (_, long_period, _) = parse_options(options)?;
        Ok(long_period - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let (short_period, long_period, signal_period) = parse_options(options)?;
        let lookback = long_period - 1;

        let output_len = input.len().saturating_sub(lookback);
        let mut macd = Vec::with_capacity(output_len);
        let mut signal = Vec::with_capacity(output_len);
        let mut hist = Vec::with_capacity(output_len);

        if input.len() <= lookback {
            return Ok(vec![macd, signal, hist]);
        }

        let (short_per, long_per) = ema_pair(short_period, long_period);
        let signal_per = 2.0 / (signal_period as Real + 1.0);

        let mut short_ema = input[0];
        let mut long_ema = input[0];
        let mut signal_ema = 0.0;

        for (index, sample) in input.iter().enumerate().skip(1) {
            short_ema = (*sample - short_ema) * short_per + short_ema;
            long_ema = (*sample - long_ema) * long_per + long_ema;
            let macd_value = short_ema - long_ema;

            if index == long_period - 1 {
                signal_ema = macd_value;
            }

            if index >= long_period - 1 {
                signal_ema = (macd_value - signal_ema) * signal_per + signal_ema;
                macd.push(macd_value);
                signal.push(signal_ema);
                hist.push(macd_value - signal_ema);
            }
        }

        Ok(vec![macd, signal, hist])
    }
}

fn parse_options(options: &[Real]) -> Result<(usize, usize, usize), IndicatorError> {
    expect_option_count(METADATA.name, options, 3)?;
    let short_period = parse_usize_option(METADATA.name, options, 0, "short_period", 1)?;
    let long_period = parse_usize_option(METADATA.name, options, 1, "long_period", 2)?;
    let signal_period = parse_usize_option(METADATA.name, options, 2, "signal_period", 1)?;

    if long_period < short_period {
        return Err(IndicatorError::InvalidOption {
            indicator: METADATA.name,
            option: "long_period",
            value: options[1],
            reason: "expected long_period >= short_period",
        });
    }

    Ok((short_period, long_period, signal_period))
}

fn ema_pair(short_period: usize, long_period: usize) -> (Real, Real) {
    if short_period == 12 && long_period == 26 {
        (0.15, 0.075)
    } else {
        (
            2.0 / (short_period as Real + 1.0),
            2.0 / (long_period as Real + 1.0),
        )
    }
}
