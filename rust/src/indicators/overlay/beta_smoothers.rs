//! TA-Lib compatibility smoother overlays exposed beside native Tulip moving averages.
//!
//! These implementations keep specialized smoothing formulas available without changing the public
//! batch, in-place, stream, or registry contracts.
use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{
    double_input, expect_option_count, parse_usize_option, single_input,
};
use crate::indicators::shared::{ExtremaKind, MonotonicQueue};

const ALMA_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "alma",
    full_name: "Arnaud Legoux Moving Average",
    category: IndicatorCategory::Overlay,
    input_names: &["real"],
    option_names: &["period", "offset", "sigma"],
    output_names: &["alma"],
};

const IKHTS_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "ikhts",
    full_name: "Ichimoku Tenkan-Sen",
    category: IndicatorCategory::Overlay,
    input_names: &["high", "low"],
    option_names: &["period"],
    output_names: &["ikhts"],
};

const RMTA_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "rmta",
    full_name: "Recursive Moving Trend Average",
    category: IndicatorCategory::Overlay,
    input_names: &["real"],
    option_names: &["period", "beta"],
    output_names: &["rmta"],
};

const MAMA_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "mama",
    full_name: "MESA Adaptive Moving Average",
    category: IndicatorCategory::Overlay,
    input_names: &["real"],
    option_names: &["fastlimit", "slowlimit"],
    output_names: &["mama", "fama"],
};

#[derive(Debug, Clone, Copy)]
pub struct Alma;
#[derive(Debug, Clone, Copy)]
pub struct Ikhts;
#[derive(Debug, Clone, Copy)]
pub struct Rmta;
#[derive(Debug, Clone, Copy)]
pub struct Mama;

impl Indicator for Alma {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &ALMA_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(parse_alma_options(options)?.0 - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(ALMA_METADATA.name, inputs)?;
        let (period, weights) = alma_weights(options)?;
        let mut output = Vec::new();

        if input.len() >= period {
            for end in (period - 1)..input.len() {
                let start = end + 1 - period;
                let sum = input[start..=end]
                    .iter()
                    .zip(weights.iter())
                    .map(|(value, weight)| value * weight)
                    .sum();
                output.push(sum);
            }
        }

        Ok(vec![output])
    }

    fn create_stream(
        &self,
        _options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(None)
    }
}

impl Indicator for Ikhts {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &IKHTS_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(parse_period(IKHTS_METADATA.name, options)? - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = IkhtsStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(IkhtsStream::new(options)?)))
    }
}

impl Indicator for Rmta {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &RMTA_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(parse_rmta_options(options)?.0 - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = RmtaStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(RmtaStream::new(options)?)))
    }
}

impl Indicator for Mama {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &MAMA_METADATA
    }

    fn lookback(&self, _options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(6)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = MamaStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(MamaStream::new(options)?)))
    }
}

struct IkhtsStream {
    period: usize,
    progress: usize,
    high_queue: MonotonicQueue,
    low_queue: MonotonicQueue,
}

impl IkhtsStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        Ok(Self {
            period: parse_period(IKHTS_METADATA.name, options)?,
            progress: 0,
            high_queue: MonotonicQueue::new(ExtremaKind::Max),
            low_queue: MonotonicQueue::new(ExtremaKind::Min),
        })
    }
}

impl IndicatorStream for IkhtsStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &IKHTS_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low) = double_input(IKHTS_METADATA.name, inputs)?;
        let mut output = Vec::new();

        for (&high_value, &low_value) in high.iter().zip(low) {
            self.high_queue.push(self.progress, high_value);
            self.low_queue.push(self.progress, low_value);
            let min_index = self.progress.saturating_add(1).saturating_sub(self.period);
            self.high_queue.evict_before(min_index);
            self.low_queue.evict_before(min_index);

            if self.progress + 1 >= self.period {
                output.push((self.high_queue.front_value() + self.low_queue.front_value()) / 2.0);
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

struct RmtaStream {
    period: usize,
    beta: Real,
    progress: usize,
    b: Option<Real>,
    rmta: Option<Real>,
}

struct MamaStream {
    progress: usize,
    fastlimit: Real,
    slowlimit: Real,
    price: RingHistory<4>,
    smooth: RingHistory<7>,
    detrender: RingHistory<7>,
    i1: RingHistory<7>,
    q1: RingHistory<7>,
    ji: RingHistory<1>,
    jq: RingHistory<1>,
    i2: RingHistory<2>,
    q2: RingHistory<2>,
    re: RingHistory<2>,
    im: RingHistory<2>,
    period: RingHistory<2>,
    smoothperiod: RingHistory<2>,
    phase: RingHistory<2>,
    deltaphase: RingHistory<1>,
    alpha: RingHistory<1>,
    mama: RingHistory<1>,
    fama: RingHistory<1>,
}

impl MamaStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let (fastlimit, slowlimit) = parse_mama_options(options)?;
        Ok(Self {
            progress: 0,
            fastlimit,
            slowlimit,
            price: RingHistory::new(),
            smooth: RingHistory::new(),
            detrender: RingHistory::new(),
            i1: RingHistory::new(),
            q1: RingHistory::new(),
            ji: RingHistory::new(),
            jq: RingHistory::new(),
            i2: RingHistory::new(),
            q2: RingHistory::new(),
            re: RingHistory::new(),
            im: RingHistory::new(),
            period: RingHistory::new(),
            smoothperiod: RingHistory::new(),
            phase: RingHistory::new(),
            deltaphase: RingHistory::new(),
            alpha: RingHistory::new(),
            mama: RingHistory::new(),
            fama: RingHistory::new(),
        })
    }

    fn seed_zero_state(&mut self, price: Real) {
        self.price.push(price);
        self.smooth.push(0.0);
        self.detrender.push(0.0);
        self.i1.push(0.0);
        self.q1.push(0.0);
        self.ji.push(0.0);
        self.jq.push(0.0);
        self.i2.push(0.0);
        self.q2.push(0.0);
        self.re.push(0.0);
        self.im.push(0.0);
        self.period.push(0.0);
        self.smoothperiod.push(0.0);
        self.phase.push(0.0);
        self.deltaphase.push(0.0);
        self.alpha.push(0.0);
        self.mama.push(0.0);
        self.fama.push(0.0);
    }
}

