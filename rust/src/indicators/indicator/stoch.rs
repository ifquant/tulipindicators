use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, triple_input};
use crate::indicators::shared::{ExtremaKind, MonotonicQueue, RingSum};
use crate::state::{IndicatorState, RingHistory};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "stoch",
    full_name: "Stochastic Oscillator",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low", "close"],
    option_names: &["k_period", "k_slowing_period", "d_period"],
    output_names: &["stoch_k", "stoch_d"],
};

#[derive(Debug, Clone, Copy)]
pub struct Stoch;

impl Stoch {
    pub fn state(options: &[Real], history_capacity: usize) -> Result<StochState, IndicatorError> {
        StochState::new(options, history_capacity)
    }
}

impl Indicator for Stoch {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let (k_period, k_slow, d_period) = parse_options(options)?;
        Ok(k_period + k_slow + d_period - 3)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close) = triple_input(METADATA.name, inputs)?;
        let (k_period, k_slow, d_period) = parse_options(options)?;
        let lookback = k_period + k_slow + d_period - 3;
        let output_len = high.len().saturating_sub(lookback);
        let mut stoch = vec![0.0; output_len];
        let mut stoch_ma = vec![0.0; output_len];

        let produced = run_stoch_batch(
            high,
            low,
            close,
            (k_period, k_slow, d_period),
            (&mut stoch, &mut stoch_ma),
        );
        debug_assert_eq!(produced, output_len);

        Ok(vec![stoch, stoch_ma])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let (high, low, close) = triple_input(METADATA.name, inputs)?;
        let (k_period, k_slow, d_period) = parse_options(options)?;
        let output_len = high.len().saturating_sub(k_period + k_slow + d_period - 3);
        validate_output_slices(&METADATA, outputs, 2)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;
        ensure_output_len(&METADATA, outputs[1].len(), output_len, 1)?;
        let (first, second) = outputs.split_at_mut(1);
        Ok(run_stoch_batch(
            high,
            low,
            close,
            (k_period, k_slow, d_period),
            (&mut first[0][..output_len], &mut second[0][..output_len]),
        ))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(StochStream::new(options)?)))
    }
}

fn run_stoch_batch(
    high: &[Real],
    low: &[Real],
    close: &[Real],
    periods: (usize, usize, usize),
    outputs: (&mut [Real], &mut [Real]),
) -> usize {
    let (k_period, k_slow, d_period) = periods;
    let (stoch, stoch_ma) = outputs;
    let lookback = k_period + k_slow + d_period - 3;
    if high.len() <= lookback {
        return 0;
    }

    let kper = 1.0 / k_slow as Real;
    let dper = 1.0 / d_period as Real;

    let mut trail = 0usize;
    let mut maxi = 0usize;
    let mut mini = 0usize;
    let mut max = high[0];
    let mut min = low[0];

    let mut k_sum = RingSum::new(k_slow);
    let mut d_sum = RingSum::new(d_period);
    let mut out_index = 0usize;

    for index in 0..high.len() {
        if index >= k_period {
            trail += 1;
        }

        let high_bar = high[index];
        if maxi < trail {
            let (next_index, next_value) = scan_max(high, trail, index);
            maxi = next_index;
            max = next_value;
        } else if high_bar >= max {
            maxi = index;
            max = high_bar;
        }

        let low_bar = low[index];
        if mini < trail {
            let (next_index, next_value) = scan_min(low, trail, index);
            mini = next_index;
            min = next_value;
        } else if low_bar <= min {
            mini = index;
            min = low_bar;
        }

        let kdiff = max - min;
        let kfast = if kdiff == 0.0 {
            0.0
        } else {
            100.0 * ((close[index] - min) / kdiff)
        };
        k_sum.push(kfast);

        if index >= k_period - 1 + k_slow - 1 {
            let k_value = k_sum.sum * kper;
            d_sum.push(k_value);

            if index >= lookback {
                stoch[out_index] = k_value;
                stoch_ma[out_index] = d_sum.sum * dper;
                out_index += 1;
            }
        }
    }

    out_index
}

struct StochStream {
    k_period: usize,
    k_slow: usize,
    d_period: usize,
    progress: usize,
    max_queue: MonotonicQueue,
    min_queue: MonotonicQueue,
    k_sum: RingSum,
    d_sum: RingSum,
}

