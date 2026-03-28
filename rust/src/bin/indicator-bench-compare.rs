use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

use tulipindicators::benchmark::{
    render_tsv, run_named_benchmarks, BenchmarkConfig, BenchmarkResult,
};

const DEFAULT_REGRESSION_WARN: f64 = 1.15;

#[derive(Debug, Clone)]
struct ExternalBenchmarkRow {
    indicator: String,
    mode: String,
    input_len: usize,
    calibration_runs: usize,
    calibration_ms: f64,
    iterations: usize,
    outputs: usize,
    sample_min_ms: f64,
    total_ms: f64,
    sample_max_ms: f64,
    sample_stddev_ms: f64,
    sample_cv: f64,
    ns_per_input: f64,
}

#[derive(Debug, Clone)]
struct CompareRow {
    indicator: String,
    mode: String,
    input_len: usize,
    c_calibration_runs: usize,
    rust_calibration_runs: usize,
    c_calibration_ms: f64,
    rust_calibration_ms: f64,
    c_ns_per_input: f64,
    rust_ns_per_input: f64,
    c_sample_min_ms: f64,
    c_sample_median_ms: f64,
    c_sample_max_ms: f64,
    c_sample_stddev_ms: f64,
    c_sample_cv: f64,
    rust_sample_min_ms: f64,
    rust_sample_median_ms: f64,
    rust_sample_max_ms: f64,
    rust_sample_stddev_ms: f64,
    rust_sample_cv: f64,
    ratio: f64,
    status: &'static str,
}

