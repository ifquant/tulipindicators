//! Stateful incremental indicator APIs.

use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::Real;
use crate::registry;

/// Bounded history wrapper for incremental indicator state.
///
/// Implementations store recent outputs and expose indexed access through
/// [`IndicatorState::latest`] and [`IndicatorState::get`].
pub trait IndicatorState {
    /// Per-sample input type accepted by [`IndicatorState::update`].
    type Input;
    /// Output row type returned by this state.
    type Output: Clone;

    /// Seed the state by replaying a slice of inputs.
    ///
    /// The default implementation forwards each item to [`IndicatorState::update`].
    fn seed(&mut self, input: &[Self::Input]) -> Result<usize, IndicatorError>
    where
        Self::Input: Copy,
    {
        let mut produced = 0usize;
        for &sample in input {
            if self.update(sample).is_some() {
                produced += 1;
            }
        }
        Ok(produced)
    }

    /// Update the state with one new input sample.
    fn update(&mut self, input: Self::Input) -> Option<Self::Output>;
    /// Return the most recently produced output, if any.
    fn latest(&self) -> Option<Self::Output>;
    /// Return an older output by offset from the latest item.
    fn get(&self, index_from_latest: usize) -> Option<Self::Output>;
    /// Return the number of stored outputs.
    fn len(&self) -> usize;
    /// Return the fixed history capacity for this state.
    fn history_capacity(&self) -> usize;
    /// Clear any stored history and internal progress.
    fn reset(&mut self);

    /// Return `true` when no outputs have been produced yet.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Return `true` after the state has produced at least one output.
    fn is_ready(&self) -> bool {
        self.latest().is_some()
    }
}

/// Factory for runtime-selected incremental state.
pub trait IndicatorStateFactory: Indicator {
    /// Create a dynamic state object with bounded history.
    ///
    /// `history_capacity` must be greater than zero.
    fn dynamic_state(
        &'static self,
        options: &[Real],
        history_capacity: usize,
    ) -> Result<DynamicIndicatorState, IndicatorError>
    where
        Self: Sized,
    {
        DynamicIndicatorState::new(self, options, history_capacity)
    }
}

impl<T: Indicator + ?Sized> IndicatorStateFactory for T {}

/// Ring buffer used by [`DynamicIndicatorState`].
#[derive(Debug, Clone)]
pub(crate) struct RingHistory<T: Clone> {
    buf: Vec<T>,
    capacity: usize,
    head: usize,
}

impl<T: Clone> RingHistory<T> {
    pub(crate) fn new(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
            capacity,
            head: 0,
        }
    }

    pub(crate) fn push(&mut self, value: T) {
        if self.buf.len() < self.capacity {
            self.buf.push(value);
            self.head = self.buf.len() - 1;
        } else {
            self.head = (self.head + 1) % self.capacity;
            self.buf[self.head] = value;
        }
    }

    pub(crate) fn latest(&self) -> Option<&T> {
        if self.buf.is_empty() {
            None
        } else {
            Some(&self.buf[self.head])
        }
    }

    pub(crate) fn get(&self, index_from_latest: usize) -> Option<&T> {
        if index_from_latest >= self.buf.len() {
            return None;
        }

        if self.buf.len() < self.capacity {
            let idx = self.buf.len() - 1 - index_from_latest;
            Some(&self.buf[idx])
        } else {
            let idx = (self.head + self.capacity - index_from_latest) % self.capacity;
            Some(&self.buf[idx])
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.buf.len()
    }

    pub(crate) fn capacity(&self) -> usize {
        self.capacity
    }

    pub(crate) fn clear(&mut self) {
        self.buf.clear();
        self.head = 0;
    }
}

/// Validate the requested history capacity.
///
/// The capacity must be greater than zero because incremental state needs at least one
/// slot to retain the most recent output row.
pub(crate) fn validate_history_capacity(
    indicator: &'static str,
    history_capacity: usize,
) -> Result<(), IndicatorError> {
    if history_capacity == 0 {
        return Err(IndicatorError::InvalidOption {
            indicator,
            option: "history_capacity",
            value: 0.0,
            reason: "must be greater than zero",
        });
    }
    Ok(())
}

/// Runtime-selected incremental indicator state.
///
/// This type can be constructed from a registry name or from a typed indicator factory.
/// It keeps a bounded output history and can seed data as either input columns or rows.
pub struct DynamicIndicatorState {
    indicator: &'static dyn Indicator,
    metadata: &'static IndicatorMetadata,
    options: Vec<Real>,
    backend: DynamicStateBackend,
    history: RingHistory<Vec<Real>>,
    output_scratch: Vec<Vec<Real>>,
}

enum DynamicStateBackend {
    Stream(Box<dyn IndicatorStream>),
    Batch(BatchState),
}

struct BatchState {
    input_history: Vec<Vec<Real>>,
    produced: usize,
}

