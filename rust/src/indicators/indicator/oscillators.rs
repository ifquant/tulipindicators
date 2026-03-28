use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{
    double_input, expect_option_count, parse_usize_option, single_input, triple_input,
};
use crate::indicators::shared::{ExtremaKind, MonotonicQueue, RingSum};
use std::collections::VecDeque;

#[allow(clippy::approx_constant)]
const LEGACY_PI: Real = 3.1415926;

const CCI_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "cci",
    full_name: "Commodity Channel Index",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low", "close"],
    option_names: &["period"],
    output_names: &["cci"],
};

const CMO_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "cmo",
    full_name: "Chande Momentum Oscillator",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["cmo"],
};

const CVI_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "cvi",
    full_name: "Chaikins Volatility",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low"],
    option_names: &["period"],
    output_names: &["cvi"],
};

const FISHER_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "fisher",
    full_name: "Fisher Transform",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low"],
    option_names: &["period"],
    output_names: &["fisher", "signal"],
};

const MD_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "md",
    full_name: "Mean Deviation",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["md"],
};

const MSW_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "msw",
    full_name: "Mesa Sine Wave",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["sine", "lead"],
};

const QSTICK_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "qstick",
    full_name: "Qstick",
    category: IndicatorCategory::Indicator,
    input_names: &["open", "close"],
    option_names: &["period"],
    output_names: &["qstick"],
};

#[derive(Debug, Clone, Copy)]
pub struct Cci;
#[derive(Debug, Clone, Copy)]
pub struct Cmo;
#[derive(Debug, Clone, Copy)]
pub struct Cvi;
#[derive(Debug, Clone, Copy)]
pub struct Fisher;
#[derive(Debug, Clone, Copy)]
pub struct Md;
#[derive(Debug, Clone, Copy)]
pub struct Msw;
#[derive(Debug, Clone, Copy)]
pub struct Qstick;

impl Indicator for Cci {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &CCI_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(CCI_METADATA.name, options)?;
        Ok((period - 1) * 2)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = CciStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(CciStream::new(options)?)))
    }
}

impl Indicator for Cmo {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &CMO_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        parse_period(CMO_METADATA.name, options)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = CmoStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(CmoStream::new(options)?)))
    }
}

impl Indicator for Cvi {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &CVI_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(CVI_METADATA.name, options)?;
        Ok(period * 2 - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = CviStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(CviStream::new(options)?)))
    }
}

impl Indicator for Fisher {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &FISHER_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(FISHER_METADATA.name, options)?;
        Ok(period - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = FisherStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(FisherStream::new(options)?)))
    }
}

impl Indicator for Md {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &MD_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(MD_METADATA.name, options)?;
        Ok(period - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = MdStream::new(options)?;
        stream.feed(inputs)
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(MD_METADATA.name, inputs)?;
        let period = parse_period(MD_METADATA.name, options)?;
        validate_output_slices(&MD_METADATA, outputs, 1)?;
        let output_len = input.len().saturating_sub(period - 1);
        ensure_output_len(&MD_METADATA, outputs[0].len(), output_len, 0)?;

        if output_len == 0 {
            return Ok(0);
        }

        let mut sum = 0.0;
        for value in input.iter().take(period) {
            sum += *value;
        }

        outputs[0][0] = mean_deviation_window(&input[..period], sum, period);

        for index in period..input.len() {
            sum += input[index];
            sum -= input[index - period];
            let output_index = index - period + 1;
            outputs[0][output_index] =
                mean_deviation_window(&input[output_index..=index], sum, period);
        }

        Ok(output_len)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(MdStream::new(options)?)))
    }
}

impl Indicator for Msw {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &MSW_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        parse_period(MSW_METADATA.name, options)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = MswStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(MswStream::new(options)?)))
    }
}

impl Indicator for Qstick {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &QSTICK_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(QSTICK_METADATA.name, options)?;
        Ok(period - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = QstickStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(QstickStream::new(options)?)))
    }
}

struct CciStream {
    period: usize,
    progress: usize,
    values: VecDeque<Real>,
    sum: Real,
}

impl CciStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        Ok(Self {
            period: parse_period(CCI_METADATA.name, options)?,
            progress: 0,
            values: VecDeque::new(),
            sum: 0.0,
        })
    }
}

