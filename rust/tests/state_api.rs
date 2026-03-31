use tulipindicators::{registry, Dm, DynamicIndicatorState, Indicator, IndicatorState, Real, Rsi};

const EPSILON: Real = 1e-12;

fn assert_real_eq(left: Real, right: Real) {
    let diff = (left - right).abs();
    assert!(
        diff <= EPSILON,
        "expected {right}, got {left}, diff {diff} exceeds epsilon {EPSILON}"
    );
}

fn assert_option_real_eq(left: Option<Real>, right: Option<Real>) {
    match (left, right) {
        (Some(left), Some(right)) => assert_real_eq(left, right),
        (None, None) => {}
        (left, right) => panic!("expected {right:?}, got {left:?}"),
    }
}

fn assert_option_pair_eq(left: Option<(Real, Real)>, right: Option<(Real, Real)>) {
    match (left, right) {
        (Some((left0, left1)), Some((right0, right1))) => {
            assert_real_eq(left0, right0);
            assert_real_eq(left1, right1);
        }
        (None, None) => {}
        (left, right) => panic!("expected {right:?}, got {left:?}"),
    }
}

fn close_series() -> Vec<Real> {
    (0..96)
        .map(|index| {
            let x = index as Real;
            100.0 + x * 0.35 + (x * 0.19).sin() * 2.5 + (x * 0.07).cos() * 1.25
        })
        .collect()
}

fn high_low_series() -> (Vec<Real>, Vec<Real>) {
    let high = (0..96)
        .map(|index| {
            let x = index as Real;
            120.0 + x * 0.21 + (x * 0.11).sin() * 3.0 + 1.8
        })
        .collect();
    let low = (0..96)
        .map(|index| {
            let x = index as Real;
            118.0 + x * 0.19 + (x * 0.09).cos() * 2.0 - 1.6
        })
        .collect();
    (high, low)
}

#[test]
fn rsi_state_seed_and_indexed_history_match_batch_output() {
    let input = close_series();
    let options = [14.0];
    let batch = Rsi.run(&[&input], &options).expect("rsi batch");
    let expected = &batch[0];

    let mut state = Rsi::state(&options, expected.len()).expect("rsi state");
    let produced = state.seed(&input).expect("rsi seed");
    assert_eq!(produced, expected.len());
    assert_eq!(state.len(), expected.len());
    assert_option_real_eq(state.latest(), expected.last().copied());

    for index in 0..expected.len() {
        assert_option_real_eq(state.get(index), Some(expected[expected.len() - 1 - index]));
    }

    state.reset();
    assert_eq!(state.len(), 0);
    assert!(!state.is_ready());
}

#[test]
fn dm_state_seed_and_indexed_history_match_batch_output() {
    let (high, low) = high_low_series();
    let options = [14.0];
    let batch = Dm.run(&[&high, &low], &options).expect("dm batch");
    let expected_plus = &batch[0];
    let expected_minus = &batch[1];

    let inputs: Vec<(Real, Real)> = high.iter().copied().zip(low.iter().copied()).collect();
    let mut state = Dm::state(&options, expected_plus.len()).expect("dm state");
    let produced = state.seed(&inputs).expect("dm seed");
    assert_eq!(produced, expected_plus.len());
    assert_eq!(state.len(), expected_plus.len());
    assert_option_pair_eq(
        state.latest(),
        Some((
            *expected_plus.last().expect("plus latest"),
            *expected_minus.last().expect("minus latest"),
        )),
    );

    for index in 0..expected_plus.len() {
        assert_option_pair_eq(
            state.get(index),
            Some((
                expected_plus[expected_plus.len() - 1 - index],
                expected_minus[expected_minus.len() - 1 - index],
            )),
        );
    }

    state.reset();
    assert_eq!(state.len(), 0);
    assert!(!state.is_ready());
}

