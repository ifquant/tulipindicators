use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};
use std::f64::consts::PI;

const LINREG_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "linreg",
    full_name: "Linear Regression",
    category: IndicatorCategory::Overlay,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["linreg"],
};

const LINREGINTERCEPT_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "linregintercept",
    full_name: "Linear Regression Intercept",
    category: IndicatorCategory::Overlay,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["linregintercept"],
};

const LINREGANGLE_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "linearregangle",
    full_name: "Linear Regression Angle",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["linearregangle"],
};

const LINREGSLOPE_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "linregslope",
    full_name: "Linear Regression Slope",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["linregslope"],
};

const TSF_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "tsf",
    full_name: "Time Series Forecast",
    category: IndicatorCategory::Overlay,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["tsf"],
};

const FOSC_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "fosc",
    full_name: "Forecast Oscillator",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["fosc"],
};

const DEG_PER_RAD: Real = 180.0 / PI;

#[derive(Debug, Clone, Copy)]
pub struct LinReg;
#[derive(Debug, Clone, Copy)]
pub struct LinRegIntercept;
#[derive(Debug, Clone, Copy)]
pub struct LinRegAngle;
#[derive(Debug, Clone, Copy)]
pub struct LinRegSlope;
#[derive(Debug, Clone, Copy)]
pub struct Tsf;
#[derive(Debug, Clone, Copy)]
pub struct Fosc;

impl Indicator for LinReg {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &LINREG_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(parse_period(LINREG_METADATA.name, options)? - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let period = parse_period(LINREG_METADATA.name, options)?;
        let input = single_input(LINREG_METADATA.name, inputs)?;
        let output_len = input.len().saturating_sub(period - 1);
        if output_len == 0 {
            return Ok(vec![Vec::new()]);
        }
        let mut output = vec![0.0; output_len];
        let written = run_regression_batch(
            input,
            period,
            RegressionProjection::ValueAt(period as Real),
            &mut output,
        );
        debug_assert_eq!(written, output.len());
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let period = parse_period(LINREG_METADATA.name, options)?;
        let input = single_input(LINREG_METADATA.name, inputs)?;
        let output_len = input.len().saturating_sub(period - 1);
        validate_output_slices(&LINREG_METADATA, outputs, 1)?;
        ensure_output_len(&LINREG_METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_regression_batch(
            input,
            period,
            RegressionProjection::ValueAt(period as Real),
            &mut outputs[0][..output_len],
        ))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        let period = parse_period(LINREG_METADATA.name, options)?;
        Ok(Some(Box::new(RegressionProjectionStream::new(
            &LINREG_METADATA,
            period,
            RegressionProjection::ValueAt(period as Real),
        )?)))
    }
}

impl Indicator for LinRegIntercept {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &LINREGINTERCEPT_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(parse_period(LINREGINTERCEPT_METADATA.name, options)? - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let period = parse_period(LINREGINTERCEPT_METADATA.name, options)?;
        let input = single_input(LINREGINTERCEPT_METADATA.name, inputs)?;
        let output_len = input.len().saturating_sub(period - 1);
        if output_len == 0 {
            return Ok(vec![Vec::new()]);
        }
        let mut output = vec![0.0; output_len];
        let written = run_regression_batch(
            input,
            period,
            RegressionProjection::ValueAt(1.0),
            &mut output,
        );
        debug_assert_eq!(written, output.len());
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let period = parse_period(LINREGINTERCEPT_METADATA.name, options)?;
        let input = single_input(LINREGINTERCEPT_METADATA.name, inputs)?;
        let output_len = input.len().saturating_sub(period - 1);
        validate_output_slices(&LINREGINTERCEPT_METADATA, outputs, 1)?;
        ensure_output_len(&LINREGINTERCEPT_METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_regression_batch(
            input,
            period,
            RegressionProjection::ValueAt(1.0),
            &mut outputs[0][..output_len],
        ))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        let period = parse_period(LINREGINTERCEPT_METADATA.name, options)?;
        Ok(Some(Box::new(RegressionProjectionStream::new(
            &LINREGINTERCEPT_METADATA,
            period,
            RegressionProjection::ValueAt(1.0),
        )?)))
    }
}

