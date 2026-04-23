//! Double Exponential Moving Average (`dema`) removes part of EMA lag with a two-stage EMA chain.
//!
//! The direct kernel keeps both EMA stages in registers and emits `2 * ema1 - ema2` after warmup.
use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};
use crate::indicators::shared::EmaState;

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "dema",
    full_name: "Double Exponential Moving Average",
    category: IndicatorCategory::Overlay,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["dema"],
};

#[derive(Debug, Clone, Copy)]
pub struct Dema;

impl Indicator for Dema {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        Ok((period - 1) * 2)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let lookback = (period - 1) * 2;
        let mut output = vec![0.0; input.len().saturating_sub(lookback)];
        let produced = run_dema_batch(input, period, &mut output);
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
        let output_len = input.len().saturating_sub((period - 1) * 2);
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_dema_batch(input, period, &mut outputs[0][..output_len]))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(DemaStream::new(options)?)))
    }
}

struct DemaStream {
    period: usize,
    progress: usize,
    ema1: EmaState,
    ema2: EmaState,
}

impl DemaStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        let multiplier = ema_multiplier(period);
        Ok(Self {
            period,
            progress: 0,
            ema1: EmaState::new(multiplier),
            ema2: EmaState::new(multiplier),
        })
    }
}

impl IndicatorStream for DemaStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let mut output = vec![0.0; input.len()];
        let mut outputs = [&mut output[..]];
        let produced = self.feed_in_place(inputs, &mut outputs)?;
        output.truncate(produced);
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
        let lookback = (self.period - 1) * 2;
        let mut out_index = 0usize;

        for sample in input {
            let ema1 = self.ema1.feed(*sample);
            let index = self.progress;

            if index >= self.period - 1 {
                let ema2 = self.ema2.feed(ema1);
                if index >= lookback {
                    outputs[0][out_index] = ema1 * 2.0 - ema2;
                    out_index += 1;
                }
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

fn ema_multiplier(period: usize) -> Real {
    2.0 / (period as Real + 1.0)
}

fn run_dema_batch(input: &[Real], period: usize, output: &mut [Real]) -> usize {
    let lookback = (period - 1) * 2;
    if input.len() <= lookback {
        return 0;
    }

    let per = ema_multiplier(period);
    let per1 = 1.0 - per;

    let mut ema = input[0];
    let mut ema2 = ema;
    let mut out_index = 0usize;

    for (index, sample) in input.iter().enumerate() {
        let sample_part = *sample * per;
        ema = ema.mul_add(per1, sample_part);
        if index == period - 1 {
            ema2 = ema;
        }
        if index >= period - 1 {
            let ema_part = ema * per;
            ema2 = ema2.mul_add(per1, ema_part);
            if index >= lookback {
                output[out_index] = ema * 2.0 - ema2;
                out_index += 1;
            }
        }
    }

    out_index
}
