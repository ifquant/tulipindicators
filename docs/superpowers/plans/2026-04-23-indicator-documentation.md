# Indicator Documentation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Prepare the Rust indicator library for release by adding comprehensive source comments, public API rustdoc, indicator usage documentation, examples, and documentation quality checks.

**Architecture:** Keep the high-performance indicator kernels unchanged and layer documentation around the existing structure: public traits and state APIs get rustdoc, indicator families get module-level guides, individual indicators get concise metadata/usage notes, and tutorials explain how to use the library. Documentation checks become part of normal `cargo test` and `cargo test --doc` so new undocumented public API or bad examples are caught early.

**Tech Stack:** Rust 2021, rustdoc, Cargo integration tests, Markdown tutorials, existing `Indicator` / `IndicatorStream` / `IndicatorState` APIs.

---

## File Structure

**Primary public API docs**
- Modify: `rust/src/lib.rs`
  - Responsibility: crate-level overview, quick-start examples, API layer navigation, release-facing entry point.
- Modify: `rust/src/core/indicator.rs`
  - Responsibility: rustdoc for `IndicatorMetadata`, `Indicator`, `IndicatorStream`, `run`, `run_single`, `run_in_place`, `feed`, `feed_single`, `feed_in_place`, and validation helpers where public.
- Modify: `rust/src/core/error.rs`
  - Responsibility: document error variants and when callers should expect each variant.
- Modify: `rust/src/core/types.rs`
  - Responsibility: document `Real` and `IndicatorCategory`.
- Modify: `rust/src/state.rs`
  - Responsibility: document typed state trait, dynamic state API, history capacity semantics, reseed/reset behavior, borrowed history accessors.
- Modify: `rust/src/registry.rs`
  - Responsibility: document registry functions and static indicator handles.
- Modify: `rust/src/candles.rs`
  - Responsibility: document candle API enough for release parity with indicator docs.

**Indicator family docs**
- Modify: `rust/src/indicators/mod.rs`
  - Responsibility: high-level map of indicator modules and naming conventions.
- Modify: `rust/src/indicators/indicator/mod.rs`
  - Responsibility: oscillator/indicator family overview and re-export documentation.
- Modify: `rust/src/indicators/overlay/mod.rs`
  - Responsibility: overlay/moving-average family overview and re-export documentation.
- Modify: `rust/src/indicators/math/mod.rs`
  - Responsibility: math indicator family overview.
- Modify: `rust/src/indicators/simple/mod.rs`
  - Responsibility: vector/simple indicator macro documentation and generated indicator behavior.
- Modify: `rust/src/indicators/shared.rs`
  - Responsibility: internal helper documentation for shared smoothing/range/window state.

**Individual indicator source comments**
- Modify: selected high-frequency typed-state indicators first:
  - `rust/src/indicators/indicator/rsi.rs`
  - `rust/src/indicators/indicator/atr.rs`
  - `rust/src/indicators/indicator/natr.rs`
  - `rust/src/indicators/indicator/dm.rs`
  - `rust/src/indicators/indicator/dx.rs`
  - `rust/src/indicators/indicator/di.rs`
  - `rust/src/indicators/indicator/adx.rs`
  - `rust/src/indicators/indicator/adxr.rs`
  - `rust/src/indicators/indicator/macd.rs`
  - `rust/src/indicators/indicator/ppo.rs`
  - `rust/src/indicators/indicator/stoch.rs`
  - `rust/src/indicators/overlay/ema.rs`
  - `rust/src/indicators/overlay/sma.rs`
  - `rust/src/indicators/overlay/wilders.rs`
  - Responsibility: explain options, input/output shape, warmup/lookback behavior, state wrapper behavior, and any performance-sensitive kernel notes.
- Modify: grouped indicator files:
  - `rust/src/indicators/indicator/oscillators.rs`
  - `rust/src/indicators/indicator/price_volume.rs`
  - `rust/src/indicators/indicator/regression.rs`
  - `rust/src/indicators/indicator/ht.rs`
  - `rust/src/indicators/overlay/prices.rs`
  - `rust/src/indicators/overlay/talib_ma.rs`
  - `rust/src/indicators/math/correlation.rs`
  - `rust/src/indicators/math/cross.rs`
  - `rust/src/indicators/math/decay.rs`
  - `rust/src/indicators/math/extrema.rs`
  - Responsibility: add module and section comments explaining grouped indicators without duplicating every formula inline.

**Release user guides**
- Modify: `README.md`
  - Responsibility: release-facing concise overview, install/build, API layers, minimal examples, links to deeper docs.
- Modify: `tutorials/state-api.md`
  - Responsibility: detailed state API guide.
- Create: `tutorials/indicator-api.md`
  - Responsibility: batch, stream, in-place, single-output, registry, options/input/output shape guide.
- Create: `tutorials/indicator-reference.md`
  - Responsibility: generated or curated reference table of indicators, categories, input names, option names, output names, typed state availability.
- Create: `examples/batch_rsi.rs`
  - Responsibility: minimal batch `run_single` example.
- Create: `examples/in_place_macd.rs`
  - Responsibility: multi-output caller-owned-buffer example.
- Create: `examples/dynamic_state.rs`
  - Responsibility: runtime-selected state example.

**Documentation checks**
- Create: `rust/tests/documentation_style.rs`
  - Responsibility: repository-specific docs style checks for rustdoc/example regressions.
- Modify: `Cargo.toml`
  - Responsibility: register `documentation_style` integration test and examples if needed.