impl Indicator for LinRegSlope {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &LINREGSLOPE_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(parse_period(LINREGSLOPE_METADATA.name, options)? - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let period = parse_period(LINREGSLOPE_METADATA.name, options)?;
        let input = single_input(LINREGSLOPE_METADATA.name, inputs)?;
        let output_len = input.len().saturating_sub(period - 1);
        if output_len == 0 {
            return Ok(vec![Vec::new()]);
        }
        let mut output = vec![0.0; output_len];
        let written = run_regression_batch(input, period, RegressionProjection::Slope, &mut output);
        debug_assert_eq!(written, output.len());
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let period = parse_period(LINREGSLOPE_METADATA.name, options)?;
        let input = single_input(LINREGSLOPE_METADATA.name, inputs)?;
        let output_len = input.len().saturating_sub(period - 1);
        validate_output_slices(&LINREGSLOPE_METADATA, outputs, 1)?;
        ensure_output_len(&LINREGSLOPE_METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_regression_batch(
            input,
            period,
            RegressionProjection::Slope,
            &mut outputs[0][..output_len],
        ))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        let period = parse_period(LINREGSLOPE_METADATA.name, options)?;
        Ok(Some(Box::new(RegressionProjectionStream::new(
            &LINREGSLOPE_METADATA,
            period,
            RegressionProjection::Slope,
        )?)))
    }
}

impl Indicator for LinRegAngle {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &LINREGANGLE_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(parse_period(LINREGANGLE_METADATA.name, options)? - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let period = parse_period(LINREGANGLE_METADATA.name, options)?;
        let input = single_input(LINREGANGLE_METADATA.name, inputs)?;
        let output_len = input.len().saturating_sub(period - 1);
        if output_len == 0 {
            return Ok(vec![Vec::new()]);
        }
        let mut output = vec![0.0; output_len];
        let written = run_linearregangle_batch(input, period, &mut output);
        debug_assert_eq!(written, output.len());
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let period = parse_period(LINREGANGLE_METADATA.name, options)?;
        let input = single_input(LINREGANGLE_METADATA.name, inputs)?;
        let output_len = input.len().saturating_sub(period - 1);
        validate_output_slices(&LINREGANGLE_METADATA, outputs, 1)?;
        ensure_output_len(&LINREGANGLE_METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_linearregangle_batch(
            input,
            period,
            &mut outputs[0][..output_len],
        ))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        let period = parse_period(LINREGANGLE_METADATA.name, options)?;
        Ok(Some(Box::new(RegressionProjectionStream::new(
            &LINREGANGLE_METADATA,
            period,
            RegressionProjection::Angle,
        )?)))
    }
}

impl Indicator for Tsf {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &TSF_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(parse_period(TSF_METADATA.name, options)? - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let period = parse_period(TSF_METADATA.name, options)?;
        let input = single_input(TSF_METADATA.name, inputs)?;
        let output_len = input.len().saturating_sub(period - 1);
        if output_len == 0 {
            return Ok(vec![Vec::new()]);
        }
        let mut output = vec![0.0; output_len];
        let written = run_regression_batch(
            input,
            period,
            RegressionProjection::ValueAt(period as Real + 1.0),
            &mut output,
        );
        debug_assert_eq!(written, output.len());
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let period = parse_period(TSF_METADATA.name, options)?;
        let input = single_input(TSF_METADATA.name, inputs)?;
        let output_len = input.len().saturating_sub(period - 1);
        validate_output_slices(&TSF_METADATA, outputs, 1)?;
        ensure_output_len(&TSF_METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_regression_batch(
            input,
            period,
            RegressionProjection::ValueAt(period as Real + 1.0),
            &mut outputs[0][..output_len],
        ))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        let period = parse_period(TSF_METADATA.name, options)?;
        Ok(Some(Box::new(RegressionProjectionStream::new(
            &TSF_METADATA,
            period,
            RegressionProjection::ValueAt(period as Real + 1.0),
        )?)))
    }
}

impl Indicator for Fosc {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &FOSC_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        parse_period(FOSC_METADATA.name, options)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let period = parse_period(FOSC_METADATA.name, options)?;
        let input = single_input(FOSC_METADATA.name, inputs)?;
        if input.len() <= period {
            return Ok(vec![Vec::new()]);
        }

