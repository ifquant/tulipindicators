//! Klinger Volume Oscillator (`kvo`) combines volume force with fast and slow EMA smoothing.
//!
//! The direct kernel keeps the trend, cumulative measurement, and EMA state together to avoid the
//! wrapper allocation cost that would come from materializing intermediate series.
use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
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
        let (high, low, close, volume) = quadruple_input(METADATA.name, inputs)?;
        let output_len = high.len().saturating_sub(1);
        let mut output = vec![0.0; output_len];
        let produced = run_kvo_batch(high, low, close, volume, options, &mut output)?;
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
        let output_len = high.len().saturating_sub(1);
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), output_len, 0)?;
        run_kvo_batch(
            high,
            low,
            close,
            volume,
            options,
            &mut outputs[0][..output_len],
        )
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
        let (high, _, _, _) = quadruple_input(METADATA.name, inputs)?;
        let mut output = vec![0.0; high.len()];
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
        let (high, low, close, volume) = quadruple_input(METADATA.name, inputs)?;
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(&METADATA, outputs[0].len(), high.len(), 0)?;

        let mut out_index = 0usize;
        for (((&high, &low), &close), &volume) in high
            .iter()
            .zip(low.iter())
            .zip(close.iter())
            .zip(volume.iter())
        {
            let hlc = high + low + close;
            let dm = high - low;

            if let Some(previous_hlc) = self.previous_hlc {
                let previous_dm = self.previous_dm.expect("kvo previous dm should exist");
                if hlc > previous_hlc && self.trend != 1 {
                    self.trend = 1;
                    self.cm = previous_dm;
                } else if hlc < previous_hlc && self.trend != 0 {
                    self.trend = 0;
                    self.cm = previous_dm;
                }

                self.cm += dm;

                let vf = signed_volume_force(dm, self.cm, volume, self.trend);

                if self.progress == 1 {
                    self.short_ema = vf;
                    self.long_ema = vf;
                } else {
                    let short_part = vf * self.short_per;
                    let long_part = vf * self.long_per;
                    self.short_ema = self.short_ema.mul_add(1.0 - self.short_per, short_part);
                    self.long_ema = self.long_ema.mul_add(1.0 - self.long_per, long_part);
                }

                outputs[0][out_index] = self.short_ema - self.long_ema;
                out_index += 1;
            }

            self.previous_hlc = Some(hlc);
            self.previous_dm = Some(dm);
            self.progress += 1;
        }

        Ok(out_index)
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

fn run_kvo_batch(
    high: &[Real],
    low: &[Real],
    close: &[Real],
    volume: &[Real],
    options: &[Real],
    output: &mut [Real],
) -> Result<usize, IndicatorError> {
    let (short_period, long_period) = parse_options(options)?;
    if high.len() < 2 {
        return Ok(0);
    }

    let short_per = 2.0 / (short_period as Real + 1.0);
    let long_per = 2.0 / (long_period as Real + 1.0);

    let mut cm = 0.0;
    let mut previous_hlc = high[0] + low[0] + close[0];
    let mut trend = -1;
    let mut short_ema = 0.0;
    let mut long_ema = 0.0;
    let mut out_index = 0usize;

    for index in 1..high.len() {
        let hlc = high[index] + low[index] + close[index];
        let dm = high[index] - low[index];

        if hlc > previous_hlc && trend != 1 {
            trend = 1;
            cm = high[index - 1] - low[index - 1];
        } else if hlc < previous_hlc && trend != 0 {
            trend = 0;
            cm = high[index - 1] - low[index - 1];
        }

        cm += dm;

        let vf = signed_volume_force(dm, cm, volume[index], trend);

        if index == 1 {
            short_ema = vf;
            long_ema = vf;
        } else {
            let short_part = vf * short_per;
            let long_part = vf * long_per;
            short_ema = short_ema.mul_add(1.0 - short_per, short_part);
            long_ema = long_ema.mul_add(1.0 - long_per, long_part);
        }

        output[out_index] = short_ema - long_ema;
        out_index += 1;
        previous_hlc = hlc;
    }

    Ok(out_index)
}

fn signed_volume_force(dm: Real, cm: Real, volume: Real, trend: i32) -> Real {
    let signal = ((trend != 0) as u8 as Real).mul_add(2.0, -1.0);
    let magnitude = volume * (dm / cm).mul_add(2.0, -1.0).abs() * 100.0;
    magnitude * signal
}
