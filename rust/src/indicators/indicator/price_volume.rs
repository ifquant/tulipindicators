use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{
    double_input, expect_option_count, parse_usize_option, quadruple_input, single_input,
    triple_input,
};
use crate::indicators::shared::{true_range, EmaState, RingSum};

const AD_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "ad",
    full_name: "Accumulation/Distribution",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low", "close", "volume"],
    option_names: &[],
    output_names: &["ad"],
};

const ADOSC_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "adosc",
    full_name: "Accumulation/Distribution Oscillator",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low", "close", "volume"],
    option_names: &["short_period", "long_period"],
    output_names: &["adosc"],
};

const BOP_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "bop",
    full_name: "Balance of Power",
    category: IndicatorCategory::Indicator,
    input_names: &["open", "high", "low", "close"],
    option_names: &[],
    output_names: &["bop"],
};

const MARKETFI_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "marketfi",
    full_name: "Market Facilitation Index",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low", "volume"],
    option_names: &[],
    output_names: &["marketfi"],
};

const NVI_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "nvi",
    full_name: "Negative Volume Index",
    category: IndicatorCategory::Indicator,
    input_names: &["close", "volume"],
    option_names: &[],
    output_names: &["nvi"],
};

const OBV_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "obv",
    full_name: "On Balance Volume",
    category: IndicatorCategory::Indicator,
    input_names: &["close", "volume"],
    option_names: &[],
    output_names: &["obv"],
};

const PVI_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "pvi",
    full_name: "Positive Volume Index",
    category: IndicatorCategory::Indicator,
    input_names: &["close", "volume"],
    option_names: &[],
    output_names: &["pvi"],
};

const TR_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "tr",
    full_name: "True Range",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low", "close"],
    option_names: &[],
    output_names: &["tr"],
};

const VOSC_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "vosc",
    full_name: "Volume Oscillator",
    category: IndicatorCategory::Indicator,
    input_names: &["volume"],
    option_names: &["short_period", "long_period"],
    output_names: &["vosc"],
};

const WAD_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "wad",
    full_name: "Williams Accumulation/Distribution",
    category: IndicatorCategory::Indicator,
    input_names: &["high", "low", "close"],
    option_names: &[],
    output_names: &["wad"],
};

#[derive(Debug, Clone, Copy)]
pub struct Ad;
#[derive(Debug, Clone, Copy)]
pub struct AdOsc;
#[derive(Debug, Clone, Copy)]
pub struct Bop;
#[derive(Debug, Clone, Copy)]
pub struct MarketFi;
#[derive(Debug, Clone, Copy)]
pub struct Nvi;
#[derive(Debug, Clone, Copy)]
pub struct Obv;
#[derive(Debug, Clone, Copy)]
pub struct Pvi;
#[derive(Debug, Clone, Copy)]
pub struct Tr;
#[derive(Debug, Clone, Copy)]
pub struct Vosc;
#[derive(Debug, Clone, Copy)]
pub struct Wad;

impl Indicator for Ad {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &AD_METADATA
    }

    fn lookback(&self, _options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(0)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = AdStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(AdStream::new(options)?)))
    }
}

impl Indicator for AdOsc {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &ADOSC_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let (_, long_period) = parse_short_long(ADOSC_METADATA.name, options)?;
        Ok(long_period - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = AdOscStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(AdOscStream::new(options)?)))
    }
}

impl Indicator for Bop {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &BOP_METADATA
    }

    fn lookback(&self, _options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(0)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        expect_option_count(BOP_METADATA.name, options, 0)?;
        let (open, high, low, close) = quadruple_input(BOP_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(open.len());

        for (((&open, &high), &low), &close) in open
            .iter()
            .zip(high.iter())
            .zip(low.iter())
            .zip(close.iter())
        {
            let hl = high - low;
            output.push(if hl <= 0.0 { 0.0 } else { (close - open) / hl });
        }

        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        expect_option_count(BOP_METADATA.name, options, 0)?;
        let (open, high, low, close) = quadruple_input(BOP_METADATA.name, inputs)?;
        validate_output_slices(&BOP_METADATA, outputs, 1)?;
        ensure_output_len(&BOP_METADATA, outputs[0].len(), open.len(), 0)?;

        for ((((dst, &open), &high), &low), &close) in outputs[0][..open.len()]
            .iter_mut()
            .zip(open.iter())
            .zip(high.iter())
            .zip(low.iter())
            .zip(close.iter())
        {
            let hl = high - low;
            *dst = if hl <= 0.0 { 0.0 } else { (close - open) / hl };
        }

        Ok(open.len())
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(BopStream::new(options)?)))
    }
}

