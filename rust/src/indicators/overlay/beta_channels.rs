use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{
    expect_option_count, parse_usize_option, quadruple_input, single_input, triple_input,
};
use crate::indicators::shared::{true_range, EmaState, ExtremaKind, MonotonicQueue, RingSum};
use std::collections::VecDeque;

const ABANDS_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "abands",
    full_name: "Acceleration Bands",
    category: IndicatorCategory::Overlay,
    input_names: &["high", "low", "close"],
    option_names: &["period"],
    output_names: &["lower_band", "upper_band", "middle_point"],
};

const DC_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "dc",
    full_name: "Donchian Channel",
    category: IndicatorCategory::Overlay,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["dc_lower", "dc_upper"],
};

const KC_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "kc",
    full_name: "Keltner Channel",
    category: IndicatorCategory::Overlay,
    input_names: &["high", "low", "close"],
    option_names: &["period", "multiple"],
    output_names: &["kc_lower", "kc_middle", "kc_upper"],
};

const PBANDS_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "pbands",
    full_name: "Projection Bands",
    category: IndicatorCategory::Overlay,
    input_names: &["high", "low", "close"],
    option_names: &["period"],
    output_names: &["pbands_lower", "pbands_upper"],
};

const PC_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "pc",
    full_name: "Price Channel",
    category: IndicatorCategory::Overlay,
    input_names: &["high", "low"],
    option_names: &["period"],
    output_names: &["pc_low", "pc_high"],
};

const VWAP_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "vwap",
    full_name: "Volume Weighted Average Price",
    category: IndicatorCategory::Overlay,
    input_names: &["high", "low", "close", "volume"],
    option_names: &["period"],
    output_names: &["vwap"],
};

#[derive(Debug, Clone, Copy)]
pub struct Abands;
#[derive(Debug, Clone, Copy)]
pub struct Dc;
#[derive(Debug, Clone, Copy)]
pub struct Kc;
#[derive(Debug, Clone, Copy)]
pub struct Pbands;
#[derive(Debug, Clone, Copy)]
pub struct Pc;
#[derive(Debug, Clone, Copy)]
pub struct Vwap;

impl Indicator for Abands {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &ABANDS_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(parse_period(ABANDS_METADATA.name, options)? - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = AbandsStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(AbandsStream::new(options)?)))
    }
}

impl Indicator for Dc {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &DC_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(parse_period(DC_METADATA.name, options)? - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = DcStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(DcStream::new(options)?)))
    }
}

impl Indicator for Kc {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &KC_METADATA
    }

    fn lookback(&self, _options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(0)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = KcStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(KcStream::new(options)?)))
    }
}

impl Indicator for Pbands {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &PBANDS_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(parse_period(PBANDS_METADATA.name, options)? - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = PbandsStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(PbandsStream::new(options)?)))
    }
}

impl Indicator for Pc {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &PC_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(parse_period(PC_METADATA.name, options)? - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = PcStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(PcStream::new(options)?)))
    }
}

impl Indicator for Vwap {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &VWAP_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(parse_period(VWAP_METADATA.name, options)? - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = VwapStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(VwapStream::new(options)?)))
    }
}

struct AbandsStream {
    period: usize,
    progress: usize,
    adjusted_highs: RingSum,
    adjusted_lows: RingSum,
    closes: RingSum,
}

impl AbandsStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(ABANDS_METADATA.name, options)?;
        Ok(Self {
            period,
            progress: 0,
            adjusted_highs: RingSum::new(period),
            adjusted_lows: RingSum::new(period),
            closes: RingSum::new(period),
        })
    }
}

