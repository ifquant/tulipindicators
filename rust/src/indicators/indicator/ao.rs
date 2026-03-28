use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::double_input;
use crate::indicators::shared::RingSum;

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "ao",
    full_name: "Awesome Oscillator",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low"],
    option_names: &[],
    output_names: &["ao"],
};

#[derive(Debug, Clone, Copy)]
pub struct Ao;

impl Indicator for Ao {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, _options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(33)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = AoStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(AoStream::new(options)?)))
    }
}

struct AoStream {
    progress: usize,
    sum5: RingSum,
    sum34: RingSum,
}

impl AoStream {
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
            sum5: RingSum::new(5),
            sum34: RingSum::new(34),
        })
    }
}

impl IndicatorStream for AoStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low) = double_input(METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(high.len().saturating_sub(33));

        for (&high, &low) in high.iter().zip(low.iter()) {
            let hl = 0.5 * (high + low);
            self.sum5.push(hl);
            self.sum34.push(hl);

            if self.sum5.is_full() && self.sum34.is_full() {
                output.push(self.sum5.sum / 5.0 - self.sum34.sum / 34.0);
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}