impl DynamicIndicatorState {
    /// Create the internal batch or stream backend for a given indicator.
    fn build_backend(
        indicator: &'static dyn Indicator,
        metadata: &'static IndicatorMetadata,
        options: &[Real],
    ) -> Result<DynamicStateBackend, IndicatorError> {
        Ok(match indicator.create_stream(options)? {
            Some(stream) => DynamicStateBackend::Stream(stream),
            None => DynamicStateBackend::Batch(BatchState {
                input_history: vec![Vec::new(); metadata.input_names.len()],
                produced: 0,
            }),
        })
    }

    /// Create a new dynamic state with the given indicator and options.
    ///
    /// `history_capacity` must be greater than zero.
    pub fn new(
        indicator: &'static dyn Indicator,
        options: &[Real],
        history_capacity: usize,
    ) -> Result<Self, IndicatorError> {
        let metadata = indicator.metadata();
        validate_history_capacity(metadata.name, history_capacity)?;
        let backend = Self::build_backend(indicator, metadata, options)?;
        let output_scratch = vec![vec![0.0; 1]; metadata.output_names.len()];

        Ok(Self {
            indicator,
            metadata,
            options: options.to_vec(),
            backend,
            history: RingHistory::new(history_capacity),
            output_scratch,
        })
    }

    /// Create a dynamic state by registry name.
    ///
    /// `history_capacity` must be greater than zero.
    pub fn from_name(
        name: &str,
        options: &[Real],
        history_capacity: usize,
    ) -> Result<Self, IndicatorError> {
        let indicator = registry::find(name).ok_or_else(|| IndicatorError::UnknownIndicator {
            name: name.to_string(),
        })?;
        Self::new(indicator, options, history_capacity)
    }

