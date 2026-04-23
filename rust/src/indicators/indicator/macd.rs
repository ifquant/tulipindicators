//! Moving Average Convergence/Divergence.
//!
//! `Macd` consumes one `real` series plus `short_period`, `long_period`, and
//! `signal_period` options, and emits `macd`, `macd_signal`, and
//! `macd_histogram`. The first output appears after the long EMA is warm, so
//! the lookback is `long_period - 1`. `MacdFix` keeps the same output shape but
//! fixes the fast/slow periods to 12/26 and only exposes `signal_period`.
//! `Macd::state` wraps the typed stream in a bounded history buffer for
//! incremental callers.

use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};
use crate::indicators::shared::EmaState;
use crate::state::{validate_history_capacity, IndicatorState, RingHistory};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "macd",
    full_name: "Moving Average Convergence/Divergence",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["short_period", "long_period", "signal_period"],
    output_names: &["macd", "macd_signal", "macd_histogram"],
};

const MACDFIX_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "macdfix",
    full_name: "Moving Average Convergence/Divergence Fix 12/26",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["signal_period"],
    output_names: &["macd", "macd_signal", "macd_histogram"],
};

/// Primary MACD indicator entry point for batch, stream, and state APIs.
#[derive(Debug, Clone, Copy)]
pub struct Macd;
/// Fixed-parameter MACD variant with 12/26 EMA periods and the same outputs.
#[derive(Debug, Clone, Copy)]
pub struct MacdFix;

impl Macd {
    /// Build a typed MACD state wrapper with a bounded history buffer.
    pub fn state(options: &[Real], history_capacity: usize) -> Result<MacdState, IndicatorError> {
        MacdState::new(options, history_capacity)
    }
}

impl Indicator for Macd {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let (_, long_period, _) = parse_options(options)?;
        Ok(long_period - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let (short_period, long_period, signal_period) = parse_options(options)?;
        let lookback = long_period - 1;

        let output_len = input.len().saturating_sub(lookback);
        let mut macd = vec![0.0; output_len];
        let mut signal = vec![0.0; output_len];
        let mut hist = vec![0.0; output_len];
        let produced = run_macd_batch(
            input,
            short_period,
            long_period,
            signal_period,
            &mut macd,
            &mut signal,
            &mut hist,
        );
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
        let (short_period, long_period, signal_period) = parse_options(options)?;
        let output_len = input.len().saturating_sub(long_period - 1);
        validate_output_slices(&METADATA, outputs, 3)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;
        ensure_output_len(&METADATA, outputs[1].len(), output_len, 1)?;
        ensure_output_len(&METADATA, outputs[2].len(), output_len, 2)?;
        let (macd_slice, rest) = outputs.split_at_mut(1);
        let (signal_slice, hist_slice) = rest.split_at_mut(1);
        Ok(run_macd_batch(
            input,
            short_period,
            long_period,
            signal_period,
            &mut macd_slice[0][..output_len],
            &mut signal_slice[0][..output_len],
            &mut hist_slice[0][..output_len],
        ))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(MacdStream::new(options)?)))
    }
}

impl Indicator for MacdFix {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &MACDFIX_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let _ = parse_signal_only(options)?;
        Ok(25)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(MACDFIX_METADATA.name, inputs)?;
        let signal_period = parse_signal_only(options)?;
        let output_len = input.len().saturating_sub(25);
        let mut macd = vec![0.0; output_len];
        let mut signal = vec![0.0; output_len];
        let mut hist = vec![0.0; output_len];
        let produced = run_macd_batch(
            input,
            12,
            26,
            signal_period,
            &mut macd,
            &mut signal,
            &mut hist,
        );
        debug_assert_eq!(produced, output_len);
        Ok(vec![macd, signal, hist])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(MACDFIX_METADATA.name, inputs)?;
        let signal_period = parse_signal_only(options)?;
        let output_len = input.len().saturating_sub(25);
        validate_output_slices(&MACDFIX_METADATA, outputs, 3)?;
        ensure_output_len(&MACDFIX_METADATA, outputs[0].len(), output_len, 0)?;
        ensure_output_len(&MACDFIX_METADATA, outputs[1].len(), output_len, 1)?;
        ensure_output_len(&MACDFIX_METADATA, outputs[2].len(), output_len, 2)?;
        let (macd_slice, rest) = outputs.split_at_mut(1);
        let (signal_slice, hist_slice) = rest.split_at_mut(1);
        Ok(run_macd_batch(
            input,
            12,
            26,
            signal_period,
            &mut macd_slice[0][..output_len],
            &mut signal_slice[0][..output_len],
            &mut hist_slice[0][..output_len],
        ))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        let signal_period = parse_signal_only(options)?;
        Ok(Some(Box::new(MacdStream::new_fixed(signal_period))))
    }
}