- Modify: `Makefile`
  - Responsibility: ensure `make rust-check` runs doc tests and documentation style checks if not already covered by `cargo test`.

---

### Task 1: Add Documentation Quality Gate

**Files:**
- Create: `rust/tests/documentation_style.rs`
- Modify: `Cargo.toml`

- [ ] **Step 1: Write the failing documentation style test**

Create `rust/tests/documentation_style.rs` with this exact content:

```rust
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
            contents.contains("//!") || contents.contains("///") || contents.contains("// "),
            "{rel} should include explanatory comments before release"
        );
    }
}

#[test]
fn release_guides_link_to_deeper_indicator_docs() {
    let readme = read("README.md");
    assert!(
        readme.contains("tutorials/indicator-api.md"),
        "README.md should link to tutorials/indicator-api.md"
    );
    assert!(
        readme.contains("tutorials/indicator-reference.md"),
        "README.md should link to tutorials/indicator-reference.md"
    );
}
```

- [ ] **Step 2: Register the failing test**

Modify `Cargo.toml` by appending:

```toml
[[test]]
name = "documentation_style"
path = "rust/tests/documentation_style.rs"
```

- [ ] **Step 3: Run the new test to verify it fails**

Run:

```bash
cargo test --test documentation_style
```

Expected: FAIL. It should complain that `README.md` does not link to `tutorials/indicator-api.md` and `tutorials/indicator-reference.md`, and may also identify Rust source files missing explanatory comments.

- [ ] **Step 4: Commit the failing quality gate**

```bash
git add Cargo.toml rust/tests/documentation_style.rs
git commit -m "test(tulipindicators-docs): add release documentation quality gate"
```

---

### Task 2: Document Core Public Interfaces

**Files:**
- Modify: `rust/src/core/indicator.rs`
- Modify: `rust/src/core/error.rs`
- Modify: `rust/src/core/types.rs`
- Modify: `rust/src/state.rs`
- Modify: `rust/src/registry.rs`
- Modify: `rust/src/candles.rs`

- [ ] **Step 1: Add rustdoc to indicator traits**

In `rust/src/core/indicator.rs`, replace the top of the file through the end of the `IndicatorStream` trait docs with documented definitions shaped like this. Keep existing function bodies unchanged.

```rust
use crate::core::error::IndicatorError;
use crate::core::types::{IndicatorCategory, Real};

/// Static description of an indicator.
///
/// The metadata is the runtime contract for generic callers: it describes the
/// required input columns, option order, and output series order. The registry
/// exposes this metadata for every indicator so applications can build dynamic
/// forms and buffers without hard-coding indicator-specific shapes.
#[derive(Debug, Clone, Copy)]
pub struct IndicatorMetadata {
    /// Short registry name, for example `"rsi"` or `"macd"`.
    pub name: &'static str,
    /// Human-readable display name.
    pub full_name: &'static str,
    /// Broad indicator category used for listings and documentation.
    pub category: IndicatorCategory,
    /// Input column names in the order expected by [`Indicator::run`].
    pub input_names: &'static [&'static str],
    /// Option names in the order expected by [`Indicator::run`].
    pub option_names: &'static [&'static str],
    /// Output series names in the order returned by [`Indicator::run`].
    pub output_names: &'static [&'static str],
}

/// Batch-oriented technical indicator interface.
///
/// Implementors provide the general owned-output [`run`](Self::run) method and
/// may override [`run_in_place`](Self::run_in_place) for caller-owned buffers.
/// Generic callers should inspect [`metadata`](Self::metadata) before building
/// input and output arrays.
pub trait Indicator: Sync {
    /// Returns the static metadata for this indicator.
    fn metadata(&self) -> &'static IndicatorMetadata;

    /// Returns the number of leading samples needed before the first output.
    fn lookback(&self, options: &[Real]) -> Result<usize, IndicatorError>;

    /// Runs the indicator over complete input columns and returns owned output
    /// columns.
    ///
    /// Use this for simple batch work. For single-output indicators, prefer
    /// [`run_single`](Self::run_single). For allocation-sensitive code, prefer
    /// [`run_in_place`](Self::run_in_place).
    fn run(&self, inputs: &[&[Real]], options: &[Real]) -> Result<Vec<Vec<Real>>, IndicatorError>;

    /// Runs a one-output indicator and returns the single output column.
    ///
    /// Multi-output indicators return [`IndicatorError::WrongOutputCount`]
    /// instead of silently dropping extra outputs.
    fn run_single(
        &self,
        inputs: &[&[Real]],
        options: &[Real],
    ) -> Result<Vec<Real>, IndicatorError> {
        let metadata = self.metadata();
        validate_single_output(metadata)?;
        let mut computed = self.run(inputs, options)?;
        validate_computed_outputs(metadata, &computed)?;
        Ok(computed.pop().unwrap_or_default())
    }
```

Then add method-level rustdoc to `run_in_place` and `create_stream`:

```rust
    /// Runs the indicator into caller-provided output buffers.
    ///
    /// Returns the number of output rows written. Output buffers must be passed
    /// in metadata output order and must be large enough for the computed rows.
    fn run_in_place(...)

    /// Creates an incremental stream implementation when the indicator supports
    /// one.
    ///
    /// Indicators without stream support return `Ok(None)`. Dynamic state falls
    /// back to batch recomputation for those indicators.
    fn create_stream(...)
```

Add rustdoc to `IndicatorStream`:

