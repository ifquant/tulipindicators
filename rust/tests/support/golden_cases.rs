use std::fs;
use std::path::Path;

use tulipindicators::{find, Real};

pub const TEST_FILES: &[&str] = &[
    "c/tests/atoz.txt",
    "c/tests/untest.txt",
    "c/tests/extra.txt",
];

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct GoldenCase {
    pub name: String,
    pub options: Vec<Real>,
    pub inputs: Vec<Vec<Real>>,
    pub outputs: Vec<Vec<Real>>,
}

pub fn parse_cases(path: &str) -> Vec<GoldenCase> {
    let content = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path));
    let lines: Vec<String> = content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToOwned::to_owned)
        .collect();

    let mut cases = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let header = &lines[index];
        if header.starts_with('{') {
            panic!(
                "unexpected data line without header in {}: {}",
                path, header
            );
        }

        index += 1;

        let mut tokens = header.split_whitespace();
        let name = tokens.next().expect("header should have a name");
        let options: Vec<Real> = tokens.map(parse_number).collect();

        if let Some(indicator) = find(name) {
            let metadata = indicator.metadata();
            let mut inputs = Vec::with_capacity(metadata.input_names.len());
            let mut outputs = Vec::with_capacity(metadata.output_names.len());

            for _ in metadata.input_names {
                inputs.push(parse_array(&lines[index], path, name));
                index += 1;
            }

            for _ in metadata.output_names {
                outputs.push(parse_array(&lines[index], path, name));
                index += 1;
            }

            if let Ok(lookback) = indicator.lookback(&options) {
                let expected_len = inputs.first().map_or(0, Vec::len).saturating_sub(lookback);
                if expected_len > 0
                    && outputs
                        .iter()
                        .all(|output| output.len() == expected_len + 1)
                {
                    for output in &mut outputs {
                        output.truncate(expected_len);
                    }
                }
            }

            cases.push(GoldenCase {
                name: name.to_string(),
                options,
                inputs,
                outputs,
            });
        } else {
            while index < lines.len() && lines[index].starts_with('{') {
                index += 1;
            }
        }
    }

    cases
}

fn parse_array(line: &str, path: &str, indicator: &str) -> Vec<Real> {
    let trimmed = line.trim().trim_end_matches(';');
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        panic!(
            "{} {} expected array line, got {}",
            path, indicator, trimmed
        );
    }

    let body = &trimmed[1..trimmed.len() - 1];
    if body.trim().is_empty() {
        return Vec::new();
    }

    body.split(',').map(parse_number).collect()
}

fn parse_number(raw: &str) -> Real {
    raw.trim()
        .parse()
        .unwrap_or_else(|error| panic!("failed to parse number `{raw}`: {error}"))
}
