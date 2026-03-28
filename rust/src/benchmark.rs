use crate::core::error::IndicatorError;
use crate::core::indicator::{output_len_for_input, Indicator};
use crate::core::types::Real;
use crate::registry;
use std::fmt::Write as _;
use std::fs;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const DEFAULT_SIZES: &[usize] = &[256, 4096, 65_536, 262_144];
const DEFAULT_STREAM_CHUNK: usize = 1024;
const DEFAULT_MIN_ITERATIONS: usize = 16;
const DEFAULT_TARGET_MS: u64 = 1_000;
const DEFAULT_REPEATS: usize = 3;

#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub sizes: Vec<usize>,
    pub stream_chunk_size: usize,
    pub min_iterations: usize,
    pub target_duration: Duration,
    pub repeats: usize,
    pub output_dir: PathBuf,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            sizes: DEFAULT_SIZES.to_vec(),
            stream_chunk_size: DEFAULT_STREAM_CHUNK,
            min_iterations: DEFAULT_MIN_ITERATIONS,
            target_duration: Duration::from_millis(DEFAULT_TARGET_MS),
            repeats: DEFAULT_REPEATS,
            output_dir: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("target")
                .join("indicator-bench"),
        }
    }
}

impl BenchmarkConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(raw) = std::env::var("TI_BENCH_SIZES") {
            let sizes = parse_csv_usize(&raw);
            if !sizes.is_empty() {
                config.sizes = sizes;
            }
        }

        if let Ok(raw) = std::env::var("TI_BENCH_STREAM_CHUNK") {
            if let Ok(value) = raw.parse::<usize>() {
                if value > 0 {
                    config.stream_chunk_size = value;
                }
            }
        }

        if let Ok(raw) = std::env::var("TI_BENCH_MIN_ITERATIONS") {
            if let Ok(value) = raw.parse::<usize>() {
                if value > 0 {
                    config.min_iterations = value;
                }
            }
        }

        if let Ok(raw) = std::env::var("TI_BENCH_TARGET_MS") {
            if let Ok(value) = raw.parse::<u64>() {
                if value > 0 {
                    config.target_duration = Duration::from_millis(value);
                }
            }
        }

        if let Ok(raw) = std::env::var("TI_BENCH_REPEATS") {
            if let Ok(value) = raw.parse::<usize>() {
                if value > 0 {
                    config.repeats = value;
                }
            }
        }

        config
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BenchmarkMode {
    Batch,
    Stream,
}

impl BenchmarkMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Batch => "batch",
            Self::Stream => "stream",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BenchmarkResult {
    pub indicator: &'static str,
    pub mode: BenchmarkMode,
    pub input_len: usize,
    pub iterations: usize,
    pub total_outputs: usize,
    pub elapsed: Duration,
    pub ns_per_input: f64,
}

pub fn run_registry_benchmarks(
    config: &BenchmarkConfig,
) -> Result<Vec<BenchmarkResult>, IndicatorError> {
    run_indicator_benchmarks(config, registry::all())
}

pub fn run_named_benchmarks(
    config: &BenchmarkConfig,
    names: &[String],
) -> Result<Vec<BenchmarkResult>, IndicatorError> {
    let mut indicators = Vec::with_capacity(names.len());
    for name in names {
        let indicator = registry::find(name).ok_or(IndicatorError::InvalidOption {
            indicator: "benchmark",
            option: "indicator",
            value: 0.0,
            reason: "unknown indicator name in benchmark filter",
        })?;
        indicators.push(indicator);
    }
    run_indicator_benchmarks(config, indicators)
}

fn run_indicator_benchmarks<'a>(
    config: &BenchmarkConfig,
    indicators: impl IntoIterator<Item = &'a dyn Indicator>,
) -> Result<Vec<BenchmarkResult>, IndicatorError> {
    let mut results = Vec::new();

    for indicator in indicators {
        for &size in &config.sizes {
            let scenario = BenchmarkScenario::new(indicator, size)?;
            results.push(run_batch_benchmark(&scenario, config)?);

            if indicator.create_stream(&scenario.options)?.is_some() {
                results.push(run_stream_benchmark(&scenario, config)?);
            }
        }
    }

    Ok(results)
}

