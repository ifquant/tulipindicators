use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, triple_input};
use crate::indicators::shared::RingSum;

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "ultosc",
    full_name: "Ultimate Oscillator",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low", "close"],
    option_names: &["short_period", "medium_period", "long_period"],
    output_names: &["ultosc"],
};

#[derive(Debug, Clone, Copy)]
pub struct UltOsc;

impl Indicator for UltOsc {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let (_, _, long_period) = parse_options(options)?;
        Ok(long_period)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close) = triple_input(METADATA.name, inputs)?;
        let mut output = vec![0.0; high.len().saturating_sub(self.lookback(options)?)];
        let output_len = run_ultosc_batch(high, low, close, options, &mut output)?;
        output.truncate(output_len);
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let (high, low, close) = triple_input(METADATA.name, inputs)?;
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(
            &METADATA,
            outputs[0].len(),
            high.len().saturating_sub(self.lookback(options)?),
            0,
        )?;
        run_ultosc_batch(high, low, close, options, outputs[0])
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(UltOscStream::new(options)?)))
    }
}

struct UltOscStream {
    progress: usize,
    previous_close: Option<Real>,
    bp_short: RingSum,
    bp_medium: RingSum,
    bp_long: RingSum,
    r_short: RingSum,
    r_medium: RingSum,
    r_long: RingSum,
}

impl UltOscStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let (short_period, medium_period, long_period) = parse_options(options)?;
        Ok(Self {
            progress: 0,
            previous_close: None,
            bp_short: RingSum::new(short_period),
            bp_medium: RingSum::new(medium_period),
            bp_long: RingSum::new(long_period),
            r_short: RingSum::new(short_period),
            r_medium: RingSum::new(medium_period),
            r_long: RingSum::new(long_period),
        })
    }
}

impl IndicatorStream for UltOscStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close) = triple_input(METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(high.len());

        for ((&high, &low), &close) in high.iter().zip(low.iter()).zip(close.iter()) {
            if let Some(previous_close) = self.previous_close {
                let true_low = low.min(previous_close);
                let true_high = high.max(previous_close);
                let bp = close - true_low;
                let range = true_high - true_low;

                self.bp_short.push(bp);
                self.bp_medium.push(bp);
                self.bp_long.push(bp);
                self.r_short.push(range);
                self.r_medium.push(range);
                self.r_long.push(range);

                if self.bp_long.is_full() {
                    let first = 4.0 * self.bp_short.sum / self.r_short.sum;
                    let second = 2.0 * self.bp_medium.sum / self.r_medium.sum;
                    let third = self.bp_long.sum / self.r_long.sum;
                    output.push((first + second + third) * 100.0 / 7.0);
                }
            }

            self.previous_close = Some(close);
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn run_ultosc_batch(
    high: &[Real],
    low: &[Real],
    close: &[Real],
    options: &[Real],
    output: &mut [Real],
) -> Result<usize, IndicatorError> {
    let (short_period, medium_period, long_period) = parse_options(options)?;
    if high.len() <= long_period {
        return Ok(0);
    }

    let mut bp_vals = vec![0.0; long_period];
    let mut r_vals = vec![0.0; long_period];
    let mut bp_long_sum = 0.0;
    let mut r_long_sum = 0.0;
    let mut bp_short_sum = 0.0;
    let mut bp_medium_sum = 0.0;
    let mut r_short_sum = 0.0;
    let mut r_medium_sum = 0.0;
    let mut ring_index = 0usize;
    let mut out_index = 0usize;

    for index in 1..high.len() {
        let true_low = low[index].min(close[index - 1]);
        let true_high = high[index].max(close[index - 1]);
        let bp = close[index] - true_low;
        let range = true_high - true_low;

        bp_short_sum += bp;
        bp_medium_sum += bp;
        r_short_sum += range;
        r_medium_sum += range;

        bp_long_sum += bp - bp_vals[ring_index];
        r_long_sum += range - r_vals[ring_index];
        bp_vals[ring_index] = bp;
        r_vals[ring_index] = range;

        if index > short_period {
            let short_index = (ring_index + long_period - short_period) % long_period;
            bp_short_sum -= bp_vals[short_index];
            r_short_sum -= r_vals[short_index];

            if index > medium_period {
                let medium_index = (ring_index + long_period - medium_period) % long_period;
                bp_medium_sum -= bp_vals[medium_index];
                r_medium_sum -= r_vals[medium_index];
            }
        }

        if index >= long_period {
            let first = 4.0 * bp_short_sum / r_short_sum;
            let second = 2.0 * bp_medium_sum / r_medium_sum;
            let third = bp_long_sum / r_long_sum;
            output[out_index] = (first + second + third) * 100.0 / 7.0;
            out_index += 1;
        }

        ring_index += 1;
        if ring_index == long_period {
            ring_index = 0;
        }
    }

    Ok(out_index)
}

fn parse_options(options: &[Real]) -> Result<(usize, usize, usize), IndicatorError> {
    expect_option_count(METADATA.name, options, 3)?;
    let short_period = parse_usize_option(METADATA.name, options, 0, "short_period", 1)?;
    let medium_period = parse_usize_option(METADATA.name, options, 1, "medium_period", 1)?;
    let long_period = parse_usize_option(METADATA.name, options, 2, "long_period", 1)?;

    if medium_period < short_period {
        return Err(IndicatorError::InvalidOption {
            indicator: METADATA.name,
            option: "medium_period",
            value: options[1],
            reason: "expected medium_period >= short_period",
        });
    }
    if long_period < medium_period {
        return Err(IndicatorError::InvalidOption {
            indicator: METADATA.name,
            option: "long_period",
            value: options[2],
            reason: "expected long_period >= medium_period",
        });
    }

    Ok((short_period, medium_period, long_period))
}
