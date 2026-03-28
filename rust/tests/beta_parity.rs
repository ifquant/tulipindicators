use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use tulipindicators::{find, Real};

const BETA_CASES: &[BetaCase] = &[
    BetaCase::new("abands", &[7.0], &["high", "low", "close"]),
    BetaCase::new("alma", &[7.0, 0.85, 6.0], &["real"]),
    BetaCase::new("ce", &[7.0, 2.0], &["high", "low", "close"]),
    BetaCase::new("cmf", &[7.0], &["high", "low", "close", "volume"]),
    BetaCase::new("copp", &[5.0, 10.0, 7.0], &["real"]),
    BetaCase::new("dc", &[7.0], &["real"]),
    BetaCase::new("fi", &[7.0], &["close", "volume"]),
    BetaCase::new("ikhts", &[9.0], &["high", "low"]),
    BetaCase::new("kc", &[7.0, 2.0], &["high", "low", "close"]),
    BetaCase::new(
        "kst",
        &[10.0, 15.0, 20.0, 30.0, 10.0, 10.0, 10.0, 15.0],
        &["real"],
    ),
    BetaCase::new("mama", &[0.5, 0.05], &["real"]),
    BetaCase::new("pbands", &[7.0], &["high", "low", "close"]),
    BetaCase::new("pc", &[7.0], &["high", "low"]),
    BetaCase::new("pfe", &[7.0, 5.0], &["real"]),
    BetaCase::new("posc", &[7.0, 5.0], &["high", "low", "close"]),
    BetaCase::new("rmi", &[5.0, 4.0], &["real"]),
    BetaCase::new("rmta", &[7.0, 0.2], &["real"]),
    BetaCase::new("rvi", &[5.0, 7.0], &["real"]),
    BetaCase::new("smi", &[7.0, 5.0, 3.0], &["high", "low", "close"]),
    BetaCase::new("tsi", &[5.0, 7.0], &["real"]),
    BetaCase::new("vwap", &[7.0], &["high", "low", "close", "volume"]),
];

const INPUT_LEN: usize = 256;
const TOLERANCE: Real = 1e-6;

#[test]
fn rust_beta_outputs_match_c_implementations() {
    let oracle = ensure_beta_oracle();

    for case in BETA_CASES {
        let indicator = find(case.name).expect("beta indicator should be registered");
        let generated_inputs = build_inputs(case.input_names, INPUT_LEN);
        let rust_inputs: Vec<&[Real]> = generated_inputs.iter().map(Vec::as_slice).collect();
        let rust_outputs = indicator
            .run(&rust_inputs, case.options)
            .unwrap_or_else(|error| panic!("rust run failed for {}: {error}", case.name));
        let c_outputs = run_c_oracle(&oracle, case, &generated_inputs);

        assert_eq!(
            rust_outputs.len(),
            c_outputs.len(),
            "{} output count mismatch",
            case.name
        );

        for (index, (rust_output, c_output)) in
            rust_outputs.iter().zip(c_outputs.iter()).enumerate()
        {
            assert_eq!(
                rust_output.len(),
                c_output.len(),
                "{} output {} length mismatch",
                case.name,
                index
            );

            for (position, (rust_value, c_value)) in
                rust_output.iter().zip(c_output.iter()).enumerate()
            {
                let delta = (rust_value - c_value).abs();
                assert!(
                    delta <= TOLERANCE,
                    "{} output {} mismatch at {}: rust={} c={} delta={}",
                    case.name,
                    index,
                    position,
                    rust_value,
                    c_value,
                    delta
                );
            }
        }
    }
}

#[derive(Clone, Copy)]
struct BetaCase {
    name: &'static str,
    options: &'static [Real],
    input_names: &'static [&'static str],
}

impl BetaCase {
    const fn new(
        name: &'static str,
        options: &'static [Real],
        input_names: &'static [&'static str],
    ) -> Self {
        Self {
            name,
            options,
            input_names,
        }
    }
}

