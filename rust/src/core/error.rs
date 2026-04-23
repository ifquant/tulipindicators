//! Error values returned by indicator and state operations.

use crate::core::types::Real;
use std::fmt;

/// Failures reported by the public indicator API.
#[derive(Debug, Clone, PartialEq)]
pub enum IndicatorError {
    /// The caller asked for a registry entry that does not exist.
    UnknownIndicator { name: String },
    /// An option value was outside the indicator's accepted range.
    InvalidOption {
        indicator: &'static str,
        option: &'static str,
        value: Real,
        reason: &'static str,
    },
    /// The caller passed the wrong number of input series.
    WrongInputCount {
        indicator: &'static str,
        expected: usize,
        actual: usize,
    },
    /// The caller passed the wrong number of option values.
    WrongOptionCount {
        indicator: &'static str,
        expected: usize,
        actual: usize,
    },
    /// The caller passed the wrong number of output buffers.
    WrongOutputCount {
        indicator: &'static str,
        expected: usize,
        actual: usize,
    },
    /// A stream-backed indicator does not support the requested batch/stream path.
    MissingStreamSupport { indicator: &'static str },
    /// Input series lengths do not match when a method requires aligned columns.
    InputLengthMismatch {
        indicator: &'static str,
        expected: usize,
        actual: usize,
        input_index: usize,
    },
    /// An output slice is too small to hold the computed values.
    OutputTooSmall {
        indicator: &'static str,
        output_index: usize,
        expected: usize,
        actual: usize,
    },
    /// The implementation returned data that violates an internal API contract.
    InternalInvariant {
        indicator: &'static str,
        reason: &'static str,
    },
}

impl fmt::Display for IndicatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownIndicator { name } => write!(f, "unknown indicator `{name}`"),
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
            Self::MissingStreamSupport { indicator } => {
                write!(f, "{indicator}: stream benchmark requested for indicator without stream support")
            }
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
            Self::InternalInvariant { indicator, reason } => {
                write!(f, "{indicator}: internal invariant violated: {reason}")
            }
        }
    }
}

impl std::error::Error for IndicatorError {}
