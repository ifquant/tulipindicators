use tulipindicators::{
    registry, Adx, Adxr, Atr, Di, Dm, Dx, DynamicIndicatorState, Ema, Indicator, IndicatorError,
    IndicatorState, IndicatorStateFactory, Macd, Natr, Ppo, Real, Rsi, Sma, Stoch, Wilders,
};

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

fn high_low_close_series() -> (Vec<Real>, Vec<Real>, Vec<Real>) {
    let high = (0..96)
        .map(|index| {
            let x = index as Real;
            120.0 + x * 0.18 + (x * 0.11).sin() * 2.6 + 1.4
        })
        .collect();
    let low = (0..96)
        .map(|index| {
            let x = index as Real;
            117.0 + x * 0.16 + (x * 0.07).cos() * 2.1 - 1.3
        })
        .collect();
    let close = (0..96)
        .map(|index| {
            let x = index as Real;
            118.5 + x * 0.17 + (x * 0.13).sin() * 1.8 + (x * 0.05).cos() * 0.9
        })
        .collect();
    (high, low, close)
}

#[test]
fn state_constructors_reject_zero_history_capacity() {
    let expected = IndicatorError::InvalidOption {
        indicator: "rsi",
        option: "history_capacity",
        value: 0.0,
        reason: "must be greater than zero",
    };

    let typed_error = match Rsi::state(&[14.0], 0) {
        Ok(_) => panic!("typed state should reject zero history"),
        Err(error) => error,
    };
    assert_eq!(typed_error, expected);

    let dynamic_error = match DynamicIndicatorState::from_name("rsi", &[14.0], 0) {
        Ok(_) => panic!("dynamic state should reject zero history"),
        Err(error) => error,
    };
    assert_eq!(dynamic_error, expected);

    let factory_error = match Rsi.dynamic_state(&[14.0], 0) {
        Ok(_) => panic!("factory dynamic state should reject zero history"),
        Err(error) => error,
    };
    assert_eq!(factory_error, expected);
}

#[test]
fn rsi_state_seed_and_indexed_history_match_batch_output() {
    let input = close_series();
    let options = [14.0];
    let expected = Rsi.run_single(&[&input], &options).expect("rsi batch");

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
    let expected = Rsi.run_single(&[&input], &options).expect("rsi batch");

    let mut state = DynamicIndicatorState::from_name("rsi", &options, expected.len())
        .expect("dynamic rsi state");
    let produced = state.seed_columns(&[&input]).expect("dynamic rsi seed");
    assert_eq!(produced, expected.len());
    assert_eq!(state.len(), expected.len());
    assert_option_real_eq(
        state.latest().map(|values| values[0]),
        expected.last().copied(),
    );
    assert_option_real_eq(
        state.latest_ref().map(|values| values[0]),
        expected.last().copied(),
    );

    for index in 0..expected.len() {
        assert_option_real_eq(
            state.get(index).map(|values| values[0]),
            Some(expected[expected.len() - 1 - index]),
        );
        assert_option_real_eq(
            state.get_ref(index).map(|values| values[0]),
            Some(expected[expected.len() - 1 - index]),
        );
    }

    let next_input = input.last().copied().expect("latest input") + 0.75;
    let mut extended = input.clone();
    extended.push(next_input);
    let expected_latest = Rsi
        .run_single(&[&extended], &options)
        .expect("extended rsi batch")
        .last()
        .copied();
    assert_option_real_eq(
        state
            .update(&[next_input])
            .expect("dynamic rsi update")
            .map(|values| values[0]),
        expected_latest,
    );
    assert_option_real_eq(state.latest().map(|values| values[0]), expected_latest);

    state.reset().expect("dynamic rsi reset");
    assert_eq!(state.len(), 0);
    assert!(!state.is_ready());
}

