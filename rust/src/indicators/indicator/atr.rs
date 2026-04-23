use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, triple_input};
use crate::indicators::shared::true_range;
use crate::state::{validate_history_capacity, IndicatorState, RingHistory};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "atr",
    full_name: "Average True Range",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low", "close"],
    option_names: &["period"],
    output_names: &["atr"],
};

#[derive(Debug, Clone, Copy)]
pub struct Atr;

impl Atr {
    pub fn state(options: &[Real], history_capacity: usize) -> Result<AtrState, IndicatorError> {
        AtrState::new(options, history_capacity)
    }
}

impl Indicator for Atr {
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
        let mut output = vec![0.0; high.len().saturating_sub(lookback)];

        if high.len() <= lookback {
            output.clear();
            return Ok(vec![output]);
        }

        let produced = run_atr_batch(high, low, close, period, &mut output);
        debug_assert_eq!(produced, output.len());

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
        Ok(run_atr_batch(
            high,
            low,
            close,
            period,
            &mut outputs[0][..output_len],
        ))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(AtrStream::new(options)?)))
    }
}

struct AtrStream {
    period: usize,
    progress: usize,
    state_progress: isize,
    sum: Real,
    last: Real,
    last_close: Real,
}

impl AtrStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        Ok(Self {
            period: parse_period(options)?,
            progress: 0,
            state_progress: -(parse_period(options)? as isize - 1),
            sum: 0.0,
            last: 0.0,
            last_close: 0.0,
        })
    }

    fn update_one(&mut self, high: Real, low: Real, close: Real) -> Option<Real> {
        let period = self.period;
        let per = 1.0 / period as Real;
        let start = -(period as isize - 1);

        if self.state_progress < 1 {
            if self.state_progress == start {
                self.sum = high - low;
                self.last_close = close;
                self.state_progress += 1;
                self.progress += 1;
                return None;
            }

            let tr = true_range(high, low, self.last_close);
            self.sum += tr;
            self.last_close = close;
            self.state_progress += 1;
            self.progress += 1;

            if self.state_progress == 1 {
                self.last = self.sum * per;
                Some(self.last)
            } else {
                None
            }
        } else {
            let tr = true_range(high, low, self.last_close);
            self.last = (tr - self.last).mul_add(per, self.last);
            self.last_close = close;
            self.state_progress += 1;
            self.progress += 1;
            Some(self.last)
        }
    }
}

impl IndicatorStream for AtrStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, _, _) = triple_input(METADATA.name, inputs)?;
        let mut output = vec![0.0; high.len()];
        let mut outputs = [&mut output[..]];
        let produced = self.feed_in_place(inputs, &mut outputs)?;
        output.truncate(produced);

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

pub struct AtrState {
    period: usize,
    stream: AtrStream,
    history: RingHistory<Real>,
}

impl AtrState {
    pub fn new(options: &[Real], history_capacity: usize) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        validate_history_capacity(METADATA.name, history_capacity)?;
        Ok(Self {
            period,
            stream: AtrStream::new(options)?,
            history: RingHistory::new(history_capacity),
        })
    }
}

impl IndicatorState for AtrState {
    type Input = (Real, Real, Real);
    type Output = Real;

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
        self.history.latest().cloned()
    }

    fn get(&self, index_from_latest: usize) -> Option<Self::Output> {
        self.history.get(index_from_latest).cloned()
    }

    fn len(&self) -> usize {
        self.history.len()
    }

    fn history_capacity(&self) -> usize {
        self.history.capacity()
    }

    fn reset(&mut self) {
        self.stream = AtrStream {
            period: self.period,
            progress: 0,
            state_progress: -(self.period as isize - 1),
            sum: 0.0,
            last: 0.0,
            last_close: 0.0,
        };
        self.history.clear();
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 1)
}

fn run_atr_batch(
    high: &[Real],
    low: &[Real],
    close: &[Real],
    period: usize,
    output: &mut [Real],
) -> usize {
    if high.len() < period {
        return 0;
    }

    let per = 1.0 / period as Real;
    let mut sum = high[0] - low[0];
    for index in 1..period {
        sum += true_range(high[index], low[index], close[index - 1]);
    }

    let mut value = sum / period as Real;
    output[0] = value;
    let mut out_index = 1usize;

    for index in period..high.len() {
        let tr = true_range(high[index], low[index], close[index - 1]);
        value = (tr - value).mul_add(per, value);
        output[out_index] = value;
        out_index += 1;
    }

    out_index
}