struct MacdStream {
    metadata: &'static IndicatorMetadata,
    long_period: usize,
    progress: usize,
    short_ema: EmaState,
    long_ema: EmaState,
    signal_multiplier: Real,
    signal_ema: Option<Real>,
}

impl MacdStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let (short_period, long_period, signal_period) = parse_options(options)?;
        let (short_per, long_per) = ema_pair(short_period, long_period);
        Ok(Self {
            metadata: &METADATA,
            long_period,
            progress: 0,
            short_ema: EmaState::new(short_per),
            long_ema: EmaState::new(long_per),
            signal_multiplier: 2.0 / (signal_period as Real + 1.0),
            signal_ema: None,
        })
    }

    fn new_fixed(signal_period: usize) -> Self {
        let (short_per, long_per) = ema_pair(12, 26);
        Self {
            metadata: &MACDFIX_METADATA,
            long_period: 26,
            progress: 0,
            short_ema: EmaState::new(short_per),
            long_ema: EmaState::new(long_per),
            signal_multiplier: 2.0 / (signal_period as Real + 1.0),
            signal_ema: None,
        }
    }

    fn update_one(&mut self, sample: Real) -> Option<(Real, Real, Real)> {
        // The stream keeps the short and long EMA states hot, then seeds the
        // signal line exactly when the long EMA first becomes available.
        let short_value = self.short_ema.feed(sample);
        let long_value = self.long_ema.feed(sample);
        let index = self.progress;
        let mut output = None;

        if index >= 1 {
            let macd_value = short_value - long_value;

            if index == self.long_period - 1 {
                self.signal_ema = Some(macd_value);
            }

            if index >= self.long_period - 1 {
                let signal_value = match self.signal_ema {
                    Some(current) if index > self.long_period - 1 => {
                        (macd_value - current).mul_add(self.signal_multiplier, current)
                    }
                    Some(current) => current,
                    None => macd_value,
                };
                self.signal_ema = Some(signal_value);
                output = Some((macd_value, signal_value, macd_value - signal_value));
            }
        }

        self.progress += 1;
        output
    }
}

impl IndicatorStream for MacdStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        self.metadata
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let mut macd = vec![0.0; input.len()];
        let mut signal = vec![0.0; input.len()];
        let mut hist = vec![0.0; input.len()];
        let mut outputs = [&mut macd[..], &mut signal[..], &mut hist[..]];
        let produced = self.feed_in_place(inputs, &mut outputs)?;
        macd.truncate(produced);
        signal.truncate(produced);
        hist.truncate(produced);
        Ok(vec![macd, signal, hist])
    }

    fn feed_in_place(
        &mut self,
        inputs: &[&[Real]],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        validate_output_slices(&METADATA, outputs, 3)?;
        ensure_output_len(&METADATA, outputs[0].len(), input.len(), 0)?;
        ensure_output_len(&METADATA, outputs[1].len(), input.len(), 1)?;
        ensure_output_len(&METADATA, outputs[2].len(), input.len(), 2)?;
        let mut out_index = 0usize;
        for &sample in input {
            if let Some((macd, signal, hist)) = self.update_one(sample) {
                outputs[0][out_index] = macd;
                outputs[1][out_index] = signal;
                outputs[2][out_index] = hist;
                out_index += 1;
            }
        }

        Ok(out_index)
    }
}

/// Typed MACD state wrapper with bounded history for incremental callers.
pub struct MacdState {
    short_period: usize,
    long_period: usize,
    signal_period: usize,
    stream: MacdStream,
    history: RingHistory<(Real, Real, Real)>,
}

