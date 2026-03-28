use std::fs;
use std::path::Path;

use tulipindicators::{all, find, Real};

const TEST_FILES: &[&str] = &["tests/atoz.txt", "tests/untest.txt", "tests/extra.txt"];
const APPROX_TOLERANCE: Real = 1e-3;
const STREAM_TOLERANCE: Real = 1e-12;
const STREAM_STEPS: &[usize] = &[1, 2, 3, 5, 7, 64];

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

fn default_options(indicator: &str) -> Vec<Real> {
    match indicator {
        "bbands" => vec![5.0, 2.0],
        "macd" => vec![12.0, 26.0, 9.0],
        _ => vec![5.0],
    }
}

#[derive(Debug, Clone)]
struct GoldenCase {
    name: String,
    options: Vec<Real>,
    inputs: Vec<Vec<Real>>,
    outputs: Vec<Vec<Real>>,
}

fn parse_cases(path: &str) -> Vec<GoldenCase> {
    let content = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path));
    let lines: Vec<String> = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToOwned::to_owned)
        .collect();

    let mut cases = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let header = &lines[index];
        if header.starts_with('{') {
            panic!(
                "unexpected data line without header in {}: {}",
                path, header
            );
        }

        index += 1;

        let mut tokens = header.split_whitespace();
        let name = tokens.next().expect("header should have a name");
        let options: Vec<Real> = tokens.map(parse_number).collect();

        if let Some(indicator) = find(name) {
            let metadata = indicator.metadata();
            let mut inputs = Vec::with_capacity(metadata.input_names.len());
            let mut outputs = Vec::with_capacity(metadata.output_names.len());

            for _ in metadata.input_names {
                inputs.push(parse_array(&lines[index], path, name));
                index += 1;
            }

            for _ in metadata.output_names {
                outputs.push(parse_array(&lines[index], path, name));
                index += 1;
            }

            cases.push(GoldenCase {
                name: name.to_string(),
                options,
                inputs,
                outputs,
            });
        } else {
            while index < lines.len() && lines[index].starts_with('{') {
                index += 1;
            }
        }
    }

    cases
}

fn parse_array(line: &str, path: &str, indicator: &str) -> Vec<Real> {
    let trimmed = line.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        panic!(
            "{} {} expected array line, got {}",
            path, indicator, trimmed
        );
    }

    let body = &trimmed[1..trimmed.len() - 1];
    if body.trim().is_empty() {
        return Vec::new();
    }

    body.split(',').map(parse_number).collect()
}

fn parse_number(raw: &str) -> Real {
    raw.trim()
        .parse()
        .unwrap_or_else(|error| panic!("failed to parse number `{raw}`: {error}"))
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
