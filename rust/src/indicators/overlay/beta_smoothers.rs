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

#[derive(Debug, Clone, Copy)]
pub struct Alma;
#[derive(Debug, Clone, Copy)]
pub struct Ikhts;
#[derive(Debug, Clone, Copy)]
pub struct Rmta;

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
