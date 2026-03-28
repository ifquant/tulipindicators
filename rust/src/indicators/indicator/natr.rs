use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, parse_usize_option, triple_input};
use crate::indicators::shared::true_range;

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "natr",
    full_name: "Normalized Average True Range",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low", "close"],
    option_names: &["period"],
    output_names: &["natr"],
};

#[derive(Debug, Clone, Copy)]
pub struct Natr;

impl Indicator for Natr {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        Ok(period - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close) = triple_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let lookback = period - 1;
        let mut output = Vec::with_capacity(high.len().saturating_sub(lookback));

        if high.len() <= lookback {
            return Ok(vec![output]);
        }

        let per = 1.0 / period as Real;
        let mut sum = high[0] - low[0];
        for index in 1..period {
            sum += true_range(high[index], low[index], close[index - 1]);
        }

        let mut value = sum / period as Real;
        output.push(100.0 * value / close[period - 1]);

        for index in period..high.len() {
            let tr = true_range(high[index], low[index], close[index - 1]);
            value = (tr - value) * per + value;
            output.push(100.0 * value / close[index]);
        }

        Ok(vec![output])
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(NatrStream::new(options)?)))
    }
}

struct NatrStream {
    period: usize,
    progress: usize,
    sum: Real,
    last_atr: Option<Real>,
    last_close: Option<Real>,
}

impl NatrStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        Ok(Self {
            period: parse_period(options)?,
            progress: 0,
            sum: 0.0,
            last_atr: None,
            last_close: None,
        })
    }
}

impl IndicatorStream for NatrStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close) = triple_input(METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(high.len());

        for index in 0..high.len() {
            let tr = match self.last_close {
                Some(previous_close) => true_range(high[index], low[index], previous_close),
                None => high[index] - low[index],
            };

            if self.progress < self.period {
                self.sum += tr;
                if self.progress + 1 == self.period {
                    let atr = self.sum / self.period as Real;
                    self.last_atr = Some(atr);
                    output.push(100.0 * atr / close[index]);
                }
            } else if let Some(last_atr) = self.last_atr {
                let atr = (tr - last_atr) / self.period as Real + last_atr;
                self.last_atr = Some(atr);
                output.push(100.0 * atr / close[index]);
            }

            self.last_close = Some(close[index]);
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 1)
}
