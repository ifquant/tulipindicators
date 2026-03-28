use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{
    double_input, expect_option_count, parse_usize_option, single_input,
};
use crate::indicators::shared::{ExtremaKind, MonotonicQueue, RingSum};

fn parse_positive_period(
    indicator: &'static str,
    options: &[Real],
) -> Result<usize, IndicatorError> {
    expect_option_count(indicator, options, 1)?;
    parse_usize_option(indicator, options, 0, "period", 1)
}

fn parse_nonnegative_period(
    indicator: &'static str,
    options: &[Real],
) -> Result<usize, IndicatorError> {
    expect_option_count(indicator, options, 1)?;
    parse_usize_option(indicator, options, 0, "period", 0)
}

fn bool_to_real(value: bool) -> Real {
    if value {
        1.0
    } else {
        0.0
    }
}

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

const MAX_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "max",
    full_name: "Maximum In Period",
    category: IndicatorCategory::Math,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["max"],
};

const MIN_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "min",
    full_name: "Minimum In Period",
    category: IndicatorCategory::Math,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["min"],
};

const SUM_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "sum",
    full_name: "Sum Over Period",
    category: IndicatorCategory::Math,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["sum"],
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

    // Use the same adjacent-element walk as the C implementation to keep the
    // hot loop simple and avoid heavier vectorized compare codegen.
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

#[derive(Debug, Clone, Copy)]
pub struct Decay;

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

#[derive(Debug, Clone, Copy)]
pub struct EDecay;

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

#[derive(Debug, Clone, Copy)]
pub struct Lag;

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

#[derive(Debug, Clone, Copy)]
pub struct Sum;

impl Indicator for Sum {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &SUM_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_positive_period(SUM_METADATA.name, options)?;
        Ok(period - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(SUM_METADATA.name, inputs)?;
        let period = parse_positive_period(SUM_METADATA.name, options)?;
        let lookback = period - 1;
        let mut output = Vec::with_capacity(input.len().saturating_sub(lookback));

        if input.len() <= lookback {
            return Ok(vec![output]);
        }

        let mut state = RingSum::new(period);
        for &sample in input {
            state.push(sample);
            if state.is_full() {
                output.push(state.sum);
            }
        }

        Ok(vec![output])
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        let period = parse_positive_period(SUM_METADATA.name, options)?;
        Ok(Some(Box::new(SumStream {
            state: RingSum::new(period),
            progress: 0,
        })))
    }
}

struct SumStream {
    state: RingSum,
    progress: usize,
}

impl IndicatorStream for SumStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &SUM_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(SUM_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(input.len());

        for &sample in input {
            self.state.push(sample);
            if self.state.is_full() {
                output.push(self.state.sum);
            }
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn run_extrema(
    metadata: &'static IndicatorMetadata,
    inputs: &[&[Real]],
    options: &[Real],
    kind: ExtremaKind,
) -> Result<Vec<Vec<Real>>, IndicatorError> {
    let input = single_input(metadata.name, inputs)?;
    let period = parse_positive_period(metadata.name, options)?;
    let lookback = period - 1;
    let mut output = Vec::with_capacity(input.len().saturating_sub(lookback));

    if input.len() <= lookback {
        return Ok(vec![output]);
    }

    let mut queue = MonotonicQueue::new(kind);
    for (index, &sample) in input.iter().enumerate() {
        queue.push(index, sample);
        if index + 1 >= period {
            queue.evict_before(index + 1 - period);
            output.push(queue.front_value());
        }
    }

    Ok(vec![output])
}

struct ExtremaStream {
    metadata: &'static IndicatorMetadata,
    period: usize,
    index: usize,
    queue: MonotonicQueue,
}

impl IndicatorStream for ExtremaStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        self.metadata
    }

    fn progress(&self) -> usize {
        self.index
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(self.metadata.name, inputs)?;
        let mut output = Vec::with_capacity(input.len());

        for &sample in input {
            self.queue.push(self.index, sample);
            self.index += 1;
            if self.index >= self.period {
                self.queue.evict_before(self.index - self.period);
                output.push(self.queue.front_value());
            }
        }

        Ok(vec![output])
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Max;

impl Indicator for Max {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &MAX_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_positive_period(MAX_METADATA.name, options)?;
        Ok(period - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        run_extrema(&MAX_METADATA, inputs, options, ExtremaKind::Max)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        let period = parse_positive_period(MAX_METADATA.name, options)?;
        Ok(Some(Box::new(ExtremaStream {
            metadata: &MAX_METADATA,
            period,
            index: 0,
            queue: MonotonicQueue::new(ExtremaKind::Max),
        })))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Min;

impl Indicator for Min {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &MIN_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_positive_period(MIN_METADATA.name, options)?;
        Ok(period - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        run_extrema(&MIN_METADATA, inputs, options, ExtremaKind::Min)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        let period = parse_positive_period(MIN_METADATA.name, options)?;
        Ok(Some(Box::new(ExtremaStream {
            metadata: &MIN_METADATA,
            period,
            index: 0,
            queue: MonotonicQueue::new(ExtremaKind::Min),
        })))
    }
}
