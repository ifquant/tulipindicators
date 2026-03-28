use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "mom",
    full_name: "Momentum",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["mom"],
};

#[derive(Debug, Clone, Copy)]
pub struct Mom;

impl Indicator for Mom {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        parse_period(options)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        if input.len() <= period {
            return Ok(vec![Vec::new()]);
        }

        let mut output = Vec::with_capacity(input.len() - period);
        for index in period..input.len() {
            output.push(input[index] - input[index - period]);
        }

        Ok(vec![output])
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        let period = parse_period(options)?;
        Ok(Some(Box::new(MomStream {
            period,
            buffer: vec![0.0; period],
            len: 0,
            cursor: 0,
            progress: 0,
        })))
    }
}

struct MomStream {
    period: usize,
    buffer: Vec<Real>,
    len: usize,
    cursor: usize,
    progress: usize,
}

impl IndicatorStream for MomStream {
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
            if self.len < self.period {
                self.buffer[self.len] = sample;
                self.len += 1;
            } else {
                let lagged = self.buffer[self.cursor];
                self.buffer[self.cursor] = sample;
                self.cursor = (self.cursor + 1) % self.period;
                output.push(sample - lagged);
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
