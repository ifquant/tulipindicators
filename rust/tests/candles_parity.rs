#[path = "support/candle_cases.rs"]
mod candle_cases;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use candle_cases::parse_candle_cases;
use tulipindicators::{
    all_candles, candle_count, find_candle, run_candle_named, run_candle_pattern, run_candles,
    CandleConfig, CandleSet, TC_ALL, TC_SHOOTING_STAR,
};

const CANDLES_FIXTURE: &str = "c/tests/candles.txt";
const CANDLE_CONFIGS: &[CandleConfig] = &[
    CandleConfig {
        period: 10,
        body_none: 0.05,
        body_short: 0.5,
        body_long: 1.4,
        wick_none: 0.05,
        wick_long: 0.6,
        near: 0.3,
    },
    CandleConfig {
        period: 7,
        body_none: 0.05,
        body_short: 0.5,
        body_long: 1.4,
        wick_none: 0.05,
        wick_long: 0.6,
        near: 0.3,
    },
    CandleConfig {
        period: 12,
        body_none: 0.05,
        body_short: 0.5,
        body_long: 1.4,
        wick_none: 0.05,
        wick_long: 0.6,
        near: 0.3,
    },
    CandleConfig {
        period: 10,
        body_none: 0.08,
        body_short: 0.55,
        body_long: 1.25,
        wick_none: 0.08,
        wick_long: 0.7,
        near: 0.2,
    },
    CandleConfig {
        period: 10,
        body_none: 0.03,
        body_short: 0.4,
        body_long: 1.6,
        wick_none: 0.03,
        wick_long: 0.5,
        near: 0.4,
    },
];

#[derive(Debug, Clone, PartialEq, Eq)]
struct CandleMetadata {
    name: String,
    full_name: String,
    pattern: CandleSet,
}