impl MacdState {
    /// Construct the typed MACD state wrapper from validated options.
    pub fn new(options: &[Real], history_capacity: usize) -> Result<Self, IndicatorError> {
        let (short_period, long_period, signal_period) = parse_options(options)?;
        validate_history_capacity(METADATA.name, history_capacity)?;
        Ok(Self {
            short_period,
            long_period,
            signal_period,
            stream: MacdStream::new(options)?,
            history: RingHistory::new(history_capacity),
        })
    }
}

impl IndicatorState for MacdState {
    type Input = Real;
    type Output = (Real, Real, Real);

    fn update(&mut self, input: Self::Input) -> Option<Self::Output> {
        let value = self.stream.update_one(input);
        if let Some(value) = value {
            self.history.push(value);
            Some(value)
        } else {
            None
        }
    }

    fn latest(&self) -> Option<Self::Output> {
        self.history.latest().cloned()
    }

    fn get(&self, index_from_latest: usize) -> Option<Self::Output> {
        self.history.get(index_from_latest).cloned()
    }

    fn len(&self) -> usize {
        self.history.len()
    }

    fn history_capacity(&self) -> usize {
        self.history.capacity()
    }

    fn reset(&mut self) {
        self.stream = MacdStream::new(&[
            self.short_period as Real,
            self.long_period as Real,
            self.signal_period as Real,
        ])
        .expect("macd state reset should reuse validated options");
        self.history.clear();
    }
}

fn parse_options(options: &[Real]) -> Result<(usize, usize, usize), IndicatorError> {
    expect_option_count(METADATA.name, options, 3)?;
    let short_period = parse_usize_option(METADATA.name, options, 0, "short_period", 1)?;
    let long_period = parse_usize_option(METADATA.name, options, 1, "long_period", 2)?;
    let signal_period = parse_usize_option(METADATA.name, options, 2, "signal_period", 1)?;

    if long_period < short_period {
        return Err(IndicatorError::InvalidOption {
            indicator: METADATA.name,
            option: "long_period",
            value: options[1],
            reason: "expected long_period >= short_period",
        });
    }

    Ok((short_period, long_period, signal_period))
}

fn parse_signal_only(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(MACDFIX_METADATA.name, options, 1)?;
    parse_usize_option(MACDFIX_METADATA.name, options, 0, "signal_period", 1)
}

fn ema_pair(short_period: usize, long_period: usize) -> (Real, Real) {
    if short_period == 12 && long_period == 26 {
        (0.15, 0.075)
    } else {
        (
            2.0 / (short_period as Real + 1.0),
            2.0 / (long_period as Real + 1.0),
        )
    }
}

fn run_macd_batch(
    input: &[Real],
    short_period: usize,
    long_period: usize,
    signal_period: usize,
    macd: &mut [Real],
    signal: &mut [Real],
    hist: &mut [Real],
) -> usize {
    let lookback = long_period - 1;
    if input.len() <= lookback {
        return 0;
    }

    // Batch execution mirrors the stream path directly: two EMA recurrences,
    // then the signal EMA and histogram, with no intermediate series materialized.
    let (short_per, long_per) = ema_pair(short_period, long_period);
    let signal_per = 2.0 / (signal_period as Real + 1.0);

    let mut short_ema = input[0];
    let mut long_ema = input[0];
    let mut signal_ema = 0.0;
    let mut out_index = 0usize;

    for (index, sample) in input.iter().enumerate().skip(1) {
        short_ema = (*sample - short_ema).mul_add(short_per, short_ema);
        long_ema = (*sample - long_ema).mul_add(long_per, long_ema);
        let macd_value = short_ema - long_ema;

        if index == long_period - 1 {
            signal_ema = macd_value;
        }

        if index >= long_period - 1 {
            signal_ema = (macd_value - signal_ema).mul_add(signal_per, signal_ema);
            macd[out_index] = macd_value;
            signal[out_index] = signal_ema;
            hist[out_index] = macd_value - signal_ema;
            out_index += 1;
        }
    }

    out_index
}
