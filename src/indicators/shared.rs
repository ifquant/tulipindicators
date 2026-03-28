use crate::core::types::Real;

#[derive(Debug, Clone, Copy)]
pub struct EmaState {
    multiplier: Real,
    value: Option<Real>,
}

impl EmaState {
    pub fn new(multiplier: Real) -> Self {
        Self {
            multiplier,
            value: None,
        }
    }

    pub fn feed(&mut self, sample: Real) -> Real {
        let next = match self.value {
            Some(current) => (sample - current) * self.multiplier + current,
            None => sample,
        };
        self.value = Some(next);
        next
    }
}
