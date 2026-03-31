use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{expect_option_count, single_input};

const SHORT_LOOKBACK: usize = 32;
const LONG_LOOKBACK: usize = 63;
const HILBERT_A: Real = 0.0962;
const HILBERT_B: Real = 0.5769;
const SMOOTH_PRICE_SIZE: usize = 50;

const HT_DCPERIOD_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "ht_dcperiod",
    full_name: "Hilbert Transform - Dominant Cycle Period",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &[],
    output_names: &["ht_dcperiod"],
};

const HT_DCPHASE_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "ht_dcphase",
    full_name: "Hilbert Transform - Dominant Cycle Phase",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &[],
    output_names: &["ht_dcphase"],
};

const HT_PHASOR_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "ht_phasor",
    full_name: "Hilbert Transform - Phasor Components",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &[],
    output_names: &["inphase", "quadrature"],
};

const HT_SINE_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "ht_sine",
    full_name: "Hilbert Transform - SineWave",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &[],
    output_names: &["sine", "lead_sine"],
};

const HT_TRENDLINE_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "ht_trendline",
    full_name: "Hilbert Transform - Instantaneous Trendline",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &[],
    output_names: &["ht_trendline"],
};

const HT_TRENDMODE_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "ht_trendmode",
    full_name: "Hilbert Transform - Trend vs Cycle Mode",
    category: IndicatorCategory::Indicator,
    input_names: &["real"],
    option_names: &[],
    output_names: &["ht_trendmode"],
};

#[derive(Debug, Clone, Copy)]
pub struct HtDcPeriod;
#[derive(Debug, Clone, Copy)]
pub struct HtDcPhase;
#[derive(Debug, Clone, Copy)]
pub struct HtPhasor;
#[derive(Debug, Clone, Copy)]
pub struct HtSine;
#[derive(Debug, Clone, Copy)]
pub struct HtTrendline;
#[derive(Debug, Clone, Copy)]
pub struct HtTrendMode;

#[derive(Clone, Copy)]
struct HilbertHistory {
    odd: [Real; 3],
    even: [Real; 3],
    prev_odd: Real,
    prev_even: Real,
    prev_input_odd: Real,
    prev_input_even: Real,
}

impl HilbertHistory {
    fn new() -> Self {
        Self {
            odd: [0.0; 3],
            even: [0.0; 3],
            prev_odd: 0.0,
            prev_even: 0.0,
            prev_input_odd: 0.0,
            prev_input_even: 0.0,
        }
    }

    fn step(
        &mut self,
        odd_bar: bool,
        hilbert_idx: usize,
        input: Real,
        adjusted_prev_period: Real,
    ) -> Real {
        let hilbert_temp = HILBERT_A * input;
        if odd_bar {
            let mut value = -self.odd[hilbert_idx];
            self.odd[hilbert_idx] = hilbert_temp;
            value += hilbert_temp;
            value -= self.prev_odd;
            self.prev_odd = HILBERT_B * self.prev_input_odd;
            value += self.prev_odd;
            self.prev_input_odd = input;
            value * adjusted_prev_period
        } else {
            let mut value = -self.even[hilbert_idx];
            self.even[hilbert_idx] = hilbert_temp;
            value += hilbert_temp;
            value -= self.prev_even;
            self.prev_even = HILBERT_B * self.prev_input_even;
            value += self.prev_even;
            self.prev_input_even = input;
            value * adjusted_prev_period
        }
    }
}

struct PriceWmaState {
    trailing_idx: usize,
    period_wma_sub: Real,
    period_wma_sum: Real,
    trailing_wma_value: Real,
}

impl PriceWmaState {
    fn initialize(input: &[Real], start_idx: usize, lookback: usize) -> (Self, usize) {
        let trailing_idx = start_idx - lookback;
        let mut today = trailing_idx;

        let first = input[today];
        today += 1;
        let second = input[today];
        today += 1;
        let third = input[today];
        today += 1;

        (
            Self {
                trailing_idx,
                period_wma_sub: first + second + third,
                period_wma_sum: first + second * 2.0 + third * 3.0,
                trailing_wma_value: 0.0,
            },
            today,
        )
    }

    fn step(&mut self, input: &[Real], new_price: Real) -> Real {
        self.period_wma_sub += new_price;
        self.period_wma_sub -= self.trailing_wma_value;
        self.period_wma_sum += new_price * 4.0;
        self.trailing_wma_value = input[self.trailing_idx];
        self.trailing_idx += 1;
        let smoothed = self.period_wma_sum * 0.1;
        self.period_wma_sum -= self.period_wma_sub;
        smoothed
    }
}

