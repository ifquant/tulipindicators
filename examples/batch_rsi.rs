use tulipindicators::{Indicator, Real, Rsi};

fn main() -> Result<(), tulipindicators::IndicatorError> {
    let closes: Vec<Real> = (0..96)
        .map(|index| {
            let x = index as Real;
            100.0 + x * 0.35 + (x * 0.19).sin() * 2.5 + (x * 0.07).cos() * 1.25
        })
        .collect();

    let values = Rsi.run_single(&[&closes], &[14.0])?;
    assert!(!values.is_empty());

    Ok(())
}
