use crate::core::types::Real;
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy)]
pub struct EmaState {
    multiplier: Real,
    value: Option<Real>,
}

impl EmaState {
    pub fn new(multiplier: Real) -> Self {
        Self {
            multiplier,
            value: None,
        }
    }

    pub fn feed(&mut self, sample: Real) -> Real {
        let next = match self.value {
            Some(current) => (sample - current) * self.multiplier + current,
            None => sample,
        };
        self.value = Some(next);
        next
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WildersAverageState {
    period: usize,
    progress: usize,
    sum: Real,
    value: Option<Real>,
}

impl WildersAverageState {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            progress: 0,
            sum: 0.0,
            value: None,
        }
    }

    pub fn feed(&mut self, sample: Real) -> Option<Real> {
        if self.progress < self.period {
            self.sum += sample;
            self.progress += 1;

            if self.progress == self.period {
                let value = self.sum / self.period as Real;
                self.value = Some(value);
                return Some(value);
            }

            return None;
        }

        let current = self.value.expect("wilders smoother should be initialized");
        let next = (sample - current) / self.period as Real + current;
        self.value = Some(next);
        self.progress += 1;
        Some(next)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RsiState {
    period: usize,
    progress: usize,
    last_input: Option<Real>,
    smooth_up: Real,
    smooth_down: Real,
}

impl RsiState {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            progress: 0,
            last_input: None,
            smooth_up: 0.0,
            smooth_down: 0.0,
        }
    }

    pub fn feed(&mut self, sample: Real) -> Option<Real> {
        let previous = match self.last_input {
            Some(previous) => previous,
            None => {
                self.last_input = Some(sample);
                self.progress += 1;
                return None;
            }
        };

        let delta = sample - previous;
        let upward = delta.max(0.0);
        let downward = (-delta).max(0.0);
        let output = if self.progress <= self.period {
            self.smooth_up += upward;
            self.smooth_down += downward;

            if self.progress == self.period {
                self.smooth_up /= self.period as Real;
                self.smooth_down /= self.period as Real;
                Some(rsi_value(self.smooth_up, self.smooth_down))
            } else {
                None
            }
        } else {
            let per = 1.0 / self.period as Real;
            self.smooth_up = (upward - self.smooth_up) * per + self.smooth_up;
            self.smooth_down = (downward - self.smooth_down) * per + self.smooth_down;
            Some(rsi_value(self.smooth_up, self.smooth_down))
        };

        self.last_input = Some(sample);
        self.progress += 1;
        output
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DirectionalMovementState {
    period: usize,
    progress: usize,
    up: Real,
    down: Real,
    last_high: Option<Real>,
    last_low: Option<Real>,
}

impl DirectionalMovementState {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            progress: 0,
            up: 0.0,
            down: 0.0,
            last_high: None,
            last_low: None,
        }
    }

    pub fn feed(&mut self, high: Real, low: Real) -> Option<(Real, Real)> {
        let (previous_high, previous_low) = match (self.last_high, self.last_low) {
            (Some(previous_high), Some(previous_low)) => (previous_high, previous_low),
            _ => {
                self.last_high = Some(high);
                self.last_low = Some(low);
                self.progress += 1;

                if self.period == 1 {
                    return Some((0.0, 0.0));
                }

                return None;
            }
        };

        let (current_up, current_down) =
            directional_movement(previous_high, high, previous_low, low);
        let output = if self.progress < self.period {
            self.up += current_up;
            self.down += current_down;

            if self.progress + 1 == self.period {
                Some((self.up, self.down))
            } else {
                None
            }
        } else {
            let per = (self.period - 1) as Real / self.period as Real;
            self.up = self.up * per + current_up;
            self.down = self.down * per + current_down;
            Some((self.up, self.down))
        };

        self.last_high = Some(high);
        self.last_low = Some(low);
        self.progress += 1;
        output
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DirectionalIndexState {
    period: usize,
    progress: usize,
    atr: Real,
    up: Real,
    down: Real,
    last_high: Option<Real>,
    last_low: Option<Real>,
    last_close: Option<Real>,
}

impl DirectionalIndexState {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            progress: 0,
            atr: 0.0,
            up: 0.0,
            down: 0.0,
            last_high: None,
            last_low: None,
            last_close: None,
        }
    }

    pub fn feed(&mut self, high: Real, low: Real, close: Real) -> Option<(Real, Real, Real)> {
        let (previous_high, previous_low, previous_close) =
            match (self.last_high, self.last_low, self.last_close) {
                (Some(previous_high), Some(previous_low), Some(previous_close)) => {
                    (previous_high, previous_low, previous_close)
                }
                _ => {
                    self.last_high = Some(high);
                    self.last_low = Some(low);
                    self.last_close = Some(close);
                    self.progress += 1;

                    if self.period == 1 {
                        return Some((0.0, 0.0, 0.0));
                    }

                    return None;
                }
            };

        let tr = true_range(high, low, previous_close);
        let (current_up, current_down) =
            directional_movement(previous_high, high, previous_low, low);
        let output = if self.progress < self.period {
            self.atr += tr;
            self.up += current_up;
            self.down += current_down;

            if self.progress + 1 == self.period {
                Some((self.up, self.down, self.atr))
            } else {
                None
            }
        } else {
            let per = (self.period - 1) as Real / self.period as Real;
            self.atr = self.atr * per + tr;
            self.up = self.up * per + current_up;
            self.down = self.down * per + current_down;
            Some((self.up, self.down, self.atr))
        };

        self.last_high = Some(high);
        self.last_low = Some(low);
        self.last_close = Some(close);
        self.progress += 1;
        output
    }
}

