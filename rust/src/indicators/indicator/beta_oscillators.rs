//! TA-Lib compatibility oscillator indicators with Tulip-style metadata and validation.
//!
//! The implementations favor direct batch kernels so callers can still use the zero-allocation
//! `run_in_place` path even for compatibility indicators.
use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{
    expect_option_count, parse_usize_option, single_input, triple_input,
};
use crate::indicators::shared::{EmaState, ExtremaKind, MonotonicQueue};
use std::collections::VecDeque;

const POSC_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "posc",
    full_name: "Projection Oscillator",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low", "close"],
    option_names: &["period", "ema_period"],
    output_names: &["posc"],
};

const RVI_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "rvi",
    full_name: "Relative Volatility Index",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["sma_period", "stddev_period"],
    output_names: &["rvi"],
};

const SMI_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "smi",
    full_name: "Stochastic Momentum Index",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low", "close"],
    option_names: &["q_period", "r_period", "s_period"],
    output_names: &["smi"],
};

#[derive(Debug, Clone, Copy)]
pub struct Posc;
#[derive(Debug, Clone, Copy)]
pub struct Rvi;
#[derive(Debug, Clone, Copy)]
pub struct Smi;

impl Indicator for Posc {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &POSC_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(parse_posc_options(options)?.0 - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = PoscStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(PoscStream::new(options)?)))
    }
}

impl Indicator for Rvi {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &RVI_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(parse_rvi_options(options)?.1 - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = RviStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(RviStream::new(options)?)))
    }
}

impl Indicator for Smi {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &SMI_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(parse_smi_options(options)?.0 - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = SmiStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(SmiStream::new(options)?)))
    }
}

struct PoscStream {
    progress: usize,
    regression: SlidingRegressionState,
    highs: VecDeque<Real>,
    lows: VecDeque<Real>,
    ema: EmaState,
}

impl PoscStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let (period, ema_period) = parse_posc_options(options)?;
        Ok(Self {
            progress: 0,
            regression: SlidingRegressionState::new(period),
            highs: VecDeque::with_capacity(period),
            lows: VecDeque::with_capacity(period),
            ema: EmaState::new(2.0 / (ema_period as Real + 1.0)),
        })
    }
}

impl IndicatorStream for PoscStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &POSC_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close) = triple_input(POSC_METADATA.name, inputs)?;
        let mut output = Vec::new();

        for ((&high_value, &low_value), &close_value) in high.iter().zip(low).zip(close) {
            if self.highs.len() == self.regression.period {
                self.highs.pop_front();
                self.lows.pop_front();
            }
            self.highs.push_back(high_value);
            self.lows.push_back(low_value);

            if let Some(values) = self.regression.feed(close_value) {
                let mut the_max = *self.highs.back().expect("high window should exist");
                let mut the_min = *self.lows.back().expect("low window should exist");
                for offset in 1..self.regression.period {
                    let high_candidate = self.highs[self.regression.period - 1 - offset]
                        + offset as Real * values.slope;
                    if high_candidate > the_max {
                        the_max = high_candidate;
                    }

                    let low_candidate = self.lows[self.regression.period - 1 - offset]
                        + offset as Real * values.slope;
                    if low_candidate < the_min {
                        the_min = low_candidate;
                    }
                }

                let osc = (close_value - the_min) / (the_max - the_min) * 100.0;
                output.push(self.ema.feed(osc));
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

struct RviStream {
    progress: usize,
    regression: SlidingRegressionState,
    smooth_mul: Real,
    gains_ema: Real,
    losses_ema: Real,
    initialized: bool,
}

impl RviStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let (sma_period, stddev_period) = parse_rvi_options(options)?;
        Ok(Self {
            progress: 0,
            regression: SlidingRegressionState::new(stddev_period),
            smooth_mul: 2.0 / (sma_period as Real + 1.0),
            gains_ema: 0.0,
            losses_ema: 0.0,
            initialized: false,
        })
    }
}

impl IndicatorStream for RviStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &RVI_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(RVI_METADATA.name, inputs)?;
        let mut output = Vec::new();

        for &sample in input {
            if let Some(values) = self.regression.feed(sample) {
                let higher = sample - values.value_at(self.regression.period as Real);
                let variance_sample = higher * higher / self.regression.period as Real;

                if !self.initialized {
                    if higher > 0.0 {
                        self.gains_ema = variance_sample;
                    } else {
                        self.losses_ema = variance_sample;
                    }
                    self.initialized = true;
                } else if higher > 0.0 {
                    self.gains_ema =
                        (variance_sample - self.gains_ema) * self.smooth_mul + self.gains_ema;
                } else {
                    self.losses_ema =
                        (variance_sample - self.losses_ema) * self.smooth_mul + self.losses_ema;
                }

                let total = self.gains_ema + self.losses_ema;
                output.push(if total == 0.0 {
                    50.0
                } else {
                    self.gains_ema / total * 100.0
                });
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

struct SmiStream {
    q_period: usize,
    progress: usize,
    high_queue: MonotonicQueue,
    low_queue: MonotonicQueue,
    ema_r_num: EmaState,
    ema_s_num: EmaState,
    ema_r_den: EmaState,
    ema_s_den: EmaState,
}

impl SmiStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let (q_period, r_period, s_period) = parse_smi_options(options)?;
        Ok(Self {
            q_period,
            progress: 0,
            high_queue: MonotonicQueue::new(ExtremaKind::Max),
            low_queue: MonotonicQueue::new(ExtremaKind::Min),
            ema_r_num: EmaState::new(2.0 / (r_period as Real + 1.0)),
            ema_s_num: EmaState::new(2.0 / (s_period as Real + 1.0)),
            ema_r_den: EmaState::new(2.0 / (r_period as Real + 1.0)),
            ema_s_den: EmaState::new(2.0 / (s_period as Real + 1.0)),
        })
    }
}