impl IndicatorStream for AbandsStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &ABANDS_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close) = triple_input(ABANDS_METADATA.name, inputs)?;
        let mut lower = Vec::new();
        let mut upper = Vec::new();
        let mut middle = Vec::new();

        for ((&high_value, &low_value), &close_value) in high.iter().zip(low).zip(close) {
            let mult = 4.0 * (high_value - low_value) / (high_value + low_value);
            self.adjusted_highs.push((1.0 + mult) * high_value);
            self.adjusted_lows.push((1.0 - mult) * low_value);
            self.closes.push(close_value);

            if self.adjusted_highs.is_full() {
                let period = self.period as Real;
                lower.push(self.adjusted_lows.sum / period);
                upper.push(self.adjusted_highs.sum / period);
                middle.push(self.closes.sum / period);
            }

            self.progress += 1;
        }

        Ok(vec![lower, upper, middle])
    }
}

struct DcStream {
    period: usize,
    progress: usize,
    max_queue: MonotonicQueue,
    min_queue: MonotonicQueue,
}

impl DcStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        Ok(Self {
            period: parse_period(DC_METADATA.name, options)?,
            progress: 0,
            max_queue: MonotonicQueue::new(ExtremaKind::Max),
            min_queue: MonotonicQueue::new(ExtremaKind::Min),
        })
    }
}

impl IndicatorStream for DcStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &DC_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(DC_METADATA.name, inputs)?;
        let mut lower = Vec::new();
        let mut upper = Vec::new();

        for &sample in input {
            self.max_queue.push(self.progress, sample);
            self.min_queue.push(self.progress, sample);
            let min_index = self.progress.saturating_add(1).saturating_sub(self.period);
            self.max_queue.evict_before(min_index);
            self.min_queue.evict_before(min_index);

            if self.progress + 1 >= self.period {
                lower.push(self.min_queue.front_value());
                upper.push(self.max_queue.front_value());
            }

            self.progress += 1;
        }

        Ok(vec![lower, upper])
    }
}

struct KcStream {
    progress: usize,
    multiplier: Real,
    price_ema: EmaState,
    tr_ema: EmaState,
    previous_close: Option<Real>,
}

impl KcStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let (period, multiplier) = parse_period_multiplier(KC_METADATA.name, options)?;
        Ok(Self {
            progress: 0,
            multiplier,
            price_ema: EmaState::new(2.0 / (period as Real + 1.0)),
            tr_ema: EmaState::new(2.0 / (period as Real + 1.0)),
            previous_close: None,
        })
    }
}

impl IndicatorStream for KcStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &KC_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close) = triple_input(KC_METADATA.name, inputs)?;
        let mut lower = Vec::with_capacity(close.len());
        let mut middle = Vec::with_capacity(close.len());
        let mut upper = Vec::with_capacity(close.len());

        for ((&high_value, &low_value), &close_value) in high.iter().zip(low).zip(close) {
            let price_ema = self.price_ema.feed(close_value);
            let tr_value = match self.previous_close {
                Some(previous_close) => true_range(high_value, low_value, previous_close),
                None => high_value - low_value,
            };
            let tr_ema = self.tr_ema.feed(tr_value);

            lower.push(price_ema - self.multiplier * tr_ema);
            middle.push(price_ema);
            upper.push(price_ema + self.multiplier * tr_ema);

            self.previous_close = Some(close_value);
            self.progress += 1;
        }

        Ok(vec![lower, middle, upper])
    }
}

struct PbandsStream {
    progress: usize,
    state: RegressionSlopeState,
    highs: VecDeque<Real>,
    lows: VecDeque<Real>,
}

impl PbandsStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(PBANDS_METADATA.name, options)?;
        Ok(Self {
            progress: 0,
            state: RegressionSlopeState::new(period),
            highs: VecDeque::with_capacity(period),
            lows: VecDeque::with_capacity(period),
        })
    }
}

