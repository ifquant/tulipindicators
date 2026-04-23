//! Ease of Movement (`emv`) relates midpoint movement to volume-adjusted high/low range.
//!
//! The stream state keeps the previous midpoint and a rolling sum so updates only touch the newest
//! candle and the value leaving the window.
use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, triple_input};

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
        expect_option_count(METADATA.name, options, 0)?;
        let (high, low, volume) = triple_input(METADATA.name, inputs)?;
        let output_len = high.len().saturating_sub(1);
        let mut output = vec![0.0; output_len];
        let produced = run_emv_batch(high, low, volume, &mut output);
        debug_assert_eq!(produced, output.len());
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        expect_option_count(METADATA.name, options, 0)?;
        let (high, low, volume) = triple_input(METADATA.name, inputs)?;
        let output_len = high.len().saturating_sub(1);
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_emv_batch(
            high,
            low,
            volume,
            &mut outputs[0][..output_len],
        ))
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

fn run_emv_batch(high: &[Real], low: &[Real], volume: &[Real], output: &mut [Real]) -> usize {
    if high.len() <= 1 {
        return 0;
    }

    let mut last_midpoint = 0.5 * (high[0] + low[0]);
    for (dst, ((&high, &low), &volume)) in output
        .iter_mut()
        .zip(high.iter().zip(low.iter()).zip(volume.iter()).skip(1))
    {
        let midpoint = 0.5 * (high + low);
        let box_ratio = volume / 10_000.0 / (high - low);
        *dst = (midpoint - last_midpoint) / box_ratio;
        last_midpoint = midpoint;
    }

    output.len()
}