        let mut output = vec![0.0; input.len() - period];
        let written = run_fosc_batch(input, period, &mut output);
        debug_assert_eq!(written, output.len());
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let period = parse_period(FOSC_METADATA.name, options)?;
        let input = single_input(FOSC_METADATA.name, inputs)?;
        let output_len = input.len().saturating_sub(period);
        validate_output_slices(&FOSC_METADATA, outputs, 1)?;
        ensure_output_len(&FOSC_METADATA, outputs[0].len(), output_len, 0)?;
        if input.len() <= period {
            return Ok(0);
        }

        Ok(run_fosc_batch(input, period, &mut outputs[0][..output_len]))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(FoscStream::new(options)?)))
    }
}

fn run_fosc_batch(input: &[Real], period: usize, output: &mut [Real]) -> usize {
    if input.len() <= period {
        return 0;
    }

    let period_real = period as Real;
    let mut x_sum = 0.0;
    let mut x2_sum = 0.0;
    let mut y_sum = 0.0;
    let mut xy_sum = 0.0;

    for (index, &sample) in input.iter().enumerate().take(period) {
        let x = (index + 1) as Real;
        x_sum += x;
        x2_sum += x * x;
        y_sum += sample;
        xy_sum += sample * x;
    }

    let inv_denom = 1.0 / (period_real * x2_sum - x_sum * x_sum);
    let inv_period = 1.0 / period_real;

    let mut slope = (period_real * xy_sum - x_sum * y_sum) * inv_denom;
    let mut intercept0 = (y_sum - slope * x_sum) * inv_period;
    let mut tsf = intercept0 + slope * (period_real + 1.0);

    for (dst, index) in output.iter_mut().zip(period..input.len()) {
        let sample = input[index];
        *dst = 100.0 * (sample - tsf) / sample;

        xy_sum = xy_sum - y_sum + sample * period_real;
        y_sum = y_sum - input[index - period] + sample;

        slope = (period_real * xy_sum - x_sum * y_sum) * inv_denom;
        intercept0 = (y_sum - slope * x_sum) * inv_period;
        tsf = intercept0 + slope * (period_real + 1.0);
    }

    output.len()
}

enum RegressionProjection {
    ValueAt(Real),
    Slope,
    Angle,
}

fn run_regression_batch(
    input: &[Real],
    period: usize,
    projection: RegressionProjection,
    output: &mut [Real],
) -> usize {
    if input.len() < period {
        return 0;
    }

    let mut x_sum = 0.0;
    let mut x2_sum = 0.0;
    let mut y_sum = 0.0;
    let mut xy_sum = 0.0;

    for (index, &sample) in input.iter().enumerate().take(period - 1) {
        let x = (index + 1) as Real;
        x_sum += x;
        x2_sum += x * x;
        xy_sum += sample * x;
        y_sum += sample;
    }

    let period_real = period as Real;
    x_sum += period_real;
    x2_sum += period_real * period_real;
    let inv_denom = 1.0 / (period_real * x2_sum - x_sum * x_sum);
    let inv_period = 1.0 / period_real;

    let mut out_index = 0usize;
    for index in (period - 1)..input.len() {
        xy_sum += input[index] * period_real;
        y_sum += input[index];

        let slope = (period_real * xy_sum - x_sum * y_sum) * inv_denom;
        let intercept0 = (y_sum - slope * x_sum) * inv_period;
        output[out_index] = match projection {
            RegressionProjection::ValueAt(x) => intercept0 + slope * x,
            RegressionProjection::Slope => slope,
            RegressionProjection::Angle => slope.atan() * (180.0 / std::f64::consts::PI),
        };
        out_index += 1;

        xy_sum -= y_sum;
        y_sum -= input[index + 1 - period];
    }

    out_index
}

