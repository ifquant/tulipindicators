use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "t3",
    full_name: "T3 Moving Average",
    category: IndicatorCategory::Overlay,
    input_names: &["real"],
    option_names: &["period", "vfactor"],
    output_names: &["t3"],
};

#[derive(Debug, Clone, Copy)]
pub struct T3;

impl Indicator for T3 {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let (period, _) = parse_options(options)?;
        Ok(6 * (period - 1))
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let (period, vfactor) = parse_options(options)?;
        let lookback = 6 * (period - 1);
        let mut output = vec![0.0; input.len().saturating_sub(lookback)];
        let produced = run_t3_batch(input, period, vfactor, &mut output);
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
        let (period, vfactor) = parse_options(options)?;
        let output_len = input.len().saturating_sub(6 * (period - 1));
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_t3_batch(
            input,
            period,
            vfactor,
            &mut outputs[0][..output_len],
        ))
    }
}

pub(crate) fn run_t3_batch(
    input: &[Real],
    period: usize,
    vfactor: Real,
    output: &mut [Real],
) -> usize {
    let lookback = 6 * (period - 1);
    if input.len() <= lookback {
        return 0;
    }

    let k = 2.0 / (period as Real + 1.0);
    let one_minus_k = 1.0 - k;

    let mut today = 0usize;
    let mut temp = input[today];
    today += 1;
    for _ in 0..(period - 1) {
        temp += input[today];
        today += 1;
    }
    let mut e1 = temp / period as Real;

    temp = e1;
    for _ in 0..(period - 1) {
        e1 = k.mul_add(input[today], one_minus_k * e1);
        today += 1;
        temp += e1;
    }
    let mut e2 = temp / period as Real;

    temp = e2;
    for _ in 0..(period - 1) {
        e1 = k.mul_add(input[today], one_minus_k * e1);
        today += 1;
        e2 = k.mul_add(e1, one_minus_k * e2);
        temp += e2;
    }
    let mut e3 = temp / period as Real;

    temp = e3;
    for _ in 0..(period - 1) {
        e1 = k.mul_add(input[today], one_minus_k * e1);
        today += 1;
        e2 = k.mul_add(e1, one_minus_k * e2);
        e3 = k.mul_add(e2, one_minus_k * e3);
        temp += e3;
    }
    let mut e4 = temp / period as Real;

    temp = e4;
    for _ in 0..(period - 1) {
        e1 = k.mul_add(input[today], one_minus_k * e1);
        today += 1;
        e2 = k.mul_add(e1, one_minus_k * e2);
        e3 = k.mul_add(e2, one_minus_k * e3);
        e4 = k.mul_add(e3, one_minus_k * e4);
        temp += e4;
    }
    let mut e5 = temp / period as Real;

    temp = e5;
    for _ in 0..(period - 1) {
        e1 = k.mul_add(input[today], one_minus_k * e1);
        today += 1;
        e2 = k.mul_add(e1, one_minus_k * e2);
        e3 = k.mul_add(e2, one_minus_k * e3);
        e4 = k.mul_add(e3, one_minus_k * e4);
        e5 = k.mul_add(e4, one_minus_k * e5);
        temp += e5;
    }
    let mut e6 = temp / period as Real;

    let v2 = vfactor * vfactor;
    let c1 = -(v2 * vfactor);
    let c2 = 3.0 * (v2 - c1);
    let c3 = -6.0 * v2 - 3.0 * (vfactor - c1);
    let c4 = 1.0 + 3.0 * vfactor - c1 + 3.0 * v2;

    let mut out_index = 0usize;
    output[out_index] = ((c1 * e6 + c2 * e5) + c3 * e4) + c4 * e3;
    out_index += 1;

    while today < input.len() {
        e1 = k.mul_add(input[today], one_minus_k * e1);
        today += 1;
        e2 = k.mul_add(e1, one_minus_k * e2);
        e3 = k.mul_add(e2, one_minus_k * e3);
        e4 = k.mul_add(e3, one_minus_k * e4);
        e5 = k.mul_add(e4, one_minus_k * e5);
        e6 = k.mul_add(e5, one_minus_k * e6);
        output[out_index] = ((c1 * e6 + c2 * e5) + c3 * e4) + c4 * e3;
        out_index += 1;
    }

    out_index
}

fn parse_options(options: &[Real]) -> Result<(usize, Real), IndicatorError> {
    expect_option_count(METADATA.name, options, 2)?;
    let period = parse_usize_option(METADATA.name, options, 0, "period", 2)?;
    let vfactor = options[1];
    if !vfactor.is_finite() || !(0.0..=1.0).contains(&vfactor) {
        return Err(IndicatorError::InvalidOption {
            indicator: METADATA.name,
            option: "vfactor",
            value: vfactor,
            reason: "expected a finite value between 0 and 1",
        });
    }
    Ok((period, vfactor))
}
