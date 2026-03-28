use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};
use std::collections::VecDeque;

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "dpo",
    full_name: "Detrended Price Oscillator",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["dpo"],
};

#[derive(Debug, Clone, Copy)]
pub struct Dpo;

impl Indicator for Dpo {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        Ok(period - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = DpoStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(DpoStream::new(options)?)))
    }
}

struct DpoStream {
    period: usize,
    back: usize,
    progress: usize,
    window: VecDeque<Real>,
    sum: Real,
}

impl DpoStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        Ok(Self {
            period,
            back: period / 2 + 1,
            progress: 0,
            window: VecDeque::with_capacity(period),
            sum: 0.0,
        })
    }
}

impl IndicatorStream for DpoStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let mut output =
            Vec::with_capacity(input.len().saturating_sub(self.period.saturating_sub(1)));

        for &sample in input {
            self.window.push_back(sample);
            self.sum += sample;

            if self.window.len() > self.period {
                let removed = self
                    .window
                    .pop_front()
                    .expect("dpo window should not be empty");
                self.sum -= removed;
            }

            if self.window.len() == self.period {
                let lag_index = self.period - 1 - self.back;
                output.push(self.window[lag_index] - self.sum / self.period as Real);
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 2)
}
