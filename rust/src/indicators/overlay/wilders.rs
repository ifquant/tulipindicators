use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
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
        let input = single_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let output_len = input.len().saturating_sub(period - 1);
        let mut output = vec![0.0; output_len];
        let produced = run_wilders_batch(input, period, &mut output);
        debug_assert_eq!(produced, output.len());
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let output_len = input.len().saturating_sub(period - 1);
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_wilders_batch(
            input,
            period,
            &mut outputs[0][..output_len],
        ))
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

    fn feed_in_place(
        &mut self,
        inputs: &[&[Real]],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), input.len(), 0)?;

        let mut out_index = 0usize;
        for &sample in input {
            if let Some(value) = self.smoother.feed(sample) {
                outputs[0][out_index] = value;
                out_index += 1;
            }
            self.progress += 1;
        }

        Ok(out_index)
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 1)
}

fn run_wilders_batch(input: &[Real], period: usize, output: &mut [Real]) -> usize {
    if input.len() < period {
        return 0;
    }

    let per = 1.0 / period as Real;
    let mut sum = 0.0;
    for &sample in &input[..period] {
        sum += sample;
    }

    let mut value = sum / period as Real;
    output[0] = value;
    for (dst, &sample) in output[1..].iter_mut().zip(&input[period..]) {
        value = (sample - value) * per + value;
        *dst = value;
    }

    output.len()
}