```rust
/// Incremental chunk-processing interface for indicators.
///
/// Streams preserve indicator state across calls. They are useful for live data
/// or chunked backfills. A stream can produce zero, one, or many outputs per
/// feed depending on warmup/lookback behavior.
pub trait IndicatorStream {
    /// Returns metadata for the stream's indicator.
    fn metadata(&self) -> &'static IndicatorMetadata;
    /// Number of input rows consumed so far.
    fn progress(&self) -> usize;
    /// Feeds one chunk and returns owned output columns.
    fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError>;
    /// Feeds one-output streams and returns the single output column.
    fn feed_single(...)
    /// Feeds into caller-provided output buffers.
    fn feed_in_place(...)
}
```

- [ ] **Step 2: Add rustdoc to errors and core types**

In `rust/src/core/error.rs`, add doc comments above the enum and each variant:

```rust
/// Error returned by indicator construction, validation, and execution.
#[derive(Debug, Clone, PartialEq)]
pub enum IndicatorError {
    /// Registry lookup failed for a user-provided indicator name.
    UnknownIndicator { name: String },
    /// An option value is invalid for the named indicator.
    InvalidOption { ... },
    /// The caller provided the wrong number of input columns.
    WrongInputCount { ... },
    /// The caller provided the wrong number of options.
    WrongOptionCount { ... },
    /// The caller provided or requested the wrong number of output buffers.
    WrongOutputCount { ... },
    /// A stream-only operation was requested for an indicator without stream support.
    MissingStreamSupport { ... },
    /// Input columns have mismatched lengths.
    InputLengthMismatch { ... },
    /// A caller-provided output buffer is too small.
    OutputTooSmall { ... },
    /// Library code violated an internal invariant.
    InternalInvariant { ... },
}
```

In `rust/src/core/types.rs`, document:

```rust
/// Floating-point type used by all indicators.
pub type Real = f64;

/// Broad indicator family used for metadata and listings.
#[derive(...)]
pub enum IndicatorCategory { ... }
```

- [ ] **Step 3: Add rustdoc to state API**

In `rust/src/state.rs`, add docs for:

```rust
/// Typed incremental state for one indicator.
///
/// Implementations store a fixed-capacity history. `history_capacity` must be
/// at least 1 when constructing concrete state objects.
pub trait IndicatorState { ... }

/// Factory extension for constructing [`DynamicIndicatorState`] from static
/// indicator handles.
pub trait IndicatorStateFactory: Indicator { ... }

/// Runtime-selected incremental indicator state.
///
/// Stream-backed indicators reuse their stream implementation. Indicators
/// without stream support fall back to batch recomputation over retained input
/// history. Re-seeding is atomic: failed seed calls leave the old state intact.
pub struct DynamicIndicatorState { ... }
```

Add method docs for:

```rust
seed_columns
seed_rows
update
latest_ref
latest
get_ref
get
reset
history_capacity
```

The exact docs must mention:

```text
history_capacity must be greater than zero
seed_columns and seed_rows replace existing state only after successful validation and seeding
latest_ref/get_ref borrow from internal history and are invalidated by mutable calls
reset returns Result because backend reconstruction can validate options
```

- [ ] **Step 4: Add rustdoc to registry and candles**

In `rust/src/registry.rs`, document:

```rust
/// Returns all registered indicators in stable registry order.
pub fn all() -> &'static [&'static dyn Indicator] { ... }

/// Finds an indicator by registry name.
pub fn find(name: &str) -> Option<&'static dyn Indicator> { ... }
```

Add docs to static handles such as `RSI`, `MACD`, `EMA`, `SMA` where they are declared:

```rust
/// Static handle for the Relative Strength Index indicator.
pub static RSI: Rsi = Rsi;
```

In `rust/src/candles.rs`, add a top-level `//!` module comment:

```rust
//! Candlestick pattern recognition API.
//!
//! Candle helpers operate on open/high/low/close columns and return pattern
//! hits by index. This module is independent from the numeric indicator traits
//! because candle outputs are sparse pattern events rather than dense series.
```

- [ ] **Step 5: Run docs and tests**

Run:

```bash
cargo fmt --all
cargo test --test documentation_style --test state_api --test run_single_style
cargo test --doc
cargo clippy --all-targets --all-features
```

Expected: PASS except `documentation_style` may still fail on indicator files not yet commented. If it fails only for missing indicator source comments, continue to Task 3 before committing.

- [ ] **Step 6: Commit if tests pass**

If all commands pass:

```bash
git add rust/src/core/indicator.rs rust/src/core/error.rs rust/src/core/types.rs rust/src/state.rs rust/src/registry.rs rust/src/candles.rs
git commit -m "docs(tulipindicators-api): document core public interfaces"
```

---

### Task 3: Add Indicator Family Module Documentation

**Files:**
- Modify: `rust/src/indicators/mod.rs`
- Modify: `rust/src/indicators/indicator/mod.rs`
- Modify: `rust/src/indicators/overlay/mod.rs`
- Modify: `rust/src/indicators/math/mod.rs`
- Modify: `rust/src/indicators/simple/mod.rs`
- Modify: `rust/src/indicators/shared.rs`

- [ ] **Step 1: Add top-level indicator module docs**

At the top of `rust/src/indicators/mod.rs`, add:

```rust
//! Indicator implementations grouped by Tulip category.
//!
//! Most users access indicators through crate-level re-exports or the registry.
//! These modules contain the concrete implementations, stream state machines,
//! typed state wrappers, and performance-sensitive batch kernels.
//!
//! Implementation convention:
//! - metadata constant
//! - zero-sized public indicator handle
//! - `Indicator` implementation
//! - optional stream implementation
//! - optional typed state wrapper
//! - parsing helpers
//! - batch kernel helpers
```