fn ensure_beta_oracle() -> PathBuf {
    static ORACLE: OnceLock<PathBuf> = OnceLock::new();
    ORACLE
        .get_or_init(|| {
            let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let target_dir = repo_root.join("target").join("beta-parity");
            let oracle = target_dir.join("beta_oracle");
            let lib = repo_root.join("c").join("libindicators.a");

            fs::create_dir_all(&target_dir).expect("failed to create beta parity target dir");

            let status = Command::new("make")
                .arg("-C")
                .arg(repo_root.join("c"))
                .arg("libindicators.a")
                .status()
                .expect("failed to build c library for beta oracle");
            assert!(
                status.success(),
                "failed to build c library for beta oracle"
            );

            let helper = repo_root
                .join("rust")
                .join("tests")
                .join("support")
                .join("beta_oracle.c");
            let indices = repo_root
                .join("rust")
                .join("tests")
                .join("support")
                .join("beta_test_indices.h");

            let mut command = Command::new("cc");
            command
                .arg("-std=c99")
                .arg("-O2")
                .arg("-I")
                .arg(repo_root.join("c"))
                .arg("-I")
                .arg(repo_root.join("c").join("indicators"))
                .arg("-include")
                .arg(indices)
                .arg(&helper);

            for entry in fs::read_dir(repo_root.join("c").join("beta"))
                .expect("failed to read beta source directory")
            {
                let path = entry.expect("failed to read beta entry").path();
                if path.extension().and_then(|ext| ext.to_str()) == Some("c") {
                    command.arg(path);
                }
            }

            let status = command
                .arg(&lib)
                .arg("-lm")
                .arg("-o")
                .arg(&oracle)
                .status()
                .expect("failed to compile beta oracle");
            assert!(status.success(), "failed to compile beta oracle");

            oracle
        })
        .clone()
}

fn run_c_oracle(oracle: &Path, case: &BetaCase, inputs: &[Vec<Real>]) -> Vec<Vec<Real>> {
    let mut payload = format!("{} {} {}\n", case.options.len(), inputs.len(), INPUT_LEN);
    for option in case.options {
        payload.push_str(&format!("{option:.17}\n"));
    }
    for series in inputs {
        for value in series {
            payload.push_str(&format!("{value:.17}\n"));
        }
    }

    let output = Command::new(oracle)
        .arg(case.name)
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
        .unwrap_or_else(|error| panic!("failed to run c oracle for {}: {error}", case.name));

    assert!(
        output.status.success(),
        "c oracle failed for {}: {}",
        case.name,
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("oracle output should be utf8");
    parse_oracle_output(case.name, &stdout)
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

fn build_inputs(input_names: &[&str], input_len: usize) -> Vec<Vec<Real>> {
    let close = build_close_series(input_len);
    let volume = build_volume_series(input_len);

    input_names
        .iter()
        .map(|name| match *name {
            "real" | "close" => close.clone(),
            "open" => close
                .iter()
                .enumerate()
                .map(|(index, close_value)| close_value - ((index % 3) as Real - 1.0) * 0.13)
                .collect(),
            "high" => close
                .iter()
                .enumerate()
                .map(|(index, close_value)| close_value + 0.35 + (index % 5) as Real * 0.03)
                .collect(),
            "low" => close
                .iter()
                .enumerate()
                .map(|(index, close_value)| close_value - 0.35 - (index % 5) as Real * 0.03)
                .collect(),
            "volume" => volume.clone(),
            other => panic!("unsupported synthetic input `{other}`"),
        })
        .collect()
}

fn build_close_series(input_len: usize) -> Vec<Real> {
    (0..input_len)
        .map(|index| {
            let trend = 100.0 + index as Real * 0.015;
            let wave = (index as Real / 17.0).sin() * 1.7;
            let ripple = (index as Real / 7.0).cos() * 0.6;
            trend + wave + ripple
        })
        .collect()
}

fn build_volume_series(input_len: usize) -> Vec<Real> {
    (0..input_len)
        .map(|index| 10_000.0 + (index % 250) as Real * 37.0 + (index % 13) as Real * 11.0)
        .collect()
}
