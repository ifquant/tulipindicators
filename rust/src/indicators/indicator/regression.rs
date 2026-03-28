use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};

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

#[derive(Debug, Clone, Copy)]
pub struct LinReg;
#[derive(Debug, Clone, Copy)]
pub struct LinRegIntercept;
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
        let mut stream = RegressionProjectionStream::new(
            &LINREG_METADATA,
            period,
            RegressionProjection::ValueAt(period as Real),
        )?;
        stream.feed(inputs)
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
        let mut stream = RegressionProjectionStream::new(
            &LINREGINTERCEPT_METADATA,
            period,
            RegressionProjection::ValueAt(1.0),
        )?;
        stream.feed(inputs)
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
        let mut stream = RegressionProjectionStream::new(
            &LINREGSLOPE_METADATA,
            period,
            RegressionProjection::Slope,
        )?;
        stream.feed(inputs)
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

impl Indicator for Tsf {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &TSF_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(parse_period(TSF_METADATA.name, options)? - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let period = parse_period(TSF_METADATA.name, options)?;
        let mut stream = RegressionProjectionStream::new(
            &TSF_METADATA,
            period,
            RegressionProjection::ValueAt(period as Real + 1.0),
        )?;
        stream.feed(inputs)
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
        Ok(parse_period(FOSC_METADATA.name, options)?)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = FoscStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(FoscStream::new(options)?)))
    }
}

enum RegressionProjection {
    ValueAt(Real),
    Slope,
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
