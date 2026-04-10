use super::parse_positive_period;
use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::double_input;

const BETA_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "beta",
    full_name: "Beta",
    category: IndicatorCategory::Math,
    input_names: &["real", "real"],
    option_names: &["period"],
    output_names: &["beta"],
};

const CORREL_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "correl",
    full_name: "Pearson Correlation Coefficient",
    category: IndicatorCategory::Math,
    input_names: &["real", "real"],
    option_names: &["period"],
    output_names: &["correl"],
};

#[derive(Debug, Clone, Copy)]
pub struct Beta;

#[derive(Debug, Clone, Copy)]
pub struct Correl;

impl Indicator for Beta {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &BETA_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        parse_positive_period(BETA_METADATA.name, options)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (left, right) = double_input(BETA_METADATA.name, inputs)?;
        let period = parse_positive_period(BETA_METADATA.name, options)?;
        let output_len = left.len().saturating_sub(period);
        if output_len == 0 {
            return Ok(vec![Vec::new()]);
        }

        let mut output = vec![0.0; output_len];
        let produced = run_beta_batch(left, right, period, &mut output);
        debug_assert_eq!(produced, output_len);
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let (left, right) = double_input(BETA_METADATA.name, inputs)?;
        let period = parse_positive_period(BETA_METADATA.name, options)?;
        let output_len = left.len().saturating_sub(period);
        validate_output_slices(&BETA_METADATA, outputs, 1)?;
        ensure_output_len(&BETA_METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_beta_batch(
            left,
            right,
            period,
            &mut outputs[0][..output_len],
        ))
    }
}

impl Indicator for Correl {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &CORREL_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(parse_positive_period(CORREL_METADATA.name, options)? - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (left, right) = double_input(CORREL_METADATA.name, inputs)?;
        let period = parse_positive_period(CORREL_METADATA.name, options)?;
        let output_len = left.len().saturating_sub(period - 1);
        if output_len == 0 {
            return Ok(vec![Vec::new()]);
        }

        let mut output = vec![0.0; output_len];
        let produced = run_correl_batch(left, right, period, &mut output);
        debug_assert_eq!(produced, output_len);
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let (left, right) = double_input(CORREL_METADATA.name, inputs)?;
        let period = parse_positive_period(CORREL_METADATA.name, options)?;
        let output_len = left.len().saturating_sub(period - 1);
        validate_output_slices(&CORREL_METADATA, outputs, 1)?;
        ensure_output_len(&CORREL_METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_correl_batch(
            left,
            right,
            period,
            &mut outputs[0][..output_len],
        ))
    }
}

fn return_rate(current: Real, previous: Real) -> Real {
    if previous != 0.0 {
        (current - previous) / previous
    } else {
        0.0
    }
}

fn run_beta_batch(left: &[Real], right: &[Real], period: usize, output: &mut [Real]) -> usize {
    if left.len() <= period {
        return 0;
    }

    let n = period as Real;
    let mut sx = 0.0;
    let mut sy = 0.0;
    let mut sxx = 0.0;
    let mut sxy = 0.0;

    for index in 1..=period {
        let x = return_rate(left[index], left[index - 1]);
        let y = return_rate(right[index], right[index - 1]);
        sx += x;
        sy += y;
        sxx += x * x;
        sxy += x * y;
    }

    let mut out_index = 0usize;
    let mut denom = (-sx).mul_add(sx, n * sxx);
    output[out_index] = if denom != 0.0 {
        ((-sx).mul_add(sy, n * sxy)) / denom
    } else {
        0.0
    };
    out_index += 1;

    for end in (period + 1)..left.len() {
        let add_x = return_rate(left[end], left[end - 1]);
        let add_y = return_rate(right[end], right[end - 1]);
        let trailing = end - period;
        let sub_x = return_rate(left[trailing], left[trailing - 1]);
        let sub_y = return_rate(right[trailing], right[trailing - 1]);

        sx += add_x - sub_x;
        sy += add_y - sub_y;
        sxx += (-sub_x).mul_add(sub_x, add_x * add_x);
        sxy += (-sub_x).mul_add(sub_y, add_x * add_y);

        denom = (-sx).mul_add(sx, n * sxx);
        output[out_index] = if denom != 0.0 {
            ((-sx).mul_add(sy, n * sxy)) / denom
        } else {
            0.0
        };
        out_index += 1;
    }

    out_index
}

fn run_correl_batch(left: &[Real], right: &[Real], period: usize, output: &mut [Real]) -> usize {
    if left.len() < period {
        return 0;
    }

    let n = period as Real;
    let mut sx = 0.0;
    let mut sy = 0.0;
    let mut sxx = 0.0;
    let mut syy = 0.0;
    let mut sxy = 0.0;

    for index in 0..period {
        let x = left[index];
        let y = right[index];
        sx += x;
        sy += y;
        sxx += x * x;
        syy += y * y;
        sxy += x * y;
    }

    let mut out_index = 0usize;
    let mut xdiff = (-sx).mul_add(sx, n * sxx);
    let mut ydiff = (-sy).mul_add(sy, n * syy);
    let mut denom = xdiff * ydiff;
    output[out_index] = if denom > 0.0 {
        ((-sx).mul_add(sy, n * sxy)) / denom.sqrt()
    } else {
        0.0
    };
    out_index += 1;

    for end in period..left.len() {
        let add_x = left[end];
        let add_y = right[end];
        let sub_x = left[end - period];
        let sub_y = right[end - period];

        sx += add_x - sub_x;
        sy += add_y - sub_y;
        sxx += (-sub_x).mul_add(sub_x, add_x * add_x);
        syy += (-sub_y).mul_add(sub_y, add_y * add_y);
        sxy += (-sub_x).mul_add(sub_y, add_x * add_y);

        xdiff = (-sx).mul_add(sx, n * sxx);
        ydiff = (-sy).mul_add(sy, n * syy);
        denom = xdiff * ydiff;
        output[out_index] = if denom > 0.0 {
            ((-sx).mul_add(sy, n * sxy)) / denom.sqrt()
        } else {
            0.0
        };
        out_index += 1;
    }

    out_index
}