- [ ] **Step 2: Add docs to category modules**

At the top of `rust/src/indicators/indicator/mod.rs`, add:

```rust
//! Oscillator, momentum, volatility, volume, and trend indicators.
//!
//! These indicators usually produce derived series below the price chart. Some
//! have typed state wrappers because they are common in live trading loops.
```

At the top of `rust/src/indicators/overlay/mod.rs`, add:

```rust
//! Price overlays and moving-average families.
//!
//! Overlay indicators generally consume price-like inputs and produce series
//! plotted on top of price. Many implementations expose in-place kernels for
//! benchmark-sensitive batch use.
```

At the top of `rust/src/indicators/math/mod.rs`, add:

```rust
//! General-purpose mathematical transforms and rolling-window utilities.
//!
//! These indicators are useful as building blocks for higher-level trading
//! systems and for compatibility with TA-Lib style APIs.
```

- [ ] **Step 3: Document simple macro-generated indicators**

At the top of `rust/src/indicators/simple/mod.rs`, add:

```rust
//! Simple vector indicators generated from small unary and binary operations.
//!
//! These indicators have no lookback and usually map one input row directly to
//! one output row. The macros in this module keep metadata, batch, and in-place
//! behavior consistent across the family.
```

Above `define_unary_indicator!`, add:

```rust
/// Defines a one-input, one-output vector indicator.
///
/// The generated indicator validates one input series and writes one output for
/// every input row.
```

Above `define_binary_indicator!`, add:

```rust
/// Defines a two-input, one-output vector indicator.
///
/// The generated indicator requires equal-length input series and produces one
/// output row for every pair of input rows.
```

- [ ] **Step 4: Document shared internals**

At the top of `rust/src/indicators/shared.rs`, add:

```rust
//! Shared state machines and math helpers used by multiple indicators.
//!
//! This module intentionally stays internal. It centralizes smoothing, true
//! range, RSI value calculation, directional movement, and rolling-window
//! helpers so individual indicators do not maintain duplicate logic.
```

Add comments above helper types:

```rust
/// Incremental EMA smoother used by stream and typed-state implementations.
pub(crate) struct EmaState { ... }

/// Incremental Wilder-style smoother used by RSI, ATR, and directional indicators.
pub(crate) struct WildersAverageState { ... }

/// Monotonic queue for rolling extrema without scanning the full window.
pub(crate) struct MonotonicQueue { ... }
```

Use actual type names from `shared.rs`; if `MonotonicQueue` is named differently, document the existing name rather than renaming it.

- [ ] **Step 5: Run documentation style test**

Run:

```bash
cargo fmt --all
cargo test --test documentation_style
```

Expected: PASS for module-level files. If it still fails on specific indicator implementation files, continue to Task 4.

- [ ] **Step 6: Commit**

```bash
git add rust/src/indicators/mod.rs rust/src/indicators/indicator/mod.rs rust/src/indicators/overlay/mod.rs rust/src/indicators/math/mod.rs rust/src/indicators/simple/mod.rs rust/src/indicators/shared.rs
git commit -m "docs(tulipindicators-indicators): document indicator module structure"
```

---

### Task 4: Document High-Frequency Typed-State Indicators

**Files:**
- Modify:
  - `rust/src/indicators/indicator/rsi.rs`
  - `rust/src/indicators/indicator/atr.rs`
  - `rust/src/indicators/indicator/natr.rs`
  - `rust/src/indicators/indicator/dm.rs`
  - `rust/src/indicators/indicator/dx.rs`
  - `rust/src/indicators/indicator/di.rs`
  - `rust/src/indicators/indicator/adx.rs`
  - `rust/src/indicators/indicator/adxr.rs`
  - `rust/src/indicators/indicator/macd.rs`
  - `rust/src/indicators/indicator/ppo.rs`
  - `rust/src/indicators/indicator/stoch.rs`
  - `rust/src/indicators/overlay/ema.rs`
  - `rust/src/indicators/overlay/sma.rs`
  - `rust/src/indicators/overlay/wilders.rs`

- [ ] **Step 1: Add a consistent file-level comment to `rsi.rs`**

At the top of `rust/src/indicators/indicator/rsi.rs`, add:

```rust
//! Relative Strength Index (RSI).
//!
//! Inputs:
//! - `real`: close or other price-like series
//!
//! Options:
//! - `period`: smoothing period, must be at least 1
//!
//! Outputs:
//! - `rsi`: oscillator in the range 0..100 for normal inputs
//!
//! The batch path uses the same Wilder-style smoothing as the stream and typed
//! state paths. `RsiState` is the preferred incremental API when the indicator
//! is known at compile time.
```

Add rustdoc to the public state type:

```rust
/// Typed incremental state for RSI.
///
/// Use [`Rsi::state`] to construct this type. The state stores a fixed-capacity
/// output history and updates one input sample at a time.
pub struct RsiState { ... }
```

- [ ] **Step 2: Add file-level comments to single-output typed indicators**

Add the same structure to each file, with exact indicator-specific text:

`rust/src/indicators/overlay/ema.rs`

```rust
//! Exponential Moving Average (EMA).
//!
//! Inputs:
//! - `real`: price-like input series
//!
//! Options:
//! - `period`: EMA period, must be at least 1
//!
//! Outputs:
//! - `ema`: smoothed output series
//!
//! The hot batch kernel uses `mul_add` so optimized builds can generate fused
//! multiply-add instructions on supported targets.
```

`rust/src/indicators/overlay/sma.rs`

