//! Directional Movement Index.
//!
//! Inputs are `high` and `low` series plus one `period` option. The output is a
//! single `dx` series that starts after the first `period - 1` bars, when the
//! shared directional-movement warmup has enough data to normalize the ratio.
//! `Dx::state` wraps the same typed `(Real, Real) -> Real` stream in a bounded
//! history buffer for incremental consumers.

use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{double_input, expect_option_count, parse_usize_option};
use crate::indicators::shared::{directional_ratio, DirectionalMovementState};
use crate::state::{validate_history_capacity, IndicatorState, RingHistory};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "dx",
    full_name: "Directional Movement Index",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low"],
    option_names: &["period"],
    output_names: &["dx"],
};

/// Typed DX indicator entry point for batch, stream, and state APIs.
#[derive(Debug, Clone, Copy)]
pub struct Dx;

impl Dx {
    /// Build a typed DX state wrapper with a bounded history buffer.
    pub fn state(options: &[Real], history_capacity: usize) -> Result<DxState, IndicatorError> {
        DxState::new(options, history_capacity)
    }
}

impl Indicator for Dx {
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
        let mut output = vec![0.0; output_len];
        let produced = run_dx_batch(high, low, period, &mut output);
        debug_assert_eq!(produced, output_len);
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
        let output_len = high.len().saturating_sub(period.saturating_sub(1));
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_dx_batch(
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
        Ok(Some(Box::new(DxStream::new(options)?)))
    }
}

struct DxStream {
    progress: usize,
    state: DirectionalMovementState,
}

impl DxStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        Ok(Self {
            progress: 0,
            state: DirectionalMovementState::new(parse_period(options)?),
        })
    }

    fn update_one(&mut self, high: Real, low: Real) -> Option<Real> {
        let output = self
            .state
            .feed(high, low)
            .map(|(up, down)| directional_ratio(up, down));
        self.progress += 1;
        output
    }
}

impl IndicatorStream for DxStream {
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
            if let Some(value) = self.update_one(high_value, low_value) {
                output.push(value);
            }
        }

        Ok(vec![output])
    }
}

/// Typed DX state wrapper with bounded history for incremental callers.
pub struct DxState {
    period: usize,
    stream: DxStream,
    history: RingHistory<Real>,
}

impl DxState {
    /// Construct the typed DX state wrapper from validated options.
    pub fn new(options: &[Real], history_capacity: usize) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        validate_history_capacity(METADATA.name, history_capacity)?;
        Ok(Self {
            period,
            stream: DxStream::new(options)?,
            history: RingHistory::new(history_capacity),
        })
    }
}

impl IndicatorState for DxState {
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
        self.stream = DxStream {
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

fn run_dx_batch(high: &[Real], low: &[Real], period: usize, output: &mut [Real]) -> usize {
    if high.len() <= period.saturating_sub(1) {
        return 0;
    }

    // The batch kernel keeps the directional sums in local scalars and updates
    // the ratio in one pass so the stream and batch math stay aligned.
    let mut up = 0.0;
    let mut down = 0.0;

    for index in 1..period {
        let (current_up, current_down) = crate::indicators::shared::directional_movement(
            high[index - 1],
            high[index],
            low[index - 1],
            low[index],
        );
        up += current_up;
        down += current_down;
    }

    output[0] = directional_ratio(up, down);
    let mut out_index = 1usize;
    let per = (period - 1) as Real / period as Real;

    for index in period..high.len() {
        let (current_up, current_down) = crate::indicators::shared::directional_movement(
            high[index - 1],
            high[index],
            low[index - 1],
            low[index],
        );
        up = up.mul_add(per, current_up);
        down = down.mul_add(per, current_down);
        output[out_index] = directional_ratio(up, down);
        out_index += 1;
    }

    out_index
}
