use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{double_input, expect_option_count, parse_usize_option};
use crate::indicators::shared::{directional_ratio, DirectionalMovementState, WildersAverageState};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "adx",
    full_name: "Average Directional Movement Index",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low"],
    option_names: &["period"],
    output_names: &["adx"],
};

#[derive(Debug, Clone, Copy)]
pub struct Adx;

impl Indicator for Adx {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        Ok((period - 1) * 2)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low) = double_input(METADATA.name, inputs)?;
        let mut dm_state = DirectionalMovementState::new(parse_period(options)?);
        let mut adx_state = WildersAverageState::new(parse_period(options)?);
        let mut output = Vec::new();

        for (&high_value, &low_value) in high.iter().zip(low.iter()) {
            if let Some((up, down)) = dm_state.feed(high_value, low_value) {
                let dx = directional_ratio(up, down);
                if let Some(adx) = adx_state.feed(dx) {
                    output.push(adx);
                }
            }
        }

        Ok(vec![output])
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(AdxStream::new(options)?)))
    }
}

struct AdxStream {
    progress: usize,
    dm_state: DirectionalMovementState,
    adx_state: WildersAverageState,
}

impl AdxStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        Ok(Self {
            progress: 0,
            dm_state: DirectionalMovementState::new(period),
            adx_state: WildersAverageState::new(period),
        })
    }
}

impl IndicatorStream for AdxStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low) = double_input(METADATA.name, inputs)?;
        let mut output = Vec::new();

        for (&high_value, &low_value) in high.iter().zip(low.iter()) {
            if let Some((up, down)) = self.dm_state.feed(high_value, low_value) {
                let dx = directional_ratio(up, down);
                if let Some(adx) = self.adx_state.feed(dx) {
                    output.push(adx);
                }
            }
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 2)
}