impl IndicatorStream for PbandsStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &PBANDS_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close) = triple_input(PBANDS_METADATA.name, inputs)?;
        let mut lower = Vec::new();
        let mut upper = Vec::new();

        for ((&high_value, &low_value), &close_value) in high.iter().zip(low).zip(close) {
            if self.highs.len() == self.state.period {
                self.highs.pop_front();
                self.lows.pop_front();
            }
            self.highs.push_back(high_value);
            self.lows.push_back(low_value);

            if let Some(slope) = self.state.feed(close_value) {
                let mut max_value = *self.highs.back().expect("high window should exist");
                let mut min_value = *self.lows.back().expect("low window should exist");

                for offset in 1..self.state.period {
                    let high_candidate =
                        self.highs[self.state.period - 1 - offset] + offset as Real * slope;
                    if high_candidate > max_value {
                        max_value = high_candidate;
                    }

                    let low_candidate =
                        self.lows[self.state.period - 1 - offset] + offset as Real * slope;
                    if low_candidate < min_value {
                        min_value = low_candidate;
                    }
                }

                lower.push(min_value);
                upper.push(max_value);
            }

            self.progress += 1;
        }

        Ok(vec![lower, upper])
    }
}

struct PcStream {
    period: usize,
    progress: usize,
    high_queue: MonotonicQueue,
    low_queue: MonotonicQueue,
}

impl PcStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        Ok(Self {
            period: parse_period(PC_METADATA.name, options)?,
            progress: 0,
            high_queue: MonotonicQueue::new(ExtremaKind::Max),
            low_queue: MonotonicQueue::new(ExtremaKind::Min),
        })
    }
}

impl IndicatorStream for PcStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &PC_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low) = crate::core::validation::double_input(PC_METADATA.name, inputs)?;
        let mut low_output = Vec::new();
        let mut high_output = Vec::new();

        for (&high_value, &low_value) in high.iter().zip(low) {
            self.high_queue.push(self.progress, high_value);
            self.low_queue.push(self.progress, low_value);
            let min_index = self.progress.saturating_add(1).saturating_sub(self.period);
            self.high_queue.evict_before(min_index);
            self.low_queue.evict_before(min_index);

            if self.progress + 1 >= self.period {
                low_output.push(self.low_queue.front_value());
                high_output.push(self.high_queue.front_value());
            }

            self.progress += 1;
        }

        Ok(vec![low_output, high_output])
    }
}

struct VwapStream {
    progress: usize,
    numerator: RingSum,
    denominator: RingSum,
}

impl VwapStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(VWAP_METADATA.name, options)?;
        Ok(Self {
            progress: 0,
            numerator: RingSum::new(period),
            denominator: RingSum::new(period),
        })
    }
}

impl IndicatorStream for VwapStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &VWAP_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close, volume) = quadruple_input(VWAP_METADATA.name, inputs)?;
        let mut output = Vec::new();

        for (((&high_value, &low_value), &close_value), &volume_value) in
            high.iter().zip(low).zip(close).zip(volume)
        {
            let typical_price = (high_value + low_value + close_value) / 3.0;
            self.numerator.push(typical_price * volume_value);
            self.denominator.push(volume_value);

            if self.numerator.is_full() {
                output.push(self.numerator.sum / self.denominator.sum);
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

#[derive(Debug, Clone)]
struct RegressionSlopeState {
    period: usize,
    x_sum: Real,
    inv_denom: Real,
    values: Vec<Real>,
    cursor: usize,
    len: usize,
    y_sum: Real,
    xy_sum: Real,
}

impl RegressionSlopeState {
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

    fn feed(&mut self, sample: Real) -> Option<Real> {
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

        Some((self.period as Real * self.xy_sum - self.x_sum * self.y_sum) * self.inv_denom)
    }
}

fn parse_period(name: &'static str, options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(name, options, 1)?;
    parse_usize_option(name, options, 0, "period", 1)
}

fn parse_period_multiplier(
    name: &'static str,
    options: &[Real],
) -> Result<(usize, Real), IndicatorError> {
    expect_option_count(name, options, 2)?;
    let period = parse_usize_option(name, options, 0, "period", 1)?;
    let multiple = options[1];
    if !multiple.is_finite() || multiple < 0.0 {
        return Err(IndicatorError::InvalidOption {
            indicator: name,
            option: "multiple",
            value: multiple,
            reason: "expected a finite value >= 0",
        });
    }
    Ok((period, multiple))
}