impl IndicatorStream for MamaStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &MAMA_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(MAMA_METADATA.name, inputs)?;
        let mut mama_output = Vec::new();
        let mut fama_output = Vec::new();

        for &sample in input {
            if self.progress < 6 {
                self.seed_zero_state(sample);
                self.progress += 1;
                continue;
            }

            self.price.push(sample);

            let smooth = (4.0 * self.price.current()
                + 3.0 * self.price.prev(1)
                + 2.0 * self.price.prev(2)
                + self.price.prev(3))
                / 10.0;
            self.smooth.push(smooth);

            let detrender = hilbert_transform(&self.smooth, self.period.current());
            self.detrender.push(detrender);

            let q1 = hilbert_transform(&self.detrender, self.period.current());
            self.q1.push(q1);

            let i1 = self.detrender.prev(3);
            self.i1.push(i1);

            let ji = hilbert_transform(&self.i1, self.period.current());
            self.ji.push(ji);

            let jq = hilbert_transform(&self.q1, self.period.current());
            self.jq.push(jq);

            let i2 = 0.2 * (self.i1.current() - self.jq.current()) + 0.8 * self.i2.current();
            self.i2.push(i2);

            let q2 = 0.2 * (self.q1.current() + self.ji.current()) + 0.8 * self.q2.current();
            self.q2.push(q2);

            let re = 0.2
                * (self.i2.current() * self.i2.prev(1) + self.q2.current() * self.q2.prev(1))
                + 0.8 * self.re.current();
            self.re.push(re);

            let im = 0.2
                * (self.i2.current() * self.q2.prev(1) - self.q2.current() * self.i2.prev(1))
                + 0.8 * self.im.current();
            self.im.push(im);

            let previous_period = self.period.current();
            let mut period_value = 0.0;
            if self.im.current() != 0.0 && self.re.current() != 0.0 {
                period_value = 360.0 / (self.im.current() / self.re.current()).atan();
            }
            if period_value > 1.5 * previous_period {
                period_value = 1.5 * previous_period;
            }
            if period_value < 0.67 * previous_period {
                period_value = 0.67 * previous_period;
            }
            period_value = period_value.clamp(6.0, 50.0);
            period_value = 0.2 * period_value + 0.8 * previous_period;
            self.period.push(period_value);

            let smoothperiod = 0.33 * self.period.current() + 0.67 * self.smoothperiod.current();
            self.smoothperiod.push(smoothperiod);

            let phase = if self.i1.current() != 0.0 {
                (self.q1.current() / self.i1.current()).atan()
            } else {
                0.0
            };
            self.phase.push(phase);

            let deltaphase = (self.phase.prev(1) - self.phase.current()).max(1.0);
            self.deltaphase.push(deltaphase);

            let alpha = self
                .slowlimit
                .max(self.fastlimit / self.deltaphase.current());
            self.alpha.push(alpha);

            let mama = self.alpha.current() * self.price.current()
                + (1.0 - self.alpha.current()) * self.mama.current();
            self.mama.push(mama);

            let fama = 0.5 * self.alpha.current() * self.mama.current()
                + (1.0 - 0.5 * self.alpha.current()) * self.fama.current();
            self.fama.push(fama);

            mama_output.push(self.mama.current());
            fama_output.push(self.fama.current());
            self.progress += 1;
        }

        Ok(vec![mama_output, fama_output])
    }
}

impl RmtaStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let (period, beta) = parse_rmta_options(options)?;
        Ok(Self {
            period,
            beta,
            progress: 0,
            b: None,
            rmta: None,
        })
    }
}

