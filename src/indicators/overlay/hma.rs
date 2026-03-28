use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
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
        let mut stream = HmaStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(HmaStream::new(options)?)))
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
