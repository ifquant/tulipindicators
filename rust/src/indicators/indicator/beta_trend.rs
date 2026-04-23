//! TA-Lib compatibility trend indicators exposed through the Tulip registry.
//!
//! These indicators preserve TA-Lib naming and output shapes while following the crate's shared
//! option parsing and buffer validation rules.
use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};
use crate::indicators::shared::{EmaState, WmaState};
use std::collections::VecDeque;

type KstOptions = (usize, usize, usize, usize, usize, usize, usize, usize);

const COPP_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "copp",
    full_name: "Coppock Curve",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["short_roc_period", "long_roc_period", "wma_period"],
    output_names: &["copp"],
};

const KST_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "kst",
    full_name: "Know Sure Thing",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &[
        "roc1_period",
        "roc2_period",
        "roc3_period",
        "roc4_period",
        "ma1_period",
        "ma2_period",
        "ma3_period",
        "ma4_period",
    ],
    output_names: &["kst", "signal"],
};

const PFE_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "pfe",
    full_name: "Polarized Fractal Efficiency",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["period", "ema_period"],
    output_names: &["pfe"],
};

const RMI_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "rmi",
    full_name: "Relative Momentum Index",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["period", "lookback_period"],
    output_names: &["rmi"],
};

#[derive(Debug, Clone, Copy)]
pub struct Copp;
#[derive(Debug, Clone, Copy)]
pub struct Kst;
#[derive(Debug, Clone, Copy)]
pub struct Pfe;
#[derive(Debug, Clone, Copy)]
pub struct Rmi;

impl Indicator for Copp {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &COPP_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let (_, long_period, wma_period) = parse_copp_options(options)?;
        Ok(long_period + wma_period - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = CoppStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(CoppStream::new(options)?)))
    }
}

impl Indicator for Kst {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &KST_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let (_, _, _, roc4, _, _, _, _) = parse_kst_options(options)?;
        Ok(roc4)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = KstStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(KstStream::new(options)?)))
    }
}

impl Indicator for Pfe {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &PFE_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(parse_pfe_options(options)?.0)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = PfeStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(PfeStream::new(options)?)))
    }
}

impl Indicator for Rmi {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &RMI_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(parse_rmi_options(options)?.1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = RmiStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(RmiStream::new(options)?)))
    }
}

struct CoppStream {
    progress: usize,
    short_period: usize,
    long_period: usize,
    prices: VecDeque<Real>,
    wma: WmaState,
}

impl CoppStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let (short_period, long_period, wma_period) = parse_copp_options(options)?;
        Ok(Self {
            progress: 0,
            short_period,
            long_period,
            prices: VecDeque::with_capacity(long_period + 1),
            wma: WmaState::new(wma_period),
        })
    }
}

impl IndicatorStream for CoppStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &COPP_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(COPP_METADATA.name, inputs)?;
        let mut output = Vec::new();

        for &sample in input {
            self.prices.push_back(sample);
            if self.prices.len() > self.long_period + 1 {
                self.prices.pop_front();
            }

            if self.prices.len() > self.long_period {
                let current = *self.prices.back().expect("current price should exist");
                let short_base = self.prices[self.prices.len() - 1 - self.short_period];
                let long_base = self.prices[0];
                let rocs = ((current / short_base - 1.0) + (current / long_base - 1.0)) * 50.0;
                if let Some(value) = self.wma.feed(rocs) {
                    output.push(value);
                }
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

struct KstStream {
    progress: usize,
    periods: [usize; 4],
    prices: VecDeque<Real>,
    emas: [EmaState; 4],
    signal: Option<EmaState>,
}

impl KstStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let (roc1, roc2, roc3, roc4, ma1, ma2, ma3, ma4) = parse_kst_options(options)?;
        Ok(Self {
            progress: 0,
            periods: [roc1, roc2, roc3, roc4],
            prices: VecDeque::with_capacity(roc4 + 1),
            emas: [
                EmaState::new(2.0 / (ma1 as Real + 1.0)),
                EmaState::new(2.0 / (ma2 as Real + 1.0)),
                EmaState::new(2.0 / (ma3 as Real + 1.0)),
                EmaState::new(2.0 / (ma4 as Real + 1.0)),
            ],
            signal: None,
        })
    }
}

