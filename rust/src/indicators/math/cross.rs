use super::bool_to_real;
use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{double_input, expect_option_count};

const CROSSANY_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "crossany",
    full_name: "Crossany",
    category: IndicatorCategory::Math,
    input_names: &["real", "real"],
    option_names: &[],
    output_names: &["crossany"],
};

const CROSSOVER_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "crossover",
    full_name: "Crossover",
    category: IndicatorCategory::Math,
    input_names: &["real", "real"],
    option_names: &[],
    output_names: &["crossover"],
};

#[derive(Debug, Clone, Copy)]
pub struct CrossAny;

impl Indicator for CrossAny {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &CROSSANY_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        expect_option_count(CROSSANY_METADATA.name, options, 0)?;
        Ok(1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        expect_option_count(CROSSANY_METADATA.name, options, 0)?;
        let (left, right) = double_input(CROSSANY_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(left.len().saturating_sub(1));

        for index in 1..left.len() {
            let crossed = (left[index] > right[index] && left[index - 1] <= right[index - 1])
                || (left[index] < right[index] && left[index - 1] >= right[index - 1]);
            output.push(bool_to_real(crossed));
        }

        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        expect_option_count(CROSSANY_METADATA.name, options, 0)?;
        let (left, right) = double_input(CROSSANY_METADATA.name, inputs)?;
        let output_len = left.len().saturating_sub(1);
        validate_output_slices(&CROSSANY_METADATA, outputs, 1)?;
        ensure_output_len(&CROSSANY_METADATA, outputs[0].len(), output_len, 0)?;

        for (dst, index) in outputs[0][..output_len].iter_mut().zip(1..left.len()) {
            let crossed = (left[index] > right[index] && left[index - 1] <= right[index - 1])
                || (left[index] < right[index] && left[index - 1] >= right[index - 1]);
            *dst = bool_to_real(crossed);
        }

        Ok(output_len)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        expect_option_count(CROSSANY_METADATA.name, options, 0)?;
        Ok(Some(Box::new(CrossAnyStream {
            previous: None,
            progress: 0,
        })))
    }
}

struct CrossAnyStream {
    previous: Option<(Real, Real)>,
    progress: usize,
}

impl IndicatorStream for CrossAnyStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &CROSSANY_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (left, right) = double_input(CROSSANY_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(left.len());

        for (&lhs, &rhs) in left.iter().zip(right.iter()) {
            if let Some((prev_lhs, prev_rhs)) = self.previous {
                let crossed =
                    (lhs > rhs && prev_lhs <= prev_rhs) || (lhs < rhs && prev_lhs >= prev_rhs);
                output.push(bool_to_real(crossed));
            }
            self.previous = Some((lhs, rhs));
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Crossover;

impl Indicator for Crossover {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &CROSSOVER_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        expect_option_count(CROSSOVER_METADATA.name, options, 0)?;
        Ok(1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        expect_option_count(CROSSOVER_METADATA.name, options, 0)?;
        let (left, right) = double_input(CROSSOVER_METADATA.name, inputs)?;
        let output_len = left.len().saturating_sub(1);
        let mut output = vec![0.0; output_len];
        let produced = run_crossover_batch(left, right, &mut output);
        debug_assert_eq!(produced, output_len);
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        expect_option_count(CROSSOVER_METADATA.name, options, 0)?;
        let (left, right) = double_input(CROSSOVER_METADATA.name, inputs)?;
        let output_len = left.len().saturating_sub(1);
        validate_output_slices(&CROSSOVER_METADATA, outputs, 1)?;
        ensure_output_len(&CROSSOVER_METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_crossover_batch(
            left,
            right,
            &mut outputs[0][..output_len],
        ))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        expect_option_count(CROSSOVER_METADATA.name, options, 0)?;
        Ok(Some(Box::new(CrossoverStream {
            previous: None,
            progress: 0,
        })))
    }
}

fn run_crossover_batch(left: &[Real], right: &[Real], output: &mut [Real]) -> usize {
    let output_len = left.len().saturating_sub(1);
    if output_len == 0 {
        return 0;
    }

    debug_assert_eq!(right.len(), left.len());
    debug_assert!(output.len() >= output_len);

    // SAFETY: the loop reads adjacent pairs from equally sized `left` and
    // `right` slices and writes exactly `output_len` values into `output`.
    // The debug assertions above guarantee matching input lengths and enough
    // output capacity.
    unsafe {
        let mut left_ptr = left.as_ptr();
        let mut right_ptr = right.as_ptr();
        let mut out_ptr = output.as_mut_ptr();

        for _ in 0..output_len {
            let prev_left = *left_ptr;
            let prev_right = *right_ptr;
            left_ptr = left_ptr.add(1);
            right_ptr = right_ptr.add(1);
            let current_left = *left_ptr;
            let current_right = *right_ptr;
            *out_ptr = if current_left > current_right && prev_left <= prev_right {
                1.0
            } else {
                0.0
            };
            out_ptr = out_ptr.add(1);
        }
    }

    output_len
}

struct CrossoverStream {
    previous: Option<(Real, Real)>,
    progress: usize,
}

impl IndicatorStream for CrossoverStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &CROSSOVER_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (left, right) = double_input(CROSSOVER_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(left.len());

        for (&lhs, &rhs) in left.iter().zip(right.iter()) {
            if let Some((prev_lhs, prev_rhs)) = self.previous {
                output.push(if lhs > rhs && prev_lhs <= prev_rhs {
                    1.0
                } else {
                    0.0
                });
            }
            self.previous = Some((lhs, rhs));
            self.progress += 1;
        }

        Ok(vec![output])
    }
}
