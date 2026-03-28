use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};

const TSI_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "tsi",
    full_name: "True Strength Index",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["y_period", "z_period"],
    output_names: &["tsi"],
};

#[derive(Debug, Clone, Copy)]
pub struct Tsi;

impl Indicator for Tsi {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &TSI_METADATA
    }

    fn lookback(&self, _options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = TsiStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(TsiStream::new(options)?)))
    }
}

struct TsiStream {
    progress: usize,
    y_mul: Real,
    z_mul: Real,
    previous_price: Option<Real>,
    y_ema_num: Real,
    z_ema_num: Real,
    y_ema_den: Real,
    z_ema_den: Real,
    initialized: bool,
}

impl TsiStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let (y_period, z_period) = parse_periods(options)?;
        Ok(Self {
            progress: 0,
            y_mul: 2.0 / (1.0 + y_period as Real),
            z_mul: 2.0 / (1.0 + z_period as Real),
            previous_price: None,
            y_ema_num: 0.0,
            z_ema_num: 0.0,
            y_ema_den: 0.0,
            z_ema_den: 0.0,
            initialized: false,
        })
    }
}

impl IndicatorStream for TsiStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &TSI_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(TSI_METADATA.name, inputs)?;
        let mut output = Vec::new();

        for &sample in input {
            match self.previous_price {
                None => {
                    self.previous_price = Some(sample);
                }
                Some(previous_price) if !self.initialized => {
                    let momentum = sample - previous_price;
                    let abs_momentum = momentum.abs();
                    self.y_ema_num = momentum;
                    self.z_ema_num = momentum;
                    self.y_ema_den = abs_momentum;
                    self.z_ema_den = abs_momentum;
                    output.push(if self.z_ema_den != 0.0 {
                        100.0 * (self.z_ema_num / self.z_ema_den)
                    } else {
                        0.0
                    });
                    self.previous_price = Some(sample);
                    self.initialized = true;
                }
                Some(previous_price) => {
                    let momentum = sample - previous_price;
                    let abs_momentum = momentum.abs();

                    self.y_ema_num = (momentum - self.y_ema_num) * self.y_mul + self.y_ema_num;
                    self.y_ema_den = (abs_momentum - self.y_ema_den) * self.y_mul + self.y_ema_den;
                    self.z_ema_num =
                        (self.y_ema_num - self.z_ema_num) * self.z_mul + self.z_ema_num;
                    self.z_ema_den =
                        (self.y_ema_den - self.z_ema_den) * self.z_mul + self.z_ema_den;

                    output.push(if self.z_ema_den != 0.0 {
                        100.0 * (self.z_ema_num / self.z_ema_den)
                    } else {
                        0.0
                    });
                    self.previous_price = Some(sample);
                }
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn parse_periods(options: &[Real]) -> Result<(usize, usize), IndicatorError> {
    expect_option_count(TSI_METADATA.name, options, 2)?;
    let y_period = parse_usize_option(TSI_METADATA.name, options, 0, "y_period", 1)?;
    let z_period = parse_usize_option(TSI_METADATA.name, options, 1, "z_period", 1)?;
    Ok((y_period, z_period))
}
