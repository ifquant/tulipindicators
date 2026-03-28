use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
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
        let lag = (period - 1) / 2;
        let lookback = zlema_lookback(period);
        let mut output = Vec::with_capacity(input.len().saturating_sub(lookback));

        if input.len() <= lookback {
            return Ok(vec![output]);
        }

        let per = 2.0 / (period as Real + 1.0);
        if lag == 0 {
            let mut value = input[0];
            output.push(value);
            for &sample in &input[1..] {
                value = (sample - value) * per + value;
                output.push(value);
            }
            return Ok(vec![output]);
        }

        let mut value = input[lag - 1];
        output.push(value);

        for index in lag..input.len() {
            let current = input[index];
            let lagged = input[index - lag];
            value = ((current + (current - lagged)) - value) * per + value;
            output.push(value);
        }

        Ok(vec![output])
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
        let mut output = Vec::with_capacity(input.len());

        for &sample in input {
            if self.lag == 0 {
                let next = match self.value {
                    Some(value) => (sample - value) * self.multiplier + value,
                    None => sample,
                };
                self.value = Some(next);
                output.push(next);
                self.progress += 1;
                continue;
            }

            if self.len < self.lag {
                self.history[self.len] = sample;
                self.len += 1;
                if self.len == self.lag {
                    self.value = Some(sample);
                    output.push(sample);
                }
            } else {
                let lagged = self.history[self.cursor];
                self.history[self.cursor] = sample;
                self.cursor = (self.cursor + 1) % self.lag;
                let previous = self.value.expect("zlema value should be initialized");
                let next = ((sample + (sample - lagged)) - previous) * self.multiplier + previous;
                self.value = Some(next);
                output.push(next);
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 1)
}

fn zlema_lookback(period: usize) -> usize {
    ((period - 1) / 2).saturating_sub(1)
}
