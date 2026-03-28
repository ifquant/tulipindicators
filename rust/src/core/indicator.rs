use crate::core::error::IndicatorError;
use crate::core::types::{IndicatorCategory, Real};

#[derive(Debug, Clone, Copy)]
pub struct IndicatorMetadata {
    pub name: &'static str,
    pub full_name: &'static str,
    pub category: IndicatorCategory,
    pub input_names: &'static [&'static str],
    pub option_names: &'static [&'static str],
    pub output_names: &'static [&'static str],
}

pub trait Indicator: Sync {
    fn metadata(&self) -> &'static IndicatorMetadata;
    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError>;
    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError>;
    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let metadata = self.metadata();
        let computed = self.run(inputs, options)?;
        validate_output_slices(metadata, outputs, metadata.output_names.len())?;
        validate_computed_outputs(metadata, &computed)?;
        for (output_index, values) in computed.iter().enumerate() {
            ensure_output_len(
                metadata,
                outputs[output_index].len(),
                values.len(),
                output_index,
            )?;
            outputs[output_index][..values.len()].copy_from_slice(values);
        }
        Ok(computed.first().map_or(0, Vec::len))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        let _ = options;
        Ok(None)
    }
}

pub trait IndicatorStream {
    fn metadata(&self) -> &'static IndicatorMetadata;
    fn progress(&self) -> usize;
    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError>;
}

pub fn output_len_for_input(input_len: usize, lookback: usize) -> usize {
    input_len.saturating_sub(lookback)
}

pub fn validate_output_slices(
    metadata: &IndicatorMetadata,
    outputs: &[&mut [Real]],
    expected: usize,
) -> Result<(), IndicatorError> {
    if outputs.len() != expected {
        return Err(IndicatorError::WrongOutputCount {
            indicator: metadata.name,
            expected,
            actual: outputs.len(),
        });
    }
    Ok(())
}

pub fn validate_computed_outputs(
    metadata: &IndicatorMetadata,
    computed: &[Vec<Real>],
) -> Result<(), IndicatorError> {
    let expected = metadata.output_names.len();
    if computed.len() != expected {
        return Err(IndicatorError::InternalInvariant {
            indicator: metadata.name,
            reason: "indicator returned an unexpected number of output series",
        });
    }
    Ok(())
}

pub fn ensure_output_len(
    metadata: &IndicatorMetadata,
    actual: usize,
    expected: usize,
    output_index: usize,
) -> Result<(), IndicatorError> {
    if actual < expected {
        return Err(IndicatorError::OutputTooSmall {
            indicator: metadata.name,
            output_index,
            expected,
            actual,
        });
    }
    Ok(())
}
