[![Build Status](https://travis-ci.com/TulipCharts/tulipindicators.svg?branch=master)](https://travis-ci.com/TulipCharts/tulipindicators)

# Tulip Indicators

Tulip Indicators is a technical-analysis library with the original ANSI C
implementation under `c/` and the Rust implementation under `rust/`.

Root `make` targets build the C side. Root `cargo` commands build and test the
Rust crate.

## Layout

 - `c/`: ANSI C implementation and header files
 - `rust/`: Rust crate, tests, and binary utilities
 - `examples/`: focused Rust examples for batch, in-place, and stateful usage

The release docs live on the project site:

- [Usage](https://tulipindicators.org/usage)
- [Bindings](https://tulipindicators.org/bindings)

## Rust API

The Rust crate exposes three layers:

- batch APIs: `run(...)`, `run_single(...)`, `run_in_place(...)`
- stream APIs: `IndicatorStream` for incremental feeds without history
- state APIs: `IndicatorState` and `DynamicIndicatorState` for bounded history

Use the batch layer for offline analysis and benchmarks. Use the stream or
state layers when you want one sample or one row at a time.

For full guides, see:

- [`tutorials/indicator-api.md`](tutorials/indicator-api.md)
- [`tutorials/state-api.md`](tutorials/state-api.md)
- [`tutorials/indicator-reference.md`](tutorials/indicator-reference.md)

Examples:

- [`examples/batch_rsi.rs`](examples/batch_rsi.rs)
- [`examples/in_place_macd.rs`](examples/in_place_macd.rs)
- [`examples/dynamic_state.rs`](examples/dynamic_state.rs)

### Typed state

```rust
use tulipindicators::{IndicatorState, Rsi};

let closes = [100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 103.5, 105.0];
let mut rsi = Rsi::state(&[3.0], 32)?;

// Seed from historical data.
let produced = rsi.seed(&closes)?;
assert!(produced <= closes.len());

// Incrementally update with one new value.
let latest = rsi.update(106.0);

// Read the latest and prior outputs.
let current = rsi.latest();
let previous = rsi.get(1);
```

Use `run_single(...)` for one-output indicators when you do not want to unpack
`Vec<Vec<Real>>` manually.

```rust
use tulipindicators::{Indicator, Rsi};

let closes = [100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 103.5, 105.0];
let values = Rsi.run_single(&[&closes], &[3.0])?;

assert!(!values.is_empty());
```

### Dynamic state

```rust
use tulipindicators::{DynamicIndicatorState, IndicatorStateFactory, RSI};

let closes = [100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 103.5, 105.0];

// Build by registry name.
let mut by_name = DynamicIndicatorState::from_name("rsi", &[3.0], 32)?;
by_name.seed_columns(&[&closes])?;
let next = by_name.update(&[106.0])?;

// Or build directly from a static indicator handle.
let mut by_factory = RSI.dynamic_state(&[3.0], 32)?;
by_factory.seed_columns(&[&closes])?;
assert_eq!(by_factory.latest(), by_name.latest());
```

## Indicators

The Rust registry currently exposes 148 indicators. See
[`tutorials/indicator-reference.md`](tutorials/indicator-reference.md) for the
full metadata table, including input names, option names, output names, and
typed-state availability.

## Special Thanks

The stochrsi indicator was sponsored by: [Gunthy](https://gunthy.org).

The candle pattern recognition was sponsored by: [Algorum](https://algorumsoftware.com)
