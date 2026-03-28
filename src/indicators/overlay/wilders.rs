use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};
use crate::indicators::shared::WildersAverageState;

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "wilders",
    full_name: "Wilders Smoothing",
    category: IndicatorCategory::Overlay,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["wilders"],
};

#[derive(Debug, Clone, Copy)]
pub struct Wilders;

impl Indicator for Wilders {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        Ok(period - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        single_input(METADATA.name, inputs)?;
        let mut stream = WildersStream::new(options)?;
        Ok(stream.feed(inputs)?)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(WildersStream::new(options)?)))
    }
}

struct WildersStream {
    progress: usize,
    smoother: WildersAverageState,
}

impl WildersStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        Ok(Self {
            progress: 0,
            smoother: WildersAverageState::new(parse_period(options)?),
        })
    }
}

impl IndicatorStream for WildersStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(input.len());

        for sample in input {
            if let Some(value) = self.smoother.feed(*sample) {
                output.push(value);
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
