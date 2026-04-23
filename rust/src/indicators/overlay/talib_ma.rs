//! TA-Lib-style moving-average overlays are dispatch wrappers: `ma` selects a
//! fixed-period smoother by MA type, while `mavp` clamps a per-bar period
//! input and reuses the same concrete MA implementations. `mavp` also owns the
//! variable-period policy: it clamps each requested period, special-cases SMA,
//! and caches repeated non-SMA periods before dispatching.

use super::beta_smoothers::Mama;
use super::{
    dema::Dema, ema::Ema, kama::Kama, sma::Sma, t3::T3, tema::Tema, trima::Trima, wma::Wma,
};
use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{
    double_input, expect_option_count, parse_usize_option, single_input,
};
use std::collections::BTreeMap;

const MA_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "ma",
    full_name: "Moving Average",
    category: IndicatorCategory::Overlay,
    input_names: &["real"],
    option_names: &["period", "ma_type"],
    output_names: &["ma"],
};

const MAVP_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "mavp",
    full_name: "Moving Average With Variable Period",
    category: IndicatorCategory::Overlay,
    input_names: &["real", "periods"],
    option_names: &["min_period", "max_period", "ma_type"],
    output_names: &["mavp"],
};

#[derive(Debug, Clone, Copy)]
pub struct Ma;

#[derive(Debug, Clone, Copy)]
pub struct Mavp;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TalibMaType {
    Sma,
    Ema,
    Wma,
    Dema,
    Tema,
    Trima,
    Kama,
    Mama,
    T3,
}

// `ma` and `mavp` are thin option parsers over the same MA type dispatch.
impl Indicator for Ma {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &MA_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let (period, ma_type) = parse_ma_options(MA_METADATA.name, options)?;
        ma_lookback(period, ma_type)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(MA_METADATA.name, inputs)?;
        let (period, ma_type) = parse_ma_options(MA_METADATA.name, options)?;
        let output_len = input.len().saturating_sub(ma_lookback(period, ma_type)?);
        let mut output = vec![0.0; output_len];
        let produced = run_ma_batch(input, period, ma_type, &mut output)?;
        debug_assert_eq!(produced, output_len);
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(MA_METADATA.name, inputs)?;
        let (period, ma_type) = parse_ma_options(MA_METADATA.name, options)?;
        let output_len = input.len().saturating_sub(ma_lookback(period, ma_type)?);
        validate_output_slices(&MA_METADATA, outputs, 1)?;
        ensure_output_len(&MA_METADATA, outputs[0].len(), output_len, 0)?;
        run_ma_batch(input, period, ma_type, &mut outputs[0][..output_len])
    }
}

impl Indicator for Mavp {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &MAVP_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let (_, max_period, ma_type) = parse_mavp_options(options)?;
        ma_lookback(max_period, ma_type)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (input, periods) = double_input(MAVP_METADATA.name, inputs)?;
        let (_, max_period, ma_type) = parse_mavp_options(options)?;
        let lookback = ma_lookback(max_period, ma_type)?;
        let mut output = vec![0.0; input.len().saturating_sub(lookback)];
        let produced = run_mavp_batch(input, periods, options, ma_type, &mut output)?;
        debug_assert_eq!(produced, output.len());
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let (input, _) = double_input(MAVP_METADATA.name, inputs)?;
        let (_, max_period, ma_type) = parse_mavp_options(options)?;
        let output_len = input
            .len()
            .saturating_sub(ma_lookback(max_period, ma_type)?);
        validate_output_slices(&MAVP_METADATA, outputs, 1)?;
        ensure_output_len(&MAVP_METADATA, outputs[0].len(), output_len, 0)?;
        let (_, periods) = double_input(MAVP_METADATA.name, inputs)?;
        run_mavp_batch(
            input,
            periods,
            options,
            ma_type,
            &mut outputs[0][..output_len],
        )
    }
}

pub(crate) fn ma_lookback(period: usize, ma_type: TalibMaType) -> Result<usize, IndicatorError> {
    match ma_type {
        TalibMaType::Sma => Sma.lookback(&[period as Real]),
        TalibMaType::Ema => Ema.lookback(&[period as Real]),
        TalibMaType::Wma => Wma.lookback(&[period as Real]),
        TalibMaType::Dema => Dema.lookback(&[period as Real]),
        TalibMaType::Tema => Tema.lookback(&[period as Real]),
        TalibMaType::Trima => Trima.lookback(&[period as Real]),
        TalibMaType::Kama => Kama.lookback(&[period as Real]),
        TalibMaType::Mama => Mama.lookback(&[0.5, 0.05]),
        TalibMaType::T3 => T3.lookback(&[period as Real, 0.7]),
    }
}