fn run_linearregangle_batch(input: &[Real], period: usize, output: &mut [Real]) -> usize {
    if input.len() < period {
        return 0;
    }

    let mut x_sum = 0.0;
    let mut x2_sum = 0.0;
    let mut y_sum = 0.0;
    let mut xy_sum = 0.0;

    for (index, &sample) in input.iter().enumerate().take(period - 1) {
        let x = (index + 1) as Real;
        x_sum += x;
        x2_sum += x * x;
        xy_sum += sample * x;
        y_sum += sample;
    }

    let period_real = period as Real;
    x_sum += period_real;
    x2_sum += period_real * period_real;
    let inv_denom = 1.0 / (period_real * x2_sum - x_sum * x_sum);

    let mut out_index = 0usize;
    for index in (period - 1)..input.len() {
        xy_sum += input[index] * period_real;
        y_sum += input[index];

        let slope = (period_real * xy_sum - x_sum * y_sum) * inv_denom;
        output[out_index] = slope.atan() * DEG_PER_RAD;
        out_index += 1;

        xy_sum -= y_sum;
        y_sum -= input[index + 1 - period];
    }

    out_index
}

struct RegressionProjectionStream {
    metadata: &'static IndicatorMetadata,
    progress: usize,
    state: LinearRegressionState,
    projection: RegressionProjection,
}

impl RegressionProjectionStream {
    fn new(
        metadata: &'static IndicatorMetadata,
        period: usize,
        projection: RegressionProjection,
    ) -> Result<Self, IndicatorError> {
        Ok(Self {
            metadata,
            progress: 0,
            state: LinearRegressionState::new(period),
            projection,
        })
    }
}

impl IndicatorStream for RegressionProjectionStream {
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
            if let Some(values) = self.state.feed(sample) {
                output.push(match self.projection {
                    RegressionProjection::ValueAt(x) => values.value_at(x),
                    RegressionProjection::Slope => values.slope,
                    RegressionProjection::Angle => values.slope.atan() * DEG_PER_RAD,
                });
            }
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

struct FoscStream {
    period: usize,
    progress: usize,
    state: LinearRegressionState,
    previous_tsf: Option<Real>,
}

impl FoscStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(FOSC_METADATA.name, options)?;
        Ok(Self {
            period,
            progress: 0,
            state: LinearRegressionState::new(period),
            previous_tsf: None,
        })
    }
}

impl IndicatorStream for FoscStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &FOSC_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(FOSC_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(input.len());

        for &sample in input {
            if let Some(values) = self.state.feed(sample) {
                let current_tsf = values.value_at(self.period as Real + 1.0);
                if let Some(previous_tsf) = self.previous_tsf {
                    output.push(100.0 * (sample - previous_tsf) / sample);
                }
                self.previous_tsf = Some(current_tsf);
            }
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

#[derive(Clone)]
struct LinearRegressionState {
    period: usize,
    x_sum: Real,
    inv_denom: Real,
    values: Vec<Real>,
    cursor: usize,
    len: usize,
    y_sum: Real,
    xy_sum: Real,
}

#[derive(Clone, Copy)]
struct RegressionValues {
    intercept0: Real,
    slope: Real,
}

impl RegressionValues {
    fn value_at(self, x: Real) -> Real {
        self.intercept0 + self.slope * x
    }
}

impl LinearRegressionState {
    fn new(period: usize) -> Self {
        let x_sum = (period * (period + 1) / 2) as Real;
        let x2_sum = (period * (period + 1) * (2 * period + 1) / 6) as Real;
        let denom = period as Real * x2_sum - x_sum * x_sum;
        Self {
            period,
            x_sum,
            inv_denom: 1.0 / denom,
            values: vec![0.0; period],
            cursor: 0,
            len: 0,
            y_sum: 0.0,
            xy_sum: 0.0,
        }
    }

    fn feed(&mut self, sample: Real) -> Option<RegressionValues> {
        if self.len < self.period {
            self.values[self.len] = sample;
            self.y_sum += sample;
            self.xy_sum += sample * (self.len + 1) as Real;
            self.len += 1;

            if self.len < self.period {
                return None;
            }
        } else {
            let oldest = self.values[self.cursor];
            self.xy_sum = self.xy_sum - self.y_sum + sample * self.period as Real;
            self.y_sum = self.y_sum - oldest + sample;
            self.values[self.cursor] = sample;
            self.cursor = (self.cursor + 1) % self.period;
        }

        let slope = (self.period as Real * self.xy_sum - self.x_sum * self.y_sum) * self.inv_denom;
        let intercept0 = (self.y_sum - slope * self.x_sum) / self.period as Real;
        Some(RegressionValues { intercept0, slope })
    }
}

fn parse_period(name: &'static str, options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(name, options, 1)?;
    parse_usize_option(name, options, 0, "period", 1)
}
