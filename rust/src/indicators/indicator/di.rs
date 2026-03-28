use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, triple_input};
use crate::indicators::shared::DirectionalIndexState;

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "di",
    full_name: "Directional Indicator",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low", "close"],
    option_names: &["period"],
    output_names: &["plus_di", "minus_di"],
};

#[derive(Debug, Clone, Copy)]
pub struct Di;

impl Indicator for Di {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        Ok(period.saturating_sub(1))
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close) = triple_input(METADATA.name, inputs)?;
        let mut state = DirectionalIndexState::new(parse_period(options)?);
        let mut plus = Vec::new();
        let mut minus = Vec::new();

        for index in 0..high.len() {
            if let Some((up, down, atr)) = state.feed(high[index], low[index], close[index]) {
                plus.push(100.0 * up / atr);
                minus.push(100.0 * down / atr);
            }
        }

        Ok(vec![plus, minus])
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(DiStream::new(options)?)))
    }
}

struct DiStream {
    progress: usize,
    state: DirectionalIndexState,
}

impl DiStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        Ok(Self {
            progress: 0,
            state: DirectionalIndexState::new(parse_period(options)?),
        })
    }
}

impl IndicatorStream for DiStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close) = triple_input(METADATA.name, inputs)?;
        let mut plus = Vec::new();
        let mut minus = Vec::new();

        for index in 0..high.len() {
            if let Some((up, down, atr)) = self.state.feed(high[index], low[index], close[index]) {
                plus.push(100.0 * up / atr);
                minus.push(100.0 * down / atr);
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