impl StochStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let (k_period, k_slow, d_period) = parse_options(options)?;
        Ok(Self {
            k_period,
            k_slow,
            d_period,
            progress: 0,
            max_queue: MonotonicQueue::new(ExtremaKind::Max),
            min_queue: MonotonicQueue::new(ExtremaKind::Min),
            k_sum: RingSum::new(k_slow),
            d_sum: RingSum::new(d_period),
        })
    }

    fn update_one(&mut self, high: Real, low: Real, close: Real) -> Option<(Real, Real)> {
        let index = self.progress;
        self.max_queue.push(index, high);
        self.min_queue.push(index, low);

        let window_start = index.saturating_sub(self.k_period.saturating_sub(1));
        self.max_queue.evict_before(window_start);
        self.min_queue.evict_before(window_start);

        let max = self.max_queue.front_value();
        let min = self.min_queue.front_value();

        let kdiff = max - min;
        let kfast = if kdiff == 0.0 {
            0.0
        } else {
            100.0 * ((close - min) / kdiff)
        };
        self.k_sum.push(kfast);

        let output = if index >= self.k_period - 1 + self.k_slow - 1 {
            let k_value = self.k_sum.sum / self.k_slow as Real;
            self.d_sum.push(k_value);

            if index >= self.k_period + self.k_slow + self.d_period - 3 {
                Some((k_value, self.d_sum.sum / self.d_period as Real))
            } else {
                None
            }
        } else {
            None
        };

        self.progress += 1;
        output
    }
}

impl IndicatorStream for StochStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close) = triple_input(METADATA.name, inputs)?;
        let mut stoch = Vec::new();
        let mut stoch_ma = Vec::new();
        for ((&high, &low), &close) in high.iter().zip(low.iter()).zip(close.iter()) {
            if let Some((k, d)) = self.update_one(high, low, close) {
                stoch.push(k);
                stoch_ma.push(d);
            }
        }

        Ok(vec![stoch, stoch_ma])
    }
}

pub struct StochState {
    periods: (usize, usize, usize),
    stream: StochStream,
    history: RingHistory<(Real, Real)>,
}

impl StochState {
    pub fn new(options: &[Real], history_capacity: usize) -> Result<Self, IndicatorError> {
        let periods = parse_options(options)?;
        Ok(Self {
            periods,
            stream: StochStream::new(options)?,
            history: RingHistory::new(history_capacity),
        })
    }
}

impl IndicatorState for StochState {
    type Input = (Real, Real, Real);
    type Output = (Real, Real);

    fn seed(&mut self, input: &[Self::Input]) -> Result<usize, IndicatorError> {
        let mut produced = 0usize;
        for &(high, low, close) in input {
            if self.update((high, low, close)).is_some() {
                produced += 1;
            }
        }
        Ok(produced)
    }

    fn update(&mut self, input: Self::Input) -> Option<Self::Output> {
        let value = self.stream.update_one(input.0, input.1, input.2);
        if let Some(value) = value {
            self.history.push(value);
            Some(value)
        } else {
            None
        }
    }

    fn latest(&self) -> Option<Self::Output> {
        self.history.latest()
    }

    fn get(&self, index_from_latest: usize) -> Option<Self::Output> {
        self.history.get(index_from_latest)
    }

    fn len(&self) -> usize {
        self.history.len()
    }

    fn history_capacity(&self) -> usize {
        self.history.capacity()
    }

    fn reset(&mut self) {
        self.stream = StochStream {
            k_period: self.periods.0,
            k_slow: self.periods.1,
            d_period: self.periods.2,
            progress: 0,
            max_queue: MonotonicQueue::new(ExtremaKind::Max),
            min_queue: MonotonicQueue::new(ExtremaKind::Min),
            k_sum: RingSum::new(self.periods.1),
            d_sum: RingSum::new(self.periods.2),
        };
        self.history.clear();
    }
}

fn scan_max(values: &[Real], start: usize, end: usize) -> (usize, Real) {
    let mut max_index = start;
    let mut max_value = values[start];

    for (index, value) in values.iter().enumerate().take(end + 1).skip(start + 1) {
        if *value >= max_value {
            max_value = *value;
            max_index = index;
        }
    }

    (max_index, max_value)
}

fn scan_min(values: &[Real], start: usize, end: usize) -> (usize, Real) {
    let mut min_index = start;
    let mut min_value = values[start];

    for (index, value) in values.iter().enumerate().take(end + 1).skip(start + 1) {
        if *value <= min_value {
            min_value = *value;
            min_index = index;
        }
    }

    (min_index, min_value)
}

fn parse_options(options: &[Real]) -> Result<(usize, usize, usize), IndicatorError> {
    expect_option_count(METADATA.name, options, 3)?;
    let k_period = parse_usize_option(METADATA.name, options, 0, "k_period", 1)?;
    let k_slow = parse_usize_option(METADATA.name, options, 1, "k_slowing_period", 1)?;
    let d_period = parse_usize_option(METADATA.name, options, 2, "d_period", 1)?;
    Ok((k_period, k_slow, d_period))
}
