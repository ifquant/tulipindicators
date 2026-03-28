use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, single_input};
use crate::indicators::shared::EmaState;

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "apo",
    full_name: "Absolute Price Oscillator",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["short_period", "long_period"],
    output_names: &["apo"],
};

#[derive(Debug, Clone, Copy)]
pub struct Apo;

impl Indicator for Apo {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let _ = parse_options(options)?;
        Ok(1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let (short_period, long_period) = parse_options(options)?;
        let mut output = Vec::with_capacity(input.len().saturating_sub(1));

        if input.len() <= 1 {
            return Ok(vec![output]);
        }

        let short_per = ema_multiplier(short_period);
        let long_per = ema_multiplier(long_period);
        let mut short_ema = input[0];
        let mut long_ema = input[0];

        for &sample in &input[1..] {
            short_ema = (sample - short_ema) * short_per + short_ema;
            long_ema = (sample - long_ema) * long_per + long_ema;
            output.push(short_ema - long_ema);
        }

        Ok(vec![output])
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(ApoStream::new(options)?)))
    }
}

struct ApoStream {
    progress: usize,
    short_ema: EmaState,
    long_ema: EmaState,
}

impl ApoStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let (short_period, long_period) = parse_options(options)?;
        Ok(Self {
            progress: 0,
            short_ema: EmaState::new(ema_multiplier(short_period)),
            long_ema: EmaState::new(ema_multiplier(long_period)),
        })
    }
}

impl IndicatorStream for ApoStream {
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
            let short_ema = self.short_ema.feed(sample);
            let long_ema = self.long_ema.feed(sample);
            if self.progress >= 1 {
                output.push(short_ema - long_ema);
            }
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn parse_options(options: &[Real]) -> Result<(usize, usize), IndicatorError> {
    expect_option_count(METADATA.name, options, 2)?;
    let short_period = parse_period(options[0], "short_period", 1)?;
    let long_period = parse_period(options[1], "long_period", 2)?;

    if long_period < short_period {
        return Err(IndicatorError::InvalidOption {
            indicator: METADATA.name,
            option: "long_period",
            value: options[1],
            reason: "expected long_period >= short_period",
        });
    }

    Ok((short_period, long_period))
}

fn parse_period(
    value: Real,
    option: &'static str,
    minimum: usize,
) -> Result<usize, IndicatorError> {
    if !value.is_finite() || value < minimum as Real || value.fract() != 0.0 {
        return Err(IndicatorError::InvalidOption {
            indicator: METADATA.name,
            option,
            value,
            reason: "expected a positive integer",
        });
    }

    Ok(value as usize)
}

fn ema_multiplier(period: usize) -> Real {
    2.0 / (period as Real + 1.0)
}
