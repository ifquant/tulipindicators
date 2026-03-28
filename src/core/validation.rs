use crate::core::error::IndicatorError;
use crate::core::types::Real;

pub fn expect_input_count(
    indicator: &'static str,
    inputs: &[&[Real]],
    expected: usize,
) -> Result<(), IndicatorError> {
    if inputs.len() != expected {
        return Err(IndicatorError::WrongInputCount {
            indicator,
            expected,
            actual: inputs.len(),
        });
    }

    Ok(())
}

pub fn expect_option_count(
    indicator: &'static str,
    options: &[Real],
    expected: usize,
) -> Result<(), IndicatorError> {
    if options.len() != expected {
        return Err(IndicatorError::WrongOptionCount {
            indicator,
            expected,
            actual: options.len(),
        });
    }

    Ok(())
}

pub fn single_input<'a>(
    indicator: &'static str,
    inputs: &'a [&'a [Real]],
) -> Result<&'a [Real], IndicatorError> {
    expect_input_count(indicator, inputs, 1)?;
    Ok(inputs[0])
}

pub fn double_input<'a>(
    indicator: &'static str,
    inputs: &'a [&'a [Real]],
) -> Result<(&'a [Real], &'a [Real]), IndicatorError> {
    expect_input_count(indicator, inputs, 2)?;
    let expected = inputs[0].len();

    if inputs[1].len() != expected {
        return Err(IndicatorError::InputLengthMismatch {
            indicator,
            expected,
            actual: inputs[1].len(),
            input_index: 1,
        });
    }

    Ok((inputs[0], inputs[1]))
}

pub fn triple_input<'a>(
    indicator: &'static str,
    inputs: &'a [&'a [Real]],
) -> Result<(&'a [Real], &'a [Real], &'a [Real]), IndicatorError> {
    expect_input_count(indicator, inputs, 3)?;
    let expected = inputs[0].len();

    for (input_index, series) in inputs.iter().enumerate().skip(1) {
        if series.len() != expected {
            return Err(IndicatorError::InputLengthMismatch {
                indicator,
                expected,
                actual: series.len(),
                input_index,
            });
        }
    }

    Ok((inputs[0], inputs[1], inputs[2]))
}

pub fn parse_usize_option(
    indicator: &'static str,
    options: &[Real],
    option_index: usize,
    option_name: &'static str,
    minimum: usize,
) -> Result<usize, IndicatorError> {
    let value = options[option_index];

    if !value.is_finite() || value < minimum as Real || value.fract() != 0.0 {
        return Err(IndicatorError::InvalidOption {
            indicator,
            option: option_name,
            value,
            reason: "expected a positive integer",
        });
    }

    Ok(value as usize)
}
