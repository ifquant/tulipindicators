use crate::core::error::IndicatorError;
use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{
    double_input, expect_option_count, parse_usize_option, quadruple_input,
};
use crate::indicators::shared::{EmaState, RingSum};

const CMF_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "cmf",
    full_name: "Chaikin Money Flow",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low", "close", "volume"],
    option_names: &["period"],
    output_names: &["cmf"],
};

const FI_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "fi",
    full_name: "Force Index",
    category: IndicatorCategory::Indicator,
    input_names: &["close", "volume"],
    option_names: &["period"],
    output_names: &["fi"],
};

#[derive(Debug, Clone, Copy)]
pub struct Cmf;
#[derive(Debug, Clone, Copy)]
pub struct Fi;

impl Indicator for Cmf {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &CMF_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(parse_period(CMF_METADATA.name, options)? - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = CmfStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(CmfStream::new(options)?)))
    }
}

impl Indicator for Fi {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &FI_METADATA
    }

    fn lookback(&self, _options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = FiStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(FiStream::new(options)?)))
    }
}

struct CmfStream {
    progress: usize,
    ad_sum: RingSum,
    volume_sum: RingSum,
}

impl CmfStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(CMF_METADATA.name, options)?;
        Ok(Self {
            progress: 0,
            ad_sum: RingSum::new(period),
            volume_sum: RingSum::new(period),
        })
    }
}

impl IndicatorStream for CmfStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &CMF_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close, volume) = quadruple_input(CMF_METADATA.name, inputs)?;
        let mut output = Vec::new();

        for (((&high_value, &low_value), &close_value), &volume_value) in
            high.iter().zip(low).zip(close).zip(volume)
        {
            let ad = if high_value != low_value {
                volume_value * ((close_value - low_value) - (high_value - close_value))
                    / (high_value - low_value)
            } else {
                0.0
            };
            self.ad_sum.push(ad);
            self.volume_sum.push(volume_value);

            if self.ad_sum.is_full() {
                output.push(self.ad_sum.sum / self.volume_sum.sum);
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

struct FiStream {
    progress: usize,
    ema: EmaState,
    previous_close: Option<Real>,
}

impl FiStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(FI_METADATA.name, options)?;
        Ok(Self {
            progress: 0,
            ema: EmaState::new(2.0 / (period as Real + 1.0)),
            previous_close: None,
        })
    }
}

impl IndicatorStream for FiStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &FI_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (close, volume) = double_input(FI_METADATA.name, inputs)?;
        let mut output = Vec::new();

        for (&close_value, &volume_value) in close.iter().zip(volume) {
            if let Some(previous_close) = self.previous_close {
                let force = volume_value * (close_value - previous_close);
                output.push(self.ema.feed(force));
            }

            self.previous_close = Some(close_value);
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn parse_period(name: &'static str, options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(name, options, 1)?;
    parse_usize_option(name, options, 0, "period", 1)
}
