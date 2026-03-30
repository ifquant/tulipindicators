use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{double_input, expect_option_count, parse_usize_option};
use crate::indicators::shared::{EmaState, RingSum};

const METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "mass",
    full_name: "Mass Index",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low"],
    option_names: &["period"],
    output_names: &["mass"],
};

const EMA_MULTIPLIER: Real = 2.0 / 10.0;

#[derive(Debug, Clone, Copy)]
pub struct Mass;

impl Indicator for Mass {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let period = parse_period(options)?;
        Ok(period + 15)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low) = double_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        let mut output = vec![0.0; high.len().saturating_sub(period + 15)];
        let produced = run_mass_batch(high, low, period, &mut output);
        output.truncate(produced);
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let (high, low) = double_input(METADATA.name, inputs)?;
        let period = parse_period(options)?;
        validate_output_slices(&METADATA, outputs, 1)?;
        ensure_output_len(
            &METADATA,
            outputs[0].len(),
            high.len().saturating_sub(period + 15),
            0,
        )?;
        Ok(run_mass_batch(high, low, period, outputs[0]))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(MassStream::new(options)?)))
    }
}

struct MassStream {
    period: usize,
    progress: usize,
    ema1: EmaState,
    ema2: Option<Real>,
    ratio_sum: RingSum,
}

impl MassStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let period = parse_period(options)?;
        Ok(Self {
            period,
            progress: 0,
            ema1: EmaState::new(EMA_MULTIPLIER),
            ema2: None,
            ratio_sum: RingSum::new(period),
        })
    }
}

impl IndicatorStream for MassStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low) = double_input(METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(high.len().saturating_sub(self.period + 15));

        for (&high, &low) in high.iter().zip(low.iter()) {
            let hl = high - low;
            let ema1 = self.ema1.feed(hl);

            if self.progress == 8 {
                self.ema2 = Some(ema1);
            }

            if self.progress >= 8 {
                let previous = self.ema2.expect("mass second EMA should be initialized");
                let ema2 = (ema1 - previous) * EMA_MULTIPLIER + previous;
                self.ema2 = Some(ema2);

                if self.progress >= 16 {
                    self.ratio_sum.push(ema1 / ema2);
                    if self.ratio_sum.is_full() {
                        output.push(self.ratio_sum.sum);
                    }
                }
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn run_mass_batch(high: &[Real], low: &[Real], period: usize, output: &mut [Real]) -> usize {
    if high.len() <= period + 15 {
        return 0;
    }

    let mut ema1 = EmaState::new(EMA_MULTIPLIER);
    let mut ema2 = None;
    let mut ratio_sum = RingSum::new(period);
    let mut out_index = 0usize;

    for (index, (&high, &low)) in high.iter().zip(low.iter()).enumerate() {
        let hl = high - low;
        let ema1_value = ema1.feed(hl);

        if index == 8 {
            ema2 = Some(ema1_value);
        }

        if index >= 8 {
            let previous = ema2.expect("mass batch second EMA should be initialized");
            let ema2_value = (ema1_value - previous).mul_add(EMA_MULTIPLIER, previous);
            ema2 = Some(ema2_value);

            if index >= 16 {
                ratio_sum.push(ema1_value / ema2_value);
                if ratio_sum.is_full() {
                    output[out_index] = ratio_sum.sum;
                    out_index += 1;
                }
            }
        }
    }

    out_index
}

fn parse_period(options: &[Real]) -> Result<usize, IndicatorError> {
    expect_option_count(METADATA.name, options, 1)?;
    parse_usize_option(METADATA.name, options, 0, "period", 1)
}
