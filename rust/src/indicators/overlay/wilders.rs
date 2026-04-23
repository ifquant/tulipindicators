use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};
use crate::state::{validate_history_capacity, IndicatorState, RingHistory};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "wilders",
    full_name: "Wilders Smoothing",
    category: IndicatorCategory::Overlay,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["wilders"],
};

#[derive(Debug, Clone, Copy)]
pub struct Wilders;

impl Wilders {
    pub fn state(
        options: &[Real],
        history_capacity: usize,
    ) -> Result<WildersState, IndicatorError> {
        WildersState::new(options, history_capacity)
    }
}

impl Indicator for Wilders {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        Ok(period - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let output_len = input.len().saturating_sub(period - 1);
        let mut output = vec![0.0; output_len];
        let produced = run_wilders_batch(input, period, &mut output);
        debug_assert_eq!(produced, output.len());
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let output_len = input.len().saturating_sub(period - 1);
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_wilders_batch(
            input,
            period,
            &mut outputs[0][..output_len],
        ))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(WildersStream::new(options)?)))
    }
}

struct WildersStream {
    progress: usize,
    period: usize,
    warmup_sum: Real,
    value: Option<Real>,
}

impl WildersStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        Ok(Self {
            progress: 0,
            period,
            warmup_sum: 0.0,
            value: None,
        })
    }

    fn feed_sample(&mut self, sample: Real) -> Option<Real> {
        if self.progress < self.period {
            self.warmup_sum += sample;
            self.progress += 1;
            if self.progress == self.period {
                let value = self.warmup_sum / self.period as Real;
                self.value = Some(value);
                return Some(value);
            }
            return None;
        }

        let current = self.value.expect("wilders stream should be initialized");
        let next = (sample - current).mul_add(1.0 / self.period as Real, current);
        self.value = Some(next);
        self.progress += 1;
        Some(next)
    }
}

impl IndicatorStream for WildersStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(input.len());

        for &sample in input {
            if let Some(value) = self.feed_sample(sample) {
                output.push(value);
            }
        }

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

        let mut out_index = 0usize;
        for &sample in input {
            if let Some(value) = self.feed_sample(sample) {
                outputs[0][out_index] = value;
                out_index += 1;
            }
        }

        Ok(out_index)
    }
}

pub struct WildersState {
    period: usize,
    stream: WildersStream,
    history: RingHistory<Real>,
}

impl WildersState {
    pub fn new(options: &[Real], history_capacity: usize) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        validate_history_capacity(METADATA.name, history_capacity)?;
        Ok(Self {
            period,
            stream: WildersStream::new(options)?,
            history: RingHistory::new(history_capacity),
        })
    }
}

impl IndicatorState for WildersState {
    type Input = Real;
    type Output = Real;

    fn update(&mut self, input: Self::Input) -> Option<Self::Output> {
        let value = self.stream.feed_sample(input);
        if let Some(value) = value {
            self.history.push(value);
            Some(value)
        } else {
            None
        }
    }

    fn latest(&self) -> Option<Self::Output> {
        self.history.latest().cloned()
    }

    fn get(&self, index_from_latest: usize) -> Option<Self::Output> {
        self.history.get(index_from_latest).cloned()
    }

    fn len(&self) -> usize {
        self.history.len()
    }

    fn history_capacity(&self) -> usize {
        self.history.capacity()
    }

    fn reset(&mut self) {
        self.stream = WildersStream {
            progress: 0,
            period: self.period,
            warmup_sum: 0.0,
            value: None,
        };
        self.history.clear();
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 1)
}

fn run_wilders_batch(input: &[Real], period: usize, output: &mut [Real]) -> usize {
    if input.len() < period {
        return 0;
    }

    let per = 1.0 / period as Real;
    let mut sum = 0.0;
    unsafe {
        let mut input_ptr = input.as_ptr();
        for _ in 0..period {
            sum += *input_ptr;
            input_ptr = input_ptr.add(1);
        }

        let mut value = sum / period as Real;
        let mut out_ptr = output.as_mut_ptr();
        *out_ptr = value;
        out_ptr = out_ptr.add(1);

        for _ in period..input.len() {
            let sample = *input_ptr;
            value = (sample - value).mul_add(per, value);
            *out_ptr = value;
            input_ptr = input_ptr.add(1);
            out_ptr = out_ptr.add(1);
        }
    }

    output.len()
}
