use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use tulipindicators::Real;

const TOLERANCE: Real = 1e-6;

#[test]
fn first_missing_batch_matches_c_and_expected_values() {
    assert_case(
        "midpoint",
        &[3.0],
        &[vec![1.0, 3.0, 2.0, 5.0, 4.0]],
        &[vec![2.0, 3.5, 3.5]],
    );
    assert_case(
        "midprice",
        &[2.0],
        &[vec![5.0, 6.0, 7.0, 8.0], vec![1.0, 2.0, 3.0, 4.0]],
        &[vec![3.5, 4.5, 5.5]],
    );
    assert_case(
        "rocr100",
        &[1.0],
        &[vec![1.0, 2.0, 4.0, 8.0]],
        &[vec![200.0, 200.0, 200.0]],
    );
    assert_case(
        "linearregangle",
        &[4.0],
        &[vec![1.0, 2.0, 3.0, 4.0]],
        &[vec![45.0]],
    );
    assert_case(
        "maxindex",
        &[3.0],
        &[vec![1.0, 6.0, 4.0, 9.0, 2.0, 3.0, 4.0]],
        &[vec![1.0, 3.0, 3.0, 3.0, 6.0]],
    );
    assert_case(
        "minindex",
        &[3.0],
        &[vec![1.0, 5.0, 7.0, 9.0, 2.0, 3.0, 4.0]],
        &[vec![0.0, 1.0, 4.0, 4.0, 4.0]],
    );
    assert_case(
        "minmax",
        &[3.0],
        &[vec![1.0, 5.0, 7.0, 9.0, 2.0, 3.0, 4.0]],
        &[vec![1.0, 5.0, 2.0, 2.0, 2.0], vec![7.0, 9.0, 9.0, 9.0, 4.0]],
    );
    assert_case(
        "minmaxindex",
        &[3.0],
        &[vec![1.0, 5.0, 7.0, 9.0, 2.0, 3.0, 4.0]],
        &[vec![0.0, 1.0, 4.0, 4.0, 4.0], vec![2.0, 3.0, 3.0, 3.0, 6.0]],
    );
    assert_case(
        "beta",
        &[3.0],
        &[
            vec![100.0, 110.0, 132.0, 171.6, 240.24],
            vec![100.0, 120.0, 168.0, 268.8, 483.84],
        ],
        &[vec![2.0, 2.0]],
    );
    assert_case(
        "correl",
        &[3.0],
        &[
            vec![1.0, 2.0, 3.0, 4.0, 5.0],
            vec![2.0, 4.0, 6.0, 8.0, 10.0],
        ],
        &[vec![1.0, 1.0, 1.0]],
    );
    assert_indicator_matches_other(
        "macdfix",
        &[9.0],
        "macd",
        &[12.0, 26.0, 9.0],
        &[vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
            17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0, 28.0, 29.0, 30.0,
            31.0, 32.0, 33.0, 34.0, 35.0, 36.0, 37.0, 38.0, 39.0, 40.0,
        ]],
    );
    assert_indicator_matches_other(
        "ma",
        &[5.0, 0.0],
        "sma",
        &[5.0],
        &[vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]],
    );
    assert_indicator_matches_other(
        "ma",
        &[5.0, 1.0],
        "ema",
        &[5.0],
        &[vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0]],
    );
    assert_indicator_matches_output(
        "ma",
        &[6.0, 7.0],
        "mama",
        &[0.5, 0.05],
        0,
        &[vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
            17.0, 18.0, 19.0, 20.0,
        ]],
    );
    assert_case(
        "mavp",
        &[4.0, 4.0, 0.0],
        &[
            vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
            vec![4.0, 4.0, 4.0, 4.0, 4.0, 4.0],
        ],
        &[vec![2.5, 3.5, 4.5]],
    );
    assert_case(
        "t3",
        &[2.0, 0.7],
        &[vec![
            10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0,
        ]],
        &[vec![10.0, 10.0, 10.0, 10.0]],
    );
    assert_indicator_matches_other(
        "macdext",
        &[12.0, 1.0, 26.0, 1.0, 9.0, 1.0],
        "macd",
        &[12.0, 26.0, 9.0],
        &[vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0,
            17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0, 24.0, 25.0, 26.0, 27.0, 28.0, 29.0, 30.0,
            31.0, 32.0, 33.0, 34.0, 35.0, 36.0, 37.0, 38.0, 39.0, 40.0,
        ]],
    );
}

