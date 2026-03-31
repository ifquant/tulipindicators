use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};
use crate::state::{IndicatorState, RingHistory};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "rsi",
    full_name: "Relative Strength Index",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["rsi"],
};

#[derive(Debug, Clone, Copy)]
pub struct Rsi;

impl Rsi {
    pub fn state(options: &[Real], history_capacity: usize) -> Result<RsiState, IndicatorError> {
        RsiState::new(options, history_capacity)
    }
}

impl Indicator for Rsi {
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
        let produced = run_rsi_batch(input, period, &mut output);
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
        let output_len = input.len().saturating_sub(period);
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_rsi_batch(input, period, &mut outputs[0][..output_len]))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(RsiStream::new(options)?)))
    }
}

struct RsiStream {
    period: usize,
    progress: usize,
    last_input: Option<Real>,
    smooth_up: Real,
    smooth_down: Real,
}

impl RsiStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        Ok(Self {
            period: parse_period(options)?,
            progress: 0,
            last_input: None,
            smooth_up: 0.0,
            smooth_down: 0.0,
        })
    }

    fn update_one(&mut self, sample: Real) -> Option<Real> {
        let per = 1.0 / self.period as Real;
        match self.last_input {
            None => {
                self.last_input = Some(sample);
                self.progress += 1;
                None
            }
            Some(previous) => {
                let delta = sample - previous;
                let upward = delta.max(0.0);
                let downward = (-delta).max(0.0);

                let output = if self.progress <= self.period {
                    self.smooth_up += upward;
                    self.smooth_down += downward;

                    if self.progress == self.period {
                        self.smooth_up /= self.period as Real;
                        self.smooth_down /= self.period as Real;
                        Some(rsi_value(self.smooth_up, self.smooth_down))
                    } else {
                        None
                    }
                } else {
                    self.smooth_up = (upward - self.smooth_up).mul_add(per, self.smooth_up);
                    self.smooth_down = (downward - self.smooth_down).mul_add(per, self.smooth_down);
                    Some(rsi_value(self.smooth_up, self.smooth_down))
                };

                self.last_input = Some(sample);
                self.progress += 1;
                output
            }
        }
    }
}

impl IndicatorStream for RsiStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let mut output = Vec::new();

        for &sample in input {
            if let Some(value) = self.update_one(sample) {
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
        ensure_output_len(
            &METADATA,
            outputs[0].len(),
            input.len().saturating_sub(1),
            0,
        )?;

        let mut out_index = 0usize;
        for &sample in input {
            if let Some(value) = self.update_one(sample) {
                outputs[0][out_index] = value;
                out_index += 1;
            }
        }

        Ok(out_index)
    }
}

pub struct RsiState {
    period: usize,
    stream: RsiStream,
    history: RingHistory<Real>,
}

impl RsiState {
    pub fn new(options: &[Real], history_capacity: usize) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        Ok(Self {
            period,
            stream: RsiStream::new(options)?,
            history: RingHistory::new(history_capacity),
        })
    }
}

impl IndicatorState for RsiState {
    type Input = Real;
    type Output = Real;

    fn seed(&mut self, input: &[Self::Input]) -> Result<usize, IndicatorError> {
        let mut produced = 0usize;
        for &sample in input {
            if let Some(value) = self.update(sample) {
                produced += 1;
                let _ = value;
            }
        }
        Ok(produced)
    }

    fn update(&mut self, input: Self::Input) -> Option<Self::Output> {
        let output = self.stream.update_one(input);
        if let Some(value) = output {
            self.history.push(value);
        }
        output
    }

    fn latest(&self) -> Option<Self::Output> {
        self.history.latest()
    }

    fn get(&self, index_from_latest: usize) -> Option<Self::Output> {
        self.history.get(index_from_latest)
    }

    fn len(&self) -> usize {
        self.history.len()
    }

    fn history_capacity(&self) -> usize {
        self.history.capacity()
    }

    fn reset(&mut self) {
        self.stream = RsiStream {
            period: self.period,
            progress: 0,
            last_input: None,
            smooth_up: 0.0,
            smooth_down: 0.0,
        };
        self.history.clear();
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 1)
}

fn rsi_value(smooth_up: Real, smooth_down: Real) -> Real {
    let total = smooth_up + smooth_down;
    if total == 0.0 {
        0.0
    } else {
        100.0 * (smooth_up / total)
    }
}

fn run_rsi_batch(input: &[Real], period: usize, output: &mut [Real]) -> usize {
    if input.len() <= period {
        return 0;
    }

    let per = 1.0 / period as Real;
    let mut smooth_up = 0.0;
    let mut smooth_down = 0.0;

    for index in 1..=period {
        let delta = input[index] - input[index - 1];
        if delta > 0.0 {
            smooth_up += delta;
        } else {
            smooth_down += -delta;
        }
    }

    smooth_up /= period as Real;
    smooth_down /= period as Real;
    output[0] = rsi_value(smooth_up, smooth_down);

    let mut out_index = 1usize;
    for index in (period + 1)..input.len() {
        let delta = input[index] - input[index - 1];
        let upward = delta.max(0.0);
        let downward = (-delta).max(0.0);

        smooth_up = (upward - smooth_up).mul_add(per, smooth_up);
        smooth_down = (downward - smooth_down).mul_add(per, smooth_down);
        output[out_index] = rsi_value(smooth_up, smooth_down);
        out_index += 1;
    }

    out_index
}