#[test]
fn dynamic_state_for_rsi_matches_batch_and_supports_updates() {
    let input = close_series();
    let options = [14.0];
    let batch = Rsi.run(&[&input], &options).expect("rsi batch");
    let expected = &batch[0];

    let mut state = DynamicIndicatorState::from_name("rsi", &options, expected.len())
        .expect("dynamic rsi state");
    let produced = state.seed_columns(&[&input]).expect("dynamic rsi seed");
    assert_eq!(produced, expected.len());
    assert_eq!(state.len(), expected.len());
    assert_option_real_eq(
        state.latest().map(|values| values[0]),
        expected.last().copied(),
    );

    for index in 0..expected.len() {
        assert_option_real_eq(
            state.get(index).map(|values| values[0]),
            Some(expected[expected.len() - 1 - index]),
        );
    }

    let next_input = input.last().copied().expect("latest input") + 0.75;
    let mut extended = input.clone();
    extended.push(next_input);
    let extended_batch = Rsi.run(&[&extended], &options).expect("extended rsi batch");
    let expected_latest = extended_batch[0].last().copied();
    assert_option_real_eq(
        state
            .update(&[next_input])
            .expect("dynamic rsi update")
            .map(|values| values[0]),
        expected_latest,
    );
    assert_option_real_eq(state.latest().map(|values| values[0]), expected_latest);

    state.reset();
    assert_eq!(state.len(), 0);
    assert!(!state.is_ready());
}

#[test]
fn dynamic_state_for_dm_matches_batch_and_supports_updates() {
    let (high, low) = high_low_series();
    let options = [14.0];
    let batch = Dm.run(&[&high, &low], &options).expect("dm batch");
    let expected_plus = &batch[0];
    let expected_minus = &batch[1];

    let mut state =
        DynamicIndicatorState::from_name("dm", &options, expected_plus.len()).expect("dynamic dm");
    let produced = state.seed_columns(&[&high, &low]).expect("dynamic dm seed");
    assert_eq!(produced, expected_plus.len());
    assert_eq!(state.len(), expected_plus.len());
    assert_option_pair_eq(
        state.latest().map(|values| (values[0], values[1])),
        Some((
            *expected_plus.last().expect("plus latest"),
            *expected_minus.last().expect("minus latest"),
        )),
    );

    for index in 0..expected_plus.len() {
        assert_option_pair_eq(
            state.get(index).map(|values| (values[0], values[1])),
            Some((
                expected_plus[expected_plus.len() - 1 - index],
                expected_minus[expected_minus.len() - 1 - index],
            )),
        );
    }

    let next_high = high.last().copied().expect("last high") + 0.9;
    let next_low = low.last().copied().expect("last low") + 0.4;
    let mut extended_high = high.clone();
    let mut extended_low = low.clone();
    extended_high.push(next_high);
    extended_low.push(next_low);
    let extended_batch = Dm
        .run(&[&extended_high, &extended_low], &options)
        .expect("extended dm batch");
    let expected_latest = Some((
        *extended_batch[0].last().expect("plus latest after update"),
        *extended_batch[1].last().expect("minus latest after update"),
    ));
    assert_option_pair_eq(
        state
            .update(&[next_high, next_low])
            .expect("dynamic dm update")
            .map(|values| (values[0], values[1])),
        expected_latest,
    );
    assert_option_pair_eq(
        state.latest().map(|values| (values[0], values[1])),
        expected_latest,
    );

    state.reset();
    assert_eq!(state.len(), 0);
    assert!(!state.is_ready());
}

#[test]
fn dynamic_state_falls_back_to_batch_for_ma() {
    let input = close_series();
    let options = [5.0, 0.0];
    let ma = registry::find("ma").expect("ma indicator");
    let batch = ma.run(&[&input], &options).expect("ma batch");
    let expected = &batch[0];

    let mut state =
        DynamicIndicatorState::from_name("ma", &options, expected.len()).expect("dynamic ma state");
    let produced = state.seed_columns(&[&input]).expect("dynamic ma seed");
    assert_eq!(produced, expected.len());
    assert_eq!(state.len(), expected.len());
    assert_option_real_eq(
        state.latest().map(|values| values[0]),
        expected.last().copied(),
    );

    for index in 0..expected.len() {
        assert_option_real_eq(
            state.get(index).map(|values| values[0]),
            Some(expected[expected.len() - 1 - index]),
        );
    }

    let next_input = input.last().copied().expect("latest input") + 0.6;
    let mut extended = input.clone();
    extended.push(next_input);
    let extended_batch = ma.run(&[&extended], &options).expect("extended ma batch");
    let expected_latest = extended_batch[0].last().copied();
    assert_option_real_eq(
        state
            .update(&[next_input])
            .expect("dynamic ma update")
            .map(|values| values[0]),
        expected_latest,
    );
    assert_option_real_eq(state.latest().map(|values| values[0]), expected_latest);

    state.reset();
    assert_eq!(state.len(), 0);
    assert!(!state.is_ready());
}
