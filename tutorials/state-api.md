# State API

This page describes the Rust-side incremental API that sits on top of the
existing Tulip batch layer.

## API Layers

The crate now has three distinct ways to use an indicator:

1. Batch convenience

- `run(...)`

Use this when you want the simplest batch API and do not care about caller-owned
output buffers.

2. Batch performance

- `run_in_place(...)`

Use this when you want the existing high-performance batch kernels and full
control over output allocation.

3. Incremental state

- typed `FooState`
- `DynamicIndicatorState`

Use this when market data arrives one sample at a time and you want:

- `seed(...)`
- `update(...)`
- `latest()`
- `get(index_from_latest)`

## History Model

State objects use a fixed-capacity ring buffer for output history.

That means:

- no full-buffer shifting when history is full
- `update(...)` stays `O(1)` for history storage
- `get(0)` returns the newest output
- `get(1)` returns the previous output

When the buffer is full, the oldest output is overwritten.

## Typed State

Typed state wrappers are the preferred interface when the indicator is known at
compile time.

Example:

```rust
use tulipindicators::{IndicatorState, Rsi};

let history = [100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 103.5, 105.0];
let mut rsi = Rsi::state(&[3.0], 32)?;

rsi.seed(&history)?;

let latest = rsi.update(106.0);
let current = rsi.latest();
let previous = rsi.get(1);

# let _ = (latest, current, previous);
# Ok::<(), tulipindicators::IndicatorError>(())
```

Typed state wrappers currently exist for:

- `RsiState`
- `DmState`
- `DxState`
- `DiState`
- `AdxState`
- `AdxrState`
- `EmaState`
- `SmaState`
- `WildersState`
- `AtrState`
- `NatrState`
- `MacdState`
- `PpoState`
- `StochState`

## Dynamic State

Use `DynamicIndicatorState` when the indicator is only known at runtime.

You can construct it by name:

```rust
use tulipindicators::DynamicIndicatorState;

let history = [100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 103.5, 105.0];
let mut state = DynamicIndicatorState::from_name("rsi", &[3.0], 32)?;

state.seed_columns(&[&history])?;
let latest = state.update(&[106.0])?;

# let _ = latest;
# Ok::<(), tulipindicators::IndicatorError>(())
```

Or by indicator handle:

```rust
use tulipindicators::{IndicatorStateFactory, RSI};

let history = [100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 103.5, 105.0];
let mut state = RSI.dynamic_state(&[3.0], 32)?;

state.seed_columns(&[&history])?;
let latest = state.update(&[106.0])?;

# let _ = latest;
# Ok::<(), tulipindicators::IndicatorError>(())
```

For multi-input indicators, the row/column layout follows indicator metadata:

- `dm.update(&[high, low])`
- `di.update(&[high, low, close])`
- `stoch.update(&[high, low, close])`

## Seed Forms

The dynamic API supports both column-oriented and row-oriented seeding:

- `seed_columns(&[&[Real]])`
- `seed_rows(&[Vec<Real>])`

Column-oriented seeding is usually the better match for existing batch data.
Row-oriented seeding is useful when historical data already exists as per-sample
records.

## Design Boundary

The state layer intentionally does not replace the batch layer.

The rule is:

- existing batch kernels stay the performance-oriented foundation
- state wrappers reuse those stream/state internals when available
- dynamic fallback only exists to make the incremental API universal

That keeps the high-performance layer stable while still making the library
practical for live incremental usage.