enum ShortHtKind {
    DcPeriod,
    Phasor,
}

enum LongHtKind {
    DcPhase,
    Sine,
    TrendMode,
}

impl Indicator for HtDcPeriod {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &HT_DCPERIOD_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        expect_option_count(HT_DCPERIOD_METADATA.name, options, 0)?;
        Ok(SHORT_LOOKBACK)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(HT_DCPERIOD_METADATA.name, inputs)?;
        expect_option_count(HT_DCPERIOD_METADATA.name, options, 0)?;
        let mut output = vec![0.0; input.len().saturating_sub(SHORT_LOOKBACK)];
        let produced = run_short_ht_batch(input, ShortHtKind::DcPeriod, &mut output, None);
        debug_assert_eq!(produced, output.len());
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(HT_DCPERIOD_METADATA.name, inputs)?;
        expect_option_count(HT_DCPERIOD_METADATA.name, options, 0)?;
        let output_len = input.len().saturating_sub(SHORT_LOOKBACK);
        validate_output_slices(&HT_DCPERIOD_METADATA, outputs, 1)?;
        ensure_output_len(&HT_DCPERIOD_METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_short_ht_batch(
            input,
            ShortHtKind::DcPeriod,
            &mut outputs[0][..output_len],
            None,
        ))
    }
}

impl Indicator for HtPhasor {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &HT_PHASOR_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        expect_option_count(HT_PHASOR_METADATA.name, options, 0)?;
        Ok(SHORT_LOOKBACK)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(HT_PHASOR_METADATA.name, inputs)?;
        expect_option_count(HT_PHASOR_METADATA.name, options, 0)?;
        let output_len = input.len().saturating_sub(SHORT_LOOKBACK);
        let mut inphase = vec![0.0; output_len];
        let mut quadrature = vec![0.0; output_len];
        let produced = run_short_ht_batch(
            input,
            ShortHtKind::Phasor,
            &mut inphase,
            Some(&mut quadrature),
        );
        debug_assert_eq!(produced, output_len);
        Ok(vec![inphase, quadrature])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(HT_PHASOR_METADATA.name, inputs)?;
        expect_option_count(HT_PHASOR_METADATA.name, options, 0)?;
        let output_len = input.len().saturating_sub(SHORT_LOOKBACK);
        validate_output_slices(&HT_PHASOR_METADATA, outputs, 2)?;
        ensure_output_len(&HT_PHASOR_METADATA, outputs[0].len(), output_len, 0)?;
        ensure_output_len(&HT_PHASOR_METADATA, outputs[1].len(), output_len, 1)?;
        let (inphase, quadrature) = outputs.split_at_mut(1);
        Ok(run_short_ht_batch(
            input,
            ShortHtKind::Phasor,
            &mut inphase[0][..output_len],
            Some(&mut quadrature[0][..output_len]),
        ))
    }
}

impl Indicator for HtDcPhase {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &HT_DCPHASE_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        expect_option_count(HT_DCPHASE_METADATA.name, options, 0)?;
        Ok(LONG_LOOKBACK)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(HT_DCPHASE_METADATA.name, inputs)?;
        expect_option_count(HT_DCPHASE_METADATA.name, options, 0)?;
        let mut output = vec![0.0; input.len().saturating_sub(LONG_LOOKBACK)];
        let produced = run_long_ht_batch(input, LongHtKind::DcPhase, &mut output, None);
        debug_assert_eq!(produced, output.len());
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(HT_DCPHASE_METADATA.name, inputs)?;
        expect_option_count(HT_DCPHASE_METADATA.name, options, 0)?;
        let output_len = input.len().saturating_sub(LONG_LOOKBACK);
        validate_output_slices(&HT_DCPHASE_METADATA, outputs, 1)?;
        ensure_output_len(&HT_DCPHASE_METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_long_ht_batch(
            input,
            LongHtKind::DcPhase,
            &mut outputs[0][..output_len],
            None,
        ))
    }
}

