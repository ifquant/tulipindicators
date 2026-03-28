use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, quadruple_input};
use crate::indicators::shared::RingSum;

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "mfi",
    full_name: "Money Flow Index",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low", "close", "volume"],
    option_names: &["period"],
    output_names: &["mfi"],
};

#[derive(Debug, Clone, Copy)]
pub struct Mfi;

impl Indicator for Mfi {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        parse_period(options)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = MfiStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(MfiStream::new(options)?)))
    }
}

struct MfiStream {
    progress: usize,
    previous_typ: Option<Real>,
    up: RingSum,
    down: RingSum,
}

impl MfiStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        Ok(Self {
            progress: 0,
            previous_typ: None,
            up: RingSum::new(period),
            down: RingSum::new(period),
        })
    }
}

impl IndicatorStream for MfiStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close, volume) = quadruple_input(METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(high.len());

        for (((&high, &low), &close), &volume) in high
            .iter()
            .zip(low.iter())
            .zip(close.iter())
            .zip(volume.iter())
        {
            let typ = (high + low + close) / 3.0;

            if let Some(previous_typ) = self.previous_typ {
                let bar = typ * volume;
                if typ > previous_typ {
                    self.up.push(bar);
                    self.down.push(0.0);
                } else if typ < previous_typ {
                    self.up.push(0.0);
                    self.down.push(bar);
                } else {
                    self.up.push(0.0);
                    self.down.push(0.0);
                }

                if self.up.is_full() {
                    output.push(self.up.sum / (self.up.sum + self.down.sum) * 100.0);
                }
            }

            self.previous_typ = Some(typ);
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 1)
}