#[test]
fn ht_family_matches_local_talib_default_semantics() {
    let input = vec![
        101.0, 102.5, 103.0, 102.0, 101.5, 102.8, 104.2, 105.7, 104.9, 103.8, 102.6, 101.9, 102.3,
        103.7, 105.1, 106.4, 107.0, 106.2, 105.3, 104.1, 103.4, 104.7, 106.0, 107.8, 108.5, 107.2,
        105.9, 104.8, 103.6, 102.9, 103.8, 105.4, 106.8, 107.1, 106.0, 104.4, 103.1, 102.5, 103.0,
        104.9, 106.7, 108.2, 109.1, 108.3, 107.0, 105.6, 104.3, 103.5, 104.1, 105.8, 107.4, 108.9,
        109.7, 108.8, 107.6, 106.1, 104.7, 103.2, 102.8, 103.9, 105.5, 106.9, 108.4, 109.2, 108.1,
        106.6, 105.2, 104.0, 103.3, 104.4, 106.2, 107.7, 109.0, 109.8, 108.9, 107.5, 106.0, 104.6,
        103.8, 104.7, 106.5, 108.1, 109.4, 110.0, 108.7, 107.1, 105.5, 104.2, 103.7, 104.8, 106.4,
        108.0, 109.5, 110.3, 109.1, 107.8,
    ];
    assert_matches_talib("ht_dcperiod", &[], std::slice::from_ref(&input));
    assert_matches_talib("ht_dcphase", &[], std::slice::from_ref(&input));
    assert_matches_talib("ht_phasor", &[], std::slice::from_ref(&input));
    assert_matches_talib("ht_sine", &[], std::slice::from_ref(&input));
    assert_matches_talib("ht_trendline", &[], std::slice::from_ref(&input));
    assert_matches_talib("ht_trendmode", &[], std::slice::from_ref(&input));
}

fn assert_case(name: &str, options: &[Real], inputs: &[Vec<Real>], expected: &[Vec<Real>]) {
    let indicator = tulipindicators::find(name).expect("indicator should be registered");
    let rust_inputs: Vec<&[Real]> = inputs.iter().map(Vec::as_slice).collect();
    let rust_outputs = indicator
        .run(&rust_inputs, options)
        .expect("rust run should succeed");
    let c_outputs = run_c_oracle(&ensure_stable_oracle(), name, options, inputs);

    assert_eq!(
        rust_outputs.len(),
        expected.len(),
        "{name} output count drifted"
    );
    assert_eq!(
        c_outputs.len(),
        expected.len(),
        "{name} c output count drifted"
    );

    for output_index in 0..expected.len() {
        let rust_output = &rust_outputs[output_index];
        let c_output = &c_outputs[output_index];
        let expected_output = &expected[output_index];

        assert_eq!(
            rust_output.len(),
            expected_output.len(),
            "{name} rust length drifted"
        );
        assert_eq!(
            c_output.len(),
            expected_output.len(),
            "{name} c length drifted"
        );

        for position in 0..expected_output.len() {
            let expected_value = expected_output[position];
            let rust_value = rust_output[position];
            let c_value = c_output[position];
            assert!(
                (rust_value - expected_value).abs() <= TOLERANCE,
                "{name} rust mismatch at {position}: got {rust_value} expected {expected_value}"
            );
            assert!(
                (c_value - expected_value).abs() <= TOLERANCE,
                "{name} c mismatch at {position}: got {c_value} expected {expected_value}"
            );
        }
    }
}