fn main() -> ExitCode {
    let config = BenchmarkConfig::from_env();
    let regression_warn = std::env::var("TI_BENCH_REGRESSION_WARN")
        .ok()
        .and_then(|raw| raw.parse::<f64>().ok())
        .filter(|value| *value > 0.0)
        .unwrap_or(DEFAULT_REGRESSION_WARN);

    match run_compare(&config, regression_warn) {
        Ok(paths) => {
            println!("Saved reports:");
            println!("- {}", paths.c_tsv.display());
            println!("- {}", paths.rust_tsv.display());
            println!("- {}", paths.compare_tsv.display());
            println!("- {}", paths.compare_md.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("benchmark compare failed: {error}");
            ExitCode::from(1)
        }
    }
}

struct ComparePaths {
    c_tsv: PathBuf,
    rust_tsv: PathBuf,
    compare_tsv: PathBuf,
    compare_md: PathBuf,
}

fn run_compare(config: &BenchmarkConfig, regression_warn: f64) -> Result<ComparePaths, String> {
    fs::create_dir_all(&config.output_dir).map_err(|error| error.to_string())?;

    build_c_contract_benchmark()?;
    let c_rows = run_c_benchmark(config)?;
    let indicator_names = unique_indicator_names(&c_rows);
    let rust_results =
        run_named_benchmarks(config, &indicator_names).map_err(|e| format!("{e:?}"))?;
    let compare_rows = compare_rows(&c_rows, &rust_results, regression_warn)?;

    let c_tsv = config.output_dir.join("c-latest.tsv");
    let rust_tsv = config.output_dir.join("rust-stable-latest.tsv");
    let compare_tsv = config.output_dir.join("compare-latest.tsv");
    let compare_md = config.output_dir.join("compare-latest.md");

    fs::write(&c_tsv, render_external_tsv(&c_rows)).map_err(|error| error.to_string())?;
    fs::write(&rust_tsv, render_tsv(&rust_results)).map_err(|error| error.to_string())?;
    fs::write(&compare_tsv, render_compare_tsv(&compare_rows))
        .map_err(|error| error.to_string())?;
    fs::write(
        &compare_md,
        render_compare_markdown(&compare_rows, regression_warn),
    )
    .map_err(|error| error.to_string())?;

    print!(
        "{}",
        render_compare_markdown(&compare_rows, regression_warn)
    );

    Ok(ComparePaths {
        c_tsv,
        rust_tsv,
        compare_tsv,
        compare_md,
    })
}

fn build_c_contract_benchmark() -> Result<(), String> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let status = Command::new("make")
        .arg("-C")
        .arg(repo_root.join("c"))
        .arg("benchmark_contract")
        .status()
        .map_err(|error| format!("failed to launch make for C benchmark: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err("failed to build C benchmark_contract".to_string())
    }
}

fn run_c_benchmark(config: &BenchmarkConfig) -> Result<Vec<ExternalBenchmarkRow>, String> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let executable = repo_root.join("c").join("benchmark_contract");
    let output = Command::new(&executable)
        .env("TI_BENCH_SIZES", join_sizes(&config.sizes))
        .env(
            "TI_BENCH_STREAM_CHUNK",
            config.stream_chunk_size.to_string(),
        )
        .env("TI_BENCH_MIN_ITERATIONS", config.min_iterations.to_string())
        .env(
            "TI_BENCH_CALIBRATION_MS",
            config.calibration_duration.as_millis().to_string(),
        )
        .env(
            "TI_BENCH_TARGET_MS",
            config.target_duration.as_millis().to_string(),
        )
        .output()
        .map_err(|error| format!("failed to run C benchmark_contract: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "C benchmark_contract failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    parse_external_tsv(&String::from_utf8_lossy(&output.stdout))
}

fn join_sizes(sizes: &[usize]) -> String {
    sizes
        .iter()
        .map(|size| size.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn parse_external_tsv(raw: &str) -> Result<Vec<ExternalBenchmarkRow>, String> {
    let mut rows = Vec::new();
    for (line_index, line) in raw.lines().enumerate() {
        if line_index == 0 {
            continue;
        }
        if line.trim().is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() != 13 {
            return Err(format!("invalid benchmark TSV row: {line}"));
        }
        rows.push(ExternalBenchmarkRow {
            indicator: parts[0].to_string(),
            mode: parts[1].to_string(),
            input_len: parts[2]
                .parse()
                .map_err(|_| format!("invalid input_len in row: {line}"))?,
            calibration_runs: parts[3]
                .parse()
                .map_err(|_| format!("invalid calibration_runs in row: {line}"))?,
            calibration_ms: parts[4]
                .parse()
                .map_err(|_| format!("invalid calibration_ms in row: {line}"))?,
            iterations: parts[5]
                .parse()
                .map_err(|_| format!("invalid iterations in row: {line}"))?,
            outputs: parts[6]
                .parse()
                .map_err(|_| format!("invalid outputs in row: {line}"))?,
            sample_min_ms: parts[7]
                .parse()
                .map_err(|_| format!("invalid sample_min_ms in row: {line}"))?,
            total_ms: parts[8]
                .parse()
                .map_err(|_| format!("invalid sample_median_ms in row: {line}"))?,
            sample_max_ms: parts[9]
                .parse()
                .map_err(|_| format!("invalid sample_max_ms in row: {line}"))?,
            sample_stddev_ms: parts[10]
                .parse()
                .map_err(|_| format!("invalid sample_stddev_ms in row: {line}"))?,
            sample_cv: parts[11]
                .parse()
                .map_err(|_| format!("invalid sample_cv in row: {line}"))?,
            ns_per_input: parts[12]
                .parse()
                .map_err(|_| format!("invalid ns_per_input in row: {line}"))?,
        });
    }
    Ok(rows)
}

fn unique_indicator_names(rows: &[ExternalBenchmarkRow]) -> Vec<String> {
    let mut names = Vec::new();
    let mut seen = BTreeSet::new();
    for row in rows {
        if seen.insert(row.indicator.clone()) {
            names.push(row.indicator.clone());
        }
    }
    names
}

fn compare_rows(
    c_rows: &[ExternalBenchmarkRow],
    rust_results: &[BenchmarkResult],
    regression_warn: f64,
) -> Result<Vec<CompareRow>, String> {
    let mut rust_map = BTreeMap::new();
    for row in rust_results {
        rust_map.insert(
            (
                row.indicator.to_string(),
                row.mode.as_str().to_string(),
                row.input_len,
            ),
            row,
        );
    }

    let mut rows = Vec::with_capacity(c_rows.len());
    for c_row in c_rows {
        let key = (c_row.indicator.clone(), c_row.mode.clone(), c_row.input_len);
        let rust_row = rust_map
            .remove(&key)
            .ok_or_else(|| format!("missing Rust benchmark row for {:?}", key))?;
        let ratio = rust_row.ns_per_input / c_row.ns_per_input;
        let status = if ratio > regression_warn {
            "regression"
        } else if ratio < 1.0 / regression_warn {
            "faster"
        } else {
            "within"
        };
        rows.push(CompareRow {
            indicator: c_row.indicator.clone(),
            mode: c_row.mode.clone(),
            input_len: c_row.input_len,
            c_calibration_runs: c_row.calibration_runs,
            rust_calibration_runs: rust_row.calibration_runs,
            c_calibration_ms: c_row.calibration_ms,
            rust_calibration_ms: rust_row.calibration_ms,
            c_ns_per_input: c_row.ns_per_input,
            rust_ns_per_input: rust_row.ns_per_input,
            c_sample_min_ms: c_row.sample_min_ms,
            c_sample_median_ms: c_row.total_ms,
            c_sample_max_ms: c_row.sample_max_ms,
            c_sample_stddev_ms: c_row.sample_stddev_ms,
            c_sample_cv: c_row.sample_cv,
            rust_sample_min_ms: rust_row.sample_min.as_secs_f64() * 1000.0,
            rust_sample_median_ms: rust_row.elapsed.as_secs_f64() * 1000.0,
            rust_sample_max_ms: rust_row.sample_max.as_secs_f64() * 1000.0,
            rust_sample_stddev_ms: rust_row.sample_stddev_ms,
            rust_sample_cv: rust_row.sample_cv,
            ratio,
            status,
        });
    }

    Ok(rows)
}

fn render_external_tsv(rows: &[ExternalBenchmarkRow]) -> String {
    let mut out = String::from(
        "indicator\tmode\tinput_len\tcalibration_runs\tcalibration_ms\titerations\toutputs\tsample_min_ms\tsample_median_ms\tsample_max_ms\tsample_stddev_ms\tsample_cv\tns_per_input\n",
    );
    for row in rows {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{:.3}\t{}\t{}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.2}\n",
            row.indicator,
            row.mode,
            row.input_len,
            row.calibration_runs,
            row.calibration_ms,
            row.iterations,
            row.outputs,
            row.sample_min_ms,
            row.total_ms,
            row.sample_max_ms,
            row.sample_stddev_ms,
            row.sample_cv,
            row.ns_per_input
        ));
    }
    out
}

fn render_compare_tsv(rows: &[CompareRow]) -> String {
    let mut out = String::from(
        "indicator\tmode\tinput_len\tc_calibration_runs\trust_calibration_runs\tc_calibration_ms\trust_calibration_ms\tc_sample_min_ms\tc_sample_median_ms\tc_sample_max_ms\tc_sample_stddev_ms\tc_sample_cv\trust_sample_min_ms\trust_sample_median_ms\trust_sample_max_ms\trust_sample_stddev_ms\trust_sample_cv\tc_ns_per_input\trust_ns_per_input\tratio\tstatus\n",
    );
    for row in rows {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.2}\t{:.2}\t{:.3}\t{}\n",
            row.indicator,
            row.mode,
            row.input_len,
            row.c_calibration_runs,
            row.rust_calibration_runs,
            row.c_calibration_ms,
            row.rust_calibration_ms,
            row.c_sample_min_ms,
            row.c_sample_median_ms,
            row.c_sample_max_ms,
            row.c_sample_stddev_ms,
            row.c_sample_cv,
            row.rust_sample_min_ms,
            row.rust_sample_median_ms,
            row.rust_sample_max_ms,
            row.rust_sample_stddev_ms,
            row.rust_sample_cv,
            row.c_ns_per_input,
            row.rust_ns_per_input,
            row.ratio,
            row.status
        ));
    }
    out
}