impl Indicator for MarketFi {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &MARKETFI_METADATA
    }

    fn lookback(&self, _options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(0)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        expect_option_count(MARKETFI_METADATA.name, options, 0)?;
        let (high, low, volume) = triple_input(MARKETFI_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(high.len());

        for ((&high, &low), &volume) in high.iter().zip(low.iter()).zip(volume.iter()) {
            output.push((high - low) / volume);
        }

        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        expect_option_count(MARKETFI_METADATA.name, options, 0)?;
        let (high, low, volume) = triple_input(MARKETFI_METADATA.name, inputs)?;
        validate_output_slices(&MARKETFI_METADATA, outputs, 1)?;
        ensure_output_len(&MARKETFI_METADATA, outputs[0].len(), high.len(), 0)?;

        for (((dst, &high), &low), &volume) in outputs[0][..high.len()]
            .iter_mut()
            .zip(high.iter())
            .zip(low.iter())
            .zip(volume.iter())
        {
            *dst = (high - low) / volume;
        }

        Ok(high.len())
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(MarketFiStream::new(options)?)))
    }
}

impl Indicator for Nvi {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &NVI_METADATA
    }

    fn lookback(&self, _options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(0)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        expect_option_count(NVI_METADATA.name, options, 0)?;
        let (close, volume) = double_input(NVI_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(close.len());
        let mut nvi = 1000.0;

        if close.is_empty() {
            return Ok(vec![output]);
        }

        output.push(nvi);
        for index in 1..close.len() {
            if volume[index] < volume[index - 1] {
                nvi += ((close[index] - close[index - 1]) / close[index - 1]) * nvi;
            }
            output.push(nvi);
        }

        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        expect_option_count(NVI_METADATA.name, options, 0)?;
        let (close, volume) = double_input(NVI_METADATA.name, inputs)?;
        validate_output_slices(&NVI_METADATA, outputs, 1)?;
        ensure_output_len(&NVI_METADATA, outputs[0].len(), close.len(), 0)?;

        if close.is_empty() {
            return Ok(0);
        }

        let mut nvi = 1000.0;
        outputs[0][0] = nvi;
        for index in 1..close.len() {
            if volume[index] < volume[index - 1] {
                nvi += ((close[index] - close[index - 1]) / close[index - 1]) * nvi;
            }
            outputs[0][index] = nvi;
        }

        Ok(close.len())
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(NviStream::new(options)?)))
    }
}

impl Indicator for Obv {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &OBV_METADATA
    }

    fn lookback(&self, _options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(0)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        expect_option_count(OBV_METADATA.name, options, 0)?;
        let (close, volume) = double_input(OBV_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(close.len());
        let mut sum = 0.0;

        if close.is_empty() {
            return Ok(vec![output]);
        }

        output.push(sum);
        let mut previous = close[0];
        for index in 1..close.len() {
            if close[index] > previous {
                sum += volume[index];
            } else if close[index] < previous {
                sum -= volume[index];
            }
            previous = close[index];
            output.push(sum);
        }

        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        expect_option_count(OBV_METADATA.name, options, 0)?;
        let (close, volume) = double_input(OBV_METADATA.name, inputs)?;
        validate_output_slices(&OBV_METADATA, outputs, 1)?;
        ensure_output_len(&OBV_METADATA, outputs[0].len(), close.len(), 0)?;

        if close.is_empty() {
            return Ok(0);
        }

        let mut sum = 0.0;
        outputs[0][0] = sum;
        let mut previous = close[0];
        for index in 1..close.len() {
            if close[index] > previous {
                sum += volume[index];
            } else if close[index] < previous {
                sum -= volume[index];
            }
            previous = close[index];
            outputs[0][index] = sum;
        }

        Ok(close.len())
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(ObvStream::new(options)?)))
    }
}