fn assert_indicator_matches_other(
    left_name: &str,
    left_options: &[Real],
    right_name: &str,
    right_options: &[Real],
    inputs: &[Vec<Real>],
) {
    let left = tulipindicators::find(left_name).expect("left indicator should be registered");
    let right = tulipindicators::find(right_name).expect("right indicator should be registered");
    let rust_inputs: Vec<&[Real]> = inputs.iter().map(Vec::as_slice).collect();

    let left_rust = left
        .run(&rust_inputs, left_options)
        .expect("left rust run should succeed");
    let right_rust = right
        .run(&rust_inputs, right_options)
        .expect("right rust run should succeed");
    let left_c = run_c_oracle(&ensure_stable_oracle(), left_name, left_options, inputs);
    let right_c = run_c_oracle(&ensure_stable_oracle(), right_name, right_options, inputs);

    assert_same_outputs(left_name, "rust", &left_rust, &right_rust);
    assert_same_outputs(left_name, "c", &left_c, &right_c);
}

fn assert_indicator_matches_output(
    left_name: &str,
    left_options: &[Real],
    right_name: &str,
    right_options: &[Real],
    right_output_index: usize,
    inputs: &[Vec<Real>],
) {
    let left = tulipindicators::find(left_name).expect("left indicator should be registered");
    let right = tulipindicators::find(right_name).expect("right indicator should be registered");
    let rust_inputs: Vec<&[Real]> = inputs.iter().map(Vec::as_slice).collect();

    let left_rust = left
        .run(&rust_inputs, left_options)
        .expect("left rust run should succeed");
    let right_rust = right
        .run(&rust_inputs, right_options)
        .expect("right rust run should succeed");
    let left_c = run_c_oracle(&ensure_stable_oracle(), left_name, left_options, inputs);
    let right_c = run_c_oracle(&ensure_stable_oracle(), right_name, right_options, inputs);

    assert_eq!(
        left_rust.len(),
        1,
        "{left_name} should have a single output"
    );
    assert_eq!(left_c.len(), 1, "{left_name} c should have a single output");
    assert_same_series(
        left_name,
        "rust",
        &left_rust[0],
        &right_rust[right_output_index],
    );
    assert_same_series(left_name, "c", &left_c[0], &right_c[right_output_index]);
}

fn assert_same_outputs(name: &str, side: &str, left: &[Vec<Real>], right: &[Vec<Real>]) {
    assert_eq!(
        left.len(),
        right.len(),
        "{name} {side} output count drifted"
    );
    for output_index in 0..left.len() {
        assert_eq!(
            left[output_index].len(),
            right[output_index].len(),
            "{name} {side} length drifted"
        );
        for position in 0..left[output_index].len() {
            let lhs = left[output_index][position];
            let rhs = right[output_index][position];
            assert!(
                (lhs - rhs).abs() <= TOLERANCE,
                "{name} {side} mismatch at output {output_index} position {position}: left {lhs} right {rhs}"
            );
        }
    }
}

fn assert_same_series(name: &str, side: &str, left: &[Real], right: &[Real]) {
    assert_eq!(left.len(), right.len(), "{name} {side} length drifted");
    for position in 0..left.len() {
        let lhs = left[position];
        let rhs = right[position];
        assert!(
            (lhs - rhs).abs() <= TOLERANCE,
            "{name} {side} mismatch at position {position}: left {lhs} right {rhs}"
        );
    }
}

fn assert_matches_talib(name: &str, options: &[Real], inputs: &[Vec<Real>]) {
    let indicator = tulipindicators::find(name).expect("indicator should be registered");
    let rust_inputs: Vec<&[Real]> = inputs.iter().map(Vec::as_slice).collect();
    let rust_outputs = indicator
        .run(&rust_inputs, options)
        .expect("rust run should succeed");
    let c_outputs = run_c_oracle(&ensure_stable_oracle(), name, options, inputs);
    let talib_outputs = run_c_oracle(&ensure_talib_oracle(), name, options, inputs);
    assert_same_outputs(name, "rust-vs-c", &rust_outputs, &c_outputs);
    assert_same_outputs(name, "rust-vs-talib", &rust_outputs, &talib_outputs);
    assert_same_outputs(name, "c-vs-talib", &c_outputs, &talib_outputs);
}

