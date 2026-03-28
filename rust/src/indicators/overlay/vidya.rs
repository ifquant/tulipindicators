use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
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
        let mut stream = VidyaStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(VidyaStream::new(options)?)))
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
                let short_stddev = short
                    .expect("short stats should be available once long period is reached")
                    .variance
                    .sqrt();
                let long_stddev = long
                    .expect("long stats should be available once long period is reached")
                    .variance
                    .sqrt();
                let mut k = short_stddev / long_stddev;
                if k.is_nan() {
                    k = 0.0;
                }
                k *= self.alpha;

                let current = self.value.expect("vidya value should be initialized");
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
