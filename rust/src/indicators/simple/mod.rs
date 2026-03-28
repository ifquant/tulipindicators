use crate::core::error::IndicatorError;
use crate::core::indicator::{
    ensure_output_len, validate_output_slices, Indicator, IndicatorMetadata, IndicatorStream,
};
use crate::core::types::{IndicatorCategory, Real};
use crate::core::validation::{double_input, expect_option_count, single_input};
use std::f64::consts::PI;

struct UnaryTransformStream {
    metadata: &'static IndicatorMetadata,
    progress: usize,
    op: fn(Real) -> Real,
}

impl UnaryTransformStream {
    fn new(metadata: &'static IndicatorMetadata, op: fn(Real) -> Real) -> Self {
        Self {
            metadata,
            progress: 0,
            op,
        }
    }
}

impl IndicatorStream for UnaryTransformStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        self.metadata
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let input = single_input(self.metadata.name, inputs)?;
        let output = input.iter().map(|&sample| (self.op)(sample)).collect();
        self.progress += input.len();
        Ok(vec![output])
    }
}

struct BinaryTransformStream {
    metadata: &'static IndicatorMetadata,
    progress: usize,
    op: fn(Real, Real) -> Real,
}

impl BinaryTransformStream {
    fn new(metadata: &'static IndicatorMetadata, op: fn(Real, Real) -> Real) -> Self {
        Self {
            metadata,
            progress: 0,
            op,
        }
    }
}

impl IndicatorStream for BinaryTransformStream {
    fn metadata(&self) -> &'static IndicatorMetadata {
        self.metadata
    }

    fn progress(&self) -> usize {
        self.progress
    }

    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError> {
        let (left, right) = double_input(self.metadata.name, inputs)?;
        let output = left
            .iter()
            .zip(right.iter())
            .map(|(&lhs, &rhs)| (self.op)(lhs, rhs))
            .collect();
        self.progress += left.len();
        Ok(vec![output])
    }
}

fn run_unary(
    metadata: &'static IndicatorMetadata,
    inputs: &[&[Real]],
    options: &[Real],
    op: fn(Real) -> Real,
) -> Result<Vec<Vec<Real>>, IndicatorError> {
    expect_option_count(metadata.name, options, 0)?;
    let input = single_input(metadata.name, inputs)?;
    let output = input.iter().map(|&sample| op(sample)).collect();
    Ok(vec![output])
}

fn run_unary_in_place(
    metadata: &'static IndicatorMetadata,
    inputs: &[&[Real]],
    options: &[Real],
    outputs: &mut [&mut [Real]],
    op: fn(Real) -> Real,
) -> Result<usize, IndicatorError> {
    expect_option_count(metadata.name, options, 0)?;
    let input = single_input(metadata.name, inputs)?;
    validate_output_slices(metadata, outputs, 1)?;
    ensure_output_len(metadata, outputs[0].len(), input.len(), 0)?;

    for (dst, &sample) in outputs[0][..input.len()].iter_mut().zip(input.iter()) {
        *dst = op(sample);
    }

    Ok(input.len())
}

fn run_binary(
    metadata: &'static IndicatorMetadata,
    inputs: &[&[Real]],
    options: &[Real],
    op: fn(Real, Real) -> Real,
) -> Result<Vec<Vec<Real>>, IndicatorError> {
    expect_option_count(metadata.name, options, 0)?;
    let (left, right) = double_input(metadata.name, inputs)?;
    let mut output = vec![0.0; left.len()];
    for index in 0..left.len() {
        output[index] = op(left[index], right[index]);
    }
    Ok(vec![output])
}

fn run_binary_in_place(
    metadata: &'static IndicatorMetadata,
    inputs: &[&[Real]],
    options: &[Real],
    outputs: &mut [&mut [Real]],
    op: fn(Real, Real) -> Real,
) -> Result<usize, IndicatorError> {
    expect_option_count(metadata.name, options, 0)?;
    let (left, right) = double_input(metadata.name, inputs)?;
    validate_output_slices(metadata, outputs, 1)?;
    ensure_output_len(metadata, outputs[0].len(), left.len(), 0)?;

    for index in 0..left.len() {
        outputs[0][index] = op(left[index], right[index]);
    }

    Ok(left.len())
}

fn abs_op(sample: Real) -> Real {
    sample.abs()
}

fn acos_op(sample: Real) -> Real {
    sample.acos()
}

fn asin_op(sample: Real) -> Real {
    sample.asin()
}

fn atan_op(sample: Real) -> Real {
    sample.atan()
}

fn ceil_op(sample: Real) -> Real {
    sample.ceil()
}

fn cos_op(sample: Real) -> Real {
    sample.cos()
}

fn cosh_op(sample: Real) -> Real {
    sample.cosh()
}

