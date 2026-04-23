use tulipindicators::{Indicator, Macd, Real};

fn main() -> Result<(), tulipindicators::IndicatorError> {
    let closes: Vec<Real> = (0..96)
        .map(|index| {
            let x = index as Real;
            100.0 + x * 0.25 + (x * 0.13).sin() * 1.8 + (x * 0.05).cos() * 0.75
        })
        .collect();

    let options = [12.0, 26.0, 9.0];
    let output_len = closes.len().saturating_sub(25);

    let mut macd = vec![0.0; output_len];
    let mut signal = vec![0.0; output_len];
    let mut histogram = vec![0.0; output_len];
    let mut outputs = [&mut macd[..], &mut signal[..], &mut histogram[..]];

    let produced = Macd.run_in_place(&[&closes], &options, &mut outputs)?;
    assert_eq!(produced, output_len);

    Ok(())
}
