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
        let mut output = vec![0.0; input.len()];
        let produced = run_ema_batch(input, 2.0 / (period as Real + 1.0), &mut output);
        debug_assert_eq!(produced, input.len());
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
        Ok(run_ema_batch(
            input,
            2.0 / (period as Real + 1.0),
            &mut outputs[0][..input.len()],
        ))
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
        let mut output = vec![0.0; input.len()];
        let produced = self.feed_in_place(inputs, &mut [&mut output])?;
        debug_assert_eq!(produced, input.len());
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
        if input.is_empty() {
            return Ok(0);
        }

        let mut input_ptr = input.as_ptr();
        let mut out_ptr = outputs[0].as_mut_ptr();
        let mut produced = 0usize;

        let mut value = match self.last {
            Some(last) => {
                let sample = unsafe { *input_ptr };
                unsafe {
                    *out_ptr = (sample - last) * self.multiplier + last;
                    value_from_ptr(out_ptr)
                }
            }
            None => {
                let sample = unsafe { *input_ptr };
                unsafe { *out_ptr = sample };
                sample
            }
        };
        self.last = Some(value);
        self.progress += 1;
        produced += 1;

        unsafe {
            input_ptr = input_ptr.add(1);
            out_ptr = out_ptr.add(1);
        }

        for _ in 1..input.len() {
            let sample = unsafe { *input_ptr };
            value = (sample - value) * self.multiplier + value;
            unsafe {
                *out_ptr = value;
                input_ptr = input_ptr.add(1);
                out_ptr = out_ptr.add(1);
            }
            self.last = Some(value);
            self.progress += 1;
            produced += 1;
        }

        Ok(produced)
    }
}

fn run_ema_batch(input: &[Real], multiplier: Real, output: &mut [Real]) -> usize {
    if input.is_empty() {
        return 0;
    }

    debug_assert!(output.len() >= input.len());

    let mut value = input[0];
    unsafe {
        let mut input_ptr = input.as_ptr().add(1);
        let mut out_ptr = output.as_mut_ptr();
        *out_ptr = value;
        out_ptr = out_ptr.add(1);

        for _ in 1..input.len() {
            let sample = *input_ptr;
            value = (sample - value) * multiplier + value;
            *out_ptr = value;
            input_ptr = input_ptr.add(1);
            out_ptr = out_ptr.add(1);
        }
    }

    input.len()
}

unsafe fn value_from_ptr(ptr: *const Real) -> Real {
    unsafe { *ptr }
}

fn parse_period(options: &[Real], indicator: &'static str) -> Result<usize, IndicatorError> {
    expect_option_count(indicator, options, 1)?;
    parse_usize_option(indicator, options, 0, "period", 1)
}
