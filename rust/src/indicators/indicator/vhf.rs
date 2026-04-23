//! Vertical Horizontal Filter (`vhf`) compares net price movement with accumulated absolute movement.
//!
//! The rolling high/low and movement sum state measure whether the input behaves more like a trend
//! or a choppy range.
use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};
use crate::indicators::shared::{ExtremaKind, MonotonicQueue};
use std::collections::VecDeque;

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "vhf",
    full_name: "Vertical Horizontal Filter",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["vhf"],
};

#[derive(Debug, Clone, Copy)]
pub struct Vhf;

impl Indicator for Vhf {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        parse_period(options)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let mut output = vec![0.0; input.len().saturating_sub(period)];
        let produced = run_vhf_batch(input, period, &mut output);
        output.truncate(produced);
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
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(
            &METADATA,
            outputs[0].len(),
            input.len().saturating_sub(period),
            0,
        )?;
        Ok(run_vhf_batch(input, period, outputs[0]))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(VhfStream::new(options)?)))
    }
}

struct VhfStream {
    period: usize,
    progress: usize,
    values: VecDeque<Real>,
    change_sum: Real,
    max_queue: MonotonicQueue,
    min_queue: MonotonicQueue,
}

impl VhfStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        Ok(Self {
            period: parse_period(options)?,
            progress: 0,
            values: VecDeque::new(),
            change_sum: 0.0,
            max_queue: MonotonicQueue::new(ExtremaKind::Max),
            min_queue: MonotonicQueue::new(ExtremaKind::Min),
        })
    }
}

impl IndicatorStream for VhfStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(input.len().saturating_sub(self.period));

        for &sample in input {
            let index = self.progress;
            if let Some(&previous) = self.values.back() {
                self.change_sum += (sample - previous).abs();
            }

            self.values.push_back(sample);
            self.max_queue.push(index, sample);
            self.min_queue.push(index, sample);

            if self.values.len() > self.period + 1 {
                let removed = self
                    .values
                    .pop_front()
                    .expect("vhf window should not be empty");
                let next = *self
                    .values
                    .front()
                    .expect("vhf window should retain at least one sample");
                self.change_sum -= (next - removed).abs();
            }

            if index >= self.period {
                let window_start = index + 1 - self.period;
                self.max_queue.evict_before(window_start);
                self.min_queue.evict_before(window_start);
                output.push(
                    (self.max_queue.front_value() - self.min_queue.front_value()).abs()
                        / self.change_sum,
                );
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn run_vhf_batch(input: &[Real], period: usize, output: &mut [Real]) -> usize {
    if input.len() <= period {
        return 0;
    }

    let mut values: VecDeque<Real> = VecDeque::with_capacity(period + 1);
    let mut change_sum = 0.0;
    let mut max_queue = MonotonicQueue::new(ExtremaKind::Max);
    let mut min_queue = MonotonicQueue::new(ExtremaKind::Min);
    let mut out_index = 0usize;

    for (index, &sample) in input.iter().enumerate() {
        if let Some(previous) = values.back().copied() {
            change_sum += (sample - previous).abs();
        }

        values.push_back(sample);
        max_queue.push(index, sample);
        min_queue.push(index, sample);

        if values.len() > period + 1 {
            let removed = values
                .pop_front()
                .expect("vhf batch window should not be empty");
            let next = *values
                .front()
                .expect("vhf batch window should retain one sample");
            change_sum -= (next - removed).abs();
        }

        if index >= period {
            let window_start = index + 1 - period;
            max_queue.evict_before(window_start);
            min_queue.evict_before(window_start);
            output[out_index] =
                (max_queue.front_value() - min_queue.front_value()).abs() / change_sum;
            out_index += 1;
        }
    }

    out_index
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 1)
}