    /// Return the indicator metadata for this state.
    pub fn metadata(&self) -> &'static IndicatorMetadata {
        self.metadata
    }

    /// Seed the state from aligned input columns.
    ///
    /// Reseeding is atomic: existing progress and history are replaced only after the
    /// new inputs validate and seed successfully.
    pub fn seed_columns(&mut self, inputs: &[&[Real]]) -> Result<usize, IndicatorError> {
        validate_dynamic_input_columns(self.metadata, inputs)?;
        let mut seeded = Self::new(self.indicator, &self.options, self.history.capacity())?;
        let produced = seeded.seed_columns_fresh(inputs)?;
        *self = seeded;
        Ok(produced)
    }

    fn seed_columns_fresh(&mut self, inputs: &[&[Real]]) -> Result<usize, IndicatorError> {
        match &mut self.backend {
            DynamicStateBackend::Stream(_) => {
                let input_len = inputs.first().map_or(0, |input| input.len());
                let mut row = vec![0.0; self.metadata.input_names.len()];
                let mut produced = 0usize;

                for row_index in 0..input_len {
                    for (input_index, values) in inputs.iter().enumerate() {
                        row[input_index] = values[row_index];
                    }

                    if self.update(&row)?.is_some() {
                        produced += 1;
                    }
                }

                Ok(produced)
            }
            DynamicStateBackend::Batch(batch) => {
                batch.input_history = inputs.iter().map(|input| input.to_vec()).collect();
                batch.produced = 0;
                self.recompute_batch_backend()
            }
        }
    }

    /// Seed the state from one row per sample.
    ///
    /// Reseeding is atomic: existing progress and history are replaced only after every
    /// row validates and the new data seeds successfully.
    pub fn seed_rows(&mut self, rows: &[Vec<Real>]) -> Result<usize, IndicatorError> {
        for row in rows {
            validate_dynamic_input_row(self.metadata, row)?;
        }
        let mut seeded = Self::new(self.indicator, &self.options, self.history.capacity())?;
        let produced = seeded.seed_rows_fresh(rows)?;
        *self = seeded;
        Ok(produced)
    }

    fn seed_rows_fresh(&mut self, rows: &[Vec<Real>]) -> Result<usize, IndicatorError> {
        match &mut self.backend {
            DynamicStateBackend::Stream(_) => {
                let mut produced = 0usize;
                for row in rows {
                    if self.update(row)?.is_some() {
                        produced += 1;
                    }
                }

                Ok(produced)
            }
            DynamicStateBackend::Batch(batch) => {
                batch.input_history =
                    vec![Vec::with_capacity(rows.len()); self.metadata.input_names.len()];
                for row in rows {
                    for (input_index, value) in row.iter().copied().enumerate() {
                        batch.input_history[input_index].push(value);
                    }
                }
                batch.produced = 0;
                self.recompute_batch_backend()
            }
        }
    }

    /// Update the state with one aligned input row.
    pub fn update(&mut self, input: &[Real]) -> Result<Option<Vec<Real>>, IndicatorError> {
        validate_dynamic_input_row(self.metadata, input)?;

        match &mut self.backend {
            DynamicStateBackend::Stream(stream) => {
                let input_refs: Vec<&[Real]> = input.iter().map(std::slice::from_ref).collect();
                let mut output_refs: Vec<&mut [Real]> = self
                    .output_scratch
                    .iter_mut()
                    .map(Vec::as_mut_slice)
                    .collect();
                let produced = stream.feed_in_place(&input_refs, &mut output_refs)?;

                match produced {
                    0 => Ok(None),
                    1 => {
                        let output: Vec<Real> =
                            self.output_scratch.iter().map(|values| values[0]).collect();
                        self.history.push(output.clone());
                        Ok(Some(output))
                    }
                    _ => Err(IndicatorError::InternalInvariant {
                        indicator: self.metadata.name,
                        reason: "single-row update produced more than one output row",
                    }),
                }
            }
            DynamicStateBackend::Batch(batch) => {
                for (input_index, value) in input.iter().copied().enumerate() {
                    batch.input_history[input_index].push(value);
                }

                let produced = self.recompute_batch_backend()?;
                if produced == 0 {
                    Ok(None)
                } else {
                    Ok(self.latest_ref().map(<[Real]>::to_vec))
                }
            }
        }
    }

    /// Return the latest stored output as a borrowed slice.
    ///
    /// The returned slice borrows this state's internal history. Use [`Self::latest`]
    /// when the value must outlive the next mutable call.
    pub fn latest_ref(&self) -> Option<&[Real]> {
        self.history.latest().map(Vec::as_slice)
    }

    /// Return the latest stored output as an owned vector.
    pub fn latest(&self) -> Option<Vec<Real>> {
        self.latest_ref().map(<[Real]>::to_vec)
    }

    /// Return a prior stored output as a borrowed slice.
    ///
    /// The returned slice borrows this state's internal history. Use [`Self::get`] when
    /// the value must outlive the next mutable call.
    pub fn get_ref(&self, index_from_latest: usize) -> Option<&[Real]> {
        self.history.get(index_from_latest).map(Vec::as_slice)
    }

    /// Return a prior stored output as an owned vector.
    pub fn get(&self, index_from_latest: usize) -> Option<Vec<Real>> {
        self.get_ref(index_from_latest).map(<[Real]>::to_vec)
    }

    /// Return the number of stored output rows.
    pub fn len(&self) -> usize {
        self.history.len()
    }

    /// Return the fixed history capacity.
    pub fn history_capacity(&self) -> usize {
        self.history.capacity()
    }

    /// Return `true` when no outputs have been stored yet.
    pub fn is_empty(&self) -> bool {
        self.history.len() == 0
    }

    /// Return `true` after the state has produced at least one output row.
    pub fn is_ready(&self) -> bool {
        self.latest_ref().is_some()
    }

    /// Clear the state and rebuild the underlying backend.
    pub fn reset(&mut self) -> Result<(), IndicatorError> {
        self.backend = Self::build_backend(self.indicator, self.metadata, &self.options)?;
        self.history.clear();
        Ok(())
    }

    fn recompute_batch_backend(&mut self) -> Result<usize, IndicatorError> {
        let DynamicStateBackend::Batch(batch) = &mut self.backend else {
            return Err(IndicatorError::InternalInvariant {
                indicator: self.metadata.name,
                reason: "batch recompute was requested for a stream-backed state",
            });
        };

        let input_refs: Vec<&[Real]> = batch.input_history.iter().map(Vec::as_slice).collect();
        let computed = self.indicator.run(&input_refs, &self.options)?;
        if computed.is_empty() {
            return Ok(0);
        }

        let output_len = computed[0].len();
        if output_len < batch.produced {
            return Err(IndicatorError::InternalInvariant {
                indicator: self.metadata.name,
                reason: "batch fallback output length regressed across updates",
            });
        }

        if batch.produced == 0 {
            for row_index in 0..output_len {
                let row = computed
                    .iter()
                    .map(|output| output[row_index])
                    .collect::<Vec<Real>>();
                self.history.push(row);
            }
        } else if output_len > batch.produced {
            let row = computed
                .iter()
                .map(|output| output[output_len - 1])
                .collect::<Vec<Real>>();
            self.history.push(row);
        }

        let produced_now = output_len.saturating_sub(batch.produced);
        batch.produced = output_len;
        Ok(produced_now)
    }
}

fn validate_dynamic_input_columns(
    metadata: &IndicatorMetadata,
    inputs: &[&[Real]],
) -> Result<(), IndicatorError> {
    if inputs.len() != metadata.input_names.len() {
        return Err(IndicatorError::WrongInputCount {
            indicator: metadata.name,
            expected: metadata.input_names.len(),
            actual: inputs.len(),
        });
    }

    if let Some(first) = inputs.first() {
        let expected = first.len();
        for (input_index, input) in inputs.iter().enumerate().skip(1) {
            if input.len() != expected {
                return Err(IndicatorError::InputLengthMismatch {
                    indicator: metadata.name,
                    expected,
                    actual: input.len(),
                    input_index,
                });
            }
        }
    }

    Ok(())
}

fn validate_dynamic_input_row(
    metadata: &IndicatorMetadata,
    input: &[Real],
) -> Result<(), IndicatorError> {
    if input.len() != metadata.input_names.len() {
        return Err(IndicatorError::WrongInputCount {
            indicator: metadata.name,
            expected: metadata.input_names.len(),
            actual: input.len(),
        });
    }
    Ok(())
}
