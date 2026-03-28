use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, single_input};
use crate::indicators::shared::{rolling_variance_batch, RollingStatsState};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "stderr",
    full_name: "Standard Error",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &["period"],
    output_names: &["stderr"],
};

#[derive(Debug, Clone, Copy)]
pub struct StdErr;

impl Indicator for StdErr {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        Ok(period - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let scale = 1.0 / (period as Real).sqrt();
        let mut output = vec![0.0; input.len().saturating_sub(period - 1)];
        let produced = rolling_variance_batch(input, period, &mut output, |variance| {
            scale * stddev_value(variance)
        });
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
        let period = parse_period(options)?;
        let output_len = input.len().saturating_sub(period - 1);
        let scale = 1.0 / (period as Real).sqrt();
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;
        Ok(rolling_variance_batch(
            input,
            period,
            &mut outputs[0][..output_len],
            |variance| scale * stddev_value(variance),
        ))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(StdErrStream::new(options)?)))
    }
}

struct StdErrStream {
    progress: usize,
    period: usize,
    stats: RollingStatsState,
}

impl StdErrStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        Ok(Self {
            progress: 0,
            period,
            stats: RollingStatsState::new(period),
        })
    }
}

impl IndicatorStream for StdErrStream {
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

        let scale = 1.0 / (self.period as Real).sqrt();
        let mut out_index = 0usize;
        for &sample in input {
            if let Some(stats) = self.stats.feed(sample) {
                outputs[0][out_index] = scale * stddev_value(stats.variance);
                out_index += 1;
            }
        }
        self.progress += input.len();
        Ok(out_index)
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 1)
}

fn stddev_value(variance: Real) -> Real {
    if variance > 0.0 {
        variance.sqrt()
    } else {
        variance
    }
}
