use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use tulipindicators::benchmark::{run_named_benchmarks, BenchmarkConfig, BenchmarkMode};

const INDICATORS: &[&str] = &[
    "linearregangle",
    "midpoint",
    "midprice",
    "rocr100",
    "maxindex",
    "minindex",
    "minmax",
    "minmaxindex",
    "beta",
    "correl",
    "macdfix",
];

#[test]
fn first_missing_batch_is_covered_by_c_and_rust_benchmarks() {
    let config = BenchmarkConfig {
        sizes: vec![4096],
        stream_chunk_size: 1024,
        min_iterations: 1,
        target_duration: Duration::from_millis(20),
        calibration_duration: Duration::from_millis(5),
        repeats: 1,
        output_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("target")
            .join("indicator-bench"),
    };

    let rust_rows = run_named_benchmarks(
        &config,
        &INDICATORS
            .iter()
            .map(|name| (*name).to_string())
            .collect::<Vec<_>>(),
    )
    .expect("rust benchmarks should run");
    let c_rows = run_c_benchmark_rows();

    let mut rust_batch = BTreeMap::new();
    for row in rust_rows {
        if matches!(row.mode, BenchmarkMode::Batch) {
            rust_batch.insert((row.indicator, row.input_len), row.ns_per_input);
        }
    }

    let mut c_batch = BTreeMap::new();
    for row in c_rows {
        if row.mode == "batch" {
            c_batch.insert((row.indicator, row.input_len), row.ns_per_input);
        }
    }

    for indicator in INDICATORS {
        let key = (*indicator, 4096usize);
        let rust = rust_batch
            .get(&key)
            .unwrap_or_else(|| panic!("missing Rust benchmark row for {indicator}"));
        let c = c_batch
            .get(&key)
            .unwrap_or_else(|| panic!("missing C benchmark row for {indicator}"));
        assert!(
            rust.is_finite() && *rust > 0.0,
            "invalid Rust ns/input for {indicator}"
        );
        assert!(
            c.is_finite() && *c > 0.0,
            "invalid C ns/input for {indicator}"
        );
        let ratio = rust / c;
        assert!(
            ratio.is_finite() && ratio > 0.0,
            "invalid C/Rust ratio for {indicator}"
        );
    }
}

#[derive(Debug)]
struct CBenchmarkRow<'a> {
    indicator: &'a str,
    mode: &'a str,
    input_len: usize,
    ns_per_input: f64,
}

fn run_c_benchmark_rows() -> Vec<CBenchmarkRow<'static>> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let c_dir = repo_root.join("c");

    let status = Command::new("make")
        .arg("-C")
        .arg(&c_dir)
        .arg("benchmark_contract")
        .status()
        .expect("failed to build c benchmark_contract");
    assert!(status.success(), "failed to build c benchmark_contract");

    let filter = INDICATORS.join(",");
    let output = Command::new(c_dir.join("benchmark_contract"))
        .env("TI_BENCH_INDICATORS", &filter)
        .env("TI_BENCH_SIZES", "4096")
        .env("TI_BENCH_TARGET_MS", "20")
        .env("TI_BENCH_CALIBRATION_MS", "5")
        .env("TI_BENCH_REPEATS", "1")
        .output()
        .expect("failed to run c benchmark_contract");

    assert!(
        output.status.success(),
        "c benchmark_contract failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    parse_c_rows(&String::from_utf8(output.stdout).expect("benchmark output should be utf8"))
}

fn parse_c_rows(tsv: &str) -> Vec<CBenchmarkRow<'static>> {
    let mut rows = Vec::new();
    for line in tsv.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<_> = line.split('\t').collect();
        assert!(parts.len() >= 13, "invalid c benchmark row: {line}");
        rows.push(CBenchmarkRow {
            indicator: Box::leak(parts[0].to_string().into_boxed_str()),
            mode: Box::leak(parts[1].to_string().into_boxed_str()),
            input_len: parts[2]
                .parse::<usize>()
                .unwrap_or_else(|_| panic!("invalid input_len in row: {line}")),
            ns_per_input: parts[12]
                .parse::<f64>()
                .unwrap_or_else(|_| panic!("invalid ns/input in row: {line}")),
        });
    }
    rows
}