pub enum ExtremaKind {
    Max,
    Min,
}

pub struct MonotonicQueue {
    kind: ExtremaKind,
    values: VecDeque<(usize, Real)>,
}

impl MonotonicQueue {
    pub fn new(kind: ExtremaKind) -> Self {
        Self {
            kind,
            values: VecDeque::new(),
        }
    }

    pub fn push(&mut self, index: usize, value: Real) {
        while let Some((_, tail)) = self.values.back().copied() {
            let should_pop = match self.kind {
                ExtremaKind::Max => value >= tail,
                ExtremaKind::Min => value <= tail,
            };
            if should_pop {
                self.values.pop_back();
            } else {
                break;
            }
        }
        self.values.push_back((index, value));
    }

    pub fn evict_before(&mut self, min_index: usize) {
        while let Some((index, _)) = self.values.front().copied() {
            if index < min_index {
                self.values.pop_front();
            } else {
                break;
            }
        }
    }

    pub fn front_value(&self) -> Real {
        self.values
            .front()
            .map(|(_, value)| *value)
            .expect("monotonic queue should not be empty")
    }
}

#[derive(Debug, Clone)]
pub struct RingSum {
    values: Vec<Real>,
    capacity: usize,
    index: usize,
    len: usize,
    pub sum: Real,
}

impl RingSum {
    pub fn new(capacity: usize) -> Self {
        Self {
            values: vec![0.0; capacity],
            capacity,
            index: 0,
            len: 0,
            sum: 0.0,
        }
    }

    pub fn push(&mut self, value: Real) {
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

#[derive(Debug, Clone, Copy)]
pub struct RollingStats {
    pub variance: Real,
}

#[derive(Debug, Clone)]
pub struct RollingStatsState {
    period: usize,
    values: Vec<Real>,
    index: usize,
    len: usize,
    sum: Real,
    sum2: Real,
}

impl RollingStatsState {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            values: vec![0.0; period],
            index: 0,
            len: 0,
            sum: 0.0,
            sum2: 0.0,
        }
    }

    pub fn feed(&mut self, sample: Real) -> Option<RollingStats> {
        if self.len < self.period {
            self.values[self.index] = sample;
            self.sum += sample;
            self.sum2 += sample * sample;
            self.len += 1;
            self.index = (self.index + 1) % self.period;

            if self.len < self.period {
                return None;
            }
        } else {
            let old = self.values[self.index];
            self.sum -= old;
            self.sum2 -= old * old;

            self.values[self.index] = sample;
            self.sum += sample;
            self.sum2 += sample * sample;
            self.index = (self.index + 1) % self.period;
        }

        let mean = self.sum / self.period as Real;
        let variance = self.sum2 / self.period as Real - mean * mean;
        Some(RollingStats { variance })
    }
}

#[derive(Debug, Clone)]
pub struct WmaState {
    period: usize,
    weight_total: Real,
    values: Vec<Real>,
    cursor: usize,
    len: usize,
    sum: Real,
    weighted_sum: Real,
}

impl WmaState {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            weight_total: (period * (period + 1) / 2) as Real,
            values: vec![0.0; period],
            cursor: 0,
            len: 0,
            sum: 0.0,
            weighted_sum: 0.0,
        }
    }

    pub fn feed(&mut self, sample: Real) -> Option<Real> {
        if self.len < self.period {
            self.values[self.len] = sample;
            self.sum += sample;
            self.weighted_sum += sample * (self.len + 1) as Real;
            self.len += 1;

            if self.len == self.period {
                return Some(self.weighted_sum / self.weight_total);
            }

            return None;
        }

        let oldest = self.values[self.cursor];
        self.weighted_sum = self.weighted_sum - self.sum + sample * self.period as Real;
        self.sum = self.sum - oldest + sample;
        self.values[self.cursor] = sample;
        self.cursor = (self.cursor + 1) % self.period;
        Some(self.weighted_sum / self.weight_total)
    }
}

pub fn true_range(high: Real, low: Real, previous_close: Real) -> Real {
    let ych = (high - previous_close).abs();
    let ycl = (low - previous_close).abs();
    let mut value = high - low;
    if ych > value {
        value = ych;
    }
    if ycl > value {
        value = ycl;
    }
    value
}

pub fn directional_movement(
    previous_high: Real,
    high: Real,
    previous_low: Real,
    low: Real,
) -> (Real, Real) {
    let mut up = high - previous_high;
    let mut down = previous_low - low;

    if up < 0.0 {
        up = 0.0;
    } else if up > down {
        down = 0.0;
    }

    if down < 0.0 {
        down = 0.0;
    } else if down > up {
        up = 0.0;
    }

    (up, down)
}

pub fn directional_ratio(up: Real, down: Real) -> Real {
    let diff = (up - down).abs();
    let sum = up + down;
    diff / sum * 100.0
}

pub fn rsi_value(smooth_up: Real, smooth_down: Real) -> Real {
    let total = smooth_up + smooth_down;
    if total == 0.0 {
        0.0
    } else {
        100.0 * (smooth_up / total)
    }
}
