use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: &str) -> String {
    fs::read_to_string(repo_root().join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
}

fn rust_files_under(path: &str) -> Vec<PathBuf> {
    let root = repo_root().join(path);
    let mut pending = vec![root];
    let mut files = Vec::new();

    while let Some(path) = pending.pop() {
        for entry in fs::read_dir(&path)
            .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
        {
            let path = entry
                .unwrap_or_else(|error| panic!("failed to read directory entry: {error}"))
                .path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
    }

    files.sort();
    files
}

fn relative(path: &Path) -> String {
    path.strip_prefix(repo_root())
        .expect("path should be under repo root")
        .to_string_lossy()
        .replace('\\', "/")
}

#[test]
fn documentation_style_harness_covers_expected_files() {
    let indicator_files = rust_files_under("rust/src/indicators");
    let indicator_paths = indicator_files
        .iter()
        .map(|path| relative(path))
        .collect::<Vec<_>>();

    assert!(
        indicator_paths
            .iter()
            .any(|path| path == "rust/src/indicators/indicator/rsi.rs"),
        "documentation style harness should scan indicator source files"
    );
    assert!(
        read("README.md").contains("## Rust API"),
        "documentation style harness should read release-facing markdown"
    );
}

#[test]
fn public_api_modules_have_crate_or_item_docs() {
    let required_files = [
        "rust/src/lib.rs",
        "rust/src/core/indicator.rs",
        "rust/src/core/error.rs",
        "rust/src/core/types.rs",
        "rust/src/state.rs",
        "rust/src/registry.rs",
        "rust/src/candles.rs",
    ];

    for path in required_files {
        let contents = read(path);
        assert!(
            contents.contains("//!") || contents.contains("///"),
            "{path} should contain rustdoc comments for release-facing API"
        );
    }
}

#[test]
#[ignore = "enabled after indicator source comments land"]
fn indicator_source_files_have_explanatory_comments() {
    let allowed_generated_or_macro_files = [
        "rust/src/indicators/simple/mod.rs",
        "rust/src/indicators/mod.rs",
        "rust/src/indicators/indicator/mod.rs",
        "rust/src/indicators/overlay/mod.rs",
        "rust/src/indicators/math/mod.rs",
    ];

    for path in rust_files_under("rust/src/indicators") {
        let rel = relative(&path);
        if allowed_generated_or_macro_files.contains(&rel.as_str()) {
            continue;
        }
        let contents = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read {rel}: {error}"));
        assert!(
            contents.contains("//!")
                || contents.contains("///")
                || contents.contains("/*!")
                || contents.contains("/**")
                || contents.contains("// "),
            "{rel} should include explanatory comments before release"
        );
    }
}

#[test]
fn release_guides_link_to_deeper_indicator_docs() {
    let readme = read("README.md");
    assert!(
        readme.contains("[`tutorials/indicator-api.md`](tutorials/indicator-api.md)")
            || readme.contains("[tutorials/indicator-api.md](tutorials/indicator-api.md)"),
        "README.md should link to tutorials/indicator-api.md with markdown link syntax"
    );
    assert!(
        readme.contains("[`tutorials/indicator-reference.md`](tutorials/indicator-reference.md)")
            || readme
                .contains("[tutorials/indicator-reference.md](tutorials/indicator-reference.md)"),
        "README.md should link to tutorials/indicator-reference.md with markdown link syntax"
    );
}