impl Indicator for Pvi {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &PVI_METADATA
    }

    fn lookback(&self, _options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(0)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        expect_option_count(PVI_METADATA.name, options, 0)?;
        let (close, volume) = double_input(PVI_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(close.len());
        let mut pvi = 1000.0;

        if close.is_empty() {
            return Ok(vec![output]);
        }

        output.push(pvi);
        for index in 1..close.len() {
            if volume[index] > volume[index - 1] {
                pvi += ((close[index] - close[index - 1]) / close[index - 1]) * pvi;
            }
            output.push(pvi);
        }

        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        expect_option_count(PVI_METADATA.name, options, 0)?;
        let (close, volume) = double_input(PVI_METADATA.name, inputs)?;
        validate_output_slices(&PVI_METADATA, outputs, 1)?;
        ensure_output_len(&PVI_METADATA, outputs[0].len(), close.len(), 0)?;

        if close.is_empty() {
            return Ok(0);
        }

        let mut pvi = 1000.0;
        outputs[0][0] = pvi;
        for index in 1..close.len() {
            if volume[index] > volume[index - 1] {
                pvi += ((close[index] - close[index - 1]) / close[index - 1]) * pvi;
            }
            outputs[0][index] = pvi;
        }

        Ok(close.len())
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(PviStream::new(options)?)))
    }
}

impl Indicator for Tr {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &TR_METADATA
    }

    fn lookback(&self, _options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(0)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        expect_option_count(TR_METADATA.name, options, 0)?;
        let (high, low, close) = triple_input(TR_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(high.len());
        let mut previous_close = None;

        for ((&high, &low), &close) in high.iter().zip(low.iter()).zip(close.iter()) {
            output.push(match previous_close {
                Some(previous_close) => true_range(high, low, previous_close),
                None => high - low,
            });
            previous_close = Some(close);
        }

        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        expect_option_count(TR_METADATA.name, options, 0)?;
        let (high, low, close) = triple_input(TR_METADATA.name, inputs)?;
        validate_output_slices(&TR_METADATA, outputs, 1)?;
        ensure_output_len(&TR_METADATA, outputs[0].len(), high.len(), 0)?;

        let mut previous_close = None;
        for (((dst, &high), &low), &close) in outputs[0][..high.len()]
            .iter_mut()
            .zip(high.iter())
            .zip(low.iter())
            .zip(close.iter())
        {
            *dst = match previous_close {
                Some(previous_close) => true_range(high, low, previous_close),
                None => high - low,
            };
            previous_close = Some(close);
        }

        Ok(high.len())
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(TrStream::new(options)?)))
    }
}

impl Indicator for Vosc {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &VOSC_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        let (_, long_period) = parse_short_long(VOSC_METADATA.name, options)?;
        Ok(long_period - 1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let mut stream = VoscStream::new(options)?;
        stream.feed(inputs)
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(VoscStream::new(options)?)))
    }
}

impl Indicator for Wad {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &WAD_METADATA
    }

    fn lookback(&self, _options: &[Real]) -> Result<usize, IndicatorError> {
        Ok(1)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        expect_option_count(WAD_METADATA.name, options, 0)?;
        let (high, low, close) = triple_input(WAD_METADATA.name, inputs)?;
        let output_len = high.len().saturating_sub(1);
        let mut output = vec![0.0; output_len];
        let produced = run_wad_batch(high, low, close, &mut output);
        debug_assert_eq!(produced, output.len());
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        expect_option_count(WAD_METADATA.name, options, 0)?;
        let (high, low, close) = triple_input(WAD_METADATA.name, inputs)?;
        let output_len = high.len().saturating_sub(1);
        validate_output_slices(&WAD_METADATA, outputs, 1)?;
        ensure_output_len(&WAD_METADATA, outputs[0].len(), output_len, 0)?;
        Ok(run_wad_batch(
            high,
            low,
            close,
            &mut outputs[0][..output_len],
        ))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        Ok(Some(Box::new(WadStream::new(options)?)))
    }
}

struct AdStream {
    progress: usize,
    sum: Real,
}

impl AdStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        expect_option_count(AD_METADATA.name, options, 0)?;
        Ok(Self {
            progress: 0,
            sum: 0.0,
        })
    }
}