impl IndicatorStream for RmtaStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &RMTA_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(RMTA_METADATA.name, inputs)?;
        let alpha = 1.0 - self.beta;
        let mut output = Vec::new();

        for &sample in input {
            match (self.b, self.rmta) {
                (None, None) => {
                    let b = (1.0 - alpha) * sample + sample;
                    let rmta = (1.0 - alpha) * sample + alpha * (sample + b);
                    self.b = Some(b);
                    self.rmta = Some(rmta);
                }
                (Some(b), Some(rmta)) => {
                    let next_b = (1.0 - alpha) * b + sample;
                    let next_rmta = (1.0 - alpha) * rmta + alpha * (sample + next_b - b);
                    self.b = Some(next_b);
                    self.rmta = Some(next_rmta);
                    if self.progress + 1 >= self.period {
                        output.push(next_rmta);
                    }
                }
                _ => unreachable!("rmta state should initialize together"),
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn parse_period(name: &'static str, options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(name, options, 1)?;
    parse_usize_option(name, options, 0, "period", 1)
}

fn parse_alma_options(options: &[Real]) -> Result<(usize, Real, Real), IndicatorError> {
    expect_option_count(ALMA_METADATA.name, options, 3)?;
    let period = parse_usize_option(ALMA_METADATA.name, options, 0, "period", 1)?;
    let offset = options[1];
    let sigma = options[2];

    if !offset.is_finite() || !(0.0..=1.0).contains(&offset) {
        return Err(IndicatorError::InvalidOption {
            indicator: ALMA_METADATA.name,
            option: "offset",
            value: offset,
            reason: "expected a finite value between 0 and 1",
        });
    }
    if !sigma.is_finite() || sigma <= 0.0 {
        return Err(IndicatorError::InvalidOption {
            indicator: ALMA_METADATA.name,
            option: "sigma",
            value: sigma,
            reason: "expected a finite value > 0",
        });
    }

    Ok((period, offset, sigma))
}

fn alma_weights(options: &[Real]) -> Result<(usize, Vec<Real>), IndicatorError> {
    let (period, offset, sigma) = parse_alma_options(options)?;
    let m = (offset * (period - 1) as Real).floor();
    let s = period as Real / sigma;
    let mut weights = Vec::with_capacity(period);
    let mut norm = 0.0;

    for index in 0..period {
        let value = (-(index as Real - m).powi(2) / (2.0 * s.powi(2))).exp();
        weights.push(value);
        norm += value;
    }

    for weight in &mut weights {
        *weight /= norm;
    }

    Ok((period, weights))
}

fn parse_rmta_options(options: &[Real]) -> Result<(usize, Real), IndicatorError> {
    expect_option_count(RMTA_METADATA.name, options, 2)?;
    let period = parse_usize_option(RMTA_METADATA.name, options, 0, "period", 1)?;
    let beta = options[1];
    if !beta.is_finite() {
        return Err(IndicatorError::InvalidOption {
            indicator: RMTA_METADATA.name,
            option: "beta",
            value: beta,
            reason: "expected a finite value",
        });
    }
    Ok((period, beta))
}

fn parse_mama_options(options: &[Real]) -> Result<(Real, Real), IndicatorError> {
    expect_option_count(MAMA_METADATA.name, options, 2)?;
    let fastlimit = options[0];
    let slowlimit = options[1];

    for (name, value) in [("fastlimit", fastlimit), ("slowlimit", slowlimit)] {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(IndicatorError::InvalidOption {
                indicator: MAMA_METADATA.name,
                option: name,
                value,
                reason: "expected a finite value between 0 and 1",
            });
        }
    }

    Ok((fastlimit, slowlimit))
}

fn hilbert_transform<const N: usize>(history: &RingHistory<N>, period: Real) -> Real {
    (0.0962 * history.current() + 0.5769 * history.prev(2)
        - 0.5769 * history.prev(4)
        - 0.0962 * history.prev(6))
        * (0.075 * period + 0.54)
}

#[derive(Debug, Clone, Copy)]
struct RingHistory<const N: usize> {
    len: usize,
    index: usize,
    values: [Real; N],
}

impl<const N: usize> RingHistory<N> {
    fn new() -> Self {
        Self {
            len: 0,
            index: 0,
            values: [0.0; N],
        }
    }

    fn push(&mut self, value: Real) {
        if self.len == 0 {
            self.values[0] = value;
            self.len = 1;
            self.index = 0;
            return;
        }
        self.index = (self.index + 1) % N;
        self.values[self.index] = value;
        if self.len < N {
            self.len += 1;
        }
    }

    fn current(&self) -> Real {
        debug_assert!(self.len > 0, "history should contain a current value");
        self.values[self.index]
    }

    fn prev(&self, steps: usize) -> Real {
        debug_assert!(
            self.len > steps,
            "history should contain the requested offset"
        );
        let offset = steps % N;
        let index = (self.index + N - offset) % N;
        self.values[index]
    }
}
