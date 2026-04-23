//! Hull Moving Average (`hma`) smooths price with weighted moving averages and a square-root period.
//!
//! The implementation materializes only the intermediate series needed by the WMA composition while
//! preserving the final one-output overlay contract.
use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};
use crate::indicators::shared::WmaState;

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "hma",
    full_name: "Hull Moving Average",
    category: IndicatorCategory::Overlay,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["hma"],
};

#[derive(Debug, Clone, Copy)]
pub struct Hma;

impl Indicator for Hma {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        let period_sqrt = sqrt_period(period);
        Ok(period + period_sqrt - 2)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let output_len = input.len().saturating_sub(self.lookback(options)?);
        if output_len == 0 {
            return Ok(vec![Vec::new()]);
        }

        let mut output = vec![0.0; output_len];
        let written = self.run_kernel(input, period, &mut output)?;
        debug_assert_eq!(written, output_len);
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
        let output_len = input.len().saturating_sub(self.lookback(options)?);
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;
        if output_len == 0 {
            return Ok(0);
        }
        self.run_kernel(input, period, &mut outputs[0][..output_len])
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(HmaStream::new(options)?)))
    }
}

impl Hma {
    fn run_kernel(
        &self,
        input: &[Real],
        period: usize,
        output: &mut [Real],
    ) -> Result<usize, IndicatorError> {
        let short_period = (period / 2).max(1);
        let period_sqrt = sqrt_period(period);
        let weights = (period * (period + 1) / 2) as Real;
        let weights_short = (short_period * (short_period + 1) / 2) as Real;
        let weights_sqrt = (period_sqrt * (period_sqrt + 1) / 2) as Real;

        let mut sum = 0.0;
        let mut weight_sum = 0.0;
        let mut short_sum = 0.0;
        let mut short_weight_sum = 0.0;
        let mut sqrt_sum = 0.0;
        let mut sqrt_weight_sum = 0.0;

        for (index, &sample) in input.iter().enumerate().take(period - 1) {
            weight_sum += sample * (index + 1) as Real;
            sum += sample;

            if index >= period - short_period {
                short_weight_sum += sample * (index + 1 - (period - short_period)) as Real;
                short_sum += sample;
            }
        }

        let mut sqrt_buffer = vec![0.0; period_sqrt];
        let mut sqrt_cursor = 0usize;
        let mut sqrt_len = 0usize;
        let mut out_index = 0usize;

        for index in (period - 1)..input.len() {
            let sample = input[index];
            weight_sum += sample * period as Real;
            sum += sample;

            short_weight_sum += sample * short_period as Real;
            short_sum += sample;

            let diff = 2.0 * (short_weight_sum / weights_short) - (weight_sum / weights);
            sqrt_weight_sum += diff * period_sqrt as Real;
            sqrt_sum += diff;

            sqrt_buffer[sqrt_cursor] = diff;
            sqrt_cursor = (sqrt_cursor + 1) % period_sqrt;
            if sqrt_len < period_sqrt {
                sqrt_len += 1;
            }

            if sqrt_len == period_sqrt {
                output[out_index] = sqrt_weight_sum / weights_sqrt;
                out_index += 1;
                sqrt_weight_sum -= sqrt_sum;
                sqrt_sum -= sqrt_buffer[sqrt_cursor];
            } else {
                sqrt_weight_sum -= sqrt_sum;
            }

            weight_sum -= sum;
            sum -= input[index + 1 - period];

            short_weight_sum -= short_sum;
            short_sum -= input[index + 1 - short_period];
        }

        Ok(out_index)
    }
}

struct HmaStream {
    progress: usize,
    short_wma: WmaState,
    long_wma: WmaState,
    final_wma: WmaState,
}

impl HmaStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        let short_period = (period / 2).max(1);
        let period_sqrt = sqrt_period(period);
        Ok(Self {
            progress: 0,
            short_wma: WmaState::new(short_period),
            long_wma: WmaState::new(period),
            final_wma: WmaState::new(period_sqrt),
        })
    }
}

impl IndicatorStream for HmaStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(input.len());

        for sample in input {
            let short = self.short_wma.feed(*sample);
            let long = self.long_wma.feed(*sample);

            if let (Some(short), Some(long)) = (short, long) {
                let diff = 2.0 * short - long;
                if let Some(value) = self.final_wma.feed(diff) {
                    output.push(value);
                }
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 1)
}

fn sqrt_period(period: usize) -> usize {
    (period as Real).sqrt() as usize
}
