use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
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
        let input = single_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        if input.len() < period {
            return Ok(vec![Vec::new()]);
        }

        let mut output = vec![0.0; input.len() - period + 1];
        let written = self.run_kernel(input, period, &mut output);
        debug_assert_eq!(written, output.len());
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
        let output_len = input.len().saturating_sub(period - 1);
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;
        if input.len() < period {
            return Ok(0);
        }
        Ok(self.run_kernel(input, period, &mut outputs[0][..output_len]))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(KamaStream::new(options)?)))
    }
}

impl Kama {
    fn run_kernel(&self, input: &[Real], period: usize, output: &mut [Real]) -> usize {
        let mut sum = 0.0;
        for index in 1..period {
            sum += (input[index] - input[index - 1]).abs();
        }

        let mut kama = input[period - 1];
        output[0] = kama;
        let mut out_index = 1usize;

        for index in period..input.len() {
            sum += (input[index] - input[index - 1]).abs();
            if index > period {
                sum -= (input[index - period] - input[index - period - 1]).abs();
            }

            let er = if sum != 0.0 {
                (input[index] - input[index - period]).abs() / sum
            } else {
                1.0
            };
            let alpha = er * (FAST_PER - SLOW_PER) + SLOW_PER;
            let sc = alpha * alpha;
            kama += sc * (input[index] - kama);
            output[out_index] = kama;
            out_index += 1;
        }

        out_index
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
        for sample in input {
            if let Some(previous) = self.last_input {
                let diff = (*sample - previous).abs();
                self.diff_sum += diff;
                self.diffs.push_back(diff);
                if self.diffs.len() > self.period {
                    self.diff_sum -= self.diffs.pop_front().unwrap_or(0.0);
                }
            }

            self.prices.push_back(*sample);
            if self.prices.len() > self.period + 1 {
                self.prices.pop_front();
            }

            if self.progress + 1 == self.period {
                self.value = Some(*sample);
                outputs[0][out_index] = *sample;
                out_index += 1;
            } else if self.progress + 1 > self.period {
                let oldest = *self.prices.front().unwrap_or(sample);
                let er = if self.diff_sum != 0.0 {
                    (*sample - oldest).abs() / self.diff_sum
                } else {
                    1.0
                };
                let alpha = er * (FAST_PER - SLOW_PER) + SLOW_PER;
                let sc = alpha * alpha;
                let current = self.value.unwrap_or(*sample);
                let next = current + sc * (*sample - current);
                self.value = Some(next);
                outputs[0][out_index] = next;
                out_index += 1;
            }

            self.last_input = Some(*sample);
            self.progress += 1;
        }

        Ok(out_index)
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 1)
}
