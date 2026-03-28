use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{double_input, expect_option_count};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "psar",
    full_name: "Parabolic SAR",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low"],
    option_names: &["acceleration_factor_step", "acceleration_factor_maximum"],
    output_names: &["psar"],
};

#[derive(Debug, Clone, Copy)]
pub struct Psar;

impl Indicator for Psar {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let _ = parse_options(options)?;
        Ok(1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = PsarStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(PsarStream::new(options)?)))
    }
}

struct PsarStream {
    accel_step: Real,
    accel_max: Real,
    progress: usize,
    first: Option<(Real, Real)>,
    state: Option<PsarState>,
}

#[derive(Clone, Copy)]
struct PsarState {
    long: bool,
    sar: Real,
    extreme: Real,
    accel: Real,
    prev_high: Real,
    prev_low: Real,
    prev2_high: Option<Real>,
    prev2_low: Option<Real>,
}

impl PsarStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let (accel_step, accel_max) = parse_options(options)?;
        Ok(Self {
            accel_step,
            accel_max,
            progress: 0,
            first: None,
            state: None,
        })
    }
}

impl IndicatorStream for PsarStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low) = double_input(METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(high.len().saturating_sub(1));

        for (&high, &low) in high.iter().zip(low.iter()) {
            match (self.first, self.state) {
                (None, None) => {
                    self.first = Some((high, low));
                }
                (Some((first_high, first_low)), None) => {
                    let long = first_high + first_low <= high + low;
                    let sar = if long { first_low } else { first_high };
                    let extreme = if long { first_high } else { first_low };
                    let mut state = PsarState {
                        long,
                        sar,
                        extreme,
                        accel: self.accel_step,
                        prev_high: first_high,
                        prev_low: first_low,
                        prev2_high: None,
                        prev2_low: None,
                    };
                    output.push(state.advance(high, low, self.accel_step, self.accel_max));
                    self.state = Some(state);
                    self.first = Some((high, low));
                }
                (_, Some(mut state)) => {
                    output.push(state.advance(high, low, self.accel_step, self.accel_max));
                    self.state = Some(state);
                    self.first = Some((high, low));
                }
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

impl PsarState {
    fn advance(&mut self, high: Real, low: Real, accel_step: Real, accel_max: Real) -> Real {
        self.sar = (self.extreme - self.sar) * self.accel + self.sar;

        if self.long {
            if let Some(prev2_low) = self.prev2_low {
                if self.sar > prev2_low {
                    self.sar = prev2_low;
                }
            }
            if self.sar > self.prev_low {
                self.sar = self.prev_low;
            }

            if self.accel < accel_max && high > self.extreme {
                self.accel = (self.accel + accel_step).min(accel_max);
            }
            if high > self.extreme {
                self.extreme = high;
            }
        } else {
            if let Some(prev2_high) = self.prev2_high {
                if self.sar < prev2_high {
                    self.sar = prev2_high;
                }
            }
            if self.sar < self.prev_high {
                self.sar = self.prev_high;
            }

            if self.accel < accel_max && low < self.extreme {
                self.accel = (self.accel + accel_step).min(accel_max);
            }
            if low < self.extreme {
                self.extreme = low;
            }
        }

        if (self.long && low < self.sar) || (!self.long && high > self.sar) {
            self.accel = accel_step;
            self.sar = self.extreme;
            self.long = !self.long;
            self.extreme = if self.long { high } else { low };
        }

        let output = self.sar;
        self.prev2_high = Some(self.prev_high);
        self.prev2_low = Some(self.prev_low);
        self.prev_high = high;
        self.prev_low = low;
        output
    }
}

fn parse_options(options: &[Real]) -> Result<(Real, Real), IndicatorError> {
    expect_option_count(METADATA.name, options, 2)?;
    let accel_step = options[0];
    let accel_max = options[1];

    if !accel_step.is_finite() || accel_step <= 0.0 {
        return Err(IndicatorError::InvalidOption {
            indicator: METADATA.name,
            option: "acceleration_factor_step",
            value: accel_step,
            reason: "expected a finite value > 0",
        });
    }

    if !accel_max.is_finite() || accel_max <= accel_step {
        return Err(IndicatorError::InvalidOption {
            indicator: METADATA.name,
            option: "acceleration_factor_maximum",
            value: accel_max,
            reason: "expected a finite value > acceleration_factor_step",
        });
    }

    Ok((accel_step, accel_max))
}
