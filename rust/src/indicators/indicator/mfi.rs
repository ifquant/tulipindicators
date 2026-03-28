use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
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
        let (high, low, close, volume) = quadruple_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let output_len = high.len().saturating_sub(period);
        let mut output = vec![0.0; output_len];
        let produced = run_mfi_batch(high, low, close, volume, period, &mut output);
        debug_assert_eq!(produced, output.len());
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let (high, low, close, volume) = quadruple_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let output_len = high.len().saturating_sub(period);
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_mfi_batch(
            high,
            low,
            close,
            volume,
            period,
            &mut outputs[0][..output_len],
        ))
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

fn run_mfi_batch(
    high: &[Real],
    low: &[Real],
    close: &[Real],
    volume: &[Real],
    period: usize,
    output: &mut [Real],
) -> usize {
    if high.len() <= period {
        return 0;
    }

    let mut up = RingSum::new(period);
    let mut down = RingSum::new(period);
    let mut previous_typ = (high[0] + low[0] + close[0]) / 3.0;
    let mut out_index = 0usize;

    for index in 1..high.len() {
        let typ = (high[index] + low[index] + close[index]) / 3.0;
        let bar = typ * volume[index];

        if typ > previous_typ {
            up.push(bar);
            down.push(0.0);
        } else if typ < previous_typ {
            up.push(0.0);
            down.push(bar);
        } else {
            up.push(0.0);
            down.push(0.0);
        }

        previous_typ = typ;

        if index >= period {
            output[out_index] = up.sum / (up.sum + down.sum) * 100.0;
            out_index += 1;
        }
    }

    out_index
}