```rust
//! Simple Moving Average (SMA).
//!
//! Inputs:
//! - `real`: price-like input series
//!
//! Options:
//! - `period`: rolling window length, must be at least 1
//!
//! Outputs:
//! - `sma`: arithmetic mean over the rolling window
```

`rust/src/indicators/overlay/wilders.rs`

```rust
//! Wilder smoothing.
//!
//! Inputs:
//! - `real`: price-like input series
//!
//! Options:
//! - `period`: smoothing period, must be at least 1
//!
//! Outputs:
//! - `wilders`: Wilder-style smoothed series
```

`rust/src/indicators/indicator/atr.rs`

```rust
//! Average True Range (ATR).
//!
//! Inputs:
//! - `high`
//! - `low`
//! - `close`
//!
//! Options:
//! - `period`: Wilder smoothing period
//!
//! Outputs:
//! - `atr`: average true range
```

`rust/src/indicators/indicator/natr.rs`

```rust
//! Normalized Average True Range (NATR).
//!
//! Inputs:
//! - `high`
//! - `low`
//! - `close`
//!
//! Options:
//! - `period`: Wilder smoothing period
//!
//! Outputs:
//! - `natr`: ATR normalized by close and scaled by 100
```

`rust/src/indicators/indicator/dx.rs`

```rust
//! Directional Movement Index (DX).
//!
//! Inputs:
//! - `high`
//! - `low`
//!
//! Options:
//! - `period`: Wilder smoothing period
//!
//! Outputs:
//! - `dx`: directional movement ratio scaled by 100
```

`rust/src/indicators/indicator/adx.rs`

```rust
//! Average Directional Movement Index (ADX).
//!
//! Inputs:
//! - `high`
//! - `low`
//!
//! Options:
//! - `period`: Wilder smoothing period
//!
//! Outputs:
//! - `adx`: smoothed trend-strength series
```

`rust/src/indicators/indicator/adxr.rs`

```rust
//! Average Directional Movement Rating (ADXR).
//!
//! Inputs:
//! - `high`
//! - `low`
//!
//! Options:
//! - `period`: ADX period and rating lag
//!
//! Outputs:
//! - `adxr`: average of current ADX and lagged ADX
```

`rust/src/indicators/indicator/ppo.rs`

```rust
//! Percentage Price Oscillator (PPO).
//!
//! Inputs:
//! - `real`: price-like input series
//!
//! Options:
//! - `short_period`
//! - `long_period`
//!
//! Outputs:
//! - `ppo`: EMA spread expressed as a percentage of the long EMA
```

- [ ] **Step 3: Add comments to multi-output typed indicators**

`rust/src/indicators/indicator/dm.rs`

```rust
//! Directional Movement (DM).
//!
//! Inputs:
//! - `high`
//! - `low`
//!
//! Options:
//! - `period`: Wilder smoothing period
//!
//! Outputs:
//! - `plus_dm`
//! - `minus_dm`
```

`rust/src/indicators/indicator/di.rs`

```rust
//! Directional Indicator (DI).
//!
//! Inputs:
//! - `high`
//! - `low`
//! - `close`
//!
//! Options:
//! - `period`: Wilder smoothing period
//!
//! Outputs:
//! - `plus_di`
//! - `minus_di`
```

`rust/src/indicators/indicator/macd.rs`

```rust
//! Moving Average Convergence/Divergence (MACD).
//!
//! Inputs:
//! - `real`: price-like input series
//!
//! Options:
//! - `short_period`
//! - `long_period`
//! - `signal_period`
//!
//! Outputs:
//! - `macd`
//! - `macd_signal`
//! - `macd_histogram`
```

`rust/src/indicators/indicator/stoch.rs`

```rust
//! Stochastic Oscillator.
//!
//! Inputs:
//! - `high`
//! - `low`
//! - `close`
//!
//! Options:
//! - `k_period`
//! - `k_slowing_period`
//! - `d_period`
//!
//! Outputs:
//! - `stoch_k`
//! - `stoch_d`
```

- [ ] **Step 4: Add rustdoc to each public typed state struct**

For every state struct in the files from this task, add:

```rust
/// Typed incremental state for <INDICATOR_NAME>.
///
/// Construct with `<Indicator>::state(options, history_capacity)`. The state
/// updates one sample at a time and stores the most recent outputs in a
/// fixed-capacity history ring.
```

Use the actual indicator name, for example:

```rust
/// Typed incremental state for MACD.
```

- [ ] **Step 5: Add targeted kernel comments only where useful**

Add comments only above non-obvious hot kernels. Do not comment simple assignments. Examples:

In EMA:

```rust
// Keep the recurrence in this shape so optimized builds can emit fused
// multiply-add instructions on targets that support them.
value = (sample - value).mul_add(multiplier, value);
```

In RSI/ATR/ADX family:

```rust
// The first `period` rows establish Wilder averages; outputs start after the
// warmup row so batch, stream, and typed state stay aligned.
```

- [ ] **Step 6: Run tests**

```bash
cargo fmt --all
cargo test --test documentation_style --test state_api --test stable_parity --test golden_indicators
cargo test --doc
cargo clippy --all-targets --all-features
```

Expected: PASS for files covered in this task. If `documentation_style` still fails on other indicator files, continue to Task 5.

- [ ] **Step 7: Commit**

