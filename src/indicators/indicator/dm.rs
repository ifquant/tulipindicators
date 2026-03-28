use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{double_input, expect_option_count, parse_usize_option};
use crate::indicators::shared::DirectionalMovementState;

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "dm",
    full_name: "Directional Movement",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low"],
    option_names: &["period"],
    output_names: &["plus_dm", "minus_dm"],
};

#[derive(Debug, Clone, Copy)]
pub struct Dm;

impl Indicator for Dm {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        Ok(period.saturating_sub(1))
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low) = double_input(METADATA.name, inputs)?;
        let mut state = DirectionalMovementState::new(parse_period(options)?);
        let mut plus = Vec::new();
        let mut minus = Vec::new();

        for (&high_value, &low_value) in high.iter().zip(low.iter()) {
            if let Some((up, down)) = state.feed(high_value, low_value) {
                plus.push(up);
                minus.push(down);
            }
        }

        Ok(vec![plus, minus])
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(DmStream::new(options)?)))
    }
}

struct DmStream {
    progress: usize,
    state: DirectionalMovementState,
}

impl DmStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        Ok(Self {
            progress: 0,
            state: DirectionalMovementState::new(parse_period(options)?),
        })
    }
}

impl IndicatorStream for DmStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low) = double_input(METADATA.name, inputs)?;
        let mut plus = Vec::new();
        let mut minus = Vec::new();

        for (&high_value, &low_value) in high.iter().zip(low.iter()) {
            if let Some((up, down)) = self.state.feed(high_value, low_value) {
                plus.push(up);
                minus.push(down);
            }
            self.progress += 1;
        }

        Ok(vec![plus, minus])
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 1)
}
