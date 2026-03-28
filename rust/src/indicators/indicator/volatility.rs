use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};
use crate::indicators::shared::RingSum;

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "volatility",
    full_name: "Volatility",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["volatility"],
};

const ANNUAL_SCALE: Real = 15.874507866387544;

#[derive(Debug, Clone, Copy)]
pub struct Volatility;

impl Indicator for Volatility {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        parse_period(options)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = VolatilityStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(VolatilityStream::new(options)?)))
    }
}

struct VolatilityStream {
    period: usize,
    progress: usize,
    previous: Option<Real>,
    changes: RingSum,
    changes2: RingSum,
}

impl VolatilityStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        Ok(Self {
            period,
            progress: 0,
            previous: None,
            changes: RingSum::new(period),
            changes2: RingSum::new(period),
        })
    }
}

impl IndicatorStream for VolatilityStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(input.len().saturating_sub(self.period));

        for &sample in input {
            if let Some(previous) = self.previous {
                let change = sample / previous - 1.0;
                self.changes.push(change);
                self.changes2.push(change * change);

                if self.changes.is_full() {
                    let mean = self.changes.sum / self.period as Real;
                    let variance = self.changes2.sum / self.period as Real - mean * mean;
                    output.push(variance.sqrt() * ANNUAL_SCALE);
                }
            }

            self.previous = Some(sample);
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 1)
}