fn exp_op(sample: Real) -> Real {
    sample.exp()
}

fn floor_op(sample: Real) -> Real {
    sample.floor()
}

fn ln_op(sample: Real) -> Real {
    sample.ln()
}

fn log10_op(sample: Real) -> Real {
    sample.log10()
}

fn round_op(sample: Real) -> Real {
    (sample + 0.5).floor()
}

fn sin_op(sample: Real) -> Real {
    sample.sin()
}

fn sinh_op(sample: Real) -> Real {
    sample.sinh()
}

fn sqrt_op(sample: Real) -> Real {
    sample.sqrt()
}

fn tan_op(sample: Real) -> Real {
    sample.tan()
}

fn tanh_op(sample: Real) -> Real {
    sample.tanh()
}

fn todeg_op(sample: Real) -> Real {
    sample * (180.0 / PI)
}

fn torad_op(sample: Real) -> Real {
    sample * (PI / 180.0)
}

fn trunc_op(sample: Real) -> Real {
    sample.trunc()
}

fn add_op(lhs: Real, rhs: Real) -> Real {
    lhs + rhs
}

fn sub_op(lhs: Real, rhs: Real) -> Real {
    lhs - rhs
}

fn mul_op(lhs: Real, rhs: Real) -> Real {
    lhs * rhs
}

fn div_op(lhs: Real, rhs: Real) -> Real {
    lhs / rhs
}

macro_rules! define_unary_indicator {
    ($struct:ident, $metadata:ident, $name:literal, $full:literal, $output:literal, $op:path) => {
        const $metadata: IndicatorMetadata = IndicatorMetadata {
            name: $name,
            full_name: $full,
            category: IndicatorCategory::Simple,
            input_names: &["real"],
            option_names: &[],
            output_names: &[$output],
        };

        #[derive(Debug, Clone, Copy)]
        pub struct $struct;

        impl Indicator for $struct {
            fn metadata(&self) -> &'static IndicatorMetadata {
                &$metadata
            }

            fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
                expect_option_count($name, options, 0)?;
                Ok(0)
            }

            fn run(
                &self,
                inputs: &[&[Real]],
                options: &[Real],
            ) -> Result<Vec<Vec<Real>>, IndicatorError> {
                run_unary(&$metadata, inputs, options, $op)
            }

            fn run_in_place(
                &self,
                inputs: &[&[Real]],
                options: &[Real],
                outputs: &mut [&mut [Real]],
            ) -> Result<usize, IndicatorError> {
                run_unary_in_place(&$metadata, inputs, options, outputs, $op)
            }

            fn create_stream(
                &self,
                options: &[Real],
            ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
                expect_option_count($name, options, 0)?;
                Ok(Some(Box::new(UnaryTransformStream::new(&$metadata, $op))))
            }
        }
    };
}

macro_rules! define_binary_indicator {
    ($struct:ident, $metadata:ident, $name:literal, $full:literal, $output:literal, $op:path) => {
        const $metadata: IndicatorMetadata = IndicatorMetadata {
            name: $name,
            full_name: $full,
            category: IndicatorCategory::Simple,
            input_names: &["real", "real"],
            option_names: &[],
            output_names: &[$output],
        };

        #[derive(Debug, Clone, Copy)]
        pub struct $struct;

        impl Indicator for $struct {
            fn metadata(&self) -> &'static IndicatorMetadata {
                &$metadata
            }

            fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
                expect_option_count($name, options, 0)?;
                Ok(0)
            }

            fn run(
                &self,
                inputs: &[&[Real]],
                options: &[Real],
            ) -> Result<Vec<Vec<Real>>, IndicatorError> {
                run_binary(&$metadata, inputs, options, $op)
            }

            fn run_in_place(
                &self,
                inputs: &[&[Real]],
                options: &[Real],
                outputs: &mut [&mut [Real]],
            ) -> Result<usize, IndicatorError> {
                run_binary_in_place(&$metadata, inputs, options, outputs, $op)
            }

            fn create_stream(
                &self,
                options: &[Real],
            ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
                expect_option_count($name, options, 0)?;
                Ok(Some(Box::new(BinaryTransformStream::new(&$metadata, $op))))
            }
        }
    };
}

