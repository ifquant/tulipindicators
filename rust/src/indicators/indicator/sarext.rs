//! Extended Parabolic SAR (`sarext`) exposes TA-Lib's additional long/short acceleration controls.
//!
//! The implementation keeps the richer reversal state local to the kernel while presenting the same
//! single-output indicator contract as ordinary PSAR.
use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{double_input, expect_option_count};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "sarext",
    full_name: "Parabolic SAR - Extended",
    category: IndicatorCategory::Overlay,
    input_names: &["high", "low"],
    option_names: &[
        "start_value",
        "offset_on_reverse",
        "acceleration_init_long",
        "acceleration_long",
        "acceleration_max_long",
        "acceleration_init_short",
        "acceleration_short",
        "acceleration_max_short",
    ],
    output_names: &["sarext"],
};

#[derive(Debug, Clone, Copy)]
pub struct Sarext;

#[derive(Debug, Clone, Copy)]
struct SarextOptions {
    start_value: Real,
    offset_on_reverse: Real,
    acceleration_init_long: Real,
    acceleration_long: Real,
    acceleration_max_long: Real,
    acceleration_init_short: Real,
    acceleration_short: Real,
    acceleration_max_short: Real,
}

impl Indicator for Sarext {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let _ = parse_options(options)?;
        Ok(1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low) = double_input(METADATA.name, inputs)?;
        let parsed = parse_options(options)?;
        let output_len = high.len().saturating_sub(1);
        let mut output = vec![0.0; output_len];
        let produced = run_sarext_batch(high, low, parsed, &mut output);
        debug_assert_eq!(produced, output_len);
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let (high, low) = double_input(METADATA.name, inputs)?;
        let parsed = parse_options(options)?;
        let output_len = high.len().saturating_sub(1);
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_sarext_batch(
            high,
            low,
            parsed,
            &mut outputs[0][..output_len],
        ))
    }
}

fn parse_options(options: &[Real]) -> Result<SarextOptions, IndicatorError> {
    expect_option_count(METADATA.name, options, 8)?;

    let parsed = SarextOptions {
        start_value: options[0],
        offset_on_reverse: options[1],
        acceleration_init_long: options[2],
        acceleration_long: options[3],
        acceleration_max_long: options[4],
        acceleration_init_short: options[5],
        acceleration_short: options[6],
        acceleration_max_short: options[7],
    };

    validate_finite("start_value", parsed.start_value)?;
    validate_nonnegative("offset_on_reverse", parsed.offset_on_reverse)?;
    validate_nonnegative("acceleration_init_long", parsed.acceleration_init_long)?;
    validate_nonnegative("acceleration_long", parsed.acceleration_long)?;
    validate_nonnegative("acceleration_max_long", parsed.acceleration_max_long)?;
    validate_nonnegative("acceleration_init_short", parsed.acceleration_init_short)?;
    validate_nonnegative("acceleration_short", parsed.acceleration_short)?;
    validate_nonnegative("acceleration_max_short", parsed.acceleration_max_short)?;

    Ok(parsed)
}

fn validate_finite(option: &'static str, value: Real) -> Result<(), IndicatorError> {
    if !value.is_finite() {
        return Err(IndicatorError::InvalidOption {
            indicator: METADATA.name,
            option,
            value,
            reason: "expected a finite value",
        });
    }
    Ok(())
}

fn validate_nonnegative(option: &'static str, value: Real) -> Result<(), IndicatorError> {
    if !value.is_finite() || value < 0.0 {
        return Err(IndicatorError::InvalidOption {
            indicator: METADATA.name,
            option,
            value,
            reason: "expected a finite value >= 0",
        });
    }
    Ok(())
}

fn run_sarext_batch(
    high: &[Real],
    low: &[Real],
    mut options: SarextOptions,
    output: &mut [Real],
) -> usize {
    if high.len() < 2 {
        return 0;
    }

    if options.acceleration_init_long > options.acceleration_max_long {
        options.acceleration_init_long = options.acceleration_max_long;
    }
    if options.acceleration_long > options.acceleration_max_long {
        options.acceleration_long = options.acceleration_max_long;
    }
    if options.acceleration_init_short > options.acceleration_max_short {
        options.acceleration_init_short = options.acceleration_max_short;
    }
    if options.acceleration_short > options.acceleration_max_short {
        options.acceleration_short = options.acceleration_max_short;
    }

    let mut is_long = if options.start_value == 0.0 {
        let up_move = high[1] - high[0];
        let down_move = low[0] - low[1];
        !(down_move > up_move && down_move > 0.0)
    } else {
        options.start_value > 0.0
    };

    let mut today = 1usize;
    let mut new_high = high[today - 1];
    let mut new_low = low[today - 1];
    let mut af_long = options.acceleration_init_long;
    let mut af_short = options.acceleration_init_short;

    let mut ep;
    let mut sar;
    if options.start_value == 0.0 {
        if is_long {
            ep = high[today];
            sar = new_low;
        } else {
            ep = low[today];
            sar = new_high;
        }
    } else if options.start_value > 0.0 {
        ep = high[today];
        sar = options.start_value;
    } else {
        ep = low[today];
        sar = options.start_value.abs();
    }

    new_high = high[today];
    new_low = low[today];

    let mut outputs = output.iter_mut();
    while today < high.len() {
        let prev_high = new_high;
        let prev_low = new_low;
        new_high = high[today];
        new_low = low[today];
        today += 1;
        let slot = outputs
            .next()
            .expect("sarext output buffer should match lookback-adjusted input length");

        if is_long {
            if new_low <= sar {
                is_long = false;
                sar = ep;

                if sar < prev_high {
                    sar = prev_high;
                }
                if sar < new_high {
                    sar = new_high;
                }

                if options.offset_on_reverse != 0.0 {
                    sar += sar * options.offset_on_reverse;
                }
                *slot = -sar;

                af_short = options.acceleration_init_short;
                ep = new_low;
                sar = (ep - sar).mul_add(af_short, sar);
                if sar < prev_high {
                    sar = prev_high;
                }
                if sar < new_high {
                    sar = new_high;
                }
            } else {
                *slot = sar;

                if new_high > ep {
                    ep = new_high;
                    af_long =
                        (af_long + options.acceleration_long).min(options.acceleration_max_long);
                }

                sar = (ep - sar).mul_add(af_long, sar);
                if sar > prev_low {
                    sar = prev_low;
                }
                if sar > new_low {
                    sar = new_low;
                }
            }
        } else if new_high >= sar {
            is_long = true;
            sar = ep;

            if sar > prev_low {
                sar = prev_low;
            }
            if sar > new_low {
                sar = new_low;
            }

            if options.offset_on_reverse != 0.0 {
                sar -= sar * options.offset_on_reverse;
            }
            *slot = sar;

            af_long = options.acceleration_init_long;
            ep = new_high;
            sar = (ep - sar).mul_add(af_long, sar);
            if sar > prev_low {
                sar = prev_low;
            }
            if sar > new_low {
                sar = new_low;
            }
        } else {
            *slot = -sar;

            if new_low < ep {
                ep = new_low;
                af_short =
                    (af_short + options.acceleration_short).min(options.acceleration_max_short);
            }

            sar = (ep - sar).mul_add(af_short, sar);
            if sar < prev_high {
                sar = prev_high;
            }
            if sar < new_high {
                sar = new_high;
            }
        }
    }

    output.len()
}
