use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};
use crate::indicators::shared::EmaState;

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "trix",
    full_name: "Trix",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["trix"],
};

#[derive(Debug, Clone, Copy)]
pub struct Trix;

impl Indicator for Trix {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        Ok(((period - 1) * 3) + 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let lookback = ((period - 1) * 3) + 1;
        let mut output = Vec::with_capacity(input.len().saturating_sub(lookback));

        if input.len() <= lookback {
            return Ok(vec![output]);
        }

        let start = (period * 3) - 2;
        let per = 2.0 / (period as Real + 1.0);

        let mut ema1 = input[0];
        let mut ema2 = 0.0;
        let mut ema3 = 0.0;

        for (index, sample) in input.iter().enumerate().take(start).skip(1) {
            ema1 = (*sample - ema1) * per + ema1;

            if index == period - 1 {
                ema2 = ema1;
            } else if index > period - 1 {
                ema2 = (ema1 - ema2) * per + ema2;

                if index == period * 2 - 2 {
                    ema3 = ema2;
                } else if index > period * 2 - 2 {
                    ema3 = (ema2 - ema3) * per + ema3;
                }
            }
        }

        for sample in input.iter().skip(start) {
            ema1 = (*sample - ema1) * per + ema1;
            ema2 = (ema1 - ema2) * per + ema2;
            let last = ema3;
            ema3 = (ema2 - ema3) * per + ema3;
            output.push((ema3 - last) / ema3 * 100.0);
        }

        Ok(vec![output])
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(TrixStream::new(options)?)))
    }
}

struct TrixStream {
    period: usize,
    progress: usize,
    ema1: EmaState,
    ema2: EmaState,
    ema3: EmaState,
    last_ema3: Option<Real>,
}

impl TrixStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        let multiplier = 2.0 / (period as Real + 1.0);
        Ok(Self {
            period,
            progress: 0,
            ema1: EmaState::new(multiplier),
            ema2: EmaState::new(multiplier),
            ema3: EmaState::new(multiplier),
            last_ema3: None,
        })
    }
}

impl IndicatorStream for TrixStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let mut output = Vec::new();
        let second_start = self.period - 1;
        let third_start = self.period * 2 - 2;
        let output_start = (self.period * 3) - 2;

        for sample in input {
            let ema1 = self.ema1.feed(*sample);
            let index = self.progress;

            if index >= second_start {
                let ema2 = self.ema2.feed(ema1);
                if index >= third_start {
                    let previous = self.last_ema3;
                    let ema3 = self.ema3.feed(ema2);
                    if index >= output_start {
                        let last = previous.expect("trix third ema should be initialized");
                        output.push((ema3 - last) / ema3 * 100.0);
                    }
                    self.last_ema3 = Some(ema3);
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
