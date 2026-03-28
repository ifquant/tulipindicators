use std::fs;
use std::path::Path;

use tulipindicators::{find_candle, CandleSet, Real};

#[derive(Debug, Clone)]
pub struct CandleExpectation {
    pub name: String,
    pub pattern: CandleSet,
    pub negate: bool,
    pub indices: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct CandleCase {
    pub inputs: [Vec<Real>; 4],
    pub expectations: Vec<CandleExpectation>,
}

pub fn parse_candle_cases(path: &str) -> Vec<CandleCase> {
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
        assert_eq!(lines[index], "candle", "expected candle block header");
        index += 1;

        let inputs = [
            parse_array(&lines[index]),
            parse_array(&lines[index + 1]),
            parse_array(&lines[index + 2]),
            parse_array(&lines[index + 3]),
        ];
        index += 4;

        let mut expectations = Vec::new();
        while index < lines.len() && lines[index] != "candle" {
            let mut parts = lines[index].split_whitespace();
            let raw_name = parts.next().expect("expectation name should exist");
            let negate = raw_name.starts_with('!');
            let name = raw_name.trim_start_matches('!');
            let pattern = find_candle(name)
                .unwrap_or_else(|| panic!("unknown candle pattern `{name}` in fixture"))
                .pattern;
            let indices = parts
                .map(|value| {
                    value.parse::<usize>().unwrap_or_else(|error| {
                        panic!("failed to parse candle index `{value}`: {error}")
                    })
                })
                .collect();
            expectations.push(CandleExpectation {
                name: name.to_string(),
                pattern,
                negate,
                indices,
            });
            index += 1;
        }

        cases.push(CandleCase {
            inputs,
            expectations,
        });
    }

    cases
}

fn parse_array(line: &str) -> Vec<Real> {
    let trimmed = line.trim();
    assert!(
        trimmed.starts_with('{') && trimmed.ends_with('}'),
        "expected array line, got {trimmed}"
    );
    let body = &trimmed[1..trimmed.len() - 1];
    if body.trim().is_empty() {
        return Vec::new();
    }
    body.split(',')
        .map(|value| {
            value
                .trim()
                .parse::<Real>()
                .unwrap_or_else(|error| panic!("failed to parse candle value `{value}`: {error}"))
        })
        .collect()
}
