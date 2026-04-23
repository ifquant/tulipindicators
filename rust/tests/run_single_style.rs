use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: &str) -> String {
    fs::read_to_string(repo_root().join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

fn rust_test_files() -> Vec<String> {
    let tests_dir = repo_root().join("rust").join("tests");
    let mut files = fs::read_dir(&tests_dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", tests_dir.display()))
        .map(|entry| {
            let path = entry
                .unwrap_or_else(|error| panic!("failed to read test directory entry: {error}"))
                .path();
            path
        })
        .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
        .map(|path| {
            path.strip_prefix(repo_root())
                .expect("test file should live under repo root")
                .to_string_lossy()
                .replace('\\', "/")
        })
        .filter(|path| path != "rust/tests/run_single_style.rs")
        .collect::<Vec<_>>();
    files.sort();
    files
}

fn rust_code_blocks(markdown: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = Vec::new();
    let mut in_rust_block = false;

    for line in markdown.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            if in_rust_block {
                blocks.push(current.join("\n"));
                current.clear();
                in_rust_block = false;
            } else if trimmed == "```rust" {
                in_rust_block = true;
            }
            continue;
        }

        if in_rust_block {
            current.push(line);
        }
    }

    blocks
}

#[test]
fn docs_examples_do_not_use_batch_zero_for_single_output_paths() {
    for path in ["README.md", "tutorials/state-api.md", "rust/src/lib.rs"] {
        let contents = read(path);
        for block in rust_code_blocks(&contents) {
            assert!(
                !block.contains("batch[0]"),
                "{path} contains `batch[0]` inside a Rust example block; prefer `run_single(...)` for one-output examples"
            );
        }
    }
}

#[test]
fn tests_only_keep_batch_zero_in_whitelisted_multi_output_or_comparison_cases() {
    let allowlisted_snippets: &[(&str, &[&str])] = &[
        (
            "rust/tests/state_api.rs",
            &[
                "let expected_plus = &batch[0];",
                "*extended_batch[0].last().expect(\"plus latest after update\"),",
                "let expected_macd = &batch[0];",
                "let expected_k = &batch[0];",
            ],
        ),
        (
            "rust/tests/single_output_api.rs",
            &["assert_eq!(single, batch[0]);"],
        ),
        (
            "rust/tests/golden_indicators.rs",
            &["assert_eq!(batch[0].len(), input.len().saturating_sub(6));"],
        ),
    ];

    for path in rust_test_files() {
        let snippets = allowlisted_snippets
            .iter()
            .find_map(|(allowed_path, snippets)| (*allowed_path == path).then_some(*snippets))
            .unwrap_or(&[]);
        let contents = read(&path);
        for line in contents.lines() {
            if !line.contains("batch[0]") {
                continue;
            }
            let trimmed = line.trim();
            assert!(
                snippets.contains(&trimmed),
                "{path} contains a non-whitelisted `batch[0]` use: {trimmed}\nPrefer `run_single(...)` for one-output tests, or extend this whitelist only for true multi-output / comparison cases."
            );
        }
    }
}