impl IndicatorStream for CciStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &CCI_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close) = triple_input(CCI_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(high.len());

        for ((&high, &low), &close) in high.iter().zip(low.iter()).zip(close.iter()) {
            let today = (high + low + close) / 3.0;
            self.values.push_back(today);
            self.sum += today;
            if self.values.len() > self.period {
                self.sum -= self
                    .values
                    .pop_front()
                    .expect("cci window should not be empty");
            }

            if self.progress + 1 >= self.period * 2 - 1 {
                let avg = self.sum / self.period as Real;
                let acc = self
                    .values
                    .iter()
                    .map(|value| (avg - *value).abs())
                    .sum::<Real>();
                let denom = acc / self.period as Real * 0.015;
                output.push((today - avg) / denom);
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

struct CmoStream {
    progress: usize,
    previous: Option<Real>,
    up: RingSum,
    down: RingSum,
}

impl CmoStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(CMO_METADATA.name, options)?;
        Ok(Self {
            progress: 0,
            previous: None,
            up: RingSum::new(period),
            down: RingSum::new(period),
        })
    }
}

impl IndicatorStream for CmoStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &CMO_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(CMO_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(input.len());

        for &sample in input {
            if let Some(previous) = self.previous {
                self.up.push(if sample > previous {
                    sample - previous
                } else {
                    0.0
                });
                self.down.push(if sample < previous {
                    previous - sample
                } else {
                    0.0
                });

                if self.up.is_full() {
                    output.push(
                        100.0 * (self.up.sum - self.down.sum) / (self.up.sum + self.down.sum),
                    );
                }
            }
            self.previous = Some(sample);
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

struct CviStream {
    period: usize,
    progress: usize,
    ema: Option<Real>,
    multiplier: Real,
    lag: LagRing,
}

impl CviStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(CVI_METADATA.name, options)?;
        Ok(Self {
            period,
            progress: 0,
            ema: None,
            multiplier: 2.0 / (period as Real + 1.0),
            lag: LagRing::new(period),
        })
    }
}

impl IndicatorStream for CviStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &CVI_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low) = double_input(CVI_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(high.len());

        for (&high, &low) in high.iter().zip(low.iter()) {
            let range = high - low;
            let ema = match self.ema {
                Some(current) => (range - current) * self.multiplier + current,
                None => range,
            };
            self.ema = Some(ema);

            if self.progress >= 1 {
                let replaced = self.lag.push(ema);
                if self.progress >= self.period * 2 - 1 {
                    let old = replaced.expect("cvi lag ring should be full after warmup");
                    output.push(100.0 * (ema - old) / old);
                }
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

struct FisherStream {
    period: usize,
    progress: usize,
    max_queue: MonotonicQueue,
    min_queue: MonotonicQueue,
    val1: Real,
    fish: Real,
}

impl FisherStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        Ok(Self {
            period: parse_period(FISHER_METADATA.name, options)?,
            progress: 0,
            max_queue: MonotonicQueue::new(ExtremaKind::Max),
            min_queue: MonotonicQueue::new(ExtremaKind::Min),
            val1: 0.0,
            fish: 0.0,
        })
    }
}

impl IndicatorStream for FisherStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &FISHER_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low) = double_input(FISHER_METADATA.name, inputs)?;
        let mut fisher = Vec::with_capacity(high.len());
        let mut signal = Vec::with_capacity(high.len());

        for (&high, &low) in high.iter().zip(low.iter()) {
            let index = self.progress;
            let hl = 0.5 * (high + low);
            self.max_queue.push(index, hl);
            self.min_queue.push(index, hl);

            if index + 1 >= self.period {
                let window_start = index + 1 - self.period;
                self.max_queue.evict_before(window_start);
                self.min_queue.evict_before(window_start);

                let max = self.max_queue.front_value();
                let min = self.min_queue.front_value();
                let mut mm = max - min;
                if mm == 0.0 {
                    mm = 0.001;
                }

                self.val1 = 0.33 * 2.0 * ((hl - min) / mm - 0.5) + 0.67 * self.val1;
                self.val1 = self.val1.clamp(-0.999, 0.999);

                signal.push(self.fish);
                self.fish = 0.5 * ((1.0 + self.val1) / (1.0 - self.val1)).ln() + 0.5 * self.fish;
                fisher.push(self.fish);
            }

            self.progress += 1;
        }

        Ok(vec![fisher, signal])
    }
}

