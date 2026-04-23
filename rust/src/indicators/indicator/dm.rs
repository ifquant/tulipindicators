//! Directional Movement.
//!
//! Inputs are `high` and `low` series plus one `period` option. The indicator
//! returns a pair of smoothed directional-movement series, `plus_dm` and
//! `minus_dm`, with the first result arriving after the initial `period - 1`
//! bar warmup. `Dm::state` wraps the same logic in a typed `(Real, Real) ->
//! (Real, Real)` history buffer for incremental consumers.

use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{double_input, expect_option_count, parse_usize_option};
use crate::indicators::shared::DirectionalMovementState;
use crate::state::{validate_history_capacity, IndicatorState, RingHistory};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "dm",
    full_name: "Directional Movement",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low"],
    option_names: &["period"],
    output_names: &["plus_dm", "minus_dm"],
};

/// Typed DM indicator entry point for batch, stream, and state APIs.
#[derive(Debug, Clone, Copy)]
pub struct Dm;

impl Dm {
    /// Build a typed DM state wrapper with a bounded history buffer.
    pub fn state(options: &[Real], history_capacity: usize) -> Result<DmState, IndicatorError> {
        DmState::new(options, history_capacity)
    }
}

impl Indicator for Dm {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        Ok(period.saturating_sub(1))
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low) = double_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let output_len = high.len().saturating_sub(period.saturating_sub(1));
        if output_len == 0 {
            return Ok(vec![Vec::new(), Vec::new()]);
        }

        let mut plus = vec![0.0; output_len];
        let mut minus = vec![0.0; output_len];
        let written = run_dm_batch(high, low, period, &mut plus, &mut minus);
        debug_assert_eq!(written, output_len);
        Ok(vec![plus, minus])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        if inputs.len() == 2 && outputs.len() == 2 {
            let high = inputs[0];
            let low = inputs[1];
            if high.len() == low.len() {
                if let Ok(period) = parse_period(options) {
                    let output_len = high.len().saturating_sub(period.saturating_sub(1));
                    if outputs[0].len() >= output_len && outputs[1].len() >= output_len {
                        let (plus_outputs, minus_outputs) = outputs.split_at_mut(1);
                        return Ok(run_dm_batch(
                            high,
                            low,
                            period,
                            &mut plus_outputs[0][..output_len],
                            &mut minus_outputs[0][..output_len],
                        ));
                    }
                }
            }
        }

        let (high, low) = double_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let output_len = high.len().saturating_sub(period.saturating_sub(1));
        validate_output_slices(&METADATA, outputs, 2)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;
        ensure_output_len(&METADATA, outputs[1].len(), output_len, 1)?;
        let (plus_outputs, minus_outputs) = outputs.split_at_mut(1);
        Ok(run_dm_batch(
            high,
            low,
            period,
            &mut plus_outputs[0][..output_len],
            &mut minus_outputs[0][..output_len],
        ))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(DmStream::new(options)?)))
    }
}

fn run_dm_batch(
    high: &[Real],
    low: &[Real],
    period: usize,
    plus: &mut [Real],
    minus: &mut [Real],
) -> usize {
    if high.len() < period {
        return 0;
    }

    // Batch execution keeps the directional sums hot and writes both outputs
    // together so the paired series stay aligned without extra buffering.
    let per = (period.saturating_sub(1)) as Real / period as Real;
    let mut dmup = 0.0;
    let mut dmdown = 0.0;

    for index in 1..period {
        let (dp, dm) =
            directional_movement(high[index - 1], high[index], low[index - 1], low[index]);
        dmup += dp;
        dmdown += dm;
    }

    plus[0] = dmup;
    minus[0] = dmdown;

    let previous_high = &high[period - 1..high.len() - 1];
    let current_high = &high[period..];
    let previous_low = &low[period - 1..low.len() - 1];
    let current_low = &low[period..];

    for ((((&previous_high, &high_value), &previous_low), &low_value), (plus_out, minus_out)) in
        previous_high
            .iter()
            .zip(current_high.iter())
            .zip(previous_low.iter())
            .zip(current_low.iter())
            .zip(plus[1..].iter_mut().zip(minus[1..].iter_mut()))
    {
        let (dp, dm) = directional_movement(previous_high, high_value, previous_low, low_value);
        dmup = dmup.mul_add(per, dp);
        dmdown = dmdown.mul_add(per, dm);
        *plus_out = dmup;
        *minus_out = dmdown;
    }

    plus.len()
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

struct DmStream {
    progress: usize,
    state: DirectionalMovementState,
}

impl DmStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        Ok(Self {
            progress: 0,
            state: DirectionalMovementState::new(parse_period(options)?),
        })
    }

    fn update_one(&mut self, high: Real, low: Real) -> Option<(Real, Real)> {
        let output = self.state.feed(high, low);
        self.progress += 1;
        output
    }
}

impl IndicatorStream for DmStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low) = double_input(METADATA.name, inputs)?;
        let mut plus = Vec::new();
        let mut minus = Vec::new();

        for (&high_value, &low_value) in high.iter().zip(low.iter()) {
            if let Some((up, down)) = self.update_one(high_value, low_value) {
                plus.push(up);
                minus.push(down);
            }
        }

        Ok(vec![plus, minus])
    }
}

/// Typed DM state wrapper with bounded history for incremental callers.
pub struct DmState {
    period: usize,
    stream: DmStream,
    history: RingHistory<(Real, Real)>,
}

impl DmState {
    /// Construct the typed DM state wrapper from validated options.
    pub fn new(options: &[Real], history_capacity: usize) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        validate_history_capacity(METADATA.name, history_capacity)?;
        Ok(Self {
            period,
            stream: DmStream::new(options)?,
            history: RingHistory::new(history_capacity),
        })
    }
}

impl IndicatorState for DmState {
    type Input = (Real, Real);
    type Output = (Real, Real);

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
        self.stream = DmStream {
            progress: 0,
            state: DirectionalMovementState::new(self.period),
        };
        self.history.clear();
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 1)
}
