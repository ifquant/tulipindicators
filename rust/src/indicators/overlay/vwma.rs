use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{double_input, expect_option_count, parse_usize_option};
use crate::indicators::shared::RingSum;

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "vwma",
    full_name: "Volume Weighted Moving Average",
    category: IndicatorCategory::Overlay,
    input_names: &["close", "volume"],
    option_names: &["period"],
    output_names: &["vwma"],
};

#[derive(Debug, Clone, Copy)]
pub struct Vwma;

impl Indicator for Vwma {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        Ok(period - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (close, volume) = double_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let lookback = period - 1;
        let mut output = Vec::with_capacity(close.len().saturating_sub(lookback));

        if close.len() <= lookback {
            return Ok(vec![output]);
        }

        let mut weighted_sum = RingSum::new(period);
        let mut volume_sum = RingSum::new(period);

        for (&price, &vol) in close.iter().zip(volume.iter()) {
            weighted_sum.push(price * vol);
            volume_sum.push(vol);
            if weighted_sum.is_full() {
                output.push(weighted_sum.sum / volume_sum.sum);
            }
        }

        Ok(vec![output])
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(VwmaStream::new(options)?)))
    }
}

struct VwmaStream {
    progress: usize,
    weighted_sum: RingSum,
    volume_sum: RingSum,
}

impl VwmaStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        Ok(Self {
            progress: 0,
            weighted_sum: RingSum::new(period),
            volume_sum: RingSum::new(period),
        })
    }
}

impl IndicatorStream for VwmaStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (close, volume) = double_input(METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(close.len());

        for (&price, &vol) in close.iter().zip(volume.iter()) {
            self.weighted_sum.push(price * vol);
            self.volume_sum.push(vol);
            if self.weighted_sum.is_full() {
                output.push(self.weighted_sum.sum / self.volume_sum.sum);
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
