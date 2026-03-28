use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

use tulipindicators::benchmark::{
    render_kernel_probe_tsv, render_tsv, run_named_benchmarks, run_named_kernel_probes,
    BenchmarkConfig, BenchmarkResult, KernelProbeResult,
};

const DEFAULT_REGRESSION_WARN: f64 = 1.15;
const DEFAULT_SELF_REGRESSION_WARN: f64 = 1.05;

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

#[derive(Debug, Clone)]
struct SelfCompareRow {
    indicator: String,
    mode: String,
    input_len: usize,
    best_calibration_runs: usize,
    current_calibration_runs: usize,
    best_calibration_ms: f64,
    current_calibration_ms: f64,
    best_ns_per_input: f64,
    current_ns_per_input: f64,
    best_sample_min_ms: f64,
    best_sample_median_ms: f64,
    best_sample_max_ms: f64,
    best_sample_stddev_ms: f64,
    best_sample_cv: f64,
    current_sample_min_ms: f64,
    current_sample_median_ms: f64,
    current_sample_max_ms: f64,
    current_sample_stddev_ms: f64,
    current_sample_cv: f64,
    ratio_to_best: f64,
    status: &'static str,
}

#[derive(Debug, Clone)]
struct KernelSplitRow {
    indicator: String,
    input_len: usize,
    run_in_place_ns_per_input: f64,
    kernel_ns_per_input: f64,
    ratio_to_kernel: f64,
    run_in_place_sample_cv: f64,
    kernel_sample_cv: f64,
    status: &'static str,
}