// Batch dispatch forwards to the concrete moving-average implementation
// selected by `ma_type`.
pub(crate) fn run_ma_batch(
    input: &[Real],
    period: usize,
    ma_type: TalibMaType,
    output: &mut [Real],
) -> Result<usize, IndicatorError> {
    let inputs = [input];
    match ma_type {
        TalibMaType::Sma => Sma.run_in_place(&inputs, &[period as Real], &mut [&mut output[..]]),
        TalibMaType::Ema => Ema.run_in_place(&inputs, &[period as Real], &mut [&mut output[..]]),
        TalibMaType::Wma => Wma.run_in_place(&inputs, &[period as Real], &mut [&mut output[..]]),
        TalibMaType::Dema => Dema.run_in_place(&inputs, &[period as Real], &mut [&mut output[..]]),
        TalibMaType::Tema => Tema.run_in_place(&inputs, &[period as Real], &mut [&mut output[..]]),
        TalibMaType::Trima => {
            Trima.run_in_place(&inputs, &[period as Real], &mut [&mut output[..]])
        }
        TalibMaType::Kama => Kama.run_in_place(&inputs, &[period as Real], &mut [&mut output[..]]),
        TalibMaType::T3 => T3.run_in_place(&inputs, &[period as Real, 0.7], &mut [&mut output[..]]),
        TalibMaType::Mama => {
            let lookback = Mama.lookback(&[0.5, 0.05])?;
            let output_len = input.len().saturating_sub(lookback);
            debug_assert!(output.len() >= output_len);
            let mut fama = vec![0.0; output_len];
            let mut outputs = [&mut output[..output_len], &mut fama[..]];
            Mama.run_in_place(&inputs, &[0.5, 0.05], &mut outputs)
        }
    }
}

// `mavp` clamps each requested period, caches repeated series where useful,
// and uses the same concrete MA dispatch as `ma`.
fn run_mavp_batch(
    input: &[Real],
    periods: &[Real],
    options: &[Real],
    ma_type: TalibMaType,
    output: &mut [Real],
) -> Result<usize, IndicatorError> {
    let (min_period, max_period, _) = parse_mavp_options(options)?;
    let lookback = ma_lookback(max_period, ma_type)?;
    if input.len() <= lookback {
        return Ok(0);
    }

    if ma_type == TalibMaType::Sma {
        return run_mavp_sma_batch(input, periods, min_period, max_period, lookback, output);
    }

    let mut cache: BTreeMap<usize, Vec<Real>> = BTreeMap::new();

    for (out_index, actual_index) in (lookback..input.len()).enumerate() {
        let period = clamp_period(periods[actual_index], min_period, max_period)?;
        let series = cache.entry(period).or_insert_with(|| {
            let len = input
                .len()
                .saturating_sub(ma_lookback(period, ma_type).unwrap_or(0));
            let mut out = vec![0.0; len];
            let _ = run_ma_batch(input, period, ma_type, &mut out);
            out
        });
        let current_lookback = ma_lookback(period, ma_type)?;
        output[out_index] = series[actual_index - current_lookback];
    }

    Ok(input.len() - lookback)
}

fn run_mavp_sma_batch(
    input: &[Real],
    periods: &[Real],
    min_period: usize,
    max_period: usize,
    lookback: usize,
    output: &mut [Real],
) -> Result<usize, IndicatorError> {
    let mut prefix = Vec::with_capacity(input.len() + 1);
    prefix.push(0.0);
    let mut running = 0.0;
    for &sample in input {
        running += sample;
        prefix.push(running);
    }

    let mut out_index = 0usize;
    for index in lookback..input.len() {
        let period = clamp_period(periods[index], min_period, max_period)?;
        let sum = prefix[index + 1] - prefix[index + 1 - period];
        output[out_index] = sum / period as Real;
        out_index += 1;
    }

    Ok(out_index)
}

fn clamp_period(
    value: Real,
    min_period: usize,
    max_period: usize,
) -> Result<usize, IndicatorError> {
    if !value.is_finite() {
        return Err(IndicatorError::InvalidOption {
            indicator: MAVP_METADATA.name,
            option: "periods",
            value,
            reason: "expected finite per-sample periods",
        });
    }
    let mut period = value as isize;
    if period < min_period as isize {
        period = min_period as isize;
    }
    if period > max_period as isize {
        period = max_period as isize;
    }
    Ok(period as usize)
}

fn parse_ma_options(
    indicator: &'static str,
    options: &[Real],
) -> Result<(usize, TalibMaType), IndicatorError> {
    expect_option_count(indicator, options, 2)?;
    let period = parse_usize_option(indicator, options, 0, "period", 1)?;
    let ma_type = parse_ma_type(indicator, options[1])?;
    Ok((period, ma_type))
}

fn parse_mavp_options(options: &[Real]) -> Result<(usize, usize, TalibMaType), IndicatorError> {
    expect_option_count(MAVP_METADATA.name, options, 3)?;
    let min_period = parse_usize_option(MAVP_METADATA.name, options, 0, "min_period", 2)?;
    let max_period = parse_usize_option(MAVP_METADATA.name, options, 1, "max_period", min_period)?;
    let ma_type = parse_ma_type(MAVP_METADATA.name, options[2])?;
    Ok((min_period, max_period, ma_type))
}

pub(crate) fn parse_ma_type(
    indicator: &'static str,
    value: Real,
) -> Result<TalibMaType, IndicatorError> {
    if !value.is_finite() || value.fract() != 0.0 {
        return Err(IndicatorError::InvalidOption {
            indicator,
            option: "ma_type",
            value,
            reason: "expected an integer MA type between 0 and 8",
        });
    }

    match value as i32 {
        0 => Ok(TalibMaType::Sma),
        1 => Ok(TalibMaType::Ema),
        2 => Ok(TalibMaType::Wma),
        3 => Ok(TalibMaType::Dema),
        4 => Ok(TalibMaType::Tema),
        5 => Ok(TalibMaType::Trima),
        6 => Ok(TalibMaType::Kama),
        7 => Ok(TalibMaType::Mama),
        8 => Ok(TalibMaType::T3),
        _ => Err(IndicatorError::InvalidOption {
            indicator,
            option: "ma_type",
            value,
            reason: "expected an MA type between 0 and 8",
        }),
    }
}
