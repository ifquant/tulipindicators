use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, triple_input};

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

        let period = self.period;
        let per = 1.0 / period as Real;
        let start = -(period as isize - 1);
        let mut progress = self.state_progress;
        let mut sum = self.sum;
        let mut last = self.last;
        let mut last_close = self.last_close;
        let mut out_index = 0usize;
        let mut index = 0usize;
        let mut processed = 0usize;

        if progress < 1 {
            if progress == start && index < high.len() {
                sum = high[0] - low[0];
                last_close = close[0];
                progress += 1;
                index += 1;
                processed += 1;
            }

            while progress <= 0 && index < high.len() {
                let tr = true_range(high[index], low[index], last_close);
                sum += tr;
                last_close = close[index];
                progress += 1;
                index += 1;
                processed += 1;
            }

            if progress == 1 {
                last = sum * per;
                outputs[0][out_index] = last;
                out_index += 1;
            }
        }

        if progress >= 1 {
            while index < high.len() {
                let tr = true_range(high[index], low[index], last_close);
                last = (tr - last) * per + last;
                outputs[0][out_index] = last;
                out_index += 1;
                last_close = close[index];
                progress += 1;
                index += 1;
                processed += 1;
            }
        }

        self.progress += processed;
        self.state_progress = progress;
        self.sum = sum;
        self.last = last;
        self.last_close = last_close;
        Ok(out_index)
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 1)
}

fn true_range(high: Real, low: Real, previous_close: Real) -> Real {
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
        value = (tr - value) * per + value;
        output[out_index] = value;
        out_index += 1;
    }

    out_index
}
