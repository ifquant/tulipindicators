use tulipindicators::{DynamicIndicatorState, Real};

fn main() -> Result<(), tulipindicators::IndicatorError> {
    let closes: Vec<Real> = (0..96)
        .map(|index| {
            let x = index as Real;
            100.0 + x * 0.35 + (x * 0.19).sin() * 2.5 + (x * 0.07).cos() * 1.25
        })
        .collect();

    let mut state = DynamicIndicatorState::from_name("rsi", &[14.0], 32)?;
    let seeded = state.seed_columns(&[&closes])?;
    assert!(seeded <= closes.len());

    let next = state.update(&[106.0])?;
    assert!(next.is_some());
    assert!(state.latest_ref().is_some());

    Ok(())
}