fn render_compare_markdown(rows: &[CompareRow], regression_warn: f64) -> String {
    let mut out = String::new();
    out.push_str("# C vs Rust Indicator Benchmarks\n\n");
    out.push_str(&format!(
        "Regression threshold: Rust slower than C by more than `{:.2}x`.\n\n",
        regression_warn
    ));
    out.push_str(
        "| indicator | mode | input_len | c ns/input | rust ns/input | ratio | c sample ms (min/med/max/stddev/cv) | rust sample ms (min/med/max/stddev/cv) | status |\n",
    );
    out.push_str("| --- | --- | ---: | ---: | ---: | ---: | --- | --- | --- |\n");
    for row in rows {
        out.push_str(&format!(
            "| {} | {} | {} | {:.2} | {:.2} | {:.3} | {:.3}/{:.3}/{:.3}/{:.3}/{:.3} | {:.3}/{:.3}/{:.3}/{:.3}/{:.3} | {} |\n",
            row.indicator,
            row.mode,
            row.input_len,
            row.c_ns_per_input,
            row.rust_ns_per_input,
            row.ratio,
            row.c_sample_min_ms,
            row.c_sample_median_ms,
            row.c_sample_max_ms,
            row.c_sample_stddev_ms,
            row.c_sample_cv,
            row.rust_sample_min_ms,
            row.rust_sample_median_ms,
            row.rust_sample_max_ms,
            row.rust_sample_stddev_ms,
            row.rust_sample_cv,
            row.status
        ));
    }
    out
}
