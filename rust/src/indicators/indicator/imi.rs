//! Intraday Momentum Index (`imi`) applies RSI-style smoothing to open/close intraday moves.
//!
//! The indicator consumes open and close inputs and emits a bounded momentum value after the period
//! window has accumulated enough gains and losses.
use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{double_input, expect_option_count, parse_usize_option};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "imi",
    full_name: "Intraday Momentum Index",
    category: IndicatorCategory::Indicator,
    input_names: &["open", "close"],
    option_names: &["period"],
    output_names: &["imi"],
};

#[derive(Debug, Clone, Copy)]
pub struct Imi;

impl Indicator for Imi {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(parse_period(options)?.saturating_sub(1))
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (open, close) = double_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let output_len = open.len().saturating_sub(period.saturating_sub(1));
        let mut output = vec![0.0; output_len];
        let produced = run_imi_batch(open, close, period, &mut output);
        debug_assert_eq!(produced, output_len);
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let (open, close) = double_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let output_len = open.len().saturating_sub(period.saturating_sub(1));
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_imi_batch(
            open,
            close,
            period,
            &mut outputs[0][..output_len],
        ))
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 2)
}

fn run_imi_batch(open: &[Real], close: &[Real], period: usize, output: &mut [Real]) -> usize {
    if open.len() < period {
        return 0;
    }

    let mut upsum = 0.0;
    let mut downsum = 0.0;

    for index in 0..period {
        let delta = close[index] - open[index];
        if delta > 0.0 {
            upsum += delta;
        } else {
            downsum -= delta;
        }
    }

    output[0] = 100.0 * (upsum / (upsum + downsum));
    let mut out_index = 1usize;

    for index in period..open.len() {
        let added = close[index] - open[index];
        let removed = close[index - period] - open[index - period];

        if added > 0.0 {
            upsum += added;
        } else {
            downsum -= added;
        }

        if removed > 0.0 {
            upsum -= removed;
        } else {
            downsum += removed;
        }

        output[out_index] = 100.0 * (upsum / (upsum + downsum));
        out_index += 1;
    }

    out_index
}