impl Indicator for HtSine {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &HT_SINE_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        expect_option_count(HT_SINE_METADATA.name, options, 0)?;
        Ok(LONG_LOOKBACK)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(HT_SINE_METADATA.name, inputs)?;
        expect_option_count(HT_SINE_METADATA.name, options, 0)?;
        let output_len = input.len().saturating_sub(LONG_LOOKBACK);
        let mut sine = vec![0.0; output_len];
        let mut lead_sine = vec![0.0; output_len];
        let produced = run_long_ht_batch(input, LongHtKind::Sine, &mut sine, Some(&mut lead_sine));
        debug_assert_eq!(produced, output_len);
        Ok(vec![sine, lead_sine])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(HT_SINE_METADATA.name, inputs)?;
        expect_option_count(HT_SINE_METADATA.name, options, 0)?;
        let output_len = input.len().saturating_sub(LONG_LOOKBACK);
        validate_output_slices(&HT_SINE_METADATA, outputs, 2)?;
        ensure_output_len(&HT_SINE_METADATA, outputs[0].len(), output_len, 0)?;
        ensure_output_len(&HT_SINE_METADATA, outputs[1].len(), output_len, 1)?;
        let (sine, lead_sine) = outputs.split_at_mut(1);
        Ok(run_long_ht_batch(
            input,
            LongHtKind::Sine,
            &mut sine[0][..output_len],
            Some(&mut lead_sine[0][..output_len]),
        ))
    }
}

impl Indicator for HtTrendline {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &HT_TRENDLINE_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        expect_option_count(HT_TRENDLINE_METADATA.name, options, 0)?;
        Ok(LONG_LOOKBACK)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(HT_TRENDLINE_METADATA.name, inputs)?;
        expect_option_count(HT_TRENDLINE_METADATA.name, options, 0)?;
        let mut output = vec![0.0; input.len().saturating_sub(LONG_LOOKBACK)];
        let produced = run_ht_trendline_batch(input, &mut output);
        debug_assert_eq!(produced, output.len());
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(HT_TRENDLINE_METADATA.name, inputs)?;
        expect_option_count(HT_TRENDLINE_METADATA.name, options, 0)?;
        let output_len = input.len().saturating_sub(LONG_LOOKBACK);
        validate_output_slices(&HT_TRENDLINE_METADATA, outputs, 1)?;
        ensure_output_len(&HT_TRENDLINE_METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_ht_trendline_batch(input, &mut outputs[0][..output_len]))
    }
}

impl Indicator for HtTrendMode {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &HT_TRENDMODE_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        expect_option_count(HT_TRENDMODE_METADATA.name, options, 0)?;
        Ok(LONG_LOOKBACK)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(HT_TRENDMODE_METADATA.name, inputs)?;
        expect_option_count(HT_TRENDMODE_METADATA.name, options, 0)?;
        let mut output = vec![0.0; input.len().saturating_sub(LONG_LOOKBACK)];
        let produced = run_long_ht_batch(input, LongHtKind::TrendMode, &mut output, None);
        debug_assert_eq!(produced, output.len());
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let input = single_input(HT_TRENDMODE_METADATA.name, inputs)?;
        expect_option_count(HT_TRENDMODE_METADATA.name, options, 0)?;
        let output_len = input.len().saturating_sub(LONG_LOOKBACK);
        validate_output_slices(&HT_TRENDMODE_METADATA, outputs, 1)?;
        ensure_output_len(&HT_TRENDMODE_METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_long_ht_batch(
            input,
            LongHtKind::TrendMode,
            &mut outputs[0][..output_len],
            None,
        ))
    }
}

