#[path = "support/golden_cases.rs"]
mod golden_cases;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use golden_cases::{parse_cases, TEST_FILES};
use tulipindicators::Real;

const TOLERANCE: Real = 1e-6;
const BETA_INDICATORS: &[&str] = &[
    "abands", "alma", "ce", "cmf", "copp", "dc", "fi", "ikhts", "kc", "kst", "mama", "pbands",
    "pc", "pfe", "posc", "rmi", "rmta", "rvi", "smi", "tsi", "vwap",
];

#[test]
fn rust_stable_outputs_match_c_implementations() {
    let oracle = ensure_stable_oracle();

    for path in TEST_FILES {
        for case in parse_cases(path) {
            if BETA_INDICATORS.contains(&case.name.as_str()) {
                continue;
            }

            let rust_indicator = tulipindicators::find(&case.name)
                .unwrap_or_else(|| panic!("{} should be registered in Rust", case.name));
            let rust_inputs: Vec<&[Real]> = case.inputs.iter().map(Vec::as_slice).collect();
            let rust_outputs = rust_indicator
                .run(&rust_inputs, &case.options)
                .unwrap_or_else(|error| panic!("rust run failed for {}: {error}", case.name));
            let c_outputs = run_c_oracle(&oracle, &case.name, &case.options, &case.inputs);

            assert_eq!(
                rust_outputs.len(),
                c_outputs.len(),
                "{} {} output count mismatch",
                path,
                case.name
            );

            for (output_index, (rust_output, c_output)) in
                rust_outputs.iter().zip(c_outputs.iter()).enumerate()
            {
                assert_eq!(
                    rust_output.len(),
                    c_output.len(),
                    "{} {} output {} length mismatch",
                    path,
                    case.name,
                    output_index
                );

                for (position, (rust_value, c_value)) in
                    rust_output.iter().zip(c_output.iter()).enumerate()
                {
                    let delta = (rust_value - c_value).abs();
                    assert!(
                        delta <= TOLERANCE,
                        "{} {} output {} mismatch at {}: rust={} c={} delta={}",
                        path,
                        case.name,
                        output_index,
                        position,
                        rust_value,
                        c_value,
                        delta
                    );
                }
            }
        }
    }
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
        .unwrap_or_else(|error| panic!("failed to run c oracle for {}: {error}", indicator));

    assert!(
        output.status.success(),
        "c oracle failed for {}: {}",
        indicator,
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("oracle output should be utf8");
    parse_oracle_output(indicator, &stdout)
}

fn parse_oracle_output(indicator: &str, stdout: &str) -> Vec<Vec<Real>> {
    let mut lines = stdout.lines();
    let header = lines
        .next()
        .unwrap_or_else(|| panic!("missing oracle header for {indicator}"));
    let mut header_parts = header.split_whitespace();
    let outputs = header_parts
        .next()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or_else(|| panic!("missing oracle output count for {indicator}"));
    let output_len = header_parts
        .next()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or_else(|| panic!("missing oracle output length for {indicator}"));

    let mut parsed = Vec::with_capacity(outputs);
    for output_index in 0..outputs {
        let line = lines.next().unwrap_or_else(|| {
            panic!(
                "missing oracle output line {} for {}",
                output_index, indicator
            )
        });
        let values: Vec<Real> = if line.trim().is_empty() {
            Vec::new()
        } else {
            line.split_whitespace()
                .map(|value| {
                    value.parse::<Real>().unwrap_or_else(|error| {
                        panic!("failed to parse oracle value `{value}` for {indicator}: {error}")
                    })
                })
                .collect()
        };
        assert_eq!(
            values.len(),
            output_len,
            "{} oracle output {} length mismatch",
            indicator,
            output_index
        );
        parsed.push(values);
    }

    parsed
}