#[test]
fn dynamic_state_reseeding_stream_backend_starts_from_clean_state() {
    let first_input = close_series();
    let second_input: Vec<Real> = first_input
        .iter()
        .enumerate()
        .map(|(index, value)| value + 10.0 + index as Real * 0.03)
        .collect();
    let options = [14.0];
    let expected = Rsi
        .run_single(&[&second_input], &options)
        .expect("second rsi batch");

    let mut state = DynamicIndicatorState::from_name("rsi", &options, expected.len())
        .expect("dynamic rsi state");
    state
        .seed_columns(&[&first_input])
        .expect("first dynamic rsi seed");
    let produced = state
        .seed_columns(&[&second_input])
        .expect("second dynamic rsi seed should reset stream backend");

    assert_eq!(produced, expected.len());
    assert_eq!(state.len(), expected.len());
    assert_option_real_eq(
        state.latest().map(|values| values[0]),
        expected.last().copied(),
    );

    let second_rows = second_input
        .iter()
        .map(|value| vec![*value])
        .collect::<Vec<_>>();
    state
        .seed_rows(&second_rows)
        .expect("row dynamic rsi seed should reset stream backend");
    assert_eq!(state.len(), expected.len());
    assert_option_real_eq(
        state.latest().map(|values| values[0]),
        expected.last().copied(),
    );
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
    assert_option_pair_eq(
        state.latest_ref().map(|values| (values[0], values[1])),
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
        assert_option_pair_eq(
            state.get_ref(index).map(|values| (values[0], values[1])),
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

    state.reset().expect("dynamic dm reset");
    assert_eq!(state.len(), 0);
    assert!(!state.is_ready());
}

#[test]
fn dynamic_state_falls_back_to_batch_for_ma() {
    let input = close_series();
    let options = [5.0, 0.0];
    let ma = registry::find("ma").expect("ma indicator");
    let expected = ma.run_single(&[&input], &options).expect("ma batch");

    let mut state =
        DynamicIndicatorState::from_name("ma", &options, expected.len()).expect("dynamic ma state");
    let produced = state.seed_columns(&[&input]).expect("dynamic ma seed");
    assert_eq!(produced, expected.len());
    assert_eq!(state.len(), expected.len());
    assert_option_real_eq(
        state.latest().map(|values| values[0]),
        expected.last().copied(),
    );
    assert_option_real_eq(
        state.latest_ref().map(|values| values[0]),
        expected.last().copied(),
    );

    for index in 0..expected.len() {
        assert_option_real_eq(
            state.get(index).map(|values| values[0]),
            Some(expected[expected.len() - 1 - index]),
        );
        assert_option_real_eq(
            state.get_ref(index).map(|values| values[0]),
            Some(expected[expected.len() - 1 - index]),
        );
    }

    let next_input = input.last().copied().expect("latest input") + 0.6;
    let mut extended = input.clone();
    extended.push(next_input);
    let expected_latest = ma
        .run_single(&[&extended], &options)
        .expect("extended ma batch")
        .last()
        .copied();
    assert_option_real_eq(
        state
            .update(&[next_input])
            .expect("dynamic ma update")
            .map(|values| values[0]),
        expected_latest,
    );
    assert_option_real_eq(state.latest().map(|values| values[0]), expected_latest);

    state.reset().expect("dynamic ma reset");
    assert_eq!(state.len(), 0);
    assert!(!state.is_ready());
}

#[test]
fn indicator_state_factory_creates_dynamic_states() {
    let input = close_series();
    let options = [14.0];
    let batch = Rsi.run_single(&[&input], &options).expect("rsi batch");
    let expected = batch.last().copied();

    let mut state = Rsi
        .dynamic_state(&options, batch.len())
        .expect("factory dynamic state");
    let produced = state.seed_columns(&[&input]).expect("factory seed");
    assert_eq!(produced, batch.len());
    assert_option_real_eq(state.latest().map(|values| values[0]), expected);
}

#[test]
fn ema_state_seed_and_indexed_history_match_batch_output() {
    let input = close_series();
    let options = [12.0];
    let expected = Ema.run_single(&[&input], &options).expect("ema batch");

    let mut state = Ema::state(&options, expected.len()).expect("ema state");
    let produced = state.seed(&input).expect("ema seed");
    assert_eq!(produced, expected.len());
    assert_eq!(state.len(), expected.len());
    assert_option_real_eq(state.latest(), expected.last().copied());

    for index in 0..expected.len() {
        assert_option_real_eq(state.get(index), Some(expected[expected.len() - 1 - index]));
    }
}

#[test]
fn sma_state_seed_and_indexed_history_match_batch_output() {
    let input = close_series();
    let options = [10.0];
    let expected = Sma.run_single(&[&input], &options).expect("sma batch");

    let mut state = Sma::state(&options, expected.len()).expect("sma state");
    let produced = state.seed(&input).expect("sma seed");
    assert_eq!(produced, expected.len());
    assert_eq!(state.len(), expected.len());
    assert_option_real_eq(state.latest(), expected.last().copied());

    for index in 0..expected.len() {
        assert_option_real_eq(state.get(index), Some(expected[expected.len() - 1 - index]));
    }
}

#[test]
fn atr_state_seed_and_indexed_history_match_batch_output() {
    let (high, low, close) = high_low_close_series();
    let options = [14.0];
    let expected = Atr
        .run_single(&[&high, &low, &close], &options)
        .expect("atr batch");

    let inputs: Vec<(Real, Real, Real)> = high
        .iter()
        .copied()
        .zip(low.iter().copied())
        .zip(close.iter().copied())
        .map(|((high, low), close)| (high, low, close))
        .collect();
    let mut state = Atr::state(&options, expected.len()).expect("atr state");
    let produced = state.seed(&inputs).expect("atr seed");
    assert_eq!(produced, expected.len());
    assert_eq!(state.len(), expected.len());
    assert_option_real_eq(state.latest(), expected.last().copied());

    for index in 0..expected.len() {
        assert_option_real_eq(state.get(index), Some(expected[expected.len() - 1 - index]));
    }
}

#[test]
fn macd_state_seed_and_indexed_history_match_batch_output() {
    let input = close_series();
    let options = [12.0, 26.0, 9.0];
    let batch = Macd.run(&[&input], &options).expect("macd batch");
    let expected_macd = &batch[0];
    let expected_signal = &batch[1];
    let expected_hist = &batch[2];

    let mut state = Macd::state(&options, expected_macd.len()).expect("macd state");
    let produced = state.seed(&input).expect("macd seed");
    assert_eq!(produced, expected_macd.len());
    assert_eq!(state.len(), expected_macd.len());
    assert_option_pair_eq(
        state.latest().map(|(macd, signal, _)| (macd, signal)),
        Some((
            *expected_macd.last().expect("macd latest"),
            *expected_signal.last().expect("signal latest"),
        )),
    );
    assert_option_real_eq(
        state.latest().map(|(_, _, hist)| hist),
        expected_hist.last().copied(),
    );

    for index in 0..expected_macd.len() {
        let actual = state.get(index);
        let expected = Some((
            expected_macd[expected_macd.len() - 1 - index],
            expected_signal[expected_signal.len() - 1 - index],
            expected_hist[expected_hist.len() - 1 - index],
        ));
        match (actual, expected) {
            (Some((am, asg, ah)), Some((em, es, eh))) => {
                assert_real_eq(am, em);
                assert_real_eq(asg, es);
                assert_real_eq(ah, eh);
            }
            (left, right) => panic!("expected {right:?}, got {left:?}"),
        }
    }
}

#[test]
fn wilders_state_seed_and_indexed_history_match_batch_output() {
    let input = close_series();
    let options = [14.0];
    let expected = Wilders
        .run_single(&[&input], &options)
        .expect("wilders batch");

    let mut state = Wilders::state(&options, expected.len()).expect("wilders state");
    let produced = state.seed(&input).expect("wilders seed");
    assert_eq!(produced, expected.len());
    assert_eq!(state.len(), expected.len());
    assert_option_real_eq(state.latest(), expected.last().copied());

    for index in 0..expected.len() {
        assert_option_real_eq(state.get(index), Some(expected[expected.len() - 1 - index]));
    }
}

#[test]
fn natr_state_seed_and_indexed_history_match_batch_output() {
    let (high, low, close) = high_low_close_series();
    let options = [14.0];
    let expected = Natr
        .run_single(&[&high, &low, &close], &options)
        .expect("natr batch");

    let inputs: Vec<(Real, Real, Real)> = high
        .iter()
        .copied()
        .zip(low.iter().copied())
        .zip(close.iter().copied())
        .map(|((high, low), close)| (high, low, close))
        .collect();
    let mut state = Natr::state(&options, expected.len()).expect("natr state");
    let produced = state.seed(&inputs).expect("natr seed");
    assert_eq!(produced, expected.len());
    assert_eq!(state.len(), expected.len());
    assert_option_real_eq(state.latest(), expected.last().copied());

    for index in 0..expected.len() {
        assert_option_real_eq(state.get(index), Some(expected[expected.len() - 1 - index]));
    }
}

#[test]
fn ppo_state_seed_and_indexed_history_match_batch_output() {
    let input = close_series();
    let options = [12.0, 26.0];
    let expected = Ppo.run_single(&[&input], &options).expect("ppo batch");

    let mut state = Ppo::state(&options, expected.len()).expect("ppo state");
    let produced = state.seed(&input).expect("ppo seed");
    assert_eq!(produced, expected.len());
    assert_eq!(state.len(), expected.len());
    assert_option_real_eq(state.latest(), expected.last().copied());

    for index in 0..expected.len() {
        assert_option_real_eq(state.get(index), Some(expected[expected.len() - 1 - index]));
    }
}

#[test]
fn stoch_state_seed_and_indexed_history_match_batch_output() {
    let (high, low, close) = high_low_close_series();
    let options = [14.0, 3.0, 3.0];
    let batch = Stoch
        .run(&[&high, &low, &close], &options)
        .expect("stoch batch");
    let expected_k = &batch[0];
    let expected_d = &batch[1];

    let inputs: Vec<(Real, Real, Real)> = high
        .iter()
        .copied()
        .zip(low.iter().copied())
        .zip(close.iter().copied())
        .map(|((high, low), close)| (high, low, close))
        .collect();
    let mut state = Stoch::state(&options, expected_k.len()).expect("stoch state");
    let produced = state.seed(&inputs).expect("stoch seed");
    assert_eq!(produced, expected_k.len());
    assert_eq!(state.len(), expected_k.len());
    assert_option_pair_eq(
        state.latest(),
        Some((
            *expected_k.last().expect("stoch k latest"),
            *expected_d.last().expect("stoch d latest"),
        )),
    );

    for index in 0..expected_k.len() {
        assert_option_pair_eq(
            state.get(index),
            Some((
                expected_k[expected_k.len() - 1 - index],
                expected_d[expected_d.len() - 1 - index],
            )),
        );
    }
}

#[test]
fn dx_state_seed_and_indexed_history_match_batch_output() {
    let (high, low) = high_low_series();
    let options = [14.0];
    let expected = Dx.run_single(&[&high, &low], &options).expect("dx batch");

    let inputs: Vec<(Real, Real)> = high.iter().copied().zip(low.iter().copied()).collect();
    let mut state = Dx::state(&options, expected.len()).expect("dx state");
    let produced = state.seed(&inputs).expect("dx seed");
    assert_eq!(produced, expected.len());
    assert_eq!(state.len(), expected.len());
    assert_option_real_eq(state.latest(), expected.last().copied());

    for index in 0..expected.len() {
        assert_option_real_eq(state.get(index), Some(expected[expected.len() - 1 - index]));
    }
}

#[test]
fn di_state_seed_and_indexed_history_match_batch_output() {
    let (high, low, close) = high_low_close_series();
    let options = [14.0];
    let batch = Di.run(&[&high, &low, &close], &options).expect("di batch");
    let expected_plus = &batch[0];
    let expected_minus = &batch[1];

    let inputs: Vec<(Real, Real, Real)> = high
        .iter()
        .copied()
        .zip(low.iter().copied())
        .zip(close.iter().copied())
        .map(|((high, low), close)| (high, low, close))
        .collect();
    let mut state = Di::state(&options, expected_plus.len()).expect("di state");
    let produced = state.seed(&inputs).expect("di seed");
    assert_eq!(produced, expected_plus.len());
    assert_eq!(state.len(), expected_plus.len());
    assert_option_pair_eq(
        state.latest(),
        Some((
            *expected_plus.last().expect("plus di latest"),
            *expected_minus.last().expect("minus di latest"),
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
}

#[test]
fn adx_state_seed_and_indexed_history_match_batch_output() {
    let (high, low) = high_low_series();
    let options = [14.0];
    let expected = Adx.run_single(&[&high, &low], &options).expect("adx batch");

    let inputs: Vec<(Real, Real)> = high.iter().copied().zip(low.iter().copied()).collect();
    let mut state = Adx::state(&options, expected.len()).expect("adx state");
    let produced = state.seed(&inputs).expect("adx seed");
    assert_eq!(produced, expected.len());
    assert_eq!(state.len(), expected.len());
    assert_option_real_eq(state.latest(), expected.last().copied());

    for index in 0..expected.len() {
        assert_option_real_eq(state.get(index), Some(expected[expected.len() - 1 - index]));
    }
}

#[test]
fn adxr_state_seed_and_indexed_history_match_batch_output() {
    let (high, low) = high_low_series();
    let options = [14.0];
    let expected = Adxr
        .run_single(&[&high, &low], &options)
        .expect("adxr batch");

    let inputs: Vec<(Real, Real)> = high.iter().copied().zip(low.iter().copied()).collect();
    let mut state = Adxr::state(&options, expected.len()).expect("adxr state");
    let produced = state.seed(&inputs).expect("adxr seed");
    assert_eq!(produced, expected.len());
    assert_eq!(state.len(), expected.len());
    assert_option_real_eq(state.latest(), expected.last().copied());

    for index in 0..expected.len() {
        assert_option_real_eq(state.get(index), Some(expected[expected.len() - 1 - index]));
    }
}