fn run_short_ht_batch(
    input: &[Real],
    kind: ShortHtKind,
    output0: &mut [Real],
    output1: Option<&mut [Real]>,
) -> usize {
    if input.len() <= SHORT_LOOKBACK {
        return 0;
    }

    let start_idx = SHORT_LOOKBACK;
    let end_idx = input.len() - 1;
    let rad2deg = 180.0 / (4.0 * Real::atan(1.0));
    let (mut price_wma, mut today) = PriceWmaState::initialize(input, start_idx, SHORT_LOOKBACK);
    for _ in 0..9 {
        let _ = price_wma.step(input, input[today]);
        today += 1;
    }

    let mut hilbert_idx = 0usize;
    let mut detrender = HilbertHistory::new();
    let mut q1 = HilbertHistory::new();
    let mut ji = HilbertHistory::new();
    let mut jq = HilbertHistory::new();
    let mut period = 0.0;
    let mut smooth_period: Real = 0.0;
    let mut prev_q2 = 0.0;
    let mut prev_i2 = 0.0;
    let mut re = 0.0;
    let mut im = 0.0;
    let mut i1_odd_prev2 = 0.0;
    let mut i1_odd_prev3 = 0.0;
    let mut i1_even_prev2 = 0.0;
    let mut i1_even_prev3 = 0.0;
    let mut out_idx = 0usize;
    let mut output1 = output1;

    while today <= end_idx {
        let adjusted_prev_period = 0.075 * period + 0.54;
        let today_value = input[today];
        let smoothed = price_wma.step(input, today_value);
        let odd_bar = (today % 2) != 0;

        if !odd_bar {
            let detrender_value =
                detrender.step(false, hilbert_idx, smoothed, adjusted_prev_period);
            let q1_value = q1.step(false, hilbert_idx, detrender_value, adjusted_prev_period);
            if matches!(kind, ShortHtKind::Phasor) && today >= start_idx {
                output0[out_idx] = i1_even_prev3;
                output1.as_deref_mut().expect("phasor output")[out_idx] = q1_value;
                out_idx += 1;
            }
            let ji_value = ji.step(false, hilbert_idx, i1_even_prev3, adjusted_prev_period);
            let jq_value = jq.step(false, hilbert_idx, q1_value, adjusted_prev_period);
            hilbert_idx = (hilbert_idx + 1) % 3;

            let q2 = 0.2 * (q1_value + ji_value) + 0.8 * prev_q2;
            let i2 = 0.2 * (i1_even_prev3 - jq_value) + 0.8 * prev_i2;

            i1_odd_prev3 = i1_odd_prev2;
            i1_odd_prev2 = detrender_value;

            re = 0.2 * ((i2 * prev_i2) + (q2 * prev_q2)) + 0.8 * re;
            im = 0.2 * ((i2 * prev_q2) - (q2 * prev_i2)) + 0.8 * im;
            prev_q2 = q2;
            prev_i2 = i2;
        } else {
            let detrender_value = detrender.step(true, hilbert_idx, smoothed, adjusted_prev_period);
            let q1_value = q1.step(true, hilbert_idx, detrender_value, adjusted_prev_period);
            if matches!(kind, ShortHtKind::Phasor) && today >= start_idx {
                output0[out_idx] = i1_odd_prev3;
                output1.as_deref_mut().expect("phasor output")[out_idx] = q1_value;
                out_idx += 1;
            }
            let ji_value = ji.step(true, hilbert_idx, i1_odd_prev3, adjusted_prev_period);
            let jq_value = jq.step(true, hilbert_idx, q1_value, adjusted_prev_period);

            let q2 = 0.2 * (q1_value + ji_value) + 0.8 * prev_q2;
            let i2 = 0.2 * (i1_odd_prev3 - jq_value) + 0.8 * prev_i2;

            i1_even_prev3 = i1_even_prev2;
            i1_even_prev2 = detrender_value;

            re = 0.2 * ((i2 * prev_i2) + (q2 * prev_q2)) + 0.8 * re;
            im = 0.2 * ((i2 * prev_q2) - (q2 * prev_i2)) + 0.8 * im;
            prev_q2 = q2;
            prev_i2 = i2;
        }

        let previous_period = period;
        if im != 0.0 && re != 0.0 {
            period = 360.0 / (Real::atan(im / re) * rad2deg);
        }
        let upper = 1.5 * previous_period;
        if period > upper {
            period = upper;
        }
        let lower = 0.67 * previous_period;
        if period < lower {
            period = lower;
        }
        period = period.clamp(6.0, 50.0);
        period = 0.2 * period + 0.8 * previous_period;

        if matches!(kind, ShortHtKind::DcPeriod) {
            smooth_period = smooth_period.mul_add(0.67 as Real, (0.33 as Real) * period);
            if today >= start_idx {
                output0[out_idx] = smooth_period;
                out_idx += 1;
            }
        }

        today += 1;
    }

    out_idx
}

