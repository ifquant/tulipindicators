use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};
use crate::indicators::shared::RollingStatsState;

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "stderr",
    full_name: "Standard Error",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["stderr"],
};

#[derive(Debug, Clone, Copy)]
pub struct StdErr;

impl Indicator for StdErr {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        Ok(period - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = StdErrStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(StdErrStream::new(options)?)))
    }
}

struct StdErrStream {
    progress: usize,
    period: usize,
    stats: RollingStatsState,
}

impl StdErrStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        Ok(Self {
            progress: 0,
            period,
            stats: RollingStatsState::new(period),
        })
    }
}

impl IndicatorStream for StdErrStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(input.len());
        let scale = 1.0 / (self.period as Real).sqrt();

        for sample in input {
            if let Some(stats) = self.stats.feed(*sample) {
                output.push(scale * stddev_value(stats.variance));
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

fn stddev_value(variance: Real) -> Real {
    if variance > 0.0 {
        variance.sqrt()
    } else {
        variance
    }
}
