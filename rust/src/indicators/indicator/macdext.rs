//! Extended MACD (`macdext`) allows independent moving-average types for fast, slow, and signal legs.
//!
//! It is a TA-Lib compatibility indicator, so the output remains `(macd, signal, histogram)` while
//! option parsing accepts the additional MA-type controls.
use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};
use crate::indicators::indicator::Macd;
use crate::indicators::overlay::talib_ma::{ma_lookback, parse_ma_type, run_ma_batch};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "macdext",
    full_name: "Moving Average Convergence/Divergence Extended",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &[
        "fast_period",
        "fast_ma_type",
        "slow_period",
        "slow_ma_type",
        "signal_period",
        "signal_ma_type",
    ],
    output_names: &["macd", "macd_signal", "macd_histogram"],
};

#[derive(Debug, Clone, Copy)]
pub struct MacdExt;

impl Indicator for MacdExt {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let parsed = parse_options(options)?;
        if uses_plain_ema_macd(&parsed) {
            return Macd.lookback(&[
                parsed.fast_period as Real,
                parsed.slow_period as Real,
                parsed.signal_period as Real,
            ]);
        }
        let fast_lb = ma_lookback(parsed.fast_period, parsed.fast_type)?;
        let slow_lb = ma_lookback(parsed.slow_period, parsed.slow_type)?;
        let signal_lb = ma_lookback(parsed.signal_period, parsed.signal_type)?;
        Ok(fast_lb.max(slow_lb) + signal_lb)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let parsed = parse_options(options)?;
        if uses_plain_ema_macd(&parsed) {
            return Macd.run(
                &[input],
                &[
                    parsed.fast_period as Real,
                    parsed.slow_period as Real,
                    parsed.signal_period as Real,
                ],
            );
        }
        let lookback = self.lookback(options)?;
        let output_len = input.len().saturating_sub(lookback);
        let mut macd = vec![0.0; output_len];
        let mut signal = vec![0.0; output_len];
        let mut hist = vec![0.0; output_len];
        let produced = run_macdext_batch(input, &parsed, &mut macd, &mut signal, &mut hist)?;
        debug_assert_eq!(produced, output_len);
        Ok(vec![macd, signal, hist])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let parsed = parse_options(options)?;
        if uses_plain_ema_macd(&parsed) {
            return Macd.run_in_place(
                &[input],
                &[
                    parsed.fast_period as Real,
                    parsed.slow_period as Real,
                    parsed.signal_period as Real,
                ],
                outputs,
            );
        }
        let fast_lb = ma_lookback(parsed.fast_period, parsed.fast_type)?;
        let slow_lb = ma_lookback(parsed.slow_period, parsed.slow_type)?;
        let signal_lb = ma_lookback(parsed.signal_period, parsed.signal_type)?;
        let output_len = input.len().saturating_sub(fast_lb.max(slow_lb) + signal_lb);
        validate_output_slices(&METADATA, outputs, 3)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;
        ensure_output_len(&METADATA, outputs[1].len(), output_len, 1)?;
        ensure_output_len(&METADATA, outputs[2].len(), output_len, 2)?;
        let (macd_slice, rest) = outputs.split_at_mut(1);
        let (signal_slice, hist_slice) = rest.split_at_mut(1);
        run_macdext_batch(
            input,
            &parsed,
            &mut macd_slice[0][..output_len],
            &mut signal_slice[0][..output_len],
            &mut hist_slice[0][..output_len],
        )
    }
}

#[derive(Clone, Copy)]
struct MacdExtOptions {
    fast_period: usize,
    fast_type: crate::indicators::overlay::talib_ma::TalibMaType,
    slow_period: usize,
    slow_type: crate::indicators::overlay::talib_ma::TalibMaType,
    signal_period: usize,
    signal_type: crate::indicators::overlay::talib_ma::TalibMaType,
}

fn uses_plain_ema_macd(options: &MacdExtOptions) -> bool {
    use crate::indicators::overlay::talib_ma::TalibMaType::Ema;

    options.fast_type == Ema && options.slow_type == Ema && options.signal_type == Ema
}

fn run_macdext_batch(
    input: &[Real],
    options: &MacdExtOptions,
    macd: &mut [Real],
    signal: &mut [Real],
    hist: &mut [Real],
) -> Result<usize, IndicatorError> {
    let fast_lb = ma_lookback(options.fast_period, options.fast_type)?;
    let slow_lb = ma_lookback(options.slow_period, options.slow_type)?;
    let lookback_largest = fast_lb.max(slow_lb);
    let signal_lb = ma_lookback(options.signal_period, options.signal_type)?;
    let total_lb = lookback_largest + signal_lb;
    if input.len() <= total_lb {
        return Ok(0);
    }

    let mut fast = vec![0.0; input.len().saturating_sub(fast_lb)];
    let mut slow = vec![0.0; input.len().saturating_sub(slow_lb)];
    run_ma_batch(input, options.fast_period, options.fast_type, &mut fast)?;
    run_ma_batch(input, options.slow_period, options.slow_type, &mut slow)?;

    let mut diff = vec![0.0; input.len() - lookback_largest];
    for actual_index in lookback_largest..input.len() {
        diff[actual_index - lookback_largest] =
            fast[actual_index - fast_lb] - slow[actual_index - slow_lb];
    }

    let mut signal_values = vec![0.0; diff.len().saturating_sub(signal_lb)];
    run_ma_batch(
        &diff,
        options.signal_period,
        options.signal_type,
        &mut signal_values,
    )?;

    let output_len = input.len() - total_lb;
    for out_index in 0..output_len {
        let macd_value = diff[signal_lb + out_index];
        let signal_value = signal_values[out_index];
        macd[out_index] = macd_value;
        signal[out_index] = signal_value;
        hist[out_index] = macd_value - signal_value;
    }

    Ok(output_len)
}

fn parse_options(options: &[Real]) -> Result<MacdExtOptions, IndicatorError> {
    expect_option_count(METADATA.name, options, 6)?;
    let mut fast_period = parse_usize_option(METADATA.name, options, 0, "fast_period", 2)?;
    let mut fast_type = parse_ma_type(METADATA.name, options[1])?;
    let mut slow_period = parse_usize_option(METADATA.name, options, 2, "slow_period", 2)?;
    let mut slow_type = parse_ma_type(METADATA.name, options[3])?;
    let signal_period = parse_usize_option(METADATA.name, options, 4, "signal_period", 1)?;
    let signal_type = parse_ma_type(METADATA.name, options[5])?;

    if slow_period < fast_period {
        std::mem::swap(&mut fast_period, &mut slow_period);
        std::mem::swap(&mut fast_type, &mut slow_type);
    }

    Ok(MacdExtOptions {
        fast_period,
        fast_type,
        slow_period,
        slow_type,
        signal_period,
        signal_type,
    })
}
