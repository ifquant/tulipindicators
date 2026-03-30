#[path = "support/golden_cases.rs"]
mod golden_cases;

use golden_cases::{parse_cases, TEST_FILES};
use tulipindicators::{all, find, Real};

const APPROX_TOLERANCE: Real = 1e-3;
const STREAM_TOLERANCE: Real = 1e-10;
const STREAM_STEPS: &[usize] = &[1, 2, 3, 4, 5, 7, 13, 100, 1024];

#[test]
fn batch_outputs_match_existing_golden_data() {
    for path in TEST_FILES {
        for case in parse_cases(path) {
            let indicator = find(&case.name).expect("indicator should be registered");
            let inputs: Vec<&[Real]> = case.inputs.iter().map(Vec::as_slice).collect();
            let outputs = indicator
                .run(&inputs, &case.options)
                .unwrap_or_else(|error| {
                    panic!("{} batch run failed for {}: {error}", path, case.name)
                });

            assert_eq!(
                outputs.len(),
                case.outputs.len(),
                "{} {} output count mismatch",
                path,
                case.name
            );

            for (expected, actual) in case.outputs.iter().zip(outputs.iter()) {
                assert_series_close(path, &case.name, expected, actual);
            }
        }
    }
}

#[test]
fn stream_outputs_match_batch_outputs_for_chunked_runs() {
    for indicator in all() {
        let Ok(Some(_)) = indicator.create_stream(&default_options(indicator.metadata().name))
        else {
            continue;
        };

        for path in TEST_FILES {
            for case in parse_cases(path)
                .into_iter()
                .filter(|case| case.name == indicator.metadata().name)
            {
                let inputs: Vec<&[Real]> = case.inputs.iter().map(Vec::as_slice).collect();
                let batch = indicator.run(&inputs, &case.options).unwrap();
                let expected_progress: usize = case.inputs.first().map_or(0, Vec::len);

                for step in STREAM_STEPS {
                    let mut stream = indicator
                        .create_stream(&case.options)
                        .expect("stream creation should succeed")
                        .expect("indicator should expose a stream");
                    let mut merged: Vec<Vec<Real>> = vec![Vec::new(); batch.len()];
                    let input_len = case.inputs.first().map_or(0, Vec::len);
                    let mut start = 0;

                    while start < input_len {
                        let end = (start + step).min(input_len);
                        let chunk_inputs: Vec<&[Real]> = case
                            .inputs
                            .iter()
                            .map(|series| &series[start..end])
                            .collect();
                        let chunk_outputs = stream.feed(&chunk_inputs).unwrap_or_else(|error| {
                            panic!(
                                "{} {} stream run failed at {}..{}: {error}",
                                path, case.name, start, end
                            )
                        });

                        for (index, chunk_output) in chunk_outputs.into_iter().enumerate() {
                            merged[index].extend(chunk_output);
                        }

                        start = end;
                    }

                    assert_eq!(
                        stream.progress(),
                        expected_progress,
                        "{} {} progress mismatch for step {}",
                        path,
                        case.name,
                        step
                    );

                    for (expected, actual) in batch.iter().zip(merged.iter()) {
                        assert_exact_close(path, &case.name, *step, expected, actual);
                    }
                }
            }
        }
    }
}

#[test]
fn mama_smoke_on_synthetic_series() {
    let indicator = find("mama").expect("mama should be registered");
    let options = vec![0.5, 0.05];
    let input: Vec<Real> = (0..256)
        .map(|index| 100.0 + index as Real * 0.02 + (index as Real / 11.0).sin() * 1.4)
        .collect();
    let inputs = vec![input.as_slice()];

    let batch = indicator
        .run(&inputs, &options)
        .expect("mama batch should run");
    assert_eq!(batch.len(), 2);
    assert_eq!(batch[0].len(), input.len().saturating_sub(6));
    assert_eq!(batch[1].len(), input.len().saturating_sub(6));
    assert!(batch.iter().flatten().all(|value| value.is_finite()));

    let mut stream = indicator
        .create_stream(&options)
        .expect("mama stream creation should succeed")
        .expect("mama should expose a stream");
    let stream_outputs = stream
        .feed(&inputs)
        .expect("mama stream should process synthetic input");
    assert_eq!(stream.progress(), input.len());
    assert_eq!(batch, stream_outputs);
}

fn default_options(indicator: &str) -> Vec<Real> {
    match indicator {
        "bbands" => vec![5.0, 2.0],
        "kst" => vec![10.0, 15.0, 20.0, 30.0, 10.0, 10.0, 10.0, 15.0],
        "macd" => vec![12.0, 26.0, 9.0],
        "macdext" => vec![12.0, 1.0, 26.0, 1.0, 9.0, 1.0],
        "mama" => vec![0.5, 0.05],
        "psar" => vec![0.02, 0.2],
        "sarext" => vec![0.0, 0.0, 0.02, 0.02, 0.2, 0.02, 0.02, 0.2],
        "ultosc" => vec![5.0, 7.0, 10.0],
        "vidya" => vec![2.0, 5.0, 0.2],
        _ => {
            let option_count = find(indicator)
                .expect("indicator should exist when choosing default stream options")
                .metadata()
                .option_names
                .len();
            match option_count {
                0 => vec![],
                1 => vec![5.0],
                2 => vec![5.0, 10.0],
                3 => vec![5.0, 10.0, 3.0],
                _ => panic!("missing default options for {indicator}"),
            }
        }
    }
}

fn assert_series_close(path: &str, indicator: &str, expected: &[Real], actual: &[Real]) {
    assert_eq!(
        expected.len(),
        actual.len(),
        "{} {} length mismatch: expected {}, got {}",
        path,
        indicator,
        expected.len(),
        actual.len()
    );

    for (index, (expected_value, actual_value)) in expected.iter().zip(actual.iter()).enumerate() {
        let delta = (expected_value - actual_value).abs();
        assert!(
            delta <= APPROX_TOLERANCE,
            "{} {} value mismatch at index {}: expected {}, got {}, delta {}",
            path,
            indicator,
            index,
            expected_value,
            actual_value,
            delta
        );
    }
}

fn assert_exact_close(
    path: &str,
    indicator: &str,
    step: usize,
    expected: &[Real],
    actual: &[Real],
) {
    assert_eq!(
        expected.len(),
        actual.len(),
        "{} {} stream length mismatch for step {}: expected {}, got {}",
        path,
        indicator,
        step,
        expected.len(),
        actual.len()
    );

    for (index, (expected_value, actual_value)) in expected.iter().zip(actual.iter()).enumerate() {
        let delta = (expected_value - actual_value).abs();
        assert!(
            delta <= STREAM_TOLERANCE,
            "{} {} stream mismatch for step {} at index {}: expected {}, got {}, delta {}",
            path,
            indicator,
            step,
            index,
            expected_value,
            actual_value,
            delta
        );
    }
}