impl IndicatorStream for KstStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &KST_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(KST_METADATA.name, inputs)?;
        let mut kst = Vec::new();
        let mut signal = Vec::new();

        for &sample in input {
            self.prices.push_back(sample);
            if self.prices.len() > self.periods[3] + 1 {
                self.prices.pop_front();
            }

            if let Some(&current) = self.prices.back() {
                for index in 0..4 {
                    let period = self.periods[index];
                    if self.prices.len() > period {
                        let base = self.prices[self.prices.len() - 1 - period];
                        let roc = (current - base) / base;
                        self.emas[index].feed(roc);
                    }
                }
            }

            if self.prices.len() > self.periods[3] {
                let mut value = 0.0;
                for index in 0..4 {
                    value += self.emas[index].value() * (index as Real + 1.0);
                }
                value /= 10.0;

                let signal_value = match self.signal.as_mut() {
                    Some(signal_ema) => signal_ema.feed(value),
                    None => {
                        let mut signal_ema = EmaState::new(2.0 / 10.0);
                        let initial = signal_ema.feed(value);
                        self.signal = Some(signal_ema);
                        initial
                    }
                };

                kst.push(value);
                signal.push(signal_value);
            }

            self.progress += 1;
        }

        Ok(vec![kst, signal])
    }
}

struct PfeStream {
    progress: usize,
    period: usize,
    prices: VecDeque<Real>,
    segments: VecDeque<Real>,
    segment_sum: Real,
    ema: EmaState,
    initialized: bool,
}

impl PfeStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let (period, ema_period) = parse_pfe_options(options)?;
        Ok(Self {
            progress: 0,
            period,
            prices: VecDeque::with_capacity(period + 1),
            segments: VecDeque::with_capacity(period),
            segment_sum: 0.0,
            ema: EmaState::new(2.0 / (ema_period as Real + 1.0)),
            initialized: false,
        })
    }
}

impl IndicatorStream for PfeStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &PFE_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(PFE_METADATA.name, inputs)?;
        let mut output = Vec::new();

        for &sample in input {
            if let Some(&previous) = self.prices.back() {
                let segment = ((sample - previous).powi(2) + 1.0).sqrt();
                self.segments.push_back(segment);
                self.segment_sum += segment;
                if self.segments.len() > self.period {
                    self.segment_sum -= self.segments.pop_front().expect("segment should exist");
                }
            }

            self.prices.push_back(sample);
            if self.prices.len() > self.period + 1 {
                self.prices.pop_front();
            }

            if self.prices.len() == self.period + 1 && self.segments.len() == self.period {
                let oldest = self.prices.front().expect("oldest price should exist");
                let sign = if sample - oldest > 0.0 { 1.0 } else { -1.0 };
                let numerator = sign * 100.0 * ((sample - oldest).powi(2) + 100.0).sqrt();
                let ratio = numerator / self.segment_sum;
                let value = self.ema.feed(ratio);
                self.initialized = true;
                output.push(value);
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

struct RmiStream {
    progress: usize,
    period_mul: Real,
    lookback_period: usize,
    prices: VecDeque<Real>,
    gains_ema: Option<Real>,
    losses_ema: Option<Real>,
}

impl RmiStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let (period, lookback_period) = parse_rmi_options(options)?;
        Ok(Self {
            progress: 0,
            period_mul: 2.0 / (period as Real + 1.0),
            lookback_period,
            prices: VecDeque::with_capacity(lookback_period + 1),
            gains_ema: None,
            losses_ema: None,
        })
    }
}

