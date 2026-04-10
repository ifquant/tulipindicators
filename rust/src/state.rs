use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::Real;
use crate::registry;

pub trait IndicatorState {
    type Input;
    type Output: Clone;

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

    fn update(&mut self, input: Self::Input) -> Option<Self::Output>;
    fn latest(&self) -> Option<Self::Output>;
    fn get(&self, index_from_latest: usize) -> Option<Self::Output>;
    fn len(&self) -> usize;
    fn history_capacity(&self) -> usize;
    fn reset(&mut self);

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn is_ready(&self) -> bool {
        self.latest().is_some()
    }
}

pub trait IndicatorStateFactory: Indicator {
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

#[derive(Debug, Clone)]
pub(crate) struct RingHistory<T: Clone> {
    buf: Vec<T>,
    capacity: usize,
    head: usize,
}

impl<T: Clone> RingHistory<T> {
    pub(crate) fn new(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity.max(1)),
            capacity: capacity.max(1),
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

    pub(crate) fn latest(&self) -> Option<T> {
        if self.buf.is_empty() {
            None
        } else {
            Some(self.buf[self.head].clone())
        }
    }

    pub(crate) fn get(&self, index_from_latest: usize) -> Option<T> {
        if index_from_latest >= self.buf.len() {
            return None;
        }

        if self.buf.len() < self.capacity {
            let idx = self.buf.len() - 1 - index_from_latest;
            Some(self.buf[idx].clone())
        } else {
            let idx = (self.head + self.capacity - index_from_latest) % self.capacity;
            Some(self.buf[idx].clone())
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
    pub fn new(
        indicator: &'static dyn Indicator,
        options: &[Real],
        history_capacity: usize,
    ) -> Result<Self, IndicatorError> {
        let metadata = indicator.metadata();
        let backend = match indicator.create_stream(options)? {
            Some(stream) => DynamicStateBackend::Stream(stream),
            None => DynamicStateBackend::Batch(BatchState {
                input_history: vec![Vec::new(); metadata.input_names.len()],
                produced: 0,
            }),
        };
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

    pub fn from_name(
        name: &str,
        options: &[Real],
        history_capacity: usize,
    ) -> Result<Self, IndicatorError> {
        let indicator = registry::find(name).ok_or(IndicatorError::InternalInvariant {
            indicator: "registry",
            reason: "indicator name was not found",
        })?;
        Self::new(indicator, options, history_capacity)
    }

    pub fn metadata(&self) -> &'static IndicatorMetadata {
        self.metadata
    }

    pub fn seed_columns(&mut self, inputs: &[&[Real]]) -> Result<usize, IndicatorError> {
        validate_dynamic_input_columns(self.metadata, inputs)?;
        self.history.clear();

        if matches!(self.backend, DynamicStateBackend::Stream(_)) {
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

            return Ok(produced);
        }

        let DynamicStateBackend::Batch(batch) = &mut self.backend else {
            unreachable!();
        };
        batch.input_history = inputs.iter().map(|input| input.to_vec()).collect();
        batch.produced = 0;
        self.recompute_batch_backend()
    }

    pub fn seed_rows(&mut self, rows: &[Vec<Real>]) -> Result<usize, IndicatorError> {
        self.history.clear();

        if matches!(self.backend, DynamicStateBackend::Stream(_)) {
            let mut produced = 0usize;
            for row in rows {
                if self.update(row)?.is_some() {
                    produced += 1;
                }
            }
            return Ok(produced);
        }

        let DynamicStateBackend::Batch(batch) = &mut self.backend else {
            unreachable!();
        };
        batch.input_history = vec![Vec::with_capacity(rows.len()); self.metadata.input_names.len()];
        for row in rows {
            validate_dynamic_input_row(self.metadata, row)?;
            for (input_index, value) in row.iter().copied().enumerate() {
                batch.input_history[input_index].push(value);
            }
        }
        batch.produced = 0;
        self.recompute_batch_backend()
    }

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
                    Ok(self.latest())
                }
            }
        }
    }

    pub fn latest(&self) -> Option<Vec<Real>> {
        self.history.latest()
    }

    pub fn get(&self, index_from_latest: usize) -> Option<Vec<Real>> {
        self.history.get(index_from_latest)
    }

    pub fn len(&self) -> usize {
        self.history.len()
    }

    pub fn history_capacity(&self) -> usize {
        self.history.capacity()
    }

    pub fn is_empty(&self) -> bool {
        self.history.len() == 0
    }

    pub fn is_ready(&self) -> bool {
        self.history.latest().is_some()
    }

    pub fn reset(&mut self) {
        self.backend = match self
            .indicator
            .create_stream(&self.options)
            .expect("indicator state reset should not fail after construction")
        {
            Some(stream) => DynamicStateBackend::Stream(stream),
            None => DynamicStateBackend::Batch(BatchState {
                input_history: vec![Vec::new(); self.metadata.input_names.len()],
                produced: 0,
            }),
        };
        self.history.clear();
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