pub fn write_report(
    config: &BenchmarkConfig,
    results: &[BenchmarkResult],
) -> std::io::Result<(PathBuf, PathBuf)> {
    fs::create_dir_all(&config.output_dir)?;

    let markdown_path = config.output_dir.join("latest.md");
    let tsv_path = config.output_dir.join("latest.tsv");

    fs::write(&markdown_path, render_markdown(results))?;
    fs::write(&tsv_path, render_tsv(results))?;

    Ok((markdown_path, tsv_path))
}

pub fn render_markdown(results: &[BenchmarkResult]) -> String {
    let mut out = String::new();
    out.push_str("# Indicator Benchmarks\n\n");
    out.push_str("| indicator | mode | input_len | iterations | outputs | total_ms | ns/input |\n");
    out.push_str("| --- | --- | ---: | ---: | ---: | ---: | ---: |\n");

    for result in results {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {} | {:.3} | {:.2} |",
            result.indicator,
            result.mode.as_str(),
            result.input_len,
            result.iterations,
            result.total_outputs,
            result.elapsed.as_secs_f64() * 1000.0,
            result.ns_per_input,
        );
    }

    out
}

pub fn render_tsv(results: &[BenchmarkResult]) -> String {
    let mut out =
        String::from("indicator\tmode\tinput_len\titerations\toutputs\ttotal_ms\tns_per_input\n");

    for result in results {
        let _ = writeln!(
            out,
            "{}\t{}\t{}\t{}\t{}\t{:.3}\t{:.2}",
            result.indicator,
            result.mode.as_str(),
            result.input_len,
            result.iterations,
            result.total_outputs,
            result.elapsed.as_secs_f64() * 1000.0,
            result.ns_per_input,
        );
    }

    out
}

struct BenchmarkScenario<'a> {
    indicator: &'a dyn Indicator,
    options: Vec<Real>,
    inputs: Vec<Vec<Real>>,
}

impl<'a> BenchmarkScenario<'a> {
    fn new(indicator: &'a dyn Indicator, input_len: usize) -> Result<Self, IndicatorError> {
        let metadata = indicator.metadata();
        let options = build_options(metadata.option_names, input_len);
        let inputs = build_inputs(metadata.input_names, input_len);

        Ok(Self {
            indicator,
            options,
            inputs,
        })
    }
}

fn run_batch_benchmark(
    scenario: &BenchmarkScenario<'_>,
    config: &BenchmarkConfig,
) -> Result<BenchmarkResult, IndicatorError> {
    let inputs: Vec<&[Real]> = scenario.inputs.iter().map(Vec::as_slice).collect();
    let metadata = scenario.indicator.metadata();
    let lookback = scenario.indicator.lookback(&scenario.options)?;
    let output_len = output_len_for_input(scenario.inputs[0].len(), lookback);
    let total_outputs = output_len * metadata.output_names.len();
    let mut output_buffers = vec![vec![0.0; output_len]; metadata.output_names.len()];
    let iterations = choose_iterations(scenario.inputs[0].len(), config);
    {
        let mut outputs: Vec<&mut [Real]> =
            output_buffers.iter_mut().map(Vec::as_mut_slice).collect();
        let produced = scenario
            .indicator
            .run_in_place(&inputs, &scenario.options, &mut outputs)?;
        if produced != output_len {
            return Err(IndicatorError::InternalInvariant {
                indicator: metadata.name,
                reason: "batch benchmark warmup produced an unexpected output length",
            });
        }
    }

    let mut samples = Vec::with_capacity(config.repeats);
    for _ in 0..config.repeats {
        let start = Instant::now();
        for _ in 0..iterations {
            let mut outputs: Vec<&mut [Real]> =
                output_buffers.iter_mut().map(Vec::as_mut_slice).collect();
            let produced =
                scenario
                    .indicator
                    .run_in_place(&inputs, &scenario.options, &mut outputs)?;
            let sink = outputs
                .first()
                .and_then(|output| output.get(produced.saturating_sub(1)))
                .copied()
                .unwrap_or(0.0);
            black_box((produced, sink));
        }
        samples.push(start.elapsed());
    }
    let elapsed = median_duration(samples);

    Ok(BenchmarkResult {
        indicator: metadata.name,
        mode: BenchmarkMode::Batch,
        input_len: scenario.inputs[0].len(),
        iterations,
        total_outputs,
        elapsed,
        ns_per_input: elapsed.as_secs_f64() * 1_000_000_000.0
            / (scenario.inputs[0].len() * iterations) as f64,
    })
}