impl IndicatorStream for RmiStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &RMI_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(RMI_METADATA.name, inputs)?;
        let mut output = Vec::new();

        for &sample in input {
            self.prices.push_back(sample);
            if self.prices.len() > self.lookback_period + 1 {
                self.prices.pop_front();
            }

            if self.prices.len() == self.lookback_period + 1 {
                let base = self.prices[0];
                let gain = (sample - base).max(0.0);
                let loss = (base - sample).max(0.0);
                match (self.gains_ema, self.losses_ema) {
                    (None, None) => {
                        self.gains_ema = Some(gain);
                        self.losses_ema = Some(loss);
                    }
                    (Some(gains_ema), Some(losses_ema)) => {
                        self.gains_ema = Some((gain - gains_ema) * self.period_mul + gains_ema);
                        self.losses_ema = Some((loss - losses_ema) * self.period_mul + losses_ema);
                    }
                    _ => unreachable!("rmi state should initialize together"),
                }

                let gains_ema = self.gains_ema.expect("gains ema should exist");
                let losses_ema = self.losses_ema.expect("losses ema should exist");
                output.push(gains_ema / (gains_ema + losses_ema) * 100.0);
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn parse_copp_options(options: &[Real]) -> Result<(usize, usize, usize), IndicatorError> {
    expect_option_count(COPP_METADATA.name, options, 3)?;
    let short_period = parse_usize_option(COPP_METADATA.name, options, 0, "short_roc_period", 1)?;
    let long_period = parse_usize_option(COPP_METADATA.name, options, 1, "long_roc_period", 1)?;
    let wma_period = parse_usize_option(COPP_METADATA.name, options, 2, "wma_period", 1)?;
    if long_period < short_period {
        return Err(IndicatorError::InvalidOption {
            indicator: COPP_METADATA.name,
            option: "long_roc_period",
            value: long_period as Real,
            reason: "expected long_roc_period >= short_roc_period",
        });
    }
    Ok((short_period, long_period, wma_period))
}

fn parse_kst_options(options: &[Real]) -> Result<KstOptions, IndicatorError> {
    expect_option_count(KST_METADATA.name, options, 8)?;
    let roc1 = parse_usize_option(KST_METADATA.name, options, 0, "roc1_period", 1)?;
    let roc2 = parse_usize_option(KST_METADATA.name, options, 1, "roc2_period", 1)?;
    let roc3 = parse_usize_option(KST_METADATA.name, options, 2, "roc3_period", 1)?;
    let roc4 = parse_usize_option(KST_METADATA.name, options, 3, "roc4_period", 1)?;
    let ma1 = parse_usize_option(KST_METADATA.name, options, 4, "ma1_period", 1)?;
    let ma2 = parse_usize_option(KST_METADATA.name, options, 5, "ma2_period", 1)?;
    let ma3 = parse_usize_option(KST_METADATA.name, options, 6, "ma3_period", 1)?;
    let ma4 = parse_usize_option(KST_METADATA.name, options, 7, "ma4_period", 1)?;
    if !(roc1 < roc2 && roc2 < roc3 && roc3 < roc4) {
        return Err(IndicatorError::InvalidOption {
            indicator: KST_METADATA.name,
            option: "roc_periods",
            value: roc4 as Real,
            reason: "expected roc1 < roc2 < roc3 < roc4",
        });
    }
    Ok((roc1, roc2, roc3, roc4, ma1, ma2, ma3, ma4))
}

fn parse_pfe_options(options: &[Real]) -> Result<(usize, usize), IndicatorError> {
    expect_option_count(PFE_METADATA.name, options, 2)?;
    let period = parse_usize_option(PFE_METADATA.name, options, 0, "period", 1)?;
    let ema_period = parse_usize_option(PFE_METADATA.name, options, 1, "ema_period", 1)?;
    Ok((period, ema_period))
}

fn parse_rmi_options(options: &[Real]) -> Result<(usize, usize), IndicatorError> {
    expect_option_count(RMI_METADATA.name, options, 2)?;
    let period = parse_usize_option(RMI_METADATA.name, options, 0, "period", 1)?;
    let lookback_period = parse_usize_option(RMI_METADATA.name, options, 1, "lookback_period", 1)?;
    Ok((period, lookback_period))
}
