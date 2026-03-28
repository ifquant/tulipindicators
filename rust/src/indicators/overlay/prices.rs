use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{double_input, expect_option_count, quadruple_input, triple_input};

struct DoubleOverlayStream {
    metadata: &'static IndicatorMetadata,
    progress: usize,
    op: fn(Real, Real) -> Real,
}

impl DoubleOverlayStream {
    fn new(metadata: &'static IndicatorMetadata, op: fn(Real, Real) -> Real) -> Self {
        Self {
            metadata,
            progress: 0,
            op,
        }
    }
}

impl IndicatorStream for DoubleOverlayStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        self.metadata
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (first, second) = double_input(self.metadata.name, inputs)?;
        let output = first
            .iter()
            .zip(second.iter())
            .map(|(&a, &b)| (self.op)(a, b))
            .collect();
        self.progress += first.len();
        Ok(vec![output])
    }

    fn feed_in_place(
        &mut self,
        inputs: &[&[Real]],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let (first, second) = double_input(self.metadata.name, inputs)?;
        validate_output_slices(self.metadata, outputs, 1)?;
        ensure_output_len(self.metadata, outputs[0].len(), first.len(), 0)?;

        for index in 0..first.len() {
            outputs[0][index] = (self.op)(first[index], second[index]);
        }

        self.progress += first.len();
        Ok(first.len())
    }
}

struct TripleOverlayStream {
    metadata: &'static IndicatorMetadata,
    progress: usize,
    op: fn(Real, Real, Real) -> Real,
}

impl TripleOverlayStream {
    fn new(metadata: &'static IndicatorMetadata, op: fn(Real, Real, Real) -> Real) -> Self {
        Self {
            metadata,
            progress: 0,
            op,
        }
    }
}

impl IndicatorStream for TripleOverlayStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        self.metadata
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (first, second, third) = triple_input(self.metadata.name, inputs)?;
        let output = first
            .iter()
            .zip(second.iter())
            .zip(third.iter())
            .map(|((&a, &b), &c)| (self.op)(a, b, c))
            .collect();
        self.progress += first.len();
        Ok(vec![output])
    }

    fn feed_in_place(
        &mut self,
        inputs: &[&[Real]],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let (first, second, third) = triple_input(self.metadata.name, inputs)?;
        validate_output_slices(self.metadata, outputs, 1)?;
        ensure_output_len(self.metadata, outputs[0].len(), first.len(), 0)?;

        for index in 0..first.len() {
            outputs[0][index] = (self.op)(first[index], second[index], third[index]);
        }

        self.progress += first.len();
        Ok(first.len())
    }
}

struct QuadOverlayStream {
    metadata: &'static IndicatorMetadata,
    progress: usize,
    op: fn(Real, Real, Real, Real) -> Real,
}

impl QuadOverlayStream {
    fn new(metadata: &'static IndicatorMetadata, op: fn(Real, Real, Real, Real) -> Real) -> Self {
        Self {
            metadata,
            progress: 0,
            op,
        }
    }
}

impl IndicatorStream for QuadOverlayStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        self.metadata
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (first, second, third, fourth) = quadruple_input(self.metadata.name, inputs)?;
        let output = first
            .iter()
            .zip(second.iter())
            .zip(third.iter())
            .zip(fourth.iter())
            .map(|(((&a, &b), &c), &d)| (self.op)(a, b, c, d))
            .collect();
        self.progress += first.len();
        Ok(vec![output])
    }

    fn feed_in_place(
        &mut self,
        inputs: &[&[Real]],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        let (first, second, third, fourth) = quadruple_input(self.metadata.name, inputs)?;
        validate_output_slices(self.metadata, outputs, 1)?;
        ensure_output_len(self.metadata, outputs[0].len(), first.len(), 0)?;

        for index in 0..first.len() {
            outputs[0][index] = (self.op)(first[index], second[index], third[index], fourth[index]);
        }

        self.progress += first.len();
        Ok(first.len())
    }
}

fn avgprice_op(open: Real, high: Real, low: Real, close: Real) -> Real {
    (open + high + low + close) * 0.25
}

fn medprice_op(high: Real, low: Real) -> Real {
    (high + low) * 0.5
}

fn typprice_op(high: Real, low: Real, close: Real) -> Real {
    (high + low + close) / 3.0
}

fn wcprice_op(high: Real, low: Real, close: Real) -> Real {
    (high + low + close + close) * 0.25
}

const AVGPRICE_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "avgprice",
    full_name: "Average Price",
    category: IndicatorCategory::Overlay,
    input_names: &["open", "high", "low", "close"],
    option_names: &[],
    output_names: &["avgprice"],
};

const MEDPRICE_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "medprice",
    full_name: "Median Price",
    category: IndicatorCategory::Overlay,
    input_names: &["high", "low"],
    option_names: &[],
    output_names: &["medprice"],
};

const TYPPRICE_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "typprice",
    full_name: "Typical Price",
    category: IndicatorCategory::Overlay,
    input_names: &["high", "low", "close"],
    option_names: &[],
    output_names: &["typprice"],
};

const WCPRICE_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "wcprice",
    full_name: "Weighted Close Price",
    category: IndicatorCategory::Overlay,
    input_names: &["high", "low", "close"],
    option_names: &[],
    output_names: &["wcprice"],
};

#[derive(Debug, Clone, Copy)]
pub struct AvgPrice;

