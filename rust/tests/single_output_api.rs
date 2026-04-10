use tulipindicators::{Indicator, Macd, Real, Rsi};

#[test]
fn run_single_matches_first_output_for_single_output_indicators() {
    let input: Vec<Real> = (0..96)
        .map(|index| {
            let x = index as Real;
            100.0 + x * 0.35 + (x * 0.19).sin() * 2.5 + (x * 0.07).cos() * 1.25
        })
        .collect();

    let batch = Rsi.run(&[&input], &[14.0]).expect("rsi batch");
    let single = Rsi.run_single(&[&input], &[14.0]).expect("rsi single");

    assert_eq!(single, batch[0]);
}

#[test]
fn run_single_rejects_multi_output_indicators() {
    let input: Vec<Real> = (0..96).map(|index| 100.0 + index as Real * 0.25).collect();
    let err = Macd
        .run_single(&[&input], &[12.0, 26.0, 9.0])
        .expect_err("macd should not support single-output batch helper");

    assert_eq!(
        err,
        tulipindicators::IndicatorError::WrongOutputCount {
            indicator: "macd",
            expected: 1,
            actual: 3,
        }
    );
}

#[test]
fn feed_single_matches_first_output_for_single_output_streams() {
    let input: Vec<Real> = (0..64)
        .map(|index| {
            let x = index as Real;
            95.0 + x * 0.28 + (x * 0.13).sin() * 1.7
        })
        .collect();

    let mut stream = Rsi
        .create_stream(&[14.0])
        .expect("rsi stream")
        .expect("stream support");
    let batch = stream.feed(&[&input]).expect("stream batch feed");

    let mut single_stream = Rsi
        .create_stream(&[14.0])
        .expect("rsi single stream")
        .expect("stream support");
    let single = single_stream
        .feed_single(&[&input])
        .expect("stream single feed");

    assert_eq!(single, batch[0]);
}

#[test]
fn feed_single_rejects_multi_output_streams() {
    let input: Vec<Real> = (0..64).map(|index| 100.0 + index as Real * 0.2).collect();
    let mut stream = Macd
        .create_stream(&[12.0, 26.0, 9.0])
        .expect("macd stream")
        .expect("stream support");
    let err = stream
        .feed_single(&[&input])
        .expect_err("macd stream should not support single-output helper");

    assert_eq!(
        err,
        tulipindicators::IndicatorError::WrongOutputCount {
            indicator: "macd",
            expected: 1,
            actual: 3,
        }
    );
}
