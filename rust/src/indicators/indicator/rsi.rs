use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "rsi",
    full_name: "Relative Strength Index",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["rsi"],
};

#[derive(Debug, Clone, Copy)]
pub struct Rsi;

impl Indicator for Rsi {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        parse_period(options)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let mut output = Vec::with_capacity(input.len().saturating_sub(period));

        if input.len() <= period {
            return Ok(vec![output]);
        }

        let per = 1.0 / period as Real;
        let mut smooth_up = 0.0;
        let mut smooth_down = 0.0;

        for index in 1..=period {
            let delta = input[index] - input[index - 1];
            if delta > 0.0 {
                smooth_up += delta;
            } else {
                smooth_down += -delta;
            }
        }

        smooth_up /= period as Real;
        smooth_down /= period as Real;
        output.push(rsi_value(smooth_up, smooth_down));

        for index in (period + 1)..input.len() {
            let delta = input[index] - input[index - 1];
            let upward = delta.max(0.0);
            let downward = (-delta).max(0.0);

            smooth_up = (upward - smooth_up) * per + smooth_up;
            smooth_down = (downward - smooth_down) * per + smooth_down;
            output.push(rsi_value(smooth_up, smooth_down));
        }

        Ok(vec![output])
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(RsiStream::new(options)?)))
    }
}

struct RsiStream {
    period: usize,
    progress: usize,
    last_input: Option<Real>,
    smooth_up: Real,
    smooth_down: Real,
}

impl RsiStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        Ok(Self {
            period: parse_period(options)?,
            progress: 0,
            last_input: None,
            smooth_up: 0.0,
            smooth_down: 0.0,
        })
    }
}

impl IndicatorStream for RsiStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let mut output = Vec::new();
        let per = 1.0 / self.period as Real;

        for sample in input {
            match self.last_input {
                None => {
                    self.last_input = Some(*sample);
                    self.progress += 1;
                    continue;
                }
                Some(previous) => {
                    let delta = *sample - previous;
                    let upward = delta.max(0.0);
                    let downward = (-delta).max(0.0);

                    if self.progress <= self.period {
                        self.smooth_up += upward;
                        self.smooth_down += downward;

                        if self.progress == self.period {
                            self.smooth_up /= self.period as Real;
                            self.smooth_down /= self.period as Real;
                            output.push(rsi_value(self.smooth_up, self.smooth_down));
                        }
                    } else {
                        self.smooth_up = (upward - self.smooth_up) * per + self.smooth_up;
                        self.smooth_down = (downward - self.smooth_down) * per + self.smooth_down;
                        output.push(rsi_value(self.smooth_up, self.smooth_down));
                    }

                    self.last_input = Some(*sample);
                    self.progress += 1;
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

fn rsi_value(smooth_up: Real, smooth_down: Real) -> Real {
    let total = smooth_up + smooth_down;
    if total == 0.0 {
        0.0
    } else {
        100.0 * (smooth_up / total)
    }
}
