use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, triple_input};

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
        let mut stoch = Vec::with_capacity(high.len().saturating_sub(lookback));
        let mut stoch_ma = Vec::with_capacity(high.len().saturating_sub(lookback));

        if high.len() <= lookback {
            return Ok(vec![stoch, stoch_ma]);
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
                    stoch.push(k_value);
                    stoch_ma.push(d_sum.sum * dper);
                }
            }
        }

        Ok(vec![stoch, stoch_ma])
    }
}

#[derive(Debug, Clone)]
struct RingSum {
    values: Vec<Real>,
    capacity: usize,
    index: usize,
    len: usize,
    sum: Real,
}

impl RingSum {
    fn new(capacity: usize) -> Self {
        Self {
            values: vec![0.0; capacity],
            capacity,
            index: 0,
            len: 0,
            sum: 0.0,
        }
    }

    fn push(&mut self, value: Real) {
        if self.len < self.capacity {
            self.values[self.index] = value;
            self.sum += value;
            self.len += 1;
        } else {
            self.sum -= self.values[self.index];
            self.values[self.index] = value;
            self.sum += value;
        }

        self.index += 1;
        if self.index == self.capacity {
            self.index = 0;
        }
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
