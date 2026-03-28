use crate::core::error::IndicatorError;
use crate::core::types::{IndicatorCategory, Real};

#[derive(Debug, Clone, Copy)]
pub struct IndicatorMetadata {
    pub name: &'static str,
    pub full_name: &'static str,
    pub category: IndicatorCategory,
    pub input_names: &'static [&'static str],
    pub option_names: &'static [&'static str],
    pub output_names: &'static [&'static str],
}

pub trait Indicator: Sync {
    fn metadata(&self) -> &'static IndicatorMetadata;
    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError>;
    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError>;

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        let _ = options;
        Ok(None)
    }
}

pub trait IndicatorStream {
    fn metadata(&self) -> &'static IndicatorMetadata;
    fn progress(&self) -> usize;
    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError>;
}
