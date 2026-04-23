//! Core indicator traits and metadata.

use crate::core::error::IndicatorError;
use crate::core::types::{IndicatorCategory, Real};

/// Static description of an indicator's public shape.
#[derive(Debug, Clone, Copy)]
pub struct IndicatorMetadata {
    /// Short registry name used by [`crate::registry::find`].
    pub name: &'static str,
    /// Human-readable name suitable for UIs and logs.
    pub full_name: &'static str,
    /// Coarse category used for grouping and discovery.
    pub category: IndicatorCategory,
    /// Expected input series names in call order.
    pub input_names: &'static [&'static str],
    /// Expected option names in call order.
    pub option_names: &'static [&'static str],
    /// Output series names in call order.
    pub output_names: &'static [&'static str],
}

/// Batch indicator interface.
///
/// Batch methods return owned output series by default. Use [`Indicator::run_in_place`]
/// when you want to supply the output buffers yourself.
pub trait Indicator: Sync {
    /// Return static metadata for this indicator.
    fn metadata(&self) -> &'static IndicatorMetadata;

    /// Return the number of leading samples required before the first output is ready.
    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError>;

    /// Compute all output series for the provided inputs and options.
    ///
    /// `inputs` contains one slice per input series, each aligned by index. `options`
    /// contains the indicator-specific parameters in metadata order. The returned value
    /// owns all computed output series.
    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError>;

    /// Compute a single-output indicator and return that output series.
    ///
    /// This is a convenience wrapper around [`Indicator::run`] for indicators with one
    /// output series. It still allocates owned output data.
    fn run_single(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
    ) -> Result<Vec<Real>, IndicatorError> {
        let metadata = self.metadata();
        validate_single_output(metadata)?;
        let mut computed = self.run(inputs, options)?;
        validate_computed_outputs(metadata, &computed)?;
        Ok(computed.pop().unwrap_or_default())
    }

    /// Compute the indicator and copy results into caller-provided output buffers.
    ///
    /// The output slice count must match the indicator metadata. Each output buffer must
    /// be large enough for the produced values. This path avoids allocating a fresh
    /// `Vec<Vec<Real>>`, but it still computes the batch output internally.
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

    /// Create a stateful stream implementation when the indicator supports one.
    ///
    /// Indicators that do not provide incremental streaming return `Ok(None)`.
    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        let _ = options;
        Ok(None)
    }
}

/// Incremental indicator interface.
///
/// Stream implementations consume aligned input columns and can return values one batch
/// at a time without rebuilding the whole history.
pub trait IndicatorStream {
    /// Return static metadata for the stream implementation.
    fn metadata(&self) -> &'static IndicatorMetadata;

    /// Return how many rows or samples have been processed so far.
    fn progress(&self) -> usize;

    /// Feed aligned input columns into the stream and return owned outputs.
    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError>;

    /// Convenience wrapper for single-output streams.
    ///
    /// This is the stream counterpart to [`Indicator::run_single`].
    fn feed_single(&mut self, inputs: &[&[Real]]) -> Result<Vec<Real>, IndicatorError> {
        let metadata = self.metadata();
        validate_single_output(metadata)?;
        let mut computed = self.feed(inputs)?;
        validate_computed_outputs(metadata, &computed)?;
        Ok(computed.pop().unwrap_or_default())
    }

    /// Feed the stream and copy the produced values into caller-provided buffers.
    fn feed_in_place(
        &mut self,
        inputs: &[&[Real]],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let metadata = self.metadata();
        let computed = self.feed(inputs)?;
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
}

/// Return the number of output rows produced for a given input length and lookback.
pub fn output_len_for_input(input_len: usize, lookback: usize) -> usize {
    input_len.saturating_sub(lookback)
}

/// Validate that the caller provided the expected number of output buffers.
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

/// Validate that the indicator is single-output before using a single-output helper.
pub fn validate_single_output(metadata: &IndicatorMetadata) -> Result<(), IndicatorError> {
    if metadata.output_names.len() != 1 {
        return Err(IndicatorError::WrongOutputCount {
            indicator: metadata.name,
            expected: 1,
            actual: metadata.output_names.len(),
        });
    }
    Ok(())
}

/// Validate that the computed output shape matches the indicator metadata.
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

/// Validate that an output buffer is large enough for the computed values.
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
