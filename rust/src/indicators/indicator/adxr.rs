use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{double_input, expect_option_count, parse_usize_option};
use crate::indicators::shared::{
    directional_movement, directional_ratio, DirectionalMovementState, WildersAverageState,
};
use std::collections::VecDeque;

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "adxr",
    full_name: "Average Directional Movement Rating",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low"],
    option_names: &["period"],
    output_names: &["adxr"],
};

#[derive(Debug, Clone, Copy)]
pub struct Adxr;

impl Indicator for Adxr {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        Ok((period - 1) * 3)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low) = double_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let mut dm_state = DirectionalMovementState::new(period);
        let mut adx_state = WildersAverageState::new(period);
        let mut history = VecDeque::with_capacity(period.saturating_sub(1));
        let mut output = Vec::new();

        for (&high_value, &low_value) in high.iter().zip(low.iter()) {
            if let Some((up, down)) = dm_state.feed(high_value, low_value) {
                let dx = directional_ratio(up, down);
                if let Some(adx) = adx_state.feed(dx) {
                    if history.len() == period - 1 {
                        output.push(0.5 * (adx + history[0]));
                    }
                    history.push_back(adx);
                    if history.len() > period - 1 {
                        history.pop_front();
                    }
                }
            }
        }

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
        validate_output_slices(&METADATA, outputs, 1)?;
        let lookback = (period - 1) * 3;
        let output_len = high.len().saturating_sub(lookback);
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;

        if output_len == 0 {
            return Ok(0);
        }

        let per = (period - 1) as Real / period as Real;
        let invper = 1.0 / period as Real;

        let mut dmup = 0.0;
        let mut dmdown = 0.0;
        for index in 1..period {
            let (up, down) =
                directional_movement(high[index - 1], high[index], low[index - 1], low[index]);
            dmup += up;
            dmdown += down;
        }

        let mut adx = directional_ratio(dmup, dmdown);
        let history_len = period - 1;
        let mut history = vec![0.0; history_len];
        let mut history_index = 0usize;
        let mut out_index = 0usize;

        for index in period..high.len() {
            let (up, down) =
                directional_movement(high[index - 1], high[index], low[index - 1], low[index]);
            dmup = dmup * per + up;
            dmdown = dmdown * per + down;
            let dx = directional_ratio(dmup, dmdown);

            let relative = index - period;
            if relative < period - 2 {
                adx += dx;
            } else if relative == period - 2 {
                adx += dx;
                history[history_index] = adx * invper;
                history_index += 1;
                if history_index == history_len {
                    history_index = 0;
                }
            } else {
                adx = adx * per + dx;
                if index >= lookback {
                    outputs[0][out_index] = 0.5 * (adx * invper + history[history_index]);
                    out_index += 1;
                }
                history[history_index] = adx * invper;
                history_index += 1;
                if history_index == history_len {
                    history_index = 0;
                }
            }
        }

        Ok(out_index)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(AdxrStream::new(options)?)))
    }
}

struct AdxrStream {
    progress: usize,
    period: usize,
    dm_state: DirectionalMovementState,
    adx_state: WildersAverageState,
    history: VecDeque<Real>,
}

impl AdxrStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        Ok(Self {
            progress: 0,
            period,
            dm_state: DirectionalMovementState::new(period),
            adx_state: WildersAverageState::new(period),
            history: VecDeque::with_capacity(period.saturating_sub(1)),
        })
    }
}

impl IndicatorStream for AdxrStream {
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
            if let Some((up, down)) = self.dm_state.feed(high_value, low_value) {
                let dx = directional_ratio(up, down);
                if let Some(adx) = self.adx_state.feed(dx) {
                    if self.history.len() == self.period - 1 {
                        output.push(0.5 * (adx + self.history[0]));
                    }
                    self.history.push_back(adx);
                    if self.history.len() > self.period - 1 {
                        self.history.pop_front();
                    }
                }
            }
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 2)
}
