use crate::core::error::IndicatorError;

pub trait IndicatorState {
    type Input;
    type Output: Copy;

    fn seed(&mut self, input: &[Self::Input]) -> Result<usize, IndicatorError>;
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

#[derive(Debug, Clone)]
pub(crate) struct RingHistory<T: Copy> {
    buf: Vec<T>,
    capacity: usize,
    head: usize,
}

impl<T: Copy> RingHistory<T> {
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
            Some(self.buf[self.head])
        }
    }

    pub(crate) fn get(&self, index_from_latest: usize) -> Option<T> {
        if index_from_latest >= self.buf.len() {
            return None;
        }

        if self.buf.len() < self.capacity {
            let idx = self.buf.len() - 1 - index_from_latest;
            Some(self.buf[idx])
        } else {
            let idx = (self.head + self.capacity - index_from_latest) % self.capacity;
            Some(self.buf[idx])
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
