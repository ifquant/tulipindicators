use super::{parse_nonnegative_period, parse_positive_period};
use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::single_input;

const DECAY_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "decay",
    full_name: "Linear Decay",
    category: IndicatorCategory::Math,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["decay"],
};

const EDECAY_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "edecay",
    full_name: "Exponential Decay",
    category: IndicatorCategory::Math,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["edecay"],
};

const LAG_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "lag",
    full_name: "Lag",
    category: IndicatorCategory::Math,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["lag"],
};

#[derive(Debug, Clone, Copy)]
pub struct Decay;

#[derive(Debug, Clone, Copy)]
pub struct EDecay;

#[derive(Debug, Clone, Copy)]
pub struct Lag;

impl Indicator for Decay {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &DECAY_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let _ = parse_positive_period(DECAY_METADATA.name, options)?;
        Ok(0)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(DECAY_METADATA.name, inputs)?;
        let period = parse_positive_period(DECAY_METADATA.name, options)?;
        let mut output = vec![0.0; input.len()];
        let produced = run_decay_batch(input, 1.0 / period as Real, false, &mut output);
        debug_assert_eq!(produced, input.len());
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(DECAY_METADATA.name, inputs)?;
        let period = parse_positive_period(DECAY_METADATA.name, options)?;
        validate_output_slices(&DECAY_METADATA, outputs, 1)?;
        ensure_output_len(&DECAY_METADATA, outputs[0].len(), input.len(), 0)?;
        Ok(run_decay_batch(
            input,
            1.0 / period as Real,
            false,
            &mut outputs[0][..input.len()],
        ))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        let period = parse_positive_period(DECAY_METADATA.name, options)?;
        Ok(Some(Box::new(DecayStream {
            metadata: &DECAY_METADATA,
            scale: 1.0 / period as Real,
            last: None,
            progress: 0,
            exponential: false,
        })))
    }
}

impl Indicator for EDecay {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &EDECAY_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let _ = parse_positive_period(EDECAY_METADATA.name, options)?;
        Ok(0)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(EDECAY_METADATA.name, inputs)?;
        let period = parse_positive_period(EDECAY_METADATA.name, options)?;
        let mut output = vec![0.0; input.len()];
        let produced = run_decay_batch(input, 1.0 - 1.0 / period as Real, true, &mut output);
        debug_assert_eq!(produced, input.len());
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(EDECAY_METADATA.name, inputs)?;
        let period = parse_positive_period(EDECAY_METADATA.name, options)?;
        validate_output_slices(&EDECAY_METADATA, outputs, 1)?;
        ensure_output_len(&EDECAY_METADATA, outputs[0].len(), input.len(), 0)?;
        Ok(run_decay_batch(
            input,
            1.0 - 1.0 / period as Real,
            true,
            &mut outputs[0][..input.len()],
        ))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        let period = parse_positive_period(EDECAY_METADATA.name, options)?;
        Ok(Some(Box::new(DecayStream {
            metadata: &EDECAY_METADATA,
            scale: 1.0 - 1.0 / period as Real,
            last: None,
            progress: 0,
            exponential: true,
        })))
    }
}

struct DecayStream {
    metadata: &'static IndicatorMetadata,
    scale: Real,
    last: Option<Real>,
    progress: usize,
    exponential: bool,
}

fn run_decay_batch(input: &[Real], scale: Real, exponential: bool, output: &mut [Real]) -> usize {
    if let Some((&first, rest)) = input.split_first() {
        output[0] = first;
        let mut last = first;
        for (dst, &sample) in output[1..].iter_mut().zip(rest.iter()) {
            let decayed = if exponential {
                last * scale
            } else {
                last - scale
            };
            last = sample.max(decayed);
            *dst = last;
        }
        input.len()
    } else {
        0
    }
}

impl IndicatorStream for DecayStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        self.metadata
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(self.metadata.name, inputs)?;
        let mut output = Vec::with_capacity(input.len());

        for &sample in input {
            let next = match self.last {
                Some(last) => {
                    let decayed = if self.exponential {
                        last * self.scale
                    } else {
                        last - self.scale
                    };
                    sample.max(decayed)
                }
                None => sample,
            };
            self.last = Some(next);
            self.progress += 1;
            output.push(next);
        }

        Ok(vec![output])
    }
}

impl Indicator for Lag {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &LAG_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        parse_nonnegative_period(LAG_METADATA.name, options)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(LAG_METADATA.name, inputs)?;
        let period = parse_nonnegative_period(LAG_METADATA.name, options)?;
        if period == 0 {
            return Ok(vec![input.to_vec()]);
        }
        if input.len() <= period {
            return Ok(vec![Vec::new()]);
        }
        Ok(vec![input[..input.len() - period].to_vec()])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(LAG_METADATA.name, inputs)?;
        let period = parse_nonnegative_period(LAG_METADATA.name, options)?;
        let output_len = input.len().saturating_sub(period);
        validate_output_slices(&LAG_METADATA, outputs, 1)?;
        ensure_output_len(&LAG_METADATA, outputs[0].len(), output_len, 0)?;

        if period == 0 {
            outputs[0][..input.len()].copy_from_slice(input);
            return Ok(input.len());
        }
        if input.len() <= period {
            return Ok(0);
        }

        outputs[0][..output_len].copy_from_slice(&input[..output_len]);
        Ok(output_len)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        let period = parse_nonnegative_period(LAG_METADATA.name, options)?;
        Ok(Some(Box::new(LagStream {
            period,
            buffer: vec![0.0; period.max(1)],
            len: 0,
            cursor: 0,
            progress: 0,
        })))
    }
}

struct LagStream {
    period: usize,
    buffer: Vec<Real>,
    len: usize,
    cursor: usize,
    progress: usize,
}

impl IndicatorStream for LagStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &LAG_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(LAG_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(input.len());

        for &sample in input {
            if self.period == 0 {
                output.push(sample);
            } else if self.len < self.period {
                self.buffer[self.len] = sample;
                self.len += 1;
            } else {
                output.push(self.buffer[self.cursor]);
                self.buffer[self.cursor] = sample;
                self.cursor = (self.cursor + 1) % self.period;
            }
            self.progress += 1;
        }

        Ok(vec![output])
    }
}