```bash
git add rust/src/indicators/indicator/rsi.rs rust/src/indicators/indicator/atr.rs rust/src/indicators/indicator/natr.rs rust/src/indicators/indicator/dm.rs rust/src/indicators/indicator/dx.rs rust/src/indicators/indicator/di.rs rust/src/indicators/indicator/adx.rs rust/src/indicators/indicator/adxr.rs rust/src/indicators/indicator/macd.rs rust/src/indicators/indicator/ppo.rs rust/src/indicators/indicator/stoch.rs rust/src/indicators/overlay/ema.rs rust/src/indicators/overlay/sma.rs rust/src/indicators/overlay/wilders.rs
git commit -m "docs(tulipindicators-state): document high-frequency typed indicators"
```

---

### Task 5: Document Remaining Grouped Indicator Files

**Files:**
- Modify grouped files:
  - `rust/src/indicators/indicator/oscillators.rs`
  - `rust/src/indicators/indicator/price_volume.rs`
  - `rust/src/indicators/indicator/regression.rs`
  - `rust/src/indicators/indicator/ht.rs`
  - `rust/src/indicators/indicator/beta_momentum.rs`
  - `rust/src/indicators/indicator/beta_oscillators.rs`
  - `rust/src/indicators/indicator/beta_trend.rs`
  - `rust/src/indicators/indicator/beta_volume.rs`
  - `rust/src/indicators/overlay/prices.rs`
  - `rust/src/indicators/overlay/talib_ma.rs`
  - `rust/src/indicators/overlay/beta_channels.rs`
  - `rust/src/indicators/overlay/beta_smoothers.rs`
  - `rust/src/indicators/math/correlation.rs`
  - `rust/src/indicators/math/cross.rs`
  - `rust/src/indicators/math/decay.rs`
  - `rust/src/indicators/math/extrema.rs`

- [ ] **Step 1: Add grouped module comments**

For each file, add a concise `//!` comment at the top. Use these exact texts:

`oscillators.rs`

```rust
//! Miscellaneous oscillator indicators implemented as direct batch kernels.
//!
//! This file groups medium-size indicators that share validation and rolling
//! window patterns but do not yet need dedicated modules.
```

`price_volume.rs`

```rust
//! Price/volume indicators.
//!
//! These indicators combine price columns with volume-like inputs. Several have
//! direct batch kernels to avoid stream-backed allocation overhead.
```

`regression.rs`

```rust
//! Rolling linear-regression family.
//!
//! The indicators in this file share the same rolling regression statistics and
//! expose different projections such as slope, intercept, angle, forecast, and
//! forecast oscillator.
```

`ht.rs`

```rust
//! Hilbert Transform indicator family.
//!
//! These indicators are TA-Lib compatibility implementations with stateful
//! recurrences. Keep comments near non-obvious recurrence blocks because small
//! ordering changes can alter parity and performance.
```

`prices.rs`

```rust
//! Price transform overlays.
//!
//! Most indicators in this file are thin vector transforms over OHLC columns.
//! The in-place paths are intentionally direct because these indicators are
//! sensitive to wrapper overhead.
```

`talib_ma.rs`

```rust
//! TA-Lib moving-average compatibility wrappers.
//!
//! `ma` and `mavp` dispatch to concrete moving-average kernels while preserving
//! TA-Lib option semantics.
```

`correlation.rs`

```rust
//! Rolling beta and Pearson correlation indicators.
//!
//! Both indicators maintain rolling sums and cross-products so each output row
//! can be computed without rescanning the full window.
```

`cross.rs`

```rust
//! Crossing indicators.
//!
//! These vector indicators detect crossings between two equal-length input
//! series. The batch kernels stay deliberately simple because they are often
//! wrapper-overhead bound.
```

`decay.rs`

```rust
//! Decay and lag indicators.
//!
//! These indicators transform one input series with simple recurrence or offset
//! rules and are useful as low-level building blocks.
```

`extrema.rs`

```rust
//! Rolling extrema indicators.
//!
//! This module contains max/min value and index variants. Implementations use
//! rolling-window helpers to avoid rescanning where practical.
```

For each beta file, add:

```rust
//! Beta indicator implementations retained for compatibility with the C beta set.
//!
//! These indicators are covered by beta parity tests and may still evolve before
//! being promoted to the stable indicator set.
```

- [ ] **Step 2: Add comments to standalone single-indicator files**

For any remaining `rust/src/indicators/indicator/*.rs` or `rust/src/indicators/overlay/*.rs` files not covered in Tasks 4 or 5, add a top-level `//!` comment with this template:

```rust
//! <Full indicator name>.
//!
//! See [`METADATA`] for the input, option, and output contract. The implementation
//! follows Tulip/TA-Lib-compatible lookback semantics and keeps batch kernels
//! separate from stream state where that improves performance.
```

Replace `<Full indicator name>` with the indicator's `METADATA.full_name` value.

- [ ] **Step 3: Run the documentation style test**

```bash
cargo fmt --all
cargo test --test documentation_style
```

Expected: PASS. If it fails, add the missing comment to the reported file and rerun.

- [ ] **Step 4: Run broader validation**

```bash
cargo test --test stable_parity --test golden_indicators --test beta_parity --test talib_missing_parity
cargo clippy --all-targets --all-features
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add rust/src/indicators
git commit -m "docs(tulipindicators-indicators): document remaining indicator families"
```

---

### Task 6: Add Release User Guides and Examples

**Files:**
- Modify: `README.md`
- Modify: `tutorials/state-api.md`
- Create: `tutorials/indicator-api.md`
- Create: `tutorials/indicator-reference.md`
- Create: `examples/batch_rsi.rs`
- Create: `examples/in_place_macd.rs`
- Create: `examples/dynamic_state.rs`
- Modify: `Cargo.toml` if examples need explicit registration

