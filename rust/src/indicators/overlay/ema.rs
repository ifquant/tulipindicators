use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "ema",
    full_name: "Exponential Moving Average",
    category: IndicatorCategory::Overlay,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["ema"],
};

#[derive(Debug, Clone, Copy)]
pub struct Ema;

impl Indicator for Ema {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let _ = parse_period(options, METADATA.name)?;
        Ok(0)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let period = parse_period(options, METADATA.name)?;
        let mut output = Vec::with_capacity(input.len());

        if input.is_empty() {
            return Ok(vec![output]);
        }

        let multiplier = 2.0 / (period as Real + 1.0);
        let mut value = input[0];
        output.push(value);

        for sample in &input[1..] {
            value = (*sample - value) * multiplier + value;
            output.push(value);
        }

        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let period = parse_period(options, METADATA.name)?;
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), input.len(), 0)?;

        if input.is_empty() {
            return Ok(0);
        }

        let multiplier = 2.0 / (period as Real + 1.0);
        let mut value = input[0];
        outputs[0][0] = value;

        for (dst, &sample) in outputs[0][1..input.len()].iter_mut().zip(input[1..].iter()) {
            value = (sample - value) * multiplier + value;
            *dst = value;
        }

        Ok(input.len())
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(EmaStream::new(options)?)))
    }
}

struct EmaStream {
    multiplier: Real,
    last: Option<Real>,
    progress: usize,
}

impl EmaStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(options, METADATA.name)?;
        Ok(Self {
            multiplier: 2.0 / (period as Real + 1.0),
            last: None,
            progress: 0,
        })
    }
}

impl IndicatorStream for EmaStream {
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
            let value = match self.last {
                Some(last) => (*sample - last) * self.multiplier + last,
                None => *sample,
            };
            self.last = Some(value);
            self.progress += 1;
            output.push(value);
        }

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
            let value = match self.last {
                Some(last) => (sample - last) * self.multiplier + last,
                None => sample,
            };
            self.last = Some(value);
            self.progress += 1;
            outputs[0][out_index] = value;
            out_index += 1;
        }

        Ok(out_index)
    }
}

fn parse_period(options: &[Real], indicator: &'static str) -> Result<usize, IndicatorError> {
    expect_option_count(indicator, options, 1)?;
    parse_usize_option(indicator, options, 0, "period", 1)
}
