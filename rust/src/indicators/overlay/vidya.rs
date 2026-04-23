//! Variable Index Dynamic Average (`vidya`) adapts EMA smoothing with a volatility index.
//!
//! The kernel tracks short and long volatility estimates and uses their ratio to scale the smoothing
//! factor applied to the prior output.
use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};
use crate::indicators::shared::RollingStatsState;

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "vidya",
    full_name: "Variable Index Dynamic Average",
    category: IndicatorCategory::Overlay,
    input_names: &["real"],
    option_names: &["short_period", "long_period", "alpha"],
    output_names: &["vidya"],
};

#[derive(Debug, Clone, Copy)]
pub struct Vidya;

impl Indicator for Vidya {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let (_, long_period, _) = parse_options(options)?;
        Ok(long_period - 2)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let (short_period, long_period, alpha) = parse_options(options)?;
        let output_len = input.len().saturating_sub(long_period - 2);
        if output_len == 0 {
            return Ok(vec![Vec::new()]);
        }

        let mut output = vec![0.0; output_len];
        let written = self.run_kernel(input, short_period, long_period, alpha, &mut output)?;
        debug_assert_eq!(written, output.len());
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let (short_period, long_period, alpha) = parse_options(options)?;
        let output_len = input.len().saturating_sub(long_period - 2);
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;
        if output_len == 0 {
            return Ok(0);
        }
        self.run_kernel(
            input,
            short_period,
            long_period,
            alpha,
            &mut outputs[0][..output_len],
        )
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(VidyaStream::new(options)?)))
    }
}

impl Vidya {
    fn run_kernel(
        &self,
        input: &[Real],
        short_period: usize,
        long_period: usize,
        alpha: Real,
        output: &mut [Real],
    ) -> Result<usize, IndicatorError> {
        let short_div = 1.0 / short_period as Real;
        let long_div = 1.0 / long_period as Real;
        let mut short_sum = 0.0;
        let mut short_sum2 = 0.0;
        let mut long_sum = 0.0;
        let mut long_sum2 = 0.0;

        for (index, &sample) in input.iter().take(long_period).enumerate() {
            long_sum += sample;
            long_sum2 += sample * sample;
            if index >= long_period - short_period {
                short_sum += sample;
                short_sum2 += sample * sample;
            }
        }

        let mut value = input[long_period - 2];
        output[0] = value;
        let mut out_index = 1usize;

        if long_period - 1 < input.len() {
            let short_stddev =
                (short_sum2 * short_div - (short_sum * short_div) * (short_sum * short_div)).sqrt();
            let long_stddev =
                (long_sum2 * long_div - (long_sum * long_div) * (long_sum * long_div)).sqrt();
            let mut k = short_stddev / long_stddev;
            if k.is_nan() {
                k = 0.0;
            }
            k *= alpha;
            value += (input[long_period - 1] - value) * k;
            output[out_index] = value;
            out_index += 1;
        }

        for index in long_period..input.len() {
            let sample = input[index];
            long_sum += sample;
            long_sum2 += sample * sample;
            short_sum += sample;
            short_sum2 += sample * sample;

            let old_long = input[index - long_period];
            long_sum -= old_long;
            long_sum2 -= old_long * old_long;

            let old_short = input[index - short_period];
            short_sum -= old_short;
            short_sum2 -= old_short * old_short;

            let short_stddev =
                (short_sum2 * short_div - (short_sum * short_div) * (short_sum * short_div)).sqrt();
            let long_stddev =
                (long_sum2 * long_div - (long_sum * long_div) * (long_sum * long_div)).sqrt();
            let mut k = short_stddev / long_stddev;
            if k.is_nan() {
                k = 0.0;
            }
            k *= alpha;
            value += (sample - value) * k;
            output[out_index] = value;
            out_index += 1;
        }

        Ok(out_index)
    }
}

struct VidyaStream {
    long_period: usize,
    alpha: Real,
    progress: usize,
    short_stats: RollingStatsState,
    long_stats: RollingStatsState,
    value: Option<Real>,
}

impl VidyaStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let (short_period, long_period, alpha) = parse_options(options)?;
        Ok(Self {
            long_period,
            alpha,
            progress: 0,
            short_stats: RollingStatsState::new(short_period),
            long_stats: RollingStatsState::new(long_period),
            value: None,
        })
    }
}

impl IndicatorStream for VidyaStream {
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
            let short = self.short_stats.feed(*sample);
            let long = self.long_stats.feed(*sample);

            if self.progress + 1 == self.long_period - 1 {
                self.value = Some(*sample);
                output.push(*sample);
            } else if self.progress + 1 >= self.long_period {
                let short_stddev = short.map(|stats| stats.variance.sqrt()).unwrap_or(0.0);
                let long_stddev = long.map(|stats| stats.variance.sqrt()).unwrap_or(0.0);
                let mut k = short_stddev / long_stddev;
                if k.is_nan() {
                    k = 0.0;
                }
                k *= self.alpha;

                let current = self.value.unwrap_or(*sample);
                let next = (*sample - current) * k + current;
                self.value = Some(next);
                output.push(next);
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn parse_options(options: &[Real]) -> Result<(usize, usize, Real), IndicatorError> {
    expect_option_count(METADATA.name, options, 3)?;
    let short_period = parse_usize_option(METADATA.name, options, 0, "short_period", 1)?;
    let long_period = parse_usize_option(METADATA.name, options, 1, "long_period", 2)?;
    let alpha = options[2];

    if long_period < short_period {
        return Err(IndicatorError::InvalidOption {
            indicator: METADATA.name,
            option: "long_period",
            value: options[1],
            reason: "expected long_period >= short_period",
        });
    }

    if !(0.0..=1.0).contains(&alpha) {
        return Err(IndicatorError::InvalidOption {
            indicator: METADATA.name,
            option: "alpha",
            value: alpha,
            reason: "expected a value between 0 and 1",
        });
    }

    Ok((short_period, long_period, alpha))
}
