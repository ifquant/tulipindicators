#[path = "support/candle_cases.rs"]
mod candle_cases;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use candle_cases::parse_candle_cases;
use tulipindicators::{run_candles, CandleConfig, CandleSet};

const CANDLES_FIXTURE: &str = "c/tests/candles.txt";

#[test]
fn rust_candles_match_named_fixture_expectations() {
    for case in parse_candle_cases(CANDLES_FIXTURE) {
        let inputs = case.inputs.each_ref().map(Vec::as_slice);
        let result = run_candles(u64::MAX >> (64 - 26), &inputs, &CandleConfig::default())
            .expect("rust candle engine should run");

        for expectation in case.expectations {
            for index in expectation.indices {
                let matched = result.at(index) & expectation.pattern != 0;
                assert_eq!(
                    matched, !expectation.negate,
                    "fixture expectation mismatch for {} at {}",
                    expectation.name, index
                );
            }
        }
    }
}

#[test]
fn rust_candles_match_c_candle_engine() {
    let oracle = ensure_candle_oracle();

    for case in parse_candle_cases(CANDLES_FIXTURE) {
        let input_len = case.inputs[0].len();
        let inputs = case.inputs.each_ref().map(Vec::as_slice);
        let result = run_candles(u64::MAX >> (64 - 26), &inputs, &CandleConfig::default())
            .expect("rust candle engine should run");
        let rust_sets: Vec<CandleSet> = (0..input_len).map(|index| result.at(index)).collect();
        let c_sets = run_c_oracle(&oracle, &case.inputs);

        assert_eq!(rust_sets, c_sets, "candle set mismatch for case");
        assert_eq!(
            result.count(),
            rust_sets.iter().filter(|&&set| set != 0).count(),
            "result count should match non-empty candle bars"
        );
        assert_eq!(
            result.pattern_count(),
            rust_sets
                .iter()
                .map(|set| set.count_ones() as usize)
                .sum::<usize>(),
            "pattern count should match total set bits"
        );
    }
}

fn ensure_candle_oracle() -> PathBuf {
    static ORACLE: OnceLock<PathBuf> = OnceLock::new();
    ORACLE
        .get_or_init(|| {
            let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let target_dir = repo_root.join("target").join("candles-parity");
            let oracle = target_dir.join("candle_oracle");
            let lib = repo_root.join("c").join("libindicators.a");

            fs::create_dir_all(&target_dir).expect("failed to create candle parity target dir");

            let status = Command::new("make")
                .arg("-C")
                .arg(repo_root.join("c"))
                .arg("libindicators.a")
                .status()
                .expect("failed to build c library for candle oracle");
            assert!(
                status.success(),
                "failed to build c library for candle oracle"
            );

            let helper = repo_root
                .join("rust")
                .join("tests")
                .join("support")
                .join("candle_oracle.c");

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
                .expect("failed to compile candle oracle");
            assert!(status.success(), "failed to compile candle oracle");

            oracle
        })
        .clone()
}

fn run_c_oracle(oracle: &Path, inputs: &[Vec<f64>; 4]) -> Vec<CandleSet> {
    let input_len = inputs[0].len();
    let mut payload = format!("{input_len}\n");
    for series in inputs {
        for value in series {
            payload.push_str(&format!("{value:.17}\n"));
        }
    }

    let output = Command::new(oracle)
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
        .expect("failed to run candle oracle");

    assert!(
        output.status.success(),
        "candle oracle failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("oracle output should be utf8");
    let mut lines = stdout.lines();
    let parsed_len = lines
        .next()
        .and_then(|value| value.parse::<usize>().ok())
        .expect("candle oracle should print input length");
    assert_eq!(parsed_len, input_len, "candle oracle input length mismatch");
    lines
        .map(|line| {
            line.parse::<u64>()
                .unwrap_or_else(|error| panic!("failed to parse candle set `{line}`: {error}"))
        })
        .collect()
}
