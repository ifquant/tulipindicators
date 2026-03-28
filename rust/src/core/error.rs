use crate::core::types::Real;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum IndicatorError {
    InvalidOption {
        indicator: &'static str,
        option: &'static str,
        value: Real,
        reason: &'static str,
    },
    WrongInputCount {
        indicator: &'static str,
        expected: usize,
        actual: usize,
    },
    WrongOptionCount {
        indicator: &'static str,
        expected: usize,
        actual: usize,
    },
    WrongOutputCount {
        indicator: &'static str,
        expected: usize,
        actual: usize,
    },
    InputLengthMismatch {
        indicator: &'static str,
        expected: usize,
        actual: usize,
        input_index: usize,
    },
    OutputTooSmall {
        indicator: &'static str,
        output_index: usize,
        expected: usize,
        actual: usize,
    },
}

impl fmt::Display for IndicatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOption {
                indicator,
                option,
                value,
                reason,
            } => write!(
                f,
                "{indicator}: invalid option `{option}` with value {value}: {reason}"
            ),
            Self::WrongInputCount {
                indicator,
                expected,
                actual,
            } => write!(
                f,
                "{indicator}: expected {expected} input series, got {actual}"
            ),
            Self::WrongOptionCount {
                indicator,
                expected,
                actual,
            } => write!(f, "{indicator}: expected {expected} options, got {actual}"),
            Self::WrongOutputCount {
                indicator,
                expected,
                actual,
            } => write!(f, "{indicator}: expected {expected} output buffers, got {actual}"),
            Self::InputLengthMismatch {
                indicator,
                expected,
                actual,
                input_index,
            } => write!(
                f,
                "{indicator}: input series {input_index} has length {actual}, expected {expected}"
            ),
            Self::OutputTooSmall {
                indicator,
                output_index,
                expected,
                actual,
            } => write!(
                f,
                "{indicator}: output buffer {output_index} has length {actual}, expected at least {expected}"
            ),
        }
    }
}

impl std::error::Error for IndicatorError {}