fn run_stream_benchmark(
    scenario: &BenchmarkScenario<'_>,
    config: &BenchmarkConfig,
) -> Result<BenchmarkResult, IndicatorError> {
    let mut sample_stream = scenario.indicator.create_stream(&scenario.options)?.ok_or(
        IndicatorError::MissingStreamSupport {
            indicator: scenario.indicator.metadata().name,
        },
    )?;
    let total_outputs = collect_stream_outputs(
        sample_stream.as_mut(),
        &scenario.inputs,
        config.stream_chunk_size,
    )?;
    let iterations = choose_iterations(scenario.inputs[0].len(), config);

    let mut samples = Vec::with_capacity(config.repeats);
    for _ in 0..config.repeats {
        let start = Instant::now();
        for _ in 0..iterations {
            let mut stream = scenario.indicator.create_stream(&scenario.options)?.ok_or(
                IndicatorError::MissingStreamSupport {
                    indicator: scenario.indicator.metadata().name,
                },
            )?;
            let outputs = collect_stream_outputs(
                stream.as_mut(),
                &scenario.inputs,
                config.stream_chunk_size,
            )?;
            black_box(outputs);
        }
        samples.push(start.elapsed());
    }
    let elapsed = median_duration(samples);

    Ok(BenchmarkResult {
        indicator: scenario.indicator.metadata().name,
        mode: BenchmarkMode::Stream,
        input_len: scenario.inputs[0].len(),
        iterations,
        total_outputs,
        elapsed,
        ns_per_input: elapsed.as_secs_f64() * 1_000_000_000.0
            / (scenario.inputs[0].len() * iterations) as f64,
    })
}

fn collect_stream_outputs(
    stream: &mut dyn crate::core::indicator::IndicatorStream,
    inputs: &[Vec<Real>],
    chunk_size: usize,
) -> Result<usize, IndicatorError> {
    let mut total_outputs = 0;
    let mut start = 0;
    let input_len = inputs.first().map_or(0, Vec::len);
    let output_count = stream.metadata().output_names.len();
    let mut output_buffers = vec![vec![0.0; chunk_size]; output_count];
    let mut chunk_inputs = Vec::with_capacity(inputs.len());
    while start < input_len {
        let end = (start + chunk_size).min(input_len);
        let current_chunk_len = end - start;
        chunk_inputs.clear();
        for series in inputs {
            chunk_inputs.push(&series[start..end]);
        }
        let mut outputs: Vec<&mut [Real]> = output_buffers
            .iter_mut()
            .map(|buffer| &mut buffer[..current_chunk_len])
            .collect();
        let produced = stream.feed_in_place(&chunk_inputs, &mut outputs)?;
        total_outputs += produced * output_count;
        start = end;
    }

    Ok(total_outputs)
}

fn choose_iterations(input_len: usize, config: &BenchmarkConfig) -> usize {
    let target_work = 1_000_000usize.saturating_mul(config.target_duration.as_millis() as usize)
        / DEFAULT_TARGET_MS as usize;
    let iterations = target_work / input_len.max(1);
    iterations.max(config.min_iterations)
}

