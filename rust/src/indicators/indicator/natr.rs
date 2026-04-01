use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, triple_input};
use crate::indicators::shared::true_range;
use crate::state::{IndicatorState, RingHistory};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "natr",
    full_name: "Normalized Average True Range",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low", "close"],
    option_names: &["period"],
    output_names: &["natr"],
};

#[derive(Debug, Clone, Copy)]
pub struct Natr;

impl Natr {
    pub fn state(options: &[Real], history_capacity: usize) -> Result<NatrState, IndicatorError> {
        NatrState::new(options, history_capacity)
    }
}

impl Indicator for Natr {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        Ok(period - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close) = triple_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let lookback = period - 1;
        let mut output = Vec::with_capacity(high.len().saturating_sub(lookback));

        if high.len() <= lookback {
            return Ok(vec![output]);
        }

        let per = 1.0 / period as Real;
        let mut sum = high[0] - low[0];
        for index in 1..period {
            sum += true_range(high[index], low[index], close[index - 1]);
        }

        let mut value = sum / period as Real;
        output.push(100.0 * value / close[period - 1]);

        for index in period..high.len() {
            let tr = true_range(high[index], low[index], close[index - 1]);
            value = (tr - value).mul_add(per, value);
            output.push(100.0 * value / close[index]);
        }

        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let (high, low, close) = triple_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let output_len = high.len().saturating_sub(period - 1);
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;

        if high.len() < period {
            return Ok(0);
        }

        let per = 1.0 / period as Real;
        let mut sum = high[0] - low[0];
        for index in 1..period {
            sum += true_range(high[index], low[index], close[index - 1]);
        }

        let mut value = sum * per;
        outputs[0][0] = 100.0 * value / close[period - 1];
        let mut out_index = 1usize;

        for index in period..high.len() {
            let tr = true_range(high[index], low[index], close[index - 1]);
            value = (tr - value).mul_add(per, value);
            outputs[0][out_index] = 100.0 * value / close[index];
            out_index += 1;
        }

        Ok(out_index)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(NatrStream::new(options)?)))
    }
}

struct NatrStream {
    period: usize,
    progress: usize,
    sum: Real,
    last_atr: Option<Real>,
    last_close: Option<Real>,
}

impl NatrStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        Ok(Self {
            period: parse_period(options)?,
            progress: 0,
            sum: 0.0,
            last_atr: None,
            last_close: None,
        })
    }

    fn update_one(&mut self, high: Real, low: Real, close: Real) -> Option<Real> {
        let tr = match self.last_close {
            Some(previous_close) => true_range(high, low, previous_close),
            None => high - low,
        };

        let output = if self.progress < self.period {
            self.sum += tr;
            if self.progress + 1 == self.period {
                let atr = self.sum / self.period as Real;
                self.last_atr = Some(atr);
                Some(100.0 * atr / close)
            } else {
                None
            }
        } else if let Some(last_atr) = self.last_atr {
            let atr = (tr - last_atr).mul_add(1.0 / self.period as Real, last_atr);
            self.last_atr = Some(atr);
            Some(100.0 * atr / close)
        } else {
            None
        };

        self.last_close = Some(close);
        self.progress += 1;
        output
    }
}

impl IndicatorStream for NatrStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close) = triple_input(METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(high.len());

        for ((&high, &low), &close) in high.iter().zip(low.iter()).zip(close.iter()) {
            if let Some(value) = self.update_one(high, low, close) {
                output.push(value);
            }
        }

        Ok(vec![output])
    }

    fn feed_in_place(
        &mut self,
        inputs: &[&[Real]],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let (high, low, close) = triple_input(METADATA.name, inputs)?;
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), high.len(), 0)?;

        let mut out_index = 0usize;
        for ((&high, &low), &close) in high.iter().zip(low.iter()).zip(close.iter()) {
            if let Some(value) = self.update_one(high, low, close) {
                outputs[0][out_index] = value;
                out_index += 1;
            }
        }
        Ok(out_index)
    }
}

pub struct NatrState {
    period: usize,
    stream: NatrStream,
    history: RingHistory<Real>,
}

impl NatrState {
    pub fn new(options: &[Real], history_capacity: usize) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        Ok(Self {
            period,
            stream: NatrStream::new(options)?,
            history: RingHistory::new(history_capacity),
        })
    }
}

impl IndicatorState for NatrState {
    type Input = (Real, Real, Real);
    type Output = Real;

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
        self.stream = NatrStream {
            period: self.period,
            progress: 0,
            sum: 0.0,
            last_atr: None,
            last_close: None,
        };
        self.history.clear();
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 1)
}