fn run_long_ht_batch(
    input: &[Real],
    kind: LongHtKind,
    output0: &mut [Real],
    output1: Option<&mut [Real]>,
) -> usize {
    if input.len() <= LONG_LOOKBACK {
        return 0;
    }

    let start_idx = LONG_LOOKBACK;
    let end_idx = input.len() - 1;
    let temp_real = Real::atan(1.0);
    let rad2deg = 45.0 / temp_real;
    let deg2rad = 1.0 / rad2deg;
    let const_deg2rad_by_360 = temp_real * 8.0;

    let (mut price_wma, mut today) = PriceWmaState::initialize(input, start_idx, LONG_LOOKBACK);
    for _ in 0..34 {
        let _ = price_wma.step(input, input[today]);
        today += 1;
    }

    let mut hilbert_idx = 0usize;
    let mut detrender = HilbertHistory::new();
    let mut q1 = HilbertHistory::new();
    let mut ji = HilbertHistory::new();
    let mut jq = HilbertHistory::new();
    let mut period = 0.0;
    let mut smooth_period = 0.0;
    let mut prev_q2 = 0.0;
    let mut prev_i2 = 0.0;
    let mut re = 0.0;
    let mut im = 0.0;
    let mut i1_odd_prev2 = 0.0;
    let mut i1_odd_prev3 = 0.0;
    let mut i1_even_prev2 = 0.0;
    let mut i1_even_prev3 = 0.0;
    let mut smooth_price = [0.0; SMOOTH_PRICE_SIZE];
    let mut smooth_price_idx = 0usize;
    let mut dc_phase = 0.0;
    let mut prev_dc_phase = 0.0;
    let mut i_trend1 = 0.0;
    let mut i_trend2 = 0.0;
    let mut i_trend3 = 0.0;
    let mut days_in_trend = 0i32;
    let mut sine = 0.0;
    let mut lead_sine = 0.0;
    let mut out_idx = 0usize;
    let mut output1 = output1;

    while today <= end_idx {
        let adjusted_prev_period = 0.075 * period + 0.54;
        let today_value = input[today];
        let smoothed = price_wma.step(input, today_value);
        smooth_price[smooth_price_idx] = smoothed;
        let odd_bar = (today % 2) != 0;

        if !odd_bar {
            let detrender_value =
                detrender.step(false, hilbert_idx, smoothed, adjusted_prev_period);
            let q1_value = q1.step(false, hilbert_idx, detrender_value, adjusted_prev_period);
            let ji_value = ji.step(false, hilbert_idx, i1_even_prev3, adjusted_prev_period);
            let jq_value = jq.step(false, hilbert_idx, q1_value, adjusted_prev_period);
            hilbert_idx = (hilbert_idx + 1) % 3;

            let q2 = 0.2 * (q1_value + ji_value) + 0.8 * prev_q2;
            let i2 = 0.2 * (i1_even_prev3 - jq_value) + 0.8 * prev_i2;

            i1_odd_prev3 = i1_odd_prev2;
            i1_odd_prev2 = detrender_value;

            re = 0.2 * ((i2 * prev_i2) + (q2 * prev_q2)) + 0.8 * re;
            im = 0.2 * ((i2 * prev_q2) - (q2 * prev_i2)) + 0.8 * im;
            prev_q2 = q2;
            prev_i2 = i2;
        } else {
            let detrender_value = detrender.step(true, hilbert_idx, smoothed, adjusted_prev_period);
            let q1_value = q1.step(true, hilbert_idx, detrender_value, adjusted_prev_period);
            let ji_value = ji.step(true, hilbert_idx, i1_odd_prev3, adjusted_prev_period);
            let jq_value = jq.step(true, hilbert_idx, q1_value, adjusted_prev_period);

            let q2 = 0.2 * (q1_value + ji_value) + 0.8 * prev_q2;
            let i2 = 0.2 * (i1_odd_prev3 - jq_value) + 0.8 * prev_i2;

            i1_even_prev3 = i1_even_prev2;
            i1_even_prev2 = detrender_value;

            re = 0.2 * ((i2 * prev_i2) + (q2 * prev_q2)) + 0.8 * re;
            im = 0.2 * ((i2 * prev_q2) - (q2 * prev_i2)) + 0.8 * im;
            prev_q2 = q2;
            prev_i2 = i2;
        }

        let previous_period = period;
        if im != 0.0 && re != 0.0 {
            period = 360.0 / (Real::atan(im / re) * rad2deg);
        }
        let upper = 1.5 * previous_period;
        if period > upper {
            period = upper;
        }
        let lower = 0.67 * previous_period;
        if period < lower {
            period = lower;
        }
        period = period.clamp(6.0, 50.0);
        period = 0.2 * period + 0.8 * previous_period;
        smooth_period = 0.33 * period + 0.67 * smooth_period;

        let dc_period = smooth_period + 0.5;
        let dc_period_int = dc_period as usize;

        if matches!(
            kind,
            LongHtKind::DcPhase | LongHtKind::Sine | LongHtKind::TrendMode
        ) {
            prev_dc_phase = dc_phase;
            let mut real_part = 0.0;
            let mut imag_part = 0.0;
            let mut idx = smooth_price_idx;
            for i in 0..dc_period_int {
                let angle = (i as Real * const_deg2rad_by_360) / dc_period_int as Real;
                let price = smooth_price[idx];
                real_part += angle.sin() * price;
                imag_part += angle.cos() * price;
                idx = if idx == 0 {
                    SMOOTH_PRICE_SIZE - 1
                } else {
                    idx - 1
                };
            }

            let imag_abs = imag_part.abs();
            if imag_abs > 0.0 {
                dc_phase = Real::atan(real_part / imag_part) * rad2deg;
            } else if imag_abs <= 0.01 {
                if real_part < 0.0 {
                    dc_phase -= 90.0;
                } else if real_part > 0.0 {
                    dc_phase += 90.0;
                }
            }
            dc_phase += 90.0;
            dc_phase += 360.0 / smooth_period;
            if imag_part < 0.0 {
                dc_phase += 180.0;
            }
            if dc_phase > 315.0 {
                dc_phase -= 360.0;
            }
        }

        let mut trendline = 0.0;
        if matches!(kind, LongHtKind::TrendMode) {
            let mut mean = 0.0;
            let mut idx = today as isize;
            for _ in 0..dc_period_int {
                mean += input[idx as usize];
                idx -= 1;
            }
            if dc_period_int > 0 {
                mean /= dc_period_int as Real;
            }
            trendline = (4.0 * mean + 3.0 * i_trend1 + 2.0 * i_trend2 + i_trend3) / 10.0;
            i_trend3 = i_trend2;
            i_trend2 = i_trend1;
            i_trend1 = mean;
        }

        if today >= start_idx {
            match kind {
                LongHtKind::DcPhase => {
                    output0[out_idx] = dc_phase;
                    out_idx += 1;
                }
                LongHtKind::Sine => {
                    output0[out_idx] = (dc_phase * deg2rad).sin();
                    output1.as_deref_mut().expect("sine output")[out_idx] =
                        ((dc_phase + 45.0) * deg2rad).sin();
                    out_idx += 1;
                }
                LongHtKind::TrendMode => {
                    let prev_sine = sine;
                    let prev_lead_sine = lead_sine;
                    sine = (dc_phase * deg2rad).sin();
                    lead_sine = ((dc_phase + 45.0) * deg2rad).sin();

                    let mut trend = 1.0;
                    if (sine > lead_sine && prev_sine <= prev_lead_sine)
                        || (sine < lead_sine && prev_sine >= prev_lead_sine)
                    {
                        days_in_trend = 0;
                        trend = 0.0;
                    }
                    days_in_trend += 1;
                    if (days_in_trend as Real) < 0.5 * smooth_period {
                        trend = 0.0;
                    }

                    let temp = dc_phase - prev_dc_phase;
                    if smooth_period != 0.0
                        && temp > 0.67 * 360.0 / smooth_period
                        && temp < 1.5 * 360.0 / smooth_period
                    {
                        trend = 0.0;
                    }

                    let smoothed = smooth_price[smooth_price_idx];
                    if trendline != 0.0 && ((smoothed - trendline) / trendline).abs() >= 0.015 {
                        trend = 1.0;
                    }

                    output0[out_idx] = trend;
                    out_idx += 1;
                }
            }
        }

        smooth_price_idx = (smooth_price_idx + 1) % SMOOTH_PRICE_SIZE;
        today += 1;
    }

    out_idx
}