fn main() -> ExitCode {
    let config = BenchmarkConfig::from_env();
    let regression_warn = std::env::var("TI_BENCH_REGRESSION_WARN")
        .ok()
        .and_then(|raw| raw.parse::<f64>().ok())
        .filter(|value| *value > 0.0)
        .unwrap_or(DEFAULT_REGRESSION_WARN);
    let self_regression_warn = std::env::var("TI_BENCH_SELF_REGRESSION_WARN")
        .ok()
        .and_then(|raw| raw.parse::<f64>().ok())
        .filter(|value| *value >= 1.0)
        .unwrap_or(DEFAULT_SELF_REGRESSION_WARN);

    match run_compare(&config, regression_warn, self_regression_warn) {
        Ok(paths) => {
            println!("Saved reports:");
            println!("- {}", paths.c_tsv.display());
            println!("- {}", paths.rust_tsv.display());
            println!("- {}", paths.rust_best_tsv.display());
            println!("- {}", paths.compare_tsv.display());
            println!("- {}", paths.compare_md.display());
            println!("- {}", paths.self_compare_tsv.display());
            println!("- {}", paths.self_compare_md.display());
            println!("- {}", paths.kernel_probe_tsv.display());
            println!("- {}", paths.kernel_split_tsv.display());
            println!("- {}", paths.kernel_split_md.display());
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
    rust_best_tsv: PathBuf,
    compare_tsv: PathBuf,
    compare_md: PathBuf,
    self_compare_tsv: PathBuf,
    self_compare_md: PathBuf,
    kernel_probe_tsv: PathBuf,
    kernel_split_tsv: PathBuf,
    kernel_split_md: PathBuf,
}

fn run_compare(
    config: &BenchmarkConfig,
    regression_warn: f64,
    self_regression_warn: f64,
) -> Result<ComparePaths, String> {
    fs::create_dir_all(&config.output_dir).map_err(|error| error.to_string())?;

    build_c_contract_benchmark()?;
    let c_rows = run_c_benchmark(config)?;
    let indicator_names = unique_indicator_names(&c_rows);
    let rust_results =
        run_named_benchmarks(config, &indicator_names).map_err(|e| format!("{e:?}"))?;
    let kernel_probes =
        run_named_kernel_probes(config, &indicator_names).map_err(|e| format!("{e:?}"))?;
    let compare_rows = compare_rows(&c_rows, &rust_results, regression_warn)?;
    let rust_best_tsv = config.output_dir.join("rust-best.tsv");
    let existing_best = if rust_best_tsv.exists() {
        parse_rust_tsv(&fs::read_to_string(&rust_best_tsv).map_err(|error| error.to_string())?)?
    } else {
        Vec::new()
    };
    let merged_best = merge_best_rows(&existing_best, &rust_results);
    let self_compare_rows = compare_self_rows(&merged_best, &rust_results, self_regression_warn)?;

    let c_tsv = config.output_dir.join("c-latest.tsv");
    let rust_tsv = config.output_dir.join("rust-stable-latest.tsv");
    let compare_tsv = config.output_dir.join("compare-latest.tsv");
    let compare_md = config.output_dir.join("compare-latest.md");
    let self_compare_tsv = config.output_dir.join("rust-self-compare-latest.tsv");
    let self_compare_md = config.output_dir.join("rust-self-compare-latest.md");
    let kernel_probe_tsv = config.output_dir.join("rust-kernel-probes-latest.tsv");
    let kernel_split_tsv = config.output_dir.join("rust-kernel-split-latest.tsv");
    let kernel_split_md = config.output_dir.join("rust-kernel-split-latest.md");
    let kernel_split_rows = compare_kernel_split_rows(&rust_results, &kernel_probes);

    fs::write(&c_tsv, render_external_tsv(&c_rows)).map_err(|error| error.to_string())?;
    fs::write(&rust_tsv, render_tsv(&rust_results)).map_err(|error| error.to_string())?;
    fs::write(&rust_best_tsv, render_tsv(&merged_best)).map_err(|error| error.to_string())?;
    fs::write(&compare_tsv, render_compare_tsv(&compare_rows))
        .map_err(|error| error.to_string())?;
    fs::write(
        &compare_md,
        render_compare_markdown(&compare_rows, regression_warn),
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        &self_compare_tsv,
        render_self_compare_tsv(&self_compare_rows),
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        &self_compare_md,
        render_self_compare_markdown(&self_compare_rows, self_regression_warn),
    )
    .map_err(|error| error.to_string())?;
    fs::write(&kernel_probe_tsv, render_kernel_probe_tsv(&kernel_probes))
        .map_err(|error| error.to_string())?;
    fs::write(
        &kernel_split_tsv,
        render_kernel_split_tsv(&kernel_split_rows),
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        &kernel_split_md,
        render_kernel_split_markdown(&kernel_split_rows),
    )
    .map_err(|error| error.to_string())?;

    print!(
        "{}\n{}\n{}",
        render_compare_markdown(&compare_rows, regression_warn),
        render_self_compare_markdown(&self_compare_rows, self_regression_warn),
        render_kernel_split_markdown(&kernel_split_rows)
    );

    Ok(ComparePaths {
        c_tsv,
        rust_tsv,
        rust_best_tsv,
        compare_tsv,
        compare_md,
        self_compare_tsv,
        self_compare_md,
        kernel_probe_tsv,
        kernel_split_tsv,
        kernel_split_md,
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

fn parse_rust_tsv(raw: &str) -> Result<Vec<BenchmarkResult>, String> {
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
            return Err(format!("invalid Rust benchmark TSV row: {line}"));
        }
        let mode = match parts[1] {
            "batch" => tulipindicators::benchmark::BenchmarkMode::Batch,
            "stream" => tulipindicators::benchmark::BenchmarkMode::Stream,
            other => return Err(format!("invalid benchmark mode in row: {other}")),
        };
        rows.push(BenchmarkResult {
            indicator: leak_str(parts[0].to_string()),
            mode,
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
            total_outputs: parts[6]
                .parse()
                .map_err(|_| format!("invalid outputs in row: {line}"))?,
            sample_min: std::time::Duration::from_secs_f64(
                parts[7]
                    .parse::<f64>()
                    .map_err(|_| format!("invalid sample_min_ms in row: {line}"))?
                    / 1000.0,
            ),
            elapsed: std::time::Duration::from_secs_f64(
                parts[8]
                    .parse::<f64>()
                    .map_err(|_| format!("invalid sample_median_ms in row: {line}"))?
                    / 1000.0,
            ),
            sample_max: std::time::Duration::from_secs_f64(
                parts[9]
                    .parse::<f64>()
                    .map_err(|_| format!("invalid sample_max_ms in row: {line}"))?
                    / 1000.0,
            ),
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

fn leak_str(value: String) -> &'static str {
    Box::leak(value.into_boxed_str())
}

fn merge_best_rows(
    existing: &[BenchmarkResult],
    current: &[BenchmarkResult],
) -> Vec<BenchmarkResult> {
    let mut map: BTreeMap<(String, String, usize), BenchmarkResult> = BTreeMap::new();
    for row in existing.iter().chain(current.iter()) {
        let key = (
            row.indicator.to_string(),
            row.mode.as_str().to_string(),
            row.input_len,
        );
        let replace = match map.get(&key) {
            Some(best) => row.ns_per_input < best.ns_per_input,
            None => true,
        };
        if replace {
            map.insert(key, row.clone());
        }
    }
    map.into_values().collect()
}

fn compare_self_rows(
    best_rows: &[BenchmarkResult],
    current_rows: &[BenchmarkResult],
    self_regression_warn: f64,
) -> Result<Vec<SelfCompareRow>, String> {
    let mut best_map = BTreeMap::new();
    for row in best_rows {
        best_map.insert(
            (
                row.indicator.to_string(),
                row.mode.as_str().to_string(),
                row.input_len,
            ),
            row,
        );
    }

    let mut rows = Vec::with_capacity(current_rows.len());
    for current in current_rows {
        let key = (
            current.indicator.to_string(),
            current.mode.as_str().to_string(),
            current.input_len,
        );
        let best = best_map
            .remove(&key)
            .ok_or_else(|| format!("missing self baseline row for {:?}", key))?;
        let ratio_to_best = current.ns_per_input / best.ns_per_input;
        let status = if ratio_to_best > self_regression_warn {
            "slower-than-best"
        } else if ratio_to_best < 0.995 {
            "new-best"
        } else {
            "near-best"
        };
        rows.push(SelfCompareRow {
            indicator: current.indicator.to_string(),
            mode: current.mode.as_str().to_string(),
            input_len: current.input_len,
            best_calibration_runs: best.calibration_runs,
            current_calibration_runs: current.calibration_runs,
            best_calibration_ms: best.calibration_ms,
            current_calibration_ms: current.calibration_ms,
            best_ns_per_input: best.ns_per_input,
            current_ns_per_input: current.ns_per_input,
            best_sample_min_ms: best.sample_min.as_secs_f64() * 1000.0,
            best_sample_median_ms: best.elapsed.as_secs_f64() * 1000.0,
            best_sample_max_ms: best.sample_max.as_secs_f64() * 1000.0,
            best_sample_stddev_ms: best.sample_stddev_ms,
            best_sample_cv: best.sample_cv,
            current_sample_min_ms: current.sample_min.as_secs_f64() * 1000.0,
            current_sample_median_ms: current.elapsed.as_secs_f64() * 1000.0,
            current_sample_max_ms: current.sample_max.as_secs_f64() * 1000.0,
            current_sample_stddev_ms: current.sample_stddev_ms,
            current_sample_cv: current.sample_cv,
            ratio_to_best,
            status,
        });
    }
    Ok(rows)
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

fn compare_kernel_split_rows(
    rust_results: &[BenchmarkResult],
    kernel_probes: &[KernelProbeResult],
) -> Vec<KernelSplitRow> {
    let mut kernel_map = BTreeMap::new();
    for row in kernel_probes {
        kernel_map.insert((row.indicator.to_string(), row.input_len), row);
    }

    let mut rows = Vec::new();
    for rust_row in rust_results {
        if !matches!(
            rust_row.mode,
            tulipindicators::benchmark::BenchmarkMode::Batch
        ) {
            continue;
        }
        let key = (rust_row.indicator.to_string(), rust_row.input_len);
        let Some(kernel_row) = kernel_map.get(&key) else {
            continue;
        };
        let ratio_to_kernel = rust_row.ns_per_input / kernel_row.ns_per_input;
        let status = if ratio_to_kernel > 1.20 {
            "heavy-wrapper"
        } else if ratio_to_kernel > 1.05 {
            "moderate-wrapper"
        } else {
            "thin-wrapper"
        };
        rows.push(KernelSplitRow {
            indicator: rust_row.indicator.to_string(),
            input_len: rust_row.input_len,
            run_in_place_ns_per_input: rust_row.ns_per_input,
            kernel_ns_per_input: kernel_row.ns_per_input,
            ratio_to_kernel,
            run_in_place_sample_cv: rust_row.sample_cv,
            kernel_sample_cv: kernel_row.sample_cv,
            status,
        });
    }
    rows.sort_by(|a, b| b.ratio_to_kernel.total_cmp(&a.ratio_to_kernel));
    rows
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

fn render_self_compare_tsv(rows: &[SelfCompareRow]) -> String {
    let mut out = String::from(
        "indicator\tmode\tinput_len\tbest_calibration_runs\tcurrent_calibration_runs\tbest_calibration_ms\tcurrent_calibration_ms\tbest_sample_min_ms\tbest_sample_median_ms\tbest_sample_max_ms\tbest_sample_stddev_ms\tbest_sample_cv\tcurrent_sample_min_ms\tcurrent_sample_median_ms\tcurrent_sample_max_ms\tcurrent_sample_stddev_ms\tcurrent_sample_cv\tbest_ns_per_input\tcurrent_ns_per_input\tratio_to_best\tstatus\n",
    );
    for row in rows {
        out.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.2}\t{:.2}\t{:.3}\t{}\n",
            row.indicator,
            row.mode,
            row.input_len,
            row.best_calibration_runs,
            row.current_calibration_runs,
            row.best_calibration_ms,
            row.current_calibration_ms,
            row.best_sample_min_ms,
            row.best_sample_median_ms,
            row.best_sample_max_ms,
            row.best_sample_stddev_ms,
            row.best_sample_cv,
            row.current_sample_min_ms,
            row.current_sample_median_ms,
            row.current_sample_max_ms,
            row.current_sample_stddev_ms,
            row.current_sample_cv,
            row.best_ns_per_input,
            row.current_ns_per_input,
            row.ratio_to_best,
            row.status
        ));
    }
    out
}

fn render_self_compare_markdown(rows: &[SelfCompareRow], self_regression_warn: f64) -> String {
    let mut out = String::new();
    out.push_str("# Rust vs Self Best Benchmarks\n\n");
    out.push_str(&format!(
        "Self-regression threshold: current Rust slower than historical best by more than `{:.2}x`.\n\n",
        self_regression_warn
    ));
    out.push_str(
        "| indicator | mode | input_len | best ns/input | current ns/input | ratio to best | best sample ms (min/med/max/stddev/cv) | current sample ms (min/med/max/stddev/cv) | status |\n",
    );
    out.push_str("| --- | --- | ---: | ---: | ---: | ---: | --- | --- | --- |\n");
    for row in rows {
        out.push_str(&format!(
            "| {} | {} | {} | {:.2} | {:.2} | {:.3} | {:.3}/{:.3}/{:.3}/{:.3}/{:.3} | {:.3}/{:.3}/{:.3}/{:.3}/{:.3} | {} |\n",
            row.indicator,
            row.mode,
            row.input_len,
            row.best_ns_per_input,
            row.current_ns_per_input,
            row.ratio_to_best,
            row.best_sample_min_ms,
            row.best_sample_median_ms,
            row.best_sample_max_ms,
            row.best_sample_stddev_ms,
            row.best_sample_cv,
            row.current_sample_min_ms,
            row.current_sample_median_ms,
            row.current_sample_max_ms,
            row.current_sample_stddev_ms,
            row.current_sample_cv,
            row.status
        ));
    }
    out
}

fn render_kernel_split_tsv(rows: &[KernelSplitRow]) -> String {
    let mut out = String::from(
        "indicator\tinput_len\trun_in_place_ns_per_input\tkernel_ns_per_input\tratio_to_kernel\trun_in_place_sample_cv\tkernel_sample_cv\tstatus\n",
    );
    for row in rows {
        out.push_str(&format!(
            "{}\t{}\t{:.2}\t{:.2}\t{:.3}\t{:.3}\t{:.3}\t{}\n",
            row.indicator,
            row.input_len,
            row.run_in_place_ns_per_input,
            row.kernel_ns_per_input,
            row.ratio_to_kernel,
            row.run_in_place_sample_cv,
            row.kernel_sample_cv,
            row.status
        ));
    }
    out
}

fn render_kernel_split_markdown(rows: &[KernelSplitRow]) -> String {
    let mut out = String::new();
    out.push_str("# Rust In-Place vs Kernel Benchmarks\n\n");
    out.push_str("| indicator | input_len | run_in_place ns/input | kernel ns/input | ratio to kernel | run_in_place cv | kernel cv | status |\n");
    out.push_str("| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |\n");
    for row in rows {
        out.push_str(&format!(
            "| {} | {} | {:.2} | {:.2} | {:.3} | {:.3} | {:.3} | {} |\n",
            row.indicator,
            row.input_len,
            row.run_in_place_ns_per_input,
            row.kernel_ns_per_input,
            row.ratio_to_kernel,
            row.run_in_place_sample_cv,
            row.kernel_sample_cv,
            row.status
        ));
    }
    out
}
