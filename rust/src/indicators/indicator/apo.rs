use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, single_input};
use crate::indicators::shared::EmaState;

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "apo",
    full_name: "Absolute Price Oscillator",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["short_period", "long_period"],
    output_names: &["apo"],
};

#[derive(Debug, Clone, Copy)]
pub struct Apo;

impl Indicator for Apo {
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
        let produced = run_apo_batch(input, short_period, long_period, &mut output);
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
        Ok(run_apo_batch(
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
        Ok(Some(Box::new(ApoStream::new(options)?)))
    }
}

struct ApoStream {
    progress: usize,
    short_ema: EmaState,
    long_ema: EmaState,
}

impl ApoStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let (short_period, long_period) = parse_options(options)?;
        Ok(Self {
            progress: 0,
            short_ema: EmaState::new(ema_multiplier(short_period)),
            long_ema: EmaState::new(ema_multiplier(long_period)),
        })
    }
}

impl IndicatorStream for ApoStream {
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
            let short_ema = self.short_ema.feed(sample);
            let long_ema = self.long_ema.feed(sample);
            if self.progress >= 1 {
                outputs[0][out_index] = short_ema - long_ema;
                out_index += 1;
            }
            self.progress += 1;
        }

        Ok(out_index)
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

fn run_apo_batch(
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
        output[out_index] = short_ema - long_ema;
        out_index += 1;
    }

    out_index
}
