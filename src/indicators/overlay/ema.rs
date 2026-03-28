use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};

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
}

fn parse_period(options: &[Real], indicator: &'static str) -> Result<usize, IndicatorError> {
    if options.len() != 1 {
        return Err(IndicatorError::WrongOptionCount {
            indicator,
            expected: 1,
            actual: options.len(),
        });
    }

    let value = options[0];
    if !value.is_finite() || value < 1.0 || value.fract() != 0.0 {
        return Err(IndicatorError::InvalidOption {
            indicator,
            option: "period",
            value,
            reason: "expected a positive integer",
        });
    }

    Ok(value as usize)
}

fn single_input<'a>(
    indicator: &'static str,
    inputs: &'a [&'a [Real]],
) -> Result<&'a [Real], IndicatorError> {
    if inputs.len() != 1 {
        return Err(IndicatorError::WrongInputCount {
            indicator,
            expected: 1,
            actual: inputs.len(),
        });
    }

    Ok(inputs[0])
}