- [ ] **Step 1: Create `tutorials/indicator-api.md`**

Create the file with this content:

```markdown
# Indicator API

This guide explains the Rust indicator API layers.

## Choosing an API

- Use `Indicator::run` for the simplest generic batch call.
- Use `Indicator::run_single` for one-output indicators such as `rsi`, `ema`, `sma`, and `atr`.
- Use `Indicator::run_in_place` when the caller owns output buffers and wants to avoid result allocation.
- Use `IndicatorStream::feed_in_place` for chunked or live data.
- Use typed `FooState` wrappers when you update one row at a time and know the indicator type at compile time.
- Use `DynamicIndicatorState` when the indicator is chosen by name at runtime.

## Input, Option, and Output Order

Every indicator exposes `IndicatorMetadata`.

```rust
use tulipindicators::{find, Indicator};

let indicator = find("rsi").expect("rsi should exist");
let metadata = indicator.metadata();
assert_eq!(metadata.input_names, &["real"]);
assert_eq!(metadata.option_names, &["period"]);
assert_eq!(metadata.output_names, &["rsi"]);
```

The order in metadata is the order required by batch, stream, and dynamic APIs.

## Single-Output Batch

```rust
use tulipindicators::{Indicator, Rsi};

let closes = [100.0, 101.0, 102.0, 103.0, 102.5, 104.0, 105.0];
let rsi = Rsi.run_single(&[&closes], &[3.0])?;

assert!(!rsi.is_empty());
# Ok::<(), tulipindicators::IndicatorError>(())
```

## Multi-Output Batch

```rust
use tulipindicators::{Indicator, Macd};

let closes = [100.0, 101.0, 102.0, 103.0, 102.5, 104.0, 105.0, 106.0, 107.0, 108.0];
let outputs = Macd.run(&[&closes], &[3.0, 5.0, 2.0])?;

let macd = &outputs[0];
let signal = &outputs[1];
let histogram = &outputs[2];
assert_eq!(macd.len(), signal.len());
assert_eq!(signal.len(), histogram.len());
# Ok::<(), tulipindicators::IndicatorError>(())
```

## Caller-Owned Output Buffers

```rust
use tulipindicators::{Indicator, Rsi};

let closes = [100.0, 101.0, 102.0, 103.0, 102.5, 104.0, 105.0];
let mut output = vec![0.0; closes.len()];
let produced = Rsi.run_in_place(&[&closes], &[3.0], &mut [&mut output])?;

output.truncate(produced);
assert_eq!(output.len(), produced);
# Ok::<(), tulipindicators::IndicatorError>(())
```
```

- [ ] **Step 2: Create `tutorials/indicator-reference.md`**

Create this initial curated file:

```markdown
# Indicator Reference

This reference describes the Rust indicator registry shape. The definitive
source of truth is each indicator's `IndicatorMetadata`.

## Reading Metadata

```rust
use tulipindicators::all;

for indicator in all() {
    let metadata = indicator.metadata();
    println!(
        "{} inputs={:?} options={:?} outputs={:?}",
        metadata.name,
        metadata.input_names,
        metadata.option_names,
        metadata.output_names
    );
}
```

## Typed State Coverage

Typed state wrappers currently exist for:

| Indicator | State Type | Input Type | Output Type |
|---|---|---|---|
| `rsi` | `RsiState` | `Real` | `Real` |
| `ema` | `EmaState` | `Real` | `Real` |
| `sma` | `SmaState` | `Real` | `Real` |
| `wilders` | `WildersState` | `Real` | `Real` |
| `atr` | `AtrState` | `(Real, Real, Real)` | `Real` |
| `natr` | `NatrState` | `(Real, Real, Real)` | `Real` |
| `dm` | `DmState` | `(Real, Real)` | `(Real, Real)` |
| `dx` | `DxState` | `(Real, Real)` | `Real` |
| `di` | `DiState` | `(Real, Real, Real)` | `(Real, Real)` |
| `adx` | `AdxState` | `(Real, Real)` | `Real` |
| `adxr` | `AdxrState` | `(Real, Real)` | `Real` |
| `macd` | `MacdState` | `Real` | `(Real, Real, Real)` |
| `ppo` | `PpoState` | `Real` | `Real` |
| `stoch` | `StochState` | `(Real, Real, Real)` | `(Real, Real)` |

All registered indicators can also be used through `DynamicIndicatorState`.
```

- [ ] **Step 3: Add examples**

Create `examples/batch_rsi.rs`:

```rust
use tulipindicators::{Indicator, Rsi};

fn main() -> Result<(), tulipindicators::IndicatorError> {
    let closes = [
        100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 103.5, 105.0, 106.0, 105.5,
    ];
    let values = Rsi.run_single(&[&closes], &[3.0])?;
    println!("last RSI: {:?}", values.last());
    Ok(())
}
```

Create `examples/in_place_macd.rs`:

```rust
use tulipindicators::{Indicator, Macd};

fn main() -> Result<(), tulipindicators::IndicatorError> {
    let closes = [
        100.0, 101.0, 102.0, 103.0, 102.5, 104.0, 105.0, 106.0, 107.0, 108.0,
    ];

    let mut macd = vec![0.0; closes.len()];
    let mut signal = vec![0.0; closes.len()];
    let mut histogram = vec![0.0; closes.len()];
    let produced = Macd.run_in_place(
        &[&closes],
        &[3.0, 5.0, 2.0],
        &mut [&mut macd, &mut signal, &mut histogram],
    )?;

    macd.truncate(produced);
    signal.truncate(produced);
    histogram.truncate(produced);
    println!("produced {produced} MACD rows");
    Ok(())
}
```