fn ensure_stable_oracle() -> PathBuf {
    static ORACLE: OnceLock<PathBuf> = OnceLock::new();
    ORACLE
        .get_or_init(|| {
            let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let target_dir = repo_root.join("target").join("stable-parity");
            let oracle = target_dir.join("stable_oracle");
            let lib = repo_root.join("c").join("libindicators.a");

            fs::create_dir_all(&target_dir).expect("failed to create stable parity target dir");

            let status = Command::new("make")
                .arg("-C")
                .arg(repo_root.join("c"))
                .arg("libindicators.a")
                .status()
                .expect("failed to build c library for stable oracle");
            assert!(
                status.success(),
                "failed to build c library for stable oracle"
            );

            let helper = repo_root
                .join("rust")
                .join("tests")
                .join("support")
                .join("stable_oracle.c");

            let status = Command::new("cc")
                .arg("-std=c99")
                .arg("-O2")
                .arg("-I")
                .arg(repo_root.join("c"))
                .arg(&helper)
                .arg(&lib)
                .arg("-lm")
                .arg("-o")
                .arg(&oracle)
                .status()
                .expect("failed to compile stable oracle");
            assert!(status.success(), "failed to compile stable oracle");

            oracle
        })
        .clone()
}

fn ensure_talib_oracle() -> PathBuf {
    static ORACLE: OnceLock<PathBuf> = OnceLock::new();
    ORACLE
        .get_or_init(|| {
            let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let target_dir = repo_root.join("target").join("talib-parity");
            let oracle = target_dir.join("talib_missing_oracle");

            fs::create_dir_all(&target_dir).expect("failed to create ta-lib parity target dir");

            let helper = repo_root
                .join("rust")
                .join("tests")
                .join("support")
                .join("talib_missing_oracle.c");

            let status = Command::new("cc")
                .arg("-std=c99")
                .arg("-O2")
                .arg("-I")
                .arg("/opt/homebrew/include")
                .arg(&helper)
                .arg("-L")
                .arg("/opt/homebrew/lib")
                .arg("-lta-lib")
                .arg("-lm")
                .arg("-o")
                .arg(&oracle)
                .status()
                .expect("failed to compile ta-lib oracle");
            assert!(status.success(), "failed to compile ta-lib oracle");

            oracle
        })
        .clone()
}

fn run_c_oracle(
    oracle: &Path,
    indicator: &str,
    options: &[Real],
    inputs: &[Vec<Real>],
) -> Vec<Vec<Real>> {
    let input_len = inputs.first().map_or(0, Vec::len);
    let mut payload = format!("{} {} {}\n", options.len(), inputs.len(), input_len);
    for option in options {
        payload.push_str(&format!("{option:.17}\n"));
    }
    for series in inputs {
        for value in series {
            payload.push_str(&format!("{value:.17}\n"));
        }
    }

    let output = Command::new(oracle)
        .arg(indicator)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            child
                .stdin
                .as_mut()
                .expect("stdin should be available")
                .write_all(payload.as_bytes())?;
            child.wait_with_output()
        })
        .expect("failed to run c oracle");

    assert!(
        output.status.success(),
        "c oracle failed for {}: {}",
        indicator,
        String::from_utf8_lossy(&output.stderr)
    );

    parse_oracle_output(&String::from_utf8(output.stdout).expect("oracle output should be utf8"))
}

fn parse_oracle_output(stdout: &str) -> Vec<Vec<Real>> {
    let mut lines = stdout.lines();
    let header = lines.next().expect("missing oracle header");
    let mut header_parts = header.split_whitespace();
    let outputs = header_parts
        .next()
        .and_then(|value| value.parse::<usize>().ok())
        .expect("missing output count");

    let mut parsed = Vec::with_capacity(outputs);
    for _ in 0..outputs {
        let line = lines.next().expect("missing output line");
        let values = if line.trim().is_empty() {
            Vec::new()
        } else {
            line.split_whitespace()
                .map(|value| value.parse::<Real>().expect("value should parse"))
                .collect()
        };
        parsed.push(values);
    }
    parsed
}
