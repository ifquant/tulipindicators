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
const DEFAULT_TARGET_MS: u64 = 300;
const DEFAULT_CALIBRATION_MS: u64 = 20;
const DEFAULT_REPEATS: usize = 3;

#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    pub sizes: Vec<usize>,
    pub stream_chunk_size: usize,
    pub min_iterations: usize,
    pub target_duration: Duration,
    pub calibration_duration: Duration,
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
            calibration_duration: Duration::from_millis(DEFAULT_CALIBRATION_MS),
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

        if let Ok(raw) = std::env::var("TI_BENCH_CALIBRATION_MS") {
            if let Ok(value) = raw.parse::<u64>() {
                if value > 0 {
                    config.calibration_duration = Duration::from_millis(value);
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
    pub calibration_runs: usize,
    pub calibration_ms: f64,
    pub iterations: usize,
    pub total_outputs: usize,
    pub elapsed: Duration,
    pub sample_min: Duration,
    pub sample_max: Duration,
    pub sample_stddev_ms: f64,
    pub sample_cv: f64,
    pub ns_per_input: f64,
}

#[derive(Debug, Clone)]
pub struct KernelProbeResult {
    pub indicator: &'static str,
    pub input_len: usize,
    pub calibration_runs: usize,
    pub calibration_ms: f64,
    pub iterations: usize,
    pub sample_min: Duration,
    pub elapsed: Duration,
    pub sample_max: Duration,
    pub sample_stddev_ms: f64,
    pub sample_cv: f64,
    pub ns_per_input: f64,
}

#[derive(Debug, Clone, Copy)]
struct CalibrationResult {
    runs: usize,
    elapsed: Duration,
    iterations: usize,
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

pub fn run_named_kernel_probes(
    config: &BenchmarkConfig,
    names: &[String],
) -> Result<Vec<KernelProbeResult>, IndicatorError> {
    let requested: std::collections::BTreeSet<&str> = names.iter().map(String::as_str).collect();
    let mut results = Vec::new();

    for &size in &config.sizes {
        for name in ["ema", "wilders", "zlema", "macd", "rsi"] {
            if !requested.contains(name) {
                continue;
            }
            let scenario = BenchmarkScenario::new(
                registry::find(name).expect("kernel probe indicator should exist"),
                size,
            )?;
            let result = match name {
                "ema" => run_ema_kernel_probe(&scenario, config)?,
                "wilders" => run_wilders_kernel_probe(&scenario, config)?,
                "zlema" => run_zlema_kernel_probe(&scenario, config)?,
                "macd" => run_macd_kernel_probe(&scenario, config)?,
                "rsi" => run_rsi_kernel_probe(&scenario, config)?,
                _ => continue,
            };
            results.push(result);
        }
    }

    Ok(results)
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
    out.push_str("| indicator | mode | input_len | calibration_runs | calibration_ms | iterations | outputs | sample_min_ms | sample_median_ms | sample_max_ms | sample_stddev_ms | sample_cv | ns/input |\n");
    out.push_str("| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |\n");

    for result in results {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {} | {:.3} | {} | {} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.2} |",
            result.indicator,
            result.mode.as_str(),
            result.input_len,
            result.calibration_runs,
            result.calibration_ms,
            result.iterations,
            result.total_outputs,
            result.sample_min.as_secs_f64() * 1000.0,
            result.elapsed.as_secs_f64() * 1000.0,
            result.sample_max.as_secs_f64() * 1000.0,
            result.sample_stddev_ms,
            result.sample_cv,
            result.ns_per_input,
        );
    }

    out
}

pub fn render_tsv(results: &[BenchmarkResult]) -> String {
    let mut out = String::from(
        "indicator\tmode\tinput_len\tcalibration_runs\tcalibration_ms\titerations\toutputs\tsample_min_ms\tsample_median_ms\tsample_max_ms\tsample_stddev_ms\tsample_cv\tns_per_input\n",
    );

    for result in results {
        let _ = writeln!(
            out,
            "{}\t{}\t{}\t{}\t{:.3}\t{}\t{}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.2}",
            result.indicator,
            result.mode.as_str(),
            result.input_len,
            result.calibration_runs,
            result.calibration_ms,
            result.iterations,
            result.total_outputs,
            result.sample_min.as_secs_f64() * 1000.0,
            result.elapsed.as_secs_f64() * 1000.0,
            result.sample_max.as_secs_f64() * 1000.0,
            result.sample_stddev_ms,
            result.sample_cv,
            result.ns_per_input,
        );
    }

    out
}

pub fn render_kernel_probe_tsv(results: &[KernelProbeResult]) -> String {
    let mut out = String::from(
        "indicator\tinput_len\tcalibration_runs\tcalibration_ms\titerations\tsample_min_ms\tsample_median_ms\tsample_max_ms\tsample_stddev_ms\tsample_cv\tns_per_input\n",
    );

    for result in results {
        let _ = writeln!(
            out,
            "{}\t{}\t{}\t{:.3}\t{}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.3}\t{:.2}",
            result.indicator,
            result.input_len,
            result.calibration_runs,
            result.calibration_ms,
            result.iterations,
            result.sample_min.as_secs_f64() * 1000.0,
            result.elapsed.as_secs_f64() * 1000.0,
            result.sample_max.as_secs_f64() * 1000.0,
            result.sample_stddev_ms,
            result.sample_cv,
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

fn finish_kernel_probe(
    indicator: &'static str,
    input_len: usize,
    calibration: CalibrationResult,
    iterations: usize,
    samples: Vec<Duration>,
) -> KernelProbeResult {
    let elapsed = median_duration(samples.clone());
    let sample_min = samples.iter().copied().min().unwrap_or(elapsed);
    let sample_max = samples.iter().copied().max().unwrap_or(elapsed);
    let sample_stddev_ms = stddev_duration_ms(&samples);
    let sample_cv = coefficient_of_variation(sample_stddev_ms, elapsed.as_secs_f64() * 1000.0);

    KernelProbeResult {
        indicator,
        input_len,
        calibration_runs: calibration.runs,
        calibration_ms: calibration.elapsed.as_secs_f64() * 1000.0,
        iterations,
        sample_min,
        elapsed,
        sample_max,
        sample_stddev_ms,
        sample_cv,
        ns_per_input: elapsed.as_secs_f64() * 1_000_000_000.0 / (input_len * iterations) as f64,
    }
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
    let calibration = calibrate_iterations(config, |runs| {
        for _ in 0..runs {
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
        Ok(())
    })?;
    let iterations = calibration.iterations;

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
    let elapsed = median_duration(samples.clone());
    let sample_min = samples.iter().copied().min().unwrap_or(elapsed);
    let sample_max = samples.iter().copied().max().unwrap_or(elapsed);
    let sample_stddev_ms = stddev_duration_ms(&samples);
    let sample_cv = coefficient_of_variation(sample_stddev_ms, elapsed.as_secs_f64() * 1000.0);

    Ok(BenchmarkResult {
        indicator: metadata.name,
        mode: BenchmarkMode::Batch,
        input_len: scenario.inputs[0].len(),
        calibration_runs: calibration.runs,
        calibration_ms: calibration.elapsed.as_secs_f64() * 1000.0,
        iterations,
        total_outputs,
        elapsed,
        sample_min,
        sample_max,
        sample_stddev_ms,
        sample_cv,
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
    let calibration = calibrate_iterations(config, |runs| {
        for _ in 0..runs {
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
        Ok(())
    })?;
    let iterations = calibration.iterations;

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
    let elapsed = median_duration(samples.clone());
    let sample_min = samples.iter().copied().min().unwrap_or(elapsed);
    let sample_max = samples.iter().copied().max().unwrap_or(elapsed);
    let sample_stddev_ms = stddev_duration_ms(&samples);
    let sample_cv = coefficient_of_variation(sample_stddev_ms, elapsed.as_secs_f64() * 1000.0);

    Ok(BenchmarkResult {
        indicator: scenario.indicator.metadata().name,
        mode: BenchmarkMode::Stream,
        input_len: scenario.inputs[0].len(),
        calibration_runs: calibration.runs,
        calibration_ms: calibration.elapsed.as_secs_f64() * 1000.0,
        iterations,
        total_outputs,
        elapsed,
        sample_min,
        sample_max,
        sample_stddev_ms,
        sample_cv,
        ns_per_input: elapsed.as_secs_f64() * 1_000_000_000.0
            / (scenario.inputs[0].len() * iterations) as f64,
    })
}

fn run_ema_kernel_probe(
    scenario: &BenchmarkScenario<'_>,
    config: &BenchmarkConfig,
) -> Result<KernelProbeResult, IndicatorError> {
    let input = &scenario.inputs[0];
    let period = scenario.options[0] as usize;
    let multiplier = 2.0 / (period as Real + 1.0);
    let mut output = vec![0.0; input.len()];
    run_ema_kernel(input, multiplier, &mut output);

    let calibration = calibrate_iterations(config, |runs| {
        for _ in 0..runs {
            run_ema_kernel(input, multiplier, &mut output);
            black_box(output[input.len().saturating_sub(1)]);
        }
        Ok(())
    })?;
    let iterations = calibration.iterations;
    let mut samples = Vec::with_capacity(config.repeats);
    for _ in 0..config.repeats {
        let start = Instant::now();
        for _ in 0..iterations {
            run_ema_kernel(input, multiplier, &mut output);
            black_box(output[input.len().saturating_sub(1)]);
        }
        samples.push(start.elapsed());
    }

    Ok(finish_kernel_probe(
        "ema",
        input.len(),
        calibration,
        iterations,
        samples,
    ))
}

fn run_wilders_kernel_probe(
    scenario: &BenchmarkScenario<'_>,
    config: &BenchmarkConfig,
) -> Result<KernelProbeResult, IndicatorError> {
    let input = &scenario.inputs[0];
    let period = scenario.options[0] as usize;
    let output_len = input.len().saturating_sub(period.saturating_sub(1));
    let mut output = vec![0.0; output_len];
    run_wilders_kernel(input, period, &mut output);

    let calibration = calibrate_iterations(config, |runs| {
        for _ in 0..runs {
            let produced = run_wilders_kernel(input, period, &mut output);
            black_box((produced, output[produced.saturating_sub(1)]));
        }
        Ok(())
    })?;
    let iterations = calibration.iterations;
    let mut samples = Vec::with_capacity(config.repeats);
    for _ in 0..config.repeats {
        let start = Instant::now();
        for _ in 0..iterations {
            let produced = run_wilders_kernel(input, period, &mut output);
            black_box((produced, output[produced.saturating_sub(1)]));
        }
        samples.push(start.elapsed());
    }

    Ok(finish_kernel_probe(
        "wilders",
        input.len(),
        calibration,
        iterations,
        samples,
    ))
}

fn run_zlema_kernel_probe(
    scenario: &BenchmarkScenario<'_>,
    config: &BenchmarkConfig,
) -> Result<KernelProbeResult, IndicatorError> {
    let input = &scenario.inputs[0];
    let period = scenario.options[0] as usize;
    let lag = (period - 1) / 2;
    let lookback = lag.saturating_sub(1);
    let mut output = vec![0.0; input.len().saturating_sub(lookback)];
    run_zlema_kernel(input, period, &mut output);

    let calibration = calibrate_iterations(config, |runs| {
        for _ in 0..runs {
            let produced = run_zlema_kernel(input, period, &mut output);
            black_box((produced, output[produced.saturating_sub(1)]));
        }
        Ok(())
    })?;
    let iterations = calibration.iterations;
    let mut samples = Vec::with_capacity(config.repeats);
    for _ in 0..config.repeats {
        let start = Instant::now();
        for _ in 0..iterations {
            let produced = run_zlema_kernel(input, period, &mut output);
            black_box((produced, output[produced.saturating_sub(1)]));
        }
        samples.push(start.elapsed());
    }

    Ok(finish_kernel_probe(
        "zlema",
        input.len(),
        calibration,
        iterations,
        samples,
    ))
}

fn run_macd_kernel_probe(
    scenario: &BenchmarkScenario<'_>,
    config: &BenchmarkConfig,
) -> Result<KernelProbeResult, IndicatorError> {
    let input = &scenario.inputs[0];
    let short_period = scenario.options[0] as usize;
    let long_period = scenario.options[1] as usize;
    let signal_period = scenario.options[2] as usize;
    let output_len = input.len().saturating_sub(long_period.saturating_sub(1));
    let mut macd = vec![0.0; output_len];
    let mut signal = vec![0.0; output_len];
    let mut hist = vec![0.0; output_len];
    run_macd_kernel(
        input,
        short_period,
        long_period,
        signal_period,
        &mut macd,
        &mut signal,
        &mut hist,
    );

    let calibration = calibrate_iterations(config, |runs| {
        for _ in 0..runs {
            let produced = run_macd_kernel(
                input,
                short_period,
                long_period,
                signal_period,
                &mut macd,
                &mut signal,
                &mut hist,
            );
            black_box((produced, hist[produced.saturating_sub(1)]));
        }
        Ok(())
    })?;
    let iterations = calibration.iterations;
    let mut samples = Vec::with_capacity(config.repeats);
    for _ in 0..config.repeats {
        let start = Instant::now();
        for _ in 0..iterations {
            let produced = run_macd_kernel(
                input,
                short_period,
                long_period,
                signal_period,
                &mut macd,
                &mut signal,
                &mut hist,
            );
            black_box((produced, hist[produced.saturating_sub(1)]));
        }
        samples.push(start.elapsed());
    }

    Ok(finish_kernel_probe(
        "macd",
        input.len(),
        calibration,
        iterations,
        samples,
    ))
}

fn run_rsi_kernel_probe(
    scenario: &BenchmarkScenario<'_>,
    config: &BenchmarkConfig,
) -> Result<KernelProbeResult, IndicatorError> {
    let input = &scenario.inputs[0];
    let period = scenario.options[0] as usize;
    let output_len = input.len().saturating_sub(period);
    let mut output = vec![0.0; output_len];
    run_rsi_kernel(input, period, &mut output);

    let calibration = calibrate_iterations(config, |runs| {
        for _ in 0..runs {
            let produced = run_rsi_kernel(input, period, &mut output);
            black_box((produced, output[produced.saturating_sub(1)]));
        }
        Ok(())
    })?;
    let iterations = calibration.iterations;
    let mut samples = Vec::with_capacity(config.repeats);
    for _ in 0..config.repeats {
        let start = Instant::now();
        for _ in 0..iterations {
            let produced = run_rsi_kernel(input, period, &mut output);
            black_box((produced, output[produced.saturating_sub(1)]));
        }
        samples.push(start.elapsed());
    }

    Ok(finish_kernel_probe(
        "rsi",
        input.len(),
        calibration,
        iterations,
        samples,
    ))
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

fn calibrate_iterations<F>(
    config: &BenchmarkConfig,
    mut run: F,
) -> Result<CalibrationResult, IndicatorError>
where
    F: FnMut(usize) -> Result<(), IndicatorError>,
{
    let mut runs = 1usize;
    let max_runs = 1usize << 20;

    let elapsed = loop {
        let start = Instant::now();
        run(runs)?;
        let elapsed = start.elapsed();
        if elapsed >= config.calibration_duration || runs >= max_runs {
            break elapsed;
        }
        runs = runs.saturating_mul(2).min(max_runs);
    };

    let elapsed_ns = elapsed.as_nanos().max(1);
    let target_ns = config.target_duration.as_nanos();
    let estimated = target_ns.saturating_mul(runs as u128).div_ceil(elapsed_ns) as usize;
    Ok(CalibrationResult {
        runs,
        elapsed,
        iterations: estimated.max(config.min_iterations),
    })
}

fn median_duration(mut samples: Vec<Duration>) -> Duration {
    samples.sort_unstable();
    samples[samples.len() / 2]
}

fn stddev_duration_ms(samples: &[Duration]) -> f64 {
    if samples.len() <= 1 {
        return 0.0;
    }
    let mean = samples
        .iter()
        .map(|sample| sample.as_secs_f64() * 1000.0)
        .sum::<f64>()
        / samples.len() as f64;
    let variance = samples
        .iter()
        .map(|sample| {
            let value = sample.as_secs_f64() * 1000.0;
            let delta = value - mean;
            delta * delta
        })
        .sum::<f64>()
        / samples.len() as f64;
    variance.sqrt()
}

fn coefficient_of_variation(stddev_ms: f64, median_ms: f64) -> f64 {
    if median_ms == 0.0 {
        0.0
    } else {
        stddev_ms / median_ms
    }
}

fn run_ema_kernel(input: &[Real], multiplier: Real, output: &mut [Real]) -> usize {
    if input.is_empty() {
        return 0;
    }

    let mut value = input[0];
    output[0] = value;
    for (dst, &sample) in output.iter_mut().skip(1).zip(input.iter().skip(1)) {
        value = (sample - value) * multiplier + value;
        *dst = value;
    }

    input.len()
}

fn run_wilders_kernel(input: &[Real], period: usize, output: &mut [Real]) -> usize {
    if input.len() < period {
        return 0;
    }

    let per = 1.0 / period as Real;
    let mut sum = 0.0;
    for &sample in &input[..period] {
        sum += sample;
    }

    let mut value = sum / period as Real;
    output[0] = value;
    let mut out_index = 1usize;
    for &sample in &input[period..] {
        value = (sample - value) * per + value;
        output[out_index] = value;
        out_index += 1;
    }

    out_index
}

fn run_zlema_kernel(input: &[Real], period: usize, output: &mut [Real]) -> usize {
    let lag = (period - 1) / 2;
    let lookback = lag.saturating_sub(1);
    if input.len() <= lookback {
        return 0;
    }

    let per = 2.0 / (period as Real + 1.0);
    if lag == 0 {
        return run_ema_kernel(input, per, output);
    }

    let mut value = input[lag - 1];
    output[0] = value;
    let mut out_index = 1usize;
    for index in lag..input.len() {
        let current = input[index];
        let lagged = input[index - lag];
        value = ((current + (current - lagged)) - value) * per + value;
        output[out_index] = value;
        out_index += 1;
    }

    out_index
}

fn run_macd_kernel(
    input: &[Real],
    short_period: usize,
    long_period: usize,
    signal_period: usize,
    macd: &mut [Real],
    signal: &mut [Real],
    hist: &mut [Real],
) -> usize {
    let lookback = long_period - 1;
    if input.len() <= lookback {
        return 0;
    }

    let (short_per, long_per) = if short_period == 12 && long_period == 26 {
        (0.15, 0.075)
    } else {
        (
            2.0 / (short_period as Real + 1.0),
            2.0 / (long_period as Real + 1.0),
        )
    };
    let signal_per = 2.0 / (signal_period as Real + 1.0);

    let mut short_ema = input[0];
    let mut long_ema = input[0];
    let mut signal_ema = 0.0;
    let mut out_index = 0usize;

    for (index, &sample) in input.iter().enumerate().skip(1) {
        short_ema = (sample - short_ema) * short_per + short_ema;
        long_ema = (sample - long_ema) * long_per + long_ema;
        let macd_value = short_ema - long_ema;

        if index == long_period - 1 {
            signal_ema = macd_value;
        }

        if index >= long_period - 1 {
            signal_ema = (macd_value - signal_ema) * signal_per + signal_ema;
            macd[out_index] = macd_value;
            signal[out_index] = signal_ema;
            hist[out_index] = macd_value - signal_ema;
            out_index += 1;
        }
    }

    out_index
}

fn run_rsi_kernel(input: &[Real], period: usize, output: &mut [Real]) -> usize {
    if input.len() <= period {
        return 0;
    }

    let per = 1.0 / period as Real;
    let mut smooth_up = 0.0;
    let mut smooth_down = 0.0;

    for index in 1..=period {
        let delta = input[index] - input[index - 1];
        if delta > 0.0 {
            smooth_up += delta;
        } else {
            smooth_down += -delta;
        }
    }

    smooth_up /= period as Real;
    smooth_down /= period as Real;
    output[0] = run_rsi_value(smooth_up, smooth_down);

    let mut out_index = 1usize;
    for index in (period + 1)..input.len() {
        let delta = input[index] - input[index - 1];
        let upward = delta.max(0.0);
        let downward = (-delta).max(0.0);

        smooth_up = (upward - smooth_up) * per + smooth_up;
        smooth_down = (downward - smooth_down) * per + smooth_down;
        output[out_index] = run_rsi_value(smooth_up, smooth_down);
        out_index += 1;
    }

    out_index
}

fn run_rsi_value(smooth_up: Real, smooth_down: Real) -> Real {
    let total = smooth_up + smooth_down;
    if total == 0.0 {
        0.0
    } else {
        100.0 * (smooth_up / total)
    }
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
