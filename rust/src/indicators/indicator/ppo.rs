use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, single_input};
use crate::indicators::shared::EmaState;
use crate::state::{validate_history_capacity, IndicatorState, RingHistory};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "ppo",
    full_name: "Percentage Price Oscillator",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["short_period", "long_period"],
    output_names: &["ppo"],
};

#[derive(Debug, Clone, Copy)]
pub struct Ppo;

impl Ppo {
    pub fn state(options: &[Real], history_capacity: usize) -> Result<PpoState, IndicatorError> {
        PpoState::new(options, history_capacity)
    }
}

impl Indicator for Ppo {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let _ = parse_options(options)?;
        Ok(1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let (short_period, long_period) = parse_options(options)?;
        let mut output = vec![0.0; input.len().saturating_sub(1)];
        let produced = run_ppo_batch(input, short_period, long_period, &mut output);
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
        let (short_period, long_period) = parse_options(options)?;
        let output_len = input.len().saturating_sub(1);
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_ppo_batch(
            input,
            short_period,
            long_period,
            &mut outputs[0][..output_len],
        ))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(PpoStream::new(options)?)))
    }
}

struct PpoStream {
    progress: usize,
    short_ema: EmaState,
    long_ema: EmaState,
}

impl PpoStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let (short_period, long_period) = parse_options(options)?;
        Ok(Self {
            progress: 0,
            short_ema: EmaState::new(ema_multiplier(short_period)),
            long_ema: EmaState::new(ema_multiplier(long_period)),
        })
    }

    fn update_one(&mut self, sample: Real) -> Option<Real> {
        let short_ema = self.short_ema.feed(sample);
        let long_ema = self.long_ema.feed(sample);
        let output = if self.progress >= 1 {
            Some(100.0 * (short_ema - long_ema) / long_ema)
        } else {
            None
        };
        self.progress += 1;
        output
    }
}

impl IndicatorStream for PpoStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let mut output = vec![0.0; input.len()];
        let mut outputs = [&mut output[..]];
        let produced = self.feed_in_place(inputs, &mut outputs)?;
        output.truncate(produced);

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
            if let Some(value) = self.update_one(sample) {
                outputs[0][out_index] = value;
                out_index += 1;
            }
        }

        Ok(out_index)
    }
}

pub struct PpoState {
    short_period: usize,
    long_period: usize,
    stream: PpoStream,
    history: RingHistory<Real>,
}

impl PpoState {
    pub fn new(options: &[Real], history_capacity: usize) -> Result<Self, IndicatorError> {
        let (short_period, long_period) = parse_options(options)?;
        validate_history_capacity(METADATA.name, history_capacity)?;
        Ok(Self {
            short_period,
            long_period,
            stream: PpoStream::new(options)?,
            history: RingHistory::new(history_capacity),
        })
    }
}

impl IndicatorState for PpoState {
    type Input = Real;
    type Output = Real;

    fn update(&mut self, input: Self::Input) -> Option<Self::Output> {
        let value = self.stream.update_one(input);
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
        self.stream = PpoStream {
            progress: 0,
            short_ema: EmaState::new(ema_multiplier(self.short_period)),
            long_ema: EmaState::new(ema_multiplier(self.long_period)),
        };
        self.history.clear();
    }
}

fn parse_options(options: &[Real]) -> Result<(usize, usize), IndicatorError> {
    expect_option_count(METADATA.name, options, 2)?;
    let short_period = parse_period(options[0], "short_period", 1)?;
    let long_period = parse_period(options[1], "long_period", 2)?;

    if long_period < short_period {
        return Err(IndicatorError::InvalidOption {
            indicator: METADATA.name,
            option: "long_period",
            value: options[1],
            reason: "expected long_period >= short_period",
        });
    }

    Ok((short_period, long_period))
}

fn parse_period(
    value: Real,
    option: &'static str,
    minimum: usize,
) -> Result<usize, IndicatorError> {
    if !value.is_finite() || value < minimum as Real || value.fract() != 0.0 {
        return Err(IndicatorError::InvalidOption {
            indicator: METADATA.name,
            option,
            value,
            reason: "expected a positive integer",
        });
    }

    Ok(value as usize)
}

fn ema_multiplier(period: usize) -> Real {
    2.0 / (period as Real + 1.0)
}

fn run_ppo_batch(
    input: &[Real],
    short_period: usize,
    long_period: usize,
    output: &mut [Real],
) -> usize {
    if input.len() <= 1 {
        return 0;
    }

    let short_per = ema_multiplier(short_period);
    let long_per = ema_multiplier(long_period);
    let mut short_ema = input[0];
    let mut long_ema = input[0];
    let mut out_index = 0usize;

    for &sample in &input[1..] {
        short_ema = (sample - short_ema).mul_add(short_per, short_ema);
        long_ema = (sample - long_ema).mul_add(long_per, long_ema);
        output[out_index] = 100.0 * (short_ema - long_ema) / long_ema;
        out_index += 1;
    }

    out_index
}
