# State API

This page covers the incremental layer. For the batch, stream, and in-place
entry points that sit underneath it, see [`tutorials/indicator-api.md`](indicator-api.md).

## What The State Layer Does

State objects keep a bounded history while they process data one sample or one
row at a time.

Use them when you need:

- incremental updates
- `latest()` / `latest_ref()`
- indexed history through `get(...)` / `get_ref(...)`
- a stable wrapper around a typed indicator or a runtime-selected indicator

The history buffer is fixed-size. When it fills up, the oldest output is
overwritten instead of shifting the whole buffer.

## Typed State

Typed state wrappers are the best fit when the indicator is known at compile
time.

```rust
use tulipindicators::{IndicatorState, Rsi};

fn main() -> Result<(), tulipindicators::IndicatorError> {
    let closes = [100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 103.5, 105.0];
    let mut rsi = Rsi::state(&[14.0], 32)?;

    let produced = rsi.seed(&closes)?;
    assert!(produced <= closes.len());

    let latest = rsi.update(106.0);
    assert_eq!(latest, rsi.latest());

    Ok(())
}
```

Typed wrappers currently exist for:

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

Use `DynamicIndicatorState` when the indicator name is only known at runtime.

```rust
use tulipindicators::DynamicIndicatorState;

fn main() -> Result<(), tulipindicators::IndicatorError> {
    let closes = [100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 103.5, 105.0];
    let mut state = DynamicIndicatorState::from_name("rsi", &[14.0], 32)?;

    state.seed_columns(&[&closes])?;
    let next = state.update(&[106.0])?;
    assert!(next.is_some());

    Ok(())
}
```

The row shape comes from registry metadata:

- one value per declared input
- one option per declared option name
- one output vector per declared output name

For multi-input indicators, the input order is the same order listed in
`IndicatorMetadata::input_names`.

## Seed Forms

Dynamic state supports both layouts:

- `seed_columns(&[&[Real]])`
- `seed_rows(&[Vec<Real>])`

Column-oriented seeding usually matches existing batch data better. Row-oriented
seeding is useful when historical samples are already stored as per-row records.

## When To Use What

- Use `run(...)` when you want the simplest owned batch output.
- Use `run_single(...)` when the indicator has one output series.
- Use `run_in_place(...)` when you want caller-owned batch buffers.
- Use typed `FooState` wrappers when the indicator is known at compile time.
- Use `DynamicIndicatorState` when the indicator is chosen by name at runtime.

For the full registry shape catalog, see
[`tutorials/indicator-reference.md`](indicator-reference.md).