Create `examples/dynamic_state.rs`:

```rust
use tulipindicators::DynamicIndicatorState;

fn main() -> Result<(), tulipindicators::IndicatorError> {
    let closes = [
        100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 103.5, 105.0, 106.0, 105.5,
    ];
    let mut state = DynamicIndicatorState::from_name("rsi", &[3.0], 16)?;
    state.seed_columns(&[&closes])?;
    let latest = state.update(&[106.5])?;
    println!("latest dynamic output: {latest:?}");
    Ok(())
}
```

- [ ] **Step 4: Link docs from README**

In `README.md`, under the existing "See also" list, ensure it contains:

```markdown
See also:
- [`tutorials/indicator-api.md`](tutorials/indicator-api.md)
- [`tutorials/indicator-reference.md`](tutorials/indicator-reference.md)
- [`tutorials/state-api.md`](tutorials/state-api.md)
```

- [ ] **Step 5: Run examples and docs**

```bash
cargo run --example batch_rsi
cargo run --example in_place_macd
cargo run --example dynamic_state
cargo test --doc
cargo test --test documentation_style
```

Expected: all PASS. Example commands should print one line and exit 0.

- [ ] **Step 6: Commit**

```bash
git add README.md tutorials/indicator-api.md tutorials/indicator-reference.md examples/batch_rsi.rs examples/in_place_macd.rs examples/dynamic_state.rs
git commit -m "docs(tulipindicators-release): add indicator API guides and examples"
```

---

### Task 7: Tighten Release Documentation in README and Crate Docs

**Files:**
- Modify: `README.md`
- Modify: `rust/src/lib.rs`

- [ ] **Step 1: Update README release summary**

In `README.md`, add a short "Rust API Status" section before "Rust State API":

```markdown
## Rust API Status

The Rust crate exposes:

- complete registry access for all implemented indicators
- owned batch output through `Indicator::run`
- one-output convenience through `Indicator::run_single`
- caller-owned buffers through `Indicator::run_in_place`
- chunked incremental streams through `IndicatorStream`
- typed state wrappers for high-frequency indicators
- dynamic state for runtime-selected indicators

The C implementation remains available in `c/` and is still used by parity
tests and benchmark comparisons.
```

- [ ] **Step 2: Add crate-level navigation in `lib.rs`**

In `rust/src/lib.rs`, after the introductory paragraph, add:

```rust
//! ## API guide
//!
//! - Use [`all`] and [`find`] for registry-driven applications.
//! - Use [`Indicator::run_single`] for one-output batch indicators.
//! - Use [`Indicator::run_in_place`] for allocation-sensitive batch loops.
//! - Use [`IndicatorStream`] for chunked incremental processing.
//! - Use [`IndicatorState`] wrappers when the indicator type is known.
//! - Use [`DynamicIndicatorState`] when the indicator is selected at runtime.
//!
//! See `tutorials/indicator-api.md` and `tutorials/state-api.md` for longer
//! examples.
```

- [ ] **Step 3: Run doc tests**

```bash
cargo test --doc
cargo test --test documentation_style
```

Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add README.md rust/src/lib.rs
git commit -m "docs(tulipindicators-release): polish release API overview"
```

---

### Task 8: Final Documentation Review and Release Readiness Check

**Files:**
- Modify only files required by failures found in this task.

- [ ] **Step 1: Run full Rust validation**

```bash
cargo fmt --all --check
cargo test
cargo test --doc
cargo clippy --all-targets --all-features
```

Expected: PASS.

- [ ] **Step 2: Build documentation locally**

```bash
cargo doc --no-deps
```

Expected: PASS. If rustdoc reports broken intra-doc links, fix the links and rerun.

- [ ] **Step 3: Inspect generated docs entry points**

Open or inspect these generated files:

```bash
test -f target/doc/tulipindicators/index.html
test -f target/doc/tulipindicators/trait.Indicator.html
test -f target/doc/tulipindicators/struct.DynamicIndicatorState.html
```

Expected: all commands exit 0.

- [ ] **Step 4: Search for forbidden placeholders**

Run:

```bash
rg -n "TBD|TODO|fill in|implement later|add appropriate|similar to Task" README.md tutorials rust/src
```

Expected: no new placeholder documentation. Existing historical commit tutorials may contain words like TODO; do not rewrite historical tutorials unless the match is in newly edited release docs or rustdoc.

- [ ] **Step 5: Commit final fixes if needed**

If Step 2 or Step 4 required fixes:

```bash
git add README.md tutorials rust/src
git commit -m "docs(tulipindicators-release): fix final documentation warnings"
```

If no fixes were needed, do not create an empty commit.

---

## Self-Review

**Spec coverage:** The user asked to prepare for release by comprehensively adding comments from source code to interfaces so others can use the library. Tasks 2 and 7 cover public interfaces. Tasks 3, 4, and 5 cover source comments across indicator modules and implementations. Task 6 covers user-facing guides and examples. Tasks 1 and 8 add quality gates and final release checks.

**Placeholder scan:** The plan avoids TBD/TODO/implement later placeholders. Each code-editing task includes concrete file paths, exact comment text or examples, commands, expected results, and commit instructions.

**Type consistency:** The plan uses existing names from the repository: `Indicator`, `IndicatorStream`, `IndicatorState`, `DynamicIndicatorState`, `IndicatorMetadata`, `IndicatorError`, `Real`, `Rsi`, `Macd`, `RsiState`, and existing test command names.