fn run_ht_trendline_batch(input: &[Real], output: &mut [Real]) -> usize {
    if input.len() <= LONG_LOOKBACK {
        return 0;
    }

    let start_idx = LONG_LOOKBACK;
    let end_idx = input.len() - 1;
    let temp_real = Real::atan(1.0);
    let rad2deg = 45.0 / temp_real;
    let mut prefix_sum = Vec::with_capacity(input.len() + 1);
    prefix_sum.push(0.0);
    let mut running_sum = 0.0;
    for &value in input {
        running_sum += value;
        prefix_sum.push(running_sum);
    }

    let (mut price_wma, mut today) = PriceWmaState::initialize(input, start_idx, LONG_LOOKBACK);
    for _ in 0..34 {
        let _ = price_wma.step(input, input[today]);
        today += 1;
    }

    let mut hilbert_idx = 0usize;
    let mut detrender = HilbertHistory::new();
    let mut q1 = HilbertHistory::new();
    let mut ji = HilbertHistory::new();
    let mut jq = HilbertHistory::new();
    let mut period = 0.0;
    let mut smooth_period = 0.0;
    let mut prev_q2 = 0.0;
    let mut prev_i2 = 0.0;
    let mut re = 0.0;
    let mut im = 0.0;
    let mut i1_odd_prev2 = 0.0;
    let mut i1_odd_prev3 = 0.0;
    let mut i1_even_prev2 = 0.0;
    let mut i1_even_prev3 = 0.0;
    let mut i_trend1 = 0.0;
    let mut i_trend2 = 0.0;
    let mut i_trend3 = 0.0;
    let mut out_idx = 0usize;

    while today <= end_idx {
        let adjusted_prev_period = 0.075 * period + 0.54;
        let smoothed = price_wma.step(input, input[today]);
        let odd_bar = (today % 2) != 0;

        if !odd_bar {
            let detrender_value =
                detrender.step(false, hilbert_idx, smoothed, adjusted_prev_period);
            let q1_value = q1.step(false, hilbert_idx, detrender_value, adjusted_prev_period);
            let ji_value = ji.step(false, hilbert_idx, i1_even_prev3, adjusted_prev_period);
            let jq_value = jq.step(false, hilbert_idx, q1_value, adjusted_prev_period);
            hilbert_idx = (hilbert_idx + 1) % 3;

            let q2 = 0.2 * (q1_value + ji_value) + 0.8 * prev_q2;
            let i2 = 0.2 * (i1_even_prev3 - jq_value) + 0.8 * prev_i2;

            i1_odd_prev3 = i1_odd_prev2;
            i1_odd_prev2 = detrender_value;
            re = 0.2 * ((i2 * prev_i2) + (q2 * prev_q2)) + 0.8 * re;
            im = 0.2 * ((i2 * prev_q2) - (q2 * prev_i2)) + 0.8 * im;
            prev_q2 = q2;
            prev_i2 = i2;
        } else {
            let detrender_value = detrender.step(true, hilbert_idx, smoothed, adjusted_prev_period);
            let q1_value = q1.step(true, hilbert_idx, detrender_value, adjusted_prev_period);
            let ji_value = ji.step(true, hilbert_idx, i1_odd_prev3, adjusted_prev_period);
            let jq_value = jq.step(true, hilbert_idx, q1_value, adjusted_prev_period);

            let q2 = 0.2 * (q1_value + ji_value) + 0.8 * prev_q2;
            let i2 = 0.2 * (i1_odd_prev3 - jq_value) + 0.8 * prev_i2;

            i1_even_prev3 = i1_even_prev2;
            i1_even_prev2 = detrender_value;
            re = 0.2 * ((i2 * prev_i2) + (q2 * prev_q2)) + 0.8 * re;
            im = 0.2 * ((i2 * prev_q2) - (q2 * prev_i2)) + 0.8 * im;
            prev_q2 = q2;
            prev_i2 = i2;
        }

        let previous_period = period;
        if im != 0.0 && re != 0.0 {
            period = 360.0 / (Real::atan(im / re) * rad2deg);
        }
        let upper = 1.5 * previous_period;
        if period > upper {
            period = upper;
        }
        let lower = 0.67 * previous_period;
        if period < lower {
            period = lower;
        }
        period = period.clamp(6.0, 50.0);
        period = 0.2 * period + 0.8 * previous_period;
        smooth_period = 0.33 * period + 0.67 * smooth_period;

        let dc_period_int = (smooth_period + 0.5) as usize;
        let average = if dc_period_int > 0 {
            let end = today + 1;
            let start = end - dc_period_int;
            (prefix_sum[end] - prefix_sum[start]) / dc_period_int as Real
        } else {
            0.0
        };

        let trendline = (4.0 * average + 3.0 * i_trend1 + 2.0 * i_trend2 + i_trend3) / 10.0;
        i_trend3 = i_trend2;
        i_trend2 = i_trend1;
        i_trend1 = average;

        if today >= start_idx {
            output[out_idx] = trendline;
            out_idx += 1;
        }

        today += 1;
    }

    out_idx
}