struct MdStream {
    period: usize,
    progress: usize,
    values: VecDeque<Real>,
    sum: Real,
}

impl MdStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        Ok(Self {
            period: parse_period(MD_METADATA.name, options)?,
            progress: 0,
            values: VecDeque::new(),
            sum: 0.0,
        })
    }
}

impl IndicatorStream for MdStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &MD_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(MD_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(input.len());

        for &sample in input {
            self.values.push_back(sample);
            self.sum += sample;
            if self.values.len() > self.period {
                self.sum -= self
                    .values
                    .pop_front()
                    .expect("md window should not be empty");
            }

            if self.values.len() == self.period {
                let avg = self.sum / self.period as Real;
                let acc = self
                    .values
                    .iter()
                    .map(|value| (avg - *value).abs())
                    .sum::<Real>();
                output.push(acc / self.period as Real);
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn mean_deviation_window(window: &[Real], sum: Real, period: usize) -> Real {
    let avg = sum / period as Real;
    let acc = window
        .iter()
        .map(|value| (avg - *value).abs())
        .sum::<Real>();
    acc / period as Real
}

struct MswStream {
    period: usize,
    progress: usize,
    values: VecDeque<Real>,
    cos_weights: Vec<Real>,
    sin_weights: Vec<Real>,
}

impl MswStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(MSW_METADATA.name, options)?;
        let tpi = 2.0 * LEGACY_PI;
        let cos_weights = (0..period)
            .map(|index| (tpi * index as Real / period as Real).cos())
            .collect();
        let sin_weights = (0..period)
            .map(|index| (tpi * index as Real / period as Real).sin())
            .collect();
        Ok(Self {
            period,
            progress: 0,
            values: VecDeque::new(),
            cos_weights,
            sin_weights,
        })
    }
}

impl IndicatorStream for MswStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &MSW_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(MSW_METADATA.name, inputs)?;
        let mut sine = Vec::with_capacity(input.len());
        let mut lead = Vec::with_capacity(input.len());
        let pi = LEGACY_PI;
        let tpi = 2.0 * pi;

        for &sample in input {
            self.values.push_back(sample);
            if self.values.len() > self.period {
                self.values.pop_front();
            }

            if self.progress >= self.period && self.values.len() == self.period {
                let mut rp = 0.0;
                let mut ip = 0.0;

                for (index, value) in self.values.iter().rev().enumerate() {
                    rp += self.cos_weights[index] * *value;
                    ip += self.sin_weights[index] * *value;
                }

                let mut phase = if rp.abs() > 0.001 {
                    (ip / rp).atan()
                } else {
                    tpi / 2.0 * if ip < 0.0 { -1.0 } else { 1.0 }
                };
                if rp < 0.0 {
                    phase += pi;
                }
                phase += pi / 2.0;
                if phase < 0.0 {
                    phase += tpi;
                }
                if phase > tpi {
                    phase -= tpi;
                }

                sine.push(phase.sin());
                lead.push((phase + pi / 4.0).sin());
            }

            self.progress += 1;
        }

        Ok(vec![sine, lead])
    }
}

struct QstickStream {
    period: usize,
    progress: usize,
    sum: RingSum,
}

impl QstickStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(QSTICK_METADATA.name, options)?;
        Ok(Self {
            period,
            progress: 0,
            sum: RingSum::new(period),
        })
    }
}

impl IndicatorStream for QstickStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &QSTICK_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (open, close) = double_input(QSTICK_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(open.len());

        for (&open, &close) in open.iter().zip(close.iter()) {
            self.sum.push(close - open);
            if self.sum.is_full() {
                output.push(self.sum.sum / self.period as Real);
            }
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

struct LagRing {
    values: Vec<Real>,
    cursor: usize,
    len: usize,
}

impl LagRing {
    fn new(capacity: usize) -> Self {
        Self {
            values: vec![0.0; capacity],
            cursor: 0,
            len: 0,
        }
    }

    fn push(&mut self, value: Real) -> Option<Real> {
        if self.len < self.values.len() {
            self.values[self.len] = value;
            self.len += 1;
            None
        } else {
            let old = self.values[self.cursor];
            self.values[self.cursor] = value;
            self.cursor = (self.cursor + 1) % self.values.len();
            Some(old)
        }
    }
}

fn parse_period(indicator: &'static str, options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(indicator, options, 1)?;
    parse_usize_option(indicator, options, 0, "period", 1)
}