#[test]
fn rust_candles_match_named_fixture_expectations() {
    for case in parse_candle_cases(CANDLES_FIXTURE) {
        let inputs = case.inputs.each_ref().map(Vec::as_slice);
        let result = run_candles(TC_ALL, &inputs, &CandleConfig::default())
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
fn rust_candle_metadata_matches_c() {
    let oracle = ensure_candle_oracle();
    let c_metadata = read_c_metadata(&oracle);
    let rust_metadata: Vec<CandleMetadata> = all_candles()
        .iter()
        .map(|info| CandleMetadata {
            name: info.name.to_string(),
            full_name: info.full_name.to_string(),
            pattern: info.pattern,
        })
        .collect();

    assert_eq!(
        candle_count(),
        all_candles().len(),
        "rust candle count drifted"
    );
    assert_eq!(
        all_candles().len(),
        c_metadata.len(),
        "rust/c candle counts differ"
    );
    assert_eq!(rust_metadata, c_metadata, "rust/c candle metadata drifted");
    assert_eq!(
        TC_ALL.count_ones() as usize,
        all_candles().len(),
        "TC_ALL bitset should cover every candle pattern exactly once"
    );

    for info in all_candles() {
        let found = find_candle(info.name).expect("every rust candle name should round-trip");
        assert_eq!(found, info, "rust candle lookup drifted for {}", info.name);
    }
}

#[test]
fn rust_candles_match_c_candle_engine() {
    let oracle = ensure_candle_oracle();

    for case in parse_candle_cases(CANDLES_FIXTURE) {
        let input_len = case.inputs[0].len();
        let inputs = case.inputs.each_ref().map(Vec::as_slice);
        let result = run_candles(TC_ALL, &inputs, &CandleConfig::default())
            .expect("rust candle engine should run");
        let rust_sets: Vec<CandleSet> = (0..input_len).map(|index| result.at(index)).collect();
        let c_sets = run_c_oracle(&oracle, TC_ALL, &CandleConfig::default(), &case.inputs);

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

#[test]
fn rust_single_pattern_helpers_match_full_engine_and_c() {
    for case in parse_candle_cases(CANDLES_FIXTURE) {
        let inputs = case.inputs.each_ref().map(Vec::as_slice);
        let full =
            run_candles(TC_ALL, &inputs, &CandleConfig::default()).expect("full engine should run");
        for expectation in &case.expectations {
            let by_name = run_candle_named(&expectation.name, &inputs, &CandleConfig::default())
                .expect("named candle helper should run");
            let by_pattern =
                run_candle_pattern(expectation.pattern, &inputs, &CandleConfig::default())
                    .expect("pattern candle helper should run");

            let rust_name_sets: Vec<CandleSet> = (0..case.inputs[0].len())
                .map(|index| by_name.at(index))
                .collect();
            let rust_pattern_sets: Vec<CandleSet> = (0..case.inputs[0].len())
                .map(|index| by_pattern.at(index))
                .collect();
            let full_sets: Vec<CandleSet> = (0..case.inputs[0].len())
                .map(|index| full.at(index) & expectation.pattern)
                .collect();

            assert_eq!(
                rust_name_sets, rust_pattern_sets,
                "single-pattern helpers disagreed for {}",
                expectation.name
            );
            assert_eq!(
                rust_pattern_sets, full_sets,
                "single-pattern helper mismatch vs full engine for {}",
                expectation.name
            );
        }
    }
}

#[test]
fn rust_candles_match_c_for_multiple_configs() {
    let oracle = ensure_candle_oracle();

    for config in CANDLE_CONFIGS {
        for case in parse_candle_cases(CANDLES_FIXTURE) {
            let input_len = case.inputs[0].len();
            let inputs = case.inputs.each_ref().map(Vec::as_slice);
            let result =
                run_candles(TC_ALL, &inputs, config).expect("rust candle engine should run");
            let rust_sets: Vec<CandleSet> = (0..input_len).map(|index| result.at(index)).collect();
            let c_sets = run_c_oracle(&oracle, TC_ALL, config, &case.inputs);
            assert_eq!(
                rust_sets, c_sets,
                "candle config parity mismatch for {:?}",
                config
            );
        }
    }
}

#[test]
fn rust_detects_shooting_star_in_targeted_case() {
    let oracle = ensure_candle_oracle();
    let config = CandleConfig {
        period: 3,
        ..CandleConfig::default()
    };
    let inputs = [
        vec![10.0, 8.0, 8.0, 12.0],
        vec![12.5, 10.5, 10.5, 14.2],
        vec![9.5, 7.5, 7.5, 12.0],
        vec![12.0, 10.0, 10.0, 12.1],
    ];
    let slices = inputs.each_ref().map(Vec::as_slice);
    let full = run_candles(TC_ALL, &slices, &config).expect("shooting star case should run");
    let single = run_candle_pattern(TC_SHOOTING_STAR, &slices, &config)
        .expect("single shooting star path should run");
    let c_sets: Vec<CandleSet> = run_c_oracle(&oracle, TC_ALL, &config, &inputs)
        .into_iter()
        .map(|set| set & TC_SHOOTING_STAR)
        .collect();

    assert_eq!(
        full.at(3) & TC_SHOOTING_STAR,
        TC_SHOOTING_STAR,
        "targeted case should produce a shooting star hit"
    );
    assert_eq!(
        (0..inputs[0].len())
            .map(|index| single.at(index))
            .collect::<Vec<_>>(),
        (0..inputs[0].len())
            .map(|index| full.at(index) & TC_SHOOTING_STAR)
            .collect::<Vec<_>>(),
        "single-pattern shooting star path should match the full-engine projection"
    );
    assert_eq!(
        (0..inputs[0].len())
            .map(|index| full.at(index) & TC_SHOOTING_STAR)
            .collect::<Vec<_>>(),
        c_sets,
        "targeted shooting star case should stay aligned with the stable C full-engine path"
    );
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

fn run_c_oracle(
    oracle: &Path,
    patterns: CandleSet,
    config: &CandleConfig,
    inputs: &[Vec<f64>; 4],
) -> Vec<CandleSet> {
    let input_len = inputs[0].len();
    let mut payload = format!(
        "{} {} {} {:.17} {:.17} {:.17} {:.17} {:.17} {:.17}\n",
        patterns,
        input_len,
        config.period,
        config.body_none,
        config.body_short,
        config.body_long,
        config.wick_none,
        config.wick_long,
        config.near,
    );
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

fn read_c_metadata(oracle: &Path) -> Vec<CandleMetadata> {
    let output = Command::new(oracle)
        .arg("--metadata")
        .output()
        .expect("failed to run candle metadata oracle");

    assert!(
        output.status.success(),
        "candle metadata oracle failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("oracle output should be utf8");
    let mut lines = stdout.lines();
    let count: usize = lines
        .next()
        .expect("metadata oracle should print a count")
        .trim()
        .parse()
        .expect("metadata count should be numeric");

    let metadata: Vec<CandleMetadata> = lines
        .map(|line| {
            let mut parts = line.split('\t');
            CandleMetadata {
                name: parts
                    .next()
                    .expect("metadata line should include name")
                    .to_string(),
                full_name: parts
                    .next()
                    .expect("metadata line should include full name")
                    .to_string(),
                pattern: parts
                    .next()
                    .expect("metadata line should include pattern")
                    .parse()
                    .expect("pattern should be numeric"),
            }
        })
        .collect();

    assert_eq!(metadata.len(), count, "metadata oracle count mismatch");
    metadata
}
