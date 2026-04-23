//! Average Directional Movement Index.
//!
//! Inputs are `high` and `low` series plus one `period` option. The output is a
//! single `adx` series that only starts after the initial DX warmup and the
//! additional averaging window, so the lookback is `(period - 1) * 2`. The
//! typed `Adx::state` wrapper stores that same `(Real, Real) -> Real` stream in
//! a bounded history buffer.

use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{double_input, expect_option_count, parse_usize_option};
use crate::indicators::shared::{directional_ratio, DirectionalMovementState, WildersAverageState};
use crate::state::{validate_history_capacity, IndicatorState, RingHistory};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "adx",
    full_name: "Average Directional Movement Index",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low"],
    option_names: &["period"],
    output_names: &["adx"],
};

/// Typed ADX indicator entry point for batch, stream, and state APIs.
#[derive(Debug, Clone, Copy)]
pub struct Adx;

impl Adx {
    /// Build a typed ADX state wrapper with a bounded history buffer.
    pub fn state(options: &[Real], history_capacity: usize) -> Result<AdxState, IndicatorError> {
        AdxState::new(options, history_capacity)
    }
}

impl Indicator for Adx {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        Ok((period - 1) * 2)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low) = double_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let output_len = high.len().saturating_sub((period - 1) * 2);
        if output_len == 0 {
            return Ok(vec![Vec::new()]);
        }

        let mut output = vec![0.0; output_len];
        let written = run_adx_batch(high, low, period, &mut output);
        debug_assert_eq!(written, output_len);
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let (high, low) = double_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let output_len = high.len().saturating_sub((period - 1) * 2);
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_adx_batch(
            high,
            low,
            period,
            &mut outputs[0][..output_len],
        ))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(AdxStream::new(options)?)))
    }
}

fn run_adx_batch(high: &[Real], low: &[Real], period: usize, output: &mut [Real]) -> usize {
    if high.len() <= (period - 1) * 2 {
        return 0;
    }

    // The batch kernel mirrors the stream path: smooth DM, derive DX, then
    // seed and update the Wilder average without allocating intermediate series.
    let per = (period - 1) as Real / period as Real;
    let invper = 1.0 / period as Real;
    let mut dmup = 0.0;
    let mut dmdown = 0.0;

    for index in 1..period {
        let (dp, dm) =
            directional_movement(high[index - 1], high[index], low[index - 1], low[index]);
        dmup += dp;
        dmdown += dm;
    }

    let mut adx = directional_ratio(dmup, dmdown);
    let mut out_index = 0usize;

    for index in period..high.len() {
        let (dp, dm) =
            directional_movement(high[index - 1], high[index], low[index - 1], low[index]);
        dmup = dmup * per + dp;
        dmdown = dmdown * per + dm;
        let dx = directional_ratio(dmup, dmdown);

        if index - period < period - 2 {
            adx += dx;
        } else if index - period == period - 2 {
            adx += dx;
            output[out_index] = adx * invper;
            out_index += 1;
        } else {
            adx = adx * per + dx;
            output[out_index] = adx * invper;
            out_index += 1;
        }
    }

    out_index
}

fn directional_movement(
    previous_high: Real,
    high: Real,
    previous_low: Real,
    low: Real,
) -> (Real, Real) {
    let up = high - previous_high;
    let down = previous_low - low;
    if up > down && up > 0.0 {
        (up, 0.0)
    } else if down > up && down > 0.0 {
        (0.0, down)
    } else {
        (0.0, 0.0)
    }
}

struct AdxStream {
    progress: usize,
    dm_state: DirectionalMovementState,
    adx_state: WildersAverageState,
}

impl AdxStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        Ok(Self {
            progress: 0,
            dm_state: DirectionalMovementState::new(period),
            adx_state: WildersAverageState::new(period),
        })
    }

    fn update_one(&mut self, high: Real, low: Real) -> Option<Real> {
        let output = self.dm_state.feed(high, low).and_then(|(up, down)| {
            let dx = directional_ratio(up, down);
            self.adx_state.feed(dx)
        });
        self.progress += 1;
        output
    }
}

impl IndicatorStream for AdxStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low) = double_input(METADATA.name, inputs)?;
        let mut output = Vec::new();

        for (&high_value, &low_value) in high.iter().zip(low.iter()) {
            if let Some(adx) = self.update_one(high_value, low_value) {
                output.push(adx);
            }
        }

        Ok(vec![output])
    }
}

/// Typed ADX state wrapper with bounded history for incremental callers.
pub struct AdxState {
    period: usize,
    stream: AdxStream,
    history: RingHistory<Real>,
}

impl AdxState {
    /// Construct the typed ADX state wrapper from validated options.
    pub fn new(options: &[Real], history_capacity: usize) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        validate_history_capacity(METADATA.name, history_capacity)?;
        Ok(Self {
            period,
            stream: AdxStream::new(options)?,
            history: RingHistory::new(history_capacity),
        })
    }
}

impl IndicatorState for AdxState {
    type Input = (Real, Real);
    type Output = Real;

    fn update(&mut self, input: Self::Input) -> Option<Self::Output> {
        let output = self.stream.update_one(input.0, input.1);
        if let Some(value) = output {
            self.history.push(value);
        }
        output
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
        self.stream = AdxStream {
            progress: 0,
            dm_state: DirectionalMovementState::new(self.period),
            adx_state: WildersAverageState::new(self.period),
        };
        self.history.clear();
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 2)
}
