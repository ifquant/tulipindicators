use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};
use std::collections::VecDeque;

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "kama",
    full_name: "Kaufman Adaptive Moving Average",
    category: IndicatorCategory::Overlay,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["kama"],
};

const FAST_PER: Real = 2.0 / (2.0 + 1.0);
const SLOW_PER: Real = 2.0 / (30.0 + 1.0);

#[derive(Debug, Clone, Copy)]
pub struct Kama;

impl Indicator for Kama {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        Ok(period - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = KamaStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(KamaStream::new(options)?)))
    }
}

struct KamaStream {
    period: usize,
    progress: usize,
    diff_sum: Real,
    diffs: VecDeque<Real>,
    prices: VecDeque<Real>,
    last_input: Option<Real>,
    value: Option<Real>,
}

impl KamaStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        Ok(Self {
            period: parse_period(options)?,
            progress: 0,
            diff_sum: 0.0,
            diffs: VecDeque::new(),
            prices: VecDeque::new(),
            last_input: None,
            value: None,
        })
    }
}

impl IndicatorStream for KamaStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(input.len());

        for sample in input {
            if let Some(previous) = self.last_input {
                let diff = (*sample - previous).abs();
                self.diff_sum += diff;
                self.diffs.push_back(diff);
                if self.diffs.len() > self.period {
                    self.diff_sum -= self
                        .diffs
                        .pop_front()
                        .expect("diff queue should not be empty");
                }
            }

            self.prices.push_back(*sample);
            if self.prices.len() > self.period + 1 {
                self.prices.pop_front();
            }

            if self.progress + 1 == self.period {
                self.value = Some(*sample);
                output.push(*sample);
            } else if self.progress + 1 > self.period {
                let oldest = *self
                    .prices
                    .front()
                    .expect("price queue should contain the period lookback");
                let er = if self.diff_sum != 0.0 {
                    (*sample - oldest).abs() / self.diff_sum
                } else {
                    1.0
                };
                let sc = (er * (FAST_PER - SLOW_PER) + SLOW_PER).powi(2);
                let current = self.value.expect("kama value should be initialized");
                let next = current + sc * (*sample - current);
                self.value = Some(next);
                output.push(next);
            }

            self.last_input = Some(*sample);
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 1)
}