impl IndicatorStream for AdStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &AD_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close, volume) = quadruple_input(AD_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(high.len());

        for (((&high, &low), &close), &volume) in high
            .iter()
            .zip(low.iter())
            .zip(close.iter())
            .zip(volume.iter())
        {
            let hl = high - low;
            if hl != 0.0 {
                self.sum += (close - low - high + close) / hl * volume;
            }
            output.push(self.sum);
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

struct AdOscStream {
    long_period: usize,
    progress: usize,
    sum: Real,
    short_ema: EmaState,
    long_ema: EmaState,
}

impl AdOscStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let (short_period, long_period) = parse_short_long(ADOSC_METADATA.name, options)?;
        Ok(Self {
            long_period,
            progress: 0,
            sum: 0.0,
            short_ema: EmaState::new(2.0 / (short_period as Real + 1.0)),
            long_ema: EmaState::new(2.0 / (long_period as Real + 1.0)),
        })
    }
}

impl IndicatorStream for AdOscStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &ADOSC_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close, volume) = quadruple_input(ADOSC_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(high.len());

        for (((&high, &low), &close), &volume) in high
            .iter()
            .zip(low.iter())
            .zip(close.iter())
            .zip(volume.iter())
        {
            let hl = high - low;
            if hl != 0.0 {
                self.sum += (close - low - high + close) / hl * volume;
            }

            let short = self.short_ema.feed(self.sum);
            let long = self.long_ema.feed(self.sum);
            if self.progress + 1 >= self.long_period {
                output.push(short - long);
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

struct BopStream {
    progress: usize,
}

impl BopStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        expect_option_count(BOP_METADATA.name, options, 0)?;
        Ok(Self { progress: 0 })
    }
}

impl IndicatorStream for BopStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &BOP_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (open, high, low, close) = quadruple_input(BOP_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(open.len());

        for (((&open, &high), &low), &close) in open
            .iter()
            .zip(high.iter())
            .zip(low.iter())
            .zip(close.iter())
        {
            let hl = high - low;
            output.push(if hl <= 0.0 { 0.0 } else { (close - open) / hl });
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

struct MarketFiStream {
    progress: usize,
}

impl MarketFiStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        expect_option_count(MARKETFI_METADATA.name, options, 0)?;
        Ok(Self { progress: 0 })
    }
}

impl IndicatorStream for MarketFiStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &MARKETFI_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, volume) = triple_input(MARKETFI_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(high.len());

        for ((&high, &low), &volume) in high.iter().zip(low.iter()).zip(volume.iter()) {
            output.push((high - low) / volume);
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

struct NviStream {
    progress: usize,
    previous_close: Option<Real>,
    previous_volume: Option<Real>,
    nvi: Real,
}

impl NviStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        expect_option_count(NVI_METADATA.name, options, 0)?;
        Ok(Self {
            progress: 0,
            previous_close: None,
            previous_volume: None,
            nvi: 1000.0,
        })
    }
}

impl IndicatorStream for NviStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &NVI_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (close, volume) = double_input(NVI_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(close.len());

        for (&close, &volume) in close.iter().zip(volume.iter()) {
            if let (Some(previous_close), Some(previous_volume)) =
                (self.previous_close, self.previous_volume)
            {
                if volume < previous_volume {
                    self.nvi += ((close - previous_close) / previous_close) * self.nvi;
                }
            }
            output.push(self.nvi);
            self.previous_close = Some(close);
            self.previous_volume = Some(volume);
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

struct ObvStream {
    progress: usize,
    previous_close: Option<Real>,
    sum: Real,
}

impl ObvStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        expect_option_count(OBV_METADATA.name, options, 0)?;
        Ok(Self {
            progress: 0,
            previous_close: None,
            sum: 0.0,
        })
    }
}

impl IndicatorStream for ObvStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &OBV_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (close, volume) = double_input(OBV_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(close.len());

        for (&close, &volume) in close.iter().zip(volume.iter()) {
            if let Some(previous_close) = self.previous_close {
                if close > previous_close {
                    self.sum += volume;
                } else if close < previous_close {
                    self.sum -= volume;
                }
            }
            output.push(self.sum);
            self.previous_close = Some(close);
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

struct PviStream {
    progress: usize,
    previous_close: Option<Real>,
    previous_volume: Option<Real>,
    pvi: Real,
}

impl PviStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        expect_option_count(PVI_METADATA.name, options, 0)?;
        Ok(Self {
            progress: 0,
            previous_close: None,
            previous_volume: None,
            pvi: 1000.0,
        })
    }
}