define_unary_indicator!(
    Abs,
    ABS_METADATA,
    "abs",
    "Vector Absolute Value",
    "abs",
    abs_op
);
define_unary_indicator!(
    Acos,
    ACOS_METADATA,
    "acos",
    "Vector Arccosine",
    "acos",
    acos_op
);
define_unary_indicator!(
    Asin,
    ASIN_METADATA,
    "asin",
    "Vector Arcsine",
    "asin",
    asin_op
);
define_unary_indicator!(
    Atan,
    ATAN_METADATA,
    "atan",
    "Vector Arctangent",
    "atan",
    atan_op
);
define_unary_indicator!(
    Ceil,
    CEIL_METADATA,
    "ceil",
    "Vector Ceiling",
    "ceil",
    ceil_op
);
define_unary_indicator!(Cos, COS_METADATA, "cos", "Vector Cosine", "cos", cos_op);
define_unary_indicator!(
    Cosh,
    COSH_METADATA,
    "cosh",
    "Vector Hyperbolic Cosine",
    "cosh",
    cosh_op
);
define_unary_indicator!(
    Exp,
    EXP_METADATA,
    "exp",
    "Vector Exponential",
    "exp",
    exp_op
);
const FLOOR_METADATA: IndicatorMetadata = IndicatorMetadata {
    name: "floor",
    full_name: "Vector Floor",
    category: IndicatorCategory::Simple,
    input_names: &["real"],
    option_names: &[],
    output_names: &["floor"],
};

#[derive(Debug, Clone, Copy)]
pub struct Floor;

impl Indicator for Floor {
    fn metadata(&self) -> &'static IndicatorMetadata {
        &FLOOR_METADATA
    }

    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError> {
        expect_option_count(FLOOR_METADATA.name, options, 0)?;
        Ok(0)
    }

    fn run(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
    ) -> Result<Vec<Vec<Real>>, IndicatorError> {
        expect_option_count(FLOOR_METADATA.name, options, 0)?;
        let input = single_input(FLOOR_METADATA.name, inputs)?;
        let mut output = vec![0.0; input.len()];
        for index in 0..input.len() {
            output[index] = input[index].floor();
        }
        Ok(vec![output])
    }

    fn run_in_place(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
        outputs: &mut [&mut [Real]],
    ) -> Result<usize, IndicatorError> {
        expect_option_count(FLOOR_METADATA.name, options, 0)?;
        let input = single_input(FLOOR_METADATA.name, inputs)?;
        validate_output_slices(&FLOOR_METADATA, outputs, 1)?;
        ensure_output_len(&FLOOR_METADATA, outputs[0].len(), input.len(), 0)?;

        for index in 0..input.len() {
            outputs[0][index] = input[index].floor();
        }

        Ok(input.len())
    }

    fn create_stream(
        &self,
        options: &[Real],
    ) -> Result<Option<Box<dyn IndicatorStream>>, IndicatorError> {
        expect_option_count(FLOOR_METADATA.name, options, 0)?;
        Ok(Some(Box::new(UnaryTransformStream::new(
            &FLOOR_METADATA,
            floor_op,
        ))))
    }
}
define_unary_indicator!(Ln, LN_METADATA, "ln", "Vector Natural Log", "ln", ln_op);
define_unary_indicator!(
    Log10,
    LOG10_METADATA,
    "log10",
    "Vector Base-10 Log",
    "log10",
    log10_op
);
define_unary_indicator!(
    Round,
    ROUND_METADATA,
    "round",
    "Vector Round",
    "round",
    round_op
);
define_unary_indicator!(Sin, SIN_METADATA, "sin", "Vector Sine", "sin", sin_op);
define_unary_indicator!(
    Sinh,
    SINH_METADATA,
    "sinh",
    "Vector Hyperbolic Sine",
    "sinh",
    sinh_op
);
define_unary_indicator!(
    Sqrt,
    SQRT_METADATA,
    "sqrt",
    "Vector Square Root",
    "sqrt",
    sqrt_op
);
define_unary_indicator!(Tan, TAN_METADATA, "tan", "Vector Tangent", "tan", tan_op);
define_unary_indicator!(
    Tanh,
    TANH_METADATA,
    "tanh",
    "Vector Hyperbolic Tangent",
    "tanh",
    tanh_op
);
define_unary_indicator!(
    ToDeg,
    TODEG_METADATA,
    "todeg",
    "Vector Degree Conversion",
    "degrees",
    todeg_op
);
define_unary_indicator!(
    ToRad,
    TORAD_METADATA,
    "torad",
    "Vector Radian Conversion",
    "radians",
    torad_op
);
define_unary_indicator!(
    Trunc,
    TRUNC_METADATA,
    "trunc",
    "Vector Truncate",
    "trunc",
    trunc_op
);

define_binary_indicator!(Add, ADD_METADATA, "add", "Vector Addition", "add", add_op);
define_binary_indicator!(
    Sub,
    SUB_METADATA,
    "sub",
    "Vector Subtraction",
    "sub",
    sub_op
);
define_binary_indicator!(
    Mul,
    MUL_METADATA,
    "mul",
    "Vector Multiplication",
    "mul",
    mul_op
);
define_binary_indicator!(Div, DIV_METADATA, "div", "Vector Division", "div", div_op);
