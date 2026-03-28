use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "zlema",
    full_name: "Zero-Lag Exponential Moving Average",
    category: IndicatorCategory::Overlay,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["zlema"],
};

#[derive(Debug, Clone, Copy)]
pub struct Zlema;

impl Indicator for Zlema {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        Ok(zlema_lookback(period))
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let lookback = zlema_lookback(period);
        let mut output = vec![0.0; input.len().saturating_sub(lookback)];
        let produced = run_zlema_batch(input, period, &mut output);
        debug_assert_eq!(produced, output.len());
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let output_len = input.len().saturating_sub(zlema_lookback(period));
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_zlema_batch(
            input,
            period,
            &mut outputs[0][..output_len],
        ))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(ZlemaStream::new(options)?)))
    }
}

struct ZlemaStream {
    lag: usize,
    progress: usize,
    multiplier: Real,
    history: Vec<Real>,
    cursor: usize,
    len: usize,
    value: Option<Real>,
}

impl ZlemaStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        let lag = (period - 1) / 2;
        Ok(Self {
            lag,
            progress: 0,
            multiplier: 2.0 / (period as Real + 1.0),
            history: vec![0.0; lag.max(1)],
            cursor: 0,
            len: 0,
            value: None,
        })
    }
}

impl IndicatorStream for ZlemaStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let mut output = vec![0.0; input.len()];
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
        let input = single_input(METADATA.name, inputs)?;
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), input.len(), 0)?;

        let mut out_index = 0usize;
        for &sample in input {
            if self.lag == 0 {
                let next = match self.value {
                    Some(value) => (sample - value).mul_add(self.multiplier, value),
                    None => sample,
                };
                self.value = Some(next);
                outputs[0][out_index] = next;
                out_index += 1;
                self.progress += 1;
                continue;
            }

            if self.len < self.lag {
                self.history[self.len] = sample;
                self.len += 1;
                if self.len == self.lag {
                    self.value = Some(sample);
                    outputs[0][out_index] = sample;
                    out_index += 1;
                }
            } else {
                let lagged = self.history[self.cursor];
                self.history[self.cursor] = sample;
                self.cursor = (self.cursor + 1) % self.lag;
                let previous = self.value.expect("zlema value should be initialized");
                let next =
                    ((sample + (sample - lagged)) - previous).mul_add(self.multiplier, previous);
                self.value = Some(next);
                outputs[0][out_index] = next;
                out_index += 1;
            }

            self.progress += 1;
        }

        Ok(out_index)
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 1)
}

fn zlema_lookback(period: usize) -> usize {
    ((period - 1) / 2).saturating_sub(1)
}

fn run_zlema_batch(input: &[Real], period: usize, output: &mut [Real]) -> usize {
    let lag = (period - 1) / 2;
    let lookback = zlema_lookback(period);
    if input.len() <= lookback {
        return 0;
    }

    let per = 2.0 / (period as Real + 1.0);
    if lag == 0 {
        let mut value = input[0];
        output[0] = value;
        let mut out_index = 1usize;
        for &sample in &input[1..] {
            value = (sample - value).mul_add(per, value);
            output[out_index] = value;
            out_index += 1;
        }
        return out_index;
    }

    let mut value = input[lag - 1];
    output[0] = value;
    let mut out_index = 1usize;
    for index in lag..input.len() {
        let current = input[index];
        let lagged = input[index - lag];
        value = ((current + (current - lagged)) - value).mul_add(per, value);
        output[out_index] = value;
        out_index += 1;
    }

    out_index
}