impl IndicatorStream for SmiStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &SMI_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close) = triple_input(SMI_METADATA.name, inputs)?;
        let mut output = Vec::new();

        for ((&high_value, &low_value), &close_value) in high.iter().zip(low).zip(close) {
            self.high_queue.push(self.progress, high_value);
            self.low_queue.push(self.progress, low_value);
            let min_index = self
                .progress
                .saturating_add(1)
                .saturating_sub(self.q_period);
            self.high_queue.evict_before(min_index);
            self.low_queue.evict_before(min_index);

            if self.progress + 1 >= self.q_period {
                let hh = self.high_queue.front_value();
                let ll = self.low_queue.front_value();
                let num = close_value - 0.5 * (hh + ll);
                let den = hh - ll;
                let ema_r_num = self.ema_r_num.feed(num);
                let ema_s_num = self.ema_s_num.feed(ema_r_num);
                let ema_r_den = self.ema_r_den.feed(den);
                let ema_s_den = self.ema_s_den.feed(ema_r_den);
                output.push(100.0 * ema_s_num / (0.5 * ema_s_den));
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

#[derive(Clone)]
struct SlidingRegressionState {
    period: usize,
    x_sum: Real,
    inv_denom: Real,
    values: Vec<Real>,
    cursor: usize,
    len: usize,
    y_sum: Real,
    xy_sum: Real,
}

#[derive(Clone, Copy)]
struct RegressionValues {
    intercept0: Real,
    slope: Real,
}

impl RegressionValues {
    fn value_at(self, x: Real) -> Real {
        self.intercept0 + self.slope * x
    }
}

impl SlidingRegressionState {
    fn new(period: usize) -> Self {
        let x_sum = (period * (period + 1) / 2) as Real;
        let x2_sum = (period * (period + 1) * (2 * period + 1) / 6) as Real;
        let denom = period as Real * x2_sum - x_sum * x_sum;
        Self {
            period,
            x_sum,
            inv_denom: 1.0 / denom,
            values: vec![0.0; period],
            cursor: 0,
            len: 0,
            y_sum: 0.0,
            xy_sum: 0.0,
        }
    }

    fn feed(&mut self, sample: Real) -> Option<RegressionValues> {
        if self.len < self.period {
            self.values[self.len] = sample;
            self.y_sum += sample;
            self.xy_sum += sample * (self.len + 1) as Real;
            self.len += 1;

            if self.len < self.period {
                return None;
            }
        } else {
            let oldest = self.values[self.cursor];
            self.xy_sum = self.xy_sum - self.y_sum + sample * self.period as Real;
            self.y_sum = self.y_sum - oldest + sample;
            self.values[self.cursor] = sample;
            self.cursor = (self.cursor + 1) % self.period;
        }

        let slope = (self.period as Real * self.xy_sum - self.x_sum * self.y_sum) * self.inv_denom;
        let intercept0 = (self.y_sum - slope * self.x_sum) / self.period as Real;
        Some(RegressionValues { intercept0, slope })
    }
}

fn parse_posc_options(options: &[Real]) -> Result<(usize, usize), IndicatorError> {
    expect_option_count(POSC_METADATA.name, options, 2)?;
    let period = parse_usize_option(POSC_METADATA.name, options, 0, "period", 1)?;
    let ema_period = parse_usize_option(POSC_METADATA.name, options, 1, "ema_period", 1)?;
    Ok((period, ema_period))
}

fn parse_rvi_options(options: &[Real]) -> Result<(usize, usize), IndicatorError> {
    expect_option_count(RVI_METADATA.name, options, 2)?;
    let sma_period = parse_usize_option(RVI_METADATA.name, options, 0, "sma_period", 1)?;
    let stddev_period = parse_usize_option(RVI_METADATA.name, options, 1, "stddev_period", 1)?;
    Ok((sma_period, stddev_period))
}

fn parse_smi_options(options: &[Real]) -> Result<(usize, usize, usize), IndicatorError> {
    expect_option_count(SMI_METADATA.name, options, 3)?;
    let q_period = parse_usize_option(SMI_METADATA.name, options, 0, "q_period", 1)?;
    let r_period = parse_usize_option(SMI_METADATA.name, options, 1, "r_period", 1)?;
    let s_period = parse_usize_option(SMI_METADATA.name, options, 2, "s_period", 1)?;
    Ok((q_period, r_period, s_period))
}