fn median_duration(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    samples[samples.len() / 2]
}

fn build_options(option_names: &[&str], input_len: usize) -> Vec<Real> {
    option_names
        .iter()
        .map(|name| match *name {
            "period" => default_period(input_len),
            "short_period" => 5.0,
            "long_period" => 10.0,
            "signal_period" => 9.0,
            "roc1_period" => 10.0,
            "roc2_period" => 15.0,
            "roc3_period" => 20.0,
            "roc4_period" => 30.0,
            "k_period" => 5.0,
            "k_slowing_period" => 3.0,
            "d_period" => 3.0,
            "ma1_period" => 10.0,
            "ma2_period" => 10.0,
            "ma3_period" => 10.0,
            "ma4_period" => 15.0,
            "stddev" => 2.0,
            "acceleration_factor_step" => 0.02,
            "acceleration_factor_maximum" => 0.2,
            "alpha" => 0.2,
            "beta" => 0.2,
            "fastlimit" => 0.5,
            "slowlimit" => 0.05,
            "offset" => 0.85,
            "sigma" => 6.0,
            other if other.contains("period") => 7.0,
            _ => 2.0,
        })
        .collect()
}

fn build_inputs(input_names: &[&str], input_len: usize) -> Vec<Vec<Real>> {
    let close = build_close_series(input_len);
    let volume = build_volume_series(input_len);

    input_names
        .iter()
        .enumerate()
        .map(|(input_index, name)| match *name {
            "real" => build_real_series(input_index, &close),
            "close" => close.clone(),
            "open" => close
                .iter()
                .enumerate()
                .map(|(index, close_value)| close_value - ((index % 3) as Real - 1.0) * 0.13)
                .collect(),
            "high" => close
                .iter()
                .enumerate()
                .map(|(index, close_value)| close_value + 0.35 + ((index % 5) as Real * 0.03))
                .collect(),
            "low" => close
                .iter()
                .enumerate()
                .map(|(index, close_value)| close_value - 0.35 - ((index % 5) as Real * 0.03))
                .collect(),
            "volume" => volume.clone(),
            _ => close.clone(),
        })
        .collect()
}

fn build_real_series(input_index: usize, close: &[Real]) -> Vec<Real> {
    match input_index % 4 {
        0 => close.to_vec(),
        1 => close
            .iter()
            .enumerate()
            .map(|(index, close_value)| close_value - ((index % 3) as Real - 1.0) * 0.13)
            .collect(),
        2 => close
            .iter()
            .enumerate()
            .map(|(index, close_value)| close_value + 0.35 + ((index % 5) as Real * 0.03))
            .collect(),
        _ => close
            .iter()
            .enumerate()
            .map(|(index, close_value)| close_value - 0.35 - ((index % 5) as Real * 0.03))
            .collect(),
    }
}

fn build_close_series(input_len: usize) -> Vec<Real> {
    (0..input_len)
        .map(|index| {
            let trend = 100.0 + index as Real * 0.015;
            let wave = ((index as Real) / 17.0).sin() * 1.7;
            let ripple = ((index as Real) / 7.0).cos() * 0.6;
            trend + wave + ripple
        })
        .collect()
}

fn build_volume_series(input_len: usize) -> Vec<Real> {
    (0..input_len)
        .map(|index| 10_000.0 + (index % 250) as Real * 37.0 + ((index % 13) as Real * 11.0))
        .collect()
}

fn default_period(input_len: usize) -> Real {
    let candidate = (input_len / 32).clamp(5, 30);
    candidate as Real
}

fn parse_csv_usize(raw: &str) -> Vec<usize> {
    raw.split(',')
        .filter_map(|item| item.trim().parse::<usize>().ok())
        .filter(|value| *value > 0)
        .collect()
}

#[allow(dead_code)]
fn _exists(path: &Path) -> bool {
    path.exists()
}
