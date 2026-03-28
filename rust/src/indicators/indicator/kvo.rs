use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, quadruple_input};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "kvo",
    full_name: "Klinger Volume Oscillator",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low", "close", "volume"],
    option_names: &["short_period", "long_period"],
    output_names: &["kvo"],
};

#[derive(Debug, Clone, Copy)]
pub struct Kvo;

impl Indicator for Kvo {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let _ = parse_options(options)?;
        Ok(1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = KvoStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(KvoStream::new(options)?)))
    }
}

struct KvoStream {
    short_per: Real,
    long_per: Real,
    progress: usize,
    previous_hlc: Option<Real>,
    previous_dm: Option<Real>,
    trend: i32,
    cm: Real,
    short_ema: Real,
    long_ema: Real,
}

impl KvoStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let (short_period, long_period) = parse_options(options)?;
        Ok(Self {
            short_per: 2.0 / (short_period as Real + 1.0),
            long_per: 2.0 / (long_period as Real + 1.0),
            progress: 0,
            previous_hlc: None,
            previous_dm: None,
            trend: -1,
            cm: 0.0,
            short_ema: 0.0,
            long_ema: 0.0,
        })
    }
}

impl IndicatorStream for KvoStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close, volume) = quadruple_input(METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(high.len().saturating_sub(1));

        for (((&high, &low), &close), &volume) in high
            .iter()
            .zip(low.iter())
            .zip(close.iter())
            .zip(volume.iter())
        {
            let hlc = high + low + close;
            let dm = high - low;

            if let Some(previous_hlc) = self.previous_hlc {
                if hlc > previous_hlc && self.trend != 1 {
                    self.trend = 1;
                    self.cm = self.previous_dm.expect("kvo previous dm should exist");
                } else if hlc < previous_hlc && self.trend != 0 {
                    self.trend = 0;
                    self.cm = self.previous_dm.expect("kvo previous dm should exist");
                }

                self.cm += dm;

                let vf = volume
                    * ((dm / self.cm) * 2.0 - 1.0).abs()
                    * 100.0
                    * if self.trend != 0 { 1.0 } else { -1.0 };

                if self.progress == 1 {
                    self.short_ema = vf;
                    self.long_ema = vf;
                } else {
                    self.short_ema = (vf - self.short_ema) * self.short_per + self.short_ema;
                    self.long_ema = (vf - self.long_ema) * self.long_per + self.long_ema;
                }

                output.push(self.short_ema - self.long_ema);
            }

            self.previous_hlc = Some(hlc);
            self.previous_dm = Some(dm);
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn parse_options(options: &[Real]) -> Result<(usize, usize), IndicatorError> {
    expect_option_count(METADATA.name, options, 2)?;
    let short_period = parse_usize_option(METADATA.name, options, 0, "short_period", 1)?;
    let long_period = parse_usize_option(METADATA.name, options, 1, "long_period", 1)?;

    if long_period < short_period {
        return Err(IndicatorError::InvalidOption {
            indicator: METADATA.name,
            option: "long_period",
            value: options[1],
            reason: "expected long_period >= short_period",
        });
    }

    Ok((short_period, long_period))
}