impl IndicatorStream for PviStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &PVI_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (close, volume) = double_input(PVI_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(close.len());

        for (&close, &volume) in close.iter().zip(volume.iter()) {
            if let (Some(previous_close), Some(previous_volume)) =
                (self.previous_close, self.previous_volume)
            {
                if volume > previous_volume {
                    self.pvi += ((close - previous_close) / previous_close) * self.pvi;
                }
            }
            output.push(self.pvi);
            self.previous_close = Some(close);
            self.previous_volume = Some(volume);
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

struct TrStream {
    progress: usize,
    previous_close: Option<Real>,
}

impl TrStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        expect_option_count(TR_METADATA.name, options, 0)?;
        Ok(Self {
            progress: 0,
            previous_close: None,
        })
    }
}

impl IndicatorStream for TrStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &TR_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close) = triple_input(TR_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(high.len());

        for ((&high, &low), &close) in high.iter().zip(low.iter()).zip(close.iter()) {
            output.push(match self.previous_close {
                Some(previous_close) => true_range(high, low, previous_close),
                None => high - low,
            });
            self.previous_close = Some(close);
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

struct VoscStream {
    short_period: usize,
    long_period: usize,
    progress: usize,
    short_sum: RingSum,
    long_sum: RingSum,
}

impl VoscStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        let (short_period, long_period) = parse_short_long(VOSC_METADATA.name, options)?;
        Ok(Self {
            short_period,
            long_period,
            progress: 0,
            short_sum: RingSum::new(short_period),
            long_sum: RingSum::new(long_period),
        })
    }
}

impl IndicatorStream for VoscStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &VOSC_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(VOSC_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(input.len());

        for &sample in input {
            self.short_sum.push(sample);
            self.long_sum.push(sample);

            if self.long_sum.is_full() {
                let short_avg = self.short_sum.sum / self.short_period as Real;
                let long_avg = self.long_sum.sum / self.long_period as Real;
                output.push(100.0 * (short_avg - long_avg) / long_avg);
            }

            self.progress += 1;
        }

        Ok(vec![output])
    }
}

struct WadStream {
    progress: usize,
    previous_close: Option<Real>,
    sum: Real,
}

impl WadStream {
    fn new(options: &[Real]) -> Result<Self, IndicatorError> {
        expect_option_count(WAD_METADATA.name, options, 0)?;
        Ok(Self {
            progress: 0,
            previous_close: None,
            sum: 0.0,
        })
    }
}

impl IndicatorStream for WadStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &WAD_METADATA
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (high, low, close) = triple_input(WAD_METADATA.name, inputs)?;
        let mut output = Vec::with_capacity(high.len().saturating_sub(1));

        for ((&high, &low), &close) in high.iter().zip(low.iter()).zip(close.iter()) {
            if let Some(previous_close) = self.previous_close {
                if close > previous_close {
                    self.sum += close - previous_close.min(low);
                } else if close < previous_close {
                    self.sum += close - previous_close.max(high);
                }
                output.push(self.sum);
            }
            self.previous_close = Some(close);
            self.progress += 1;
        }

        Ok(vec![output])
    }
}

fn run_wad_batch(high: &[Real], low: &[Real], close: &[Real], output: &mut [Real]) -> usize {
    if close.len() <= 1 {
        return 0;
    }

    let mut sum = 0.0;
    let mut previous_close = close[0];
    for (dst, ((&high, &low), &close)) in output
        .iter_mut()
        .zip(high.iter().zip(low.iter()).zip(close.iter()).skip(1))
    {
        if close > previous_close {
            sum += close - previous_close.min(low);
        } else if close < previous_close {
            sum += close - previous_close.max(high);
        }
        *dst = sum;
        previous_close = close;
    }

    output.len()
}

fn parse_short_long(
    indicator: &'static str,
    options: &[Real],
) -> Result<(usize, usize), IndicatorError> {
    expect_option_count(indicator, options, 2)?;
    let short_period = parse_usize_option(indicator, options, 0, "short_period", 1)?;
    let long_period = parse_usize_option(indicator, options, 1, "long_period", 1)?;
    if long_period < short_period {
        return Err(IndicatorError::InvalidOption {
            indicator,
            option: "long_period",
            value: options[1],
            reason: "expected long_period >= short_period",
        });
    }
    Ok((short_period, long_period))
}
