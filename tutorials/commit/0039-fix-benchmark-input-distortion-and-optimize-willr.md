# Fix Benchmark Input Distortion And Optimize WillR

## Background

The performance report showed two suspicious hotspots:

- `div` looked absurdly slow in Rust, especially on large inputs
- `willr` looked much slower than its C version

Those two numbers turned out to have different meanings.

`willr` was a real implementation hotspot.

`div` was mostly a benchmark-design problem.

That is an important lesson: before optimizing code, make sure the benchmark is measuring the thing you think it is measuring.

## Main Goal

Do two different kinds of repair in one focused step:

1. make the shared C/Rust benchmark contract more representative for binary `real, real` indicators
2. move Rust `willr` batch execution onto a direct C-style rolling kernel

The point was not just "make numbers smaller."

The point was to separate fake regressions from real ones.

## What Changed

### 1. Fixed a benchmark contract distortion for repeated `real` inputs

Both benchmark drivers used to map every `real` input to the same `close` series.

For indicators like:

- `add`
- `sub`
- `mul`
- `div`

that means the benchmark was really measuring things like:

- `close + close`
- `close / close`

That is legal, but it is not a very representative contract for vector-vs-vector arithmetic.

The shared benchmark generators now assign repeated `real` inputs to distinct deterministic series:

- first `real` -> `close`
- second `real` -> `open`-like variant
- third `real` -> `high`-like variant
- fourth `real` -> `low`-like variant

This was implemented in both:

- [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/benchmark.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/benchmark.rs)
- [`/Users/dev/workspace2/hc_apps/tulipindicators/c/benchmark_contract.c`](/Users/dev/workspace2/hc_apps/tulipindicators/c/benchmark_contract.c)

That keeps the compare contract aligned while making it harder for one side to get misleadingly easy arithmetic.

### 2. Moved `willr` batch onto a direct kernel

[`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/willr.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/willr.rs) used to run batch mode through a general monotonic-queue approach.

That was correct, but not especially cheap.

The C implementation uses a tighter rolling-window strategy:

- track the current max index
- track the current min index
- rescan only when the previous winner falls out of the window

Rust now uses the same style for batch mode in `run_willr_batch`, and `run_in_place` calls that kernel directly.

This is exactly the kind of optimization that is worth doing:

- same algorithmic meaning
- less framework overhead
- much closer to the reference implementation's cost model

### 3. Kept C parity exact while optimizing

The first draft of the Rust kernel failed the historical `willr` fixtures.

Why?

Because the C implementation starts with:

- `maxi = -1`
- `mini = -1`

That detail affects the first rescan behavior.

The final Rust version keeps that same initialization meaning, so the new kernel is fast without silently changing old results.

## Result

Two useful things happened after this change.

### Benchmark interpretation got better

The huge old `div` anomaly stopped looking like a 30x Rust disaster.

At `4096`, `div` is now effectively at parity.

That means the old outlier was mostly benchmark input distortion, not a real arithmetic-kernel failure.

### `willr` batch got much healthier

With the direct batch kernel, `willr` moved into parity or better depending on input size.

So `willr` was a real hotspot, and this change addressed it directly.

## Rust Knowledge Nugget 1: a benchmark can be perfectly consistent and still be misleading

The old benchmark was consistent:

- C and Rust both used it
- the input generator was deterministic

But it still created a misleading scenario for repeated `real` inputs.

That is a useful beginner lesson:

- "fair" does not automatically mean "representative"

You want both.

## Rust Knowledge Nugget 2: copying C exactly sometimes means copying a tiny initialization detail

When you port an optimized loop from C to Rust, the big structure is obvious:

- main loop
- rolling max
- rolling min

But the first bug often hides in a tiny detail:

- a sentinel value
- an off-by-one boundary
- whether the first rescan happens before or after a window shift

In this case, `-1` for the first max/min index was not cosmetic.
It was part of the behavior.

That is why parity tests matter even after the algorithm "looks the same."

## Why This Matters For Human + AI Collaboration

This step is a good example of why tutorials should record both the code fix and the reasoning fix.

The important output was not only:

- "WillR got faster"

It was also:

- "one hotspot was real"
- "one hotspot was mostly a benchmark artifact"

That distinction helps a human reviewer trust the optimization work, and it helps future AI work avoid optimizing the wrong thing.
