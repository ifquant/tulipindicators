# Indicator API

This page is the release-facing overview of the public indicator surface. It
covers the batch helpers, the stream helpers, registry lookup, and the shape
contract that ties them together.

For incremental state usage, see [`tutorials/state-api.md`](state-api.md).
For the current indicator inventory, see
[`tutorials/indicator-reference.md`](indicator-reference.md).

## API Layers

| Layer | Entry point | Use when |
| --- | --- | --- |
| Batch | `run(...)` | you want owned output series back as `Vec<Vec<Real>>` |
| Single-output batch | `run_single(...)` | the indicator has one output and you do not want to unwrap `batch[0]` |
| In-place batch | `run_in_place(...)` | you want to supply output buffers yourself |
| Stream | `create_stream(...)`, `feed(...)`, `feed_single(...)`, `feed_in_place(...)` | you want incremental batch-sized chunks |

The batch APIs return owned vectors by default. The in-place variants keep the
same computation path but write into caller-provided buffers instead of
allocating a fresh `Vec<Vec<Real>>`.

## Shape Contract

Every indicator exposes `IndicatorMetadata`:

- `name`
- `full_name`
- `category`
- `input_names`
- `option_names`
- `output_names`

That metadata defines the public call shape:

- one input slice per declared input name
- one option value per declared option name
- one output buffer per declared output name

```rust
use tulipindicators::{registry, Indicator};

let indicator = registry::find("macd").expect("macd is registered");
let meta = indicator.metadata();

assert_eq!(meta.name, "macd");
assert_eq!(meta.input_names, &["real"]);
assert_eq!(meta.option_names, &["short_period", "long_period", "signal_period"]);
assert_eq!(meta.output_names, &["macd", "macd_signal", "macd_histogram"]);
```

## Registry Lookup

Use the registry when you want to resolve an indicator by name at runtime.

```rust
use tulipindicators::registry;

let macd = registry::find("macd").expect("indicator exists");
let inventory = registry::all();

assert_eq!(macd.metadata().name, "macd");
assert!(!inventory.is_empty());
```

The registry is also the source of truth for the release reference table. If
you need the current surface in code, prefer `registry::find(...)` and
`registry::all()` over hard-coding a copied list.

## Batch Examples

`run_single(...)` is the preferred batch entry point for one-output indicators.

```rust
use tulipindicators::{Indicator, Rsi};

fn main() -> Result<(), tulipindicators::IndicatorError> {
    let closes = [100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 103.5, 105.0];
    let values = Rsi.run_single(&[&closes], &[14.0])?;

    assert!(!values.is_empty());
    Ok(())
}
```

Use `run_in_place(...)` when you already manage the output allocation.

```rust
use tulipindicators::{Indicator, Macd, Real};

fn main() -> Result<(), tulipindicators::IndicatorError> {
    let closes: Vec<Real> = (0..96).map(|i| 100.0 + i as Real * 0.25).collect();
    let options = [12.0, 26.0, 9.0];
    let output_len = closes.len().saturating_sub(25);

    let mut macd = vec![0.0; output_len];
    let mut signal = vec![0.0; output_len];
    let mut hist = vec![0.0; output_len];
    let mut outputs = [&mut macd[..], &mut signal[..], &mut hist[..]];

    let produced = Macd.run_in_place(&[&closes], &options, &mut outputs)?;
    assert_eq!(produced, output_len);
    Ok(())
}
```

## Stream Examples

Stream objects mirror the batch shape, but they keep incremental progress
instead of recomputing the whole history.

```rust
use tulipindicators::{Indicator, IndicatorStream, Rsi};

fn main() -> Result<(), tulipindicators::IndicatorError> {
    let closes = [100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 103.5, 105.0];
    let mut stream = Rsi.create_stream(&[14.0])?.expect("rsi supports streaming");
    let values = stream.feed_single(&[&closes])?;

    assert!(!values.is_empty());
    Ok(())
}
```

Use `feed_in_place(...)` when you want the stream path but still want to own
the output buffers.

## State Bridge

The state API builds on the same indicator metadata. The only extra decision is
whether the indicator is known at compile time or chosen by name at runtime.

- `Rsi::state(...)` returns a typed wrapper
- `DynamicIndicatorState::from_name(...)` resolves by registry name

For the incremental API details, see [`tutorials/state-api.md`](state-api.md).
