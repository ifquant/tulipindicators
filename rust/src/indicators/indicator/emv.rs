use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::triple_input;

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "emv",
    full_name: "Ease of Movement",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low", "volume"],
    option_names: &[],
    output_names: &["emv"],
};

#[derive(Debug, Clone, Copy)]
pub struct Emv;

impl Indicator for Emv {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, _options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = EmvStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(EmvStream::new(options)?)))
    }
}

struct EmvStream {
    progress: usize,
    last_midpoint: Option<Real>,
}

impl EmvStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        if !options.is_empty() {
            return Err(IndicatorError::WrongOptionCount {
                indicator: METADATA.name,
                expected: 0,
                actual: options.len(),
            });
        }

        Ok(Self {
            progress: 0,
            last_midpoint: None,
        })
    }
}

impl IndicatorStream for EmvStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, volume) = triple_input(METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(high.len().saturating_sub(1));

        for ((&high, &low), &volume) in high.iter().zip(low.iter()).zip(volume.iter()) {
            let midpoint = 0.5 * (high + low);
            if let Some(last) = self.last_midpoint {
                let box_ratio = volume / 10_000.0 / (high - low);
                output.push((midpoint - last) / box_ratio);
            }
            self.last_midpoint = Some(midpoint);
            self.progress += 1;
        }

        Ok(vec![output])
    }
}