impl Indicator for AvgPrice {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &AVGPRICE_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        expect_option_count(AVGPRICE_METADATA.name, options, 0)?;
        Ok(0)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        expect_option_count(AVGPRICE_METADATA.name, options, 0)?;
        let (open, high, low, close) = quadruple_input(AVGPRICE_METADATA.name, inputs)?;
        let mut output = vec![0.0; open.len()];
        let produced = run_avgprice_batch(open, high, low, close, &mut output);
        debug_assert_eq!(produced, output.len());
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        expect_option_count(AVGPRICE_METADATA.name, options, 0)?;
        let (open, high, low, close) = quadruple_input(AVGPRICE_METADATA.name, inputs)?;
        validate_output_slices(&AVGPRICE_METADATA, outputs, 1)?;
        ensure_output_len(&AVGPRICE_METADATA, outputs[0].len(), open.len(), 0)?;
        Ok(run_avgprice_batch(
            open,
            high,
            low,
            close,
            &mut outputs[0][..open.len()],
        ))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        expect_option_count(AVGPRICE_METADATA.name, options, 0)?;
        Ok(Some(Box::new(QuadOverlayStream::new(
            &AVGPRICE_METADATA,
            avgprice_op,
        ))))
    }
}

fn run_avgprice_batch(
    open: &[Real],
    high: &[Real],
    low: &[Real],
    close: &[Real],
    output: &mut [Real],
) -> usize {
    for ((((dst, &open), &high), &low), &close) in output
        .iter_mut()
        .zip(open.iter())
        .zip(high.iter())
        .zip(low.iter())
        .zip(close.iter())
    {
        *dst = (open + high + low + close) * 0.25;
    }

    output.len()
}

#[derive(Debug, Clone, Copy)]
pub struct MedPrice;

impl Indicator for MedPrice {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &MEDPRICE_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        expect_option_count(MEDPRICE_METADATA.name, options, 0)?;
        Ok(0)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        expect_option_count(MEDPRICE_METADATA.name, options, 0)?;
        let (high, low) = double_input(MEDPRICE_METADATA.name, inputs)?;
        let mut output = vec![0.0; high.len()];
        let produced = run_medprice_batch(high, low, &mut output);
        debug_assert_eq!(produced, output.len());
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        expect_option_count(MEDPRICE_METADATA.name, options, 0)?;
        let (high, low) = double_input(MEDPRICE_METADATA.name, inputs)?;
        validate_output_slices(&MEDPRICE_METADATA, outputs, 1)?;
        ensure_output_len(&MEDPRICE_METADATA, outputs[0].len(), high.len(), 0)?;
        Ok(run_medprice_batch(high, low, &mut outputs[0][..high.len()]))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        expect_option_count(MEDPRICE_METADATA.name, options, 0)?;
        Ok(Some(Box::new(DoubleOverlayStream::new(
            &MEDPRICE_METADATA,
            medprice_op,
        ))))
    }
}

fn run_medprice_batch(high: &[Real], low: &[Real], output: &mut [Real]) -> usize {
    for index in 0..high.len() {
        output[index] = (high[index] + low[index]) * 0.5;
    }
    output.len()
}

#[derive(Debug, Clone, Copy)]
pub struct TypPrice;

impl Indicator for TypPrice {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &TYPPRICE_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        expect_option_count(TYPPRICE_METADATA.name, options, 0)?;
        Ok(0)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        expect_option_count(TYPPRICE_METADATA.name, options, 0)?;
        let (high, low, close) = triple_input(TYPPRICE_METADATA.name, inputs)?;
        let mut output = vec![0.0; high.len()];
        let produced = run_typprice_batch(high, low, close, &mut output);
        debug_assert_eq!(produced, output.len());
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        expect_option_count(TYPPRICE_METADATA.name, options, 0)?;
        let (high, low, close) = triple_input(TYPPRICE_METADATA.name, inputs)?;
        validate_output_slices(&TYPPRICE_METADATA, outputs, 1)?;
        ensure_output_len(&TYPPRICE_METADATA, outputs[0].len(), high.len(), 0)?;
        Ok(run_typprice_batch(
            high,
            low,
            close,
            &mut outputs[0][..high.len()],
        ))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        expect_option_count(TYPPRICE_METADATA.name, options, 0)?;
        Ok(Some(Box::new(TripleOverlayStream::new(
            &TYPPRICE_METADATA,
            typprice_op,
        ))))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WcPrice;

impl Indicator for WcPrice {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &WCPRICE_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        expect_option_count(WCPRICE_METADATA.name, options, 0)?;
        Ok(0)
    }

    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        expect_option_count(WCPRICE_METADATA.name, options, 0)?;
        let (high, low, close) = triple_input(WCPRICE_METADATA.name, inputs)?;
        let mut output = vec![0.0; high.len()];
        let produced = run_wcprice_batch(high, low, close, &mut output);
        debug_assert_eq!(produced, output.len());
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        expect_option_count(WCPRICE_METADATA.name, options, 0)?;
        let (high, low, close) = triple_input(WCPRICE_METADATA.name, inputs)?;
        validate_output_slices(&WCPRICE_METADATA, outputs, 1)?;
        ensure_output_len(&WCPRICE_METADATA, outputs[0].len(), high.len(), 0)?;
        Ok(run_wcprice_batch(
            high,
            low,
            close,
            &mut outputs[0][..high.len()],
        ))
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        expect_option_count(WCPRICE_METADATA.name, options, 0)?;
        Ok(Some(Box::new(TripleOverlayStream::new(
            &WCPRICE_METADATA,
            wcprice_op,
        ))))
    }
}

fn run_typprice_batch(high: &[Real], low: &[Real], close: &[Real], output: &mut [Real]) -> usize {
    for index in 0..high.len() {
        output[index] = (high[index] + low[index] + close[index]) * (1.0 / 3.0);
    }
    output.len()
}

fn run_wcprice_batch(high: &[Real], low: &[Real], close: &[Real], output: &mut [Real]) -> usize {
    for index in 0..high.len() {
        output[index] = (high[index] + low[index] + close[index] + close[index]) * 0.25;
    }
    output.len()
}
