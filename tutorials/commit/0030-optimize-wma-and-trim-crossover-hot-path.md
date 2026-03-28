# Optimize WMA And Trim Crossover Hot Path

## Background

This commit continues the benchmark-guided optimization pass after the earlier lightweight indicators were moved onto `run_in_place`.

The next hotspot worth attacking was `wma`. It is not a trivial one-line indicator, but it is still simple enough that the C version gets most of its speed from a thin batch loop with running sums. The Rust version was still routing batch work through the stream state machine, which meant extra abstraction cost on a path that should stay direct.

`crossover` was a much smaller follow-up. Its code was already simple, but the hot path still went through a helper that turned `bool` into `f64`. That is not a big architectural issue, but once the bigger problems are gone, tiny hot-path details start becoming visible.

## Main Goal

Bring `wma` closer to the C performance model without changing its public behavior:

- batch mode should use a direct loop
- high-performance callers should be able to use `run_in_place`
- stream mode should still exist, but batch should no longer pay for stream-style state management

## What Changed

### 1. WMA now has a real batch kernel

In [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/wma.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/wma.rs), `Wma::run()` no longer creates a stream and feeds samples through it.

Instead, it now:

- precomputes the total weight
- keeps a running plain sum
- keeps a running weighted sum
- writes outputs in a direct batch loop

This is much closer to the C implementation style and avoids paying stream overhead on a batch benchmark.

### 2. WMA also implements `run_in_place`

The same file now overrides `run_in_place()`, so the benchmark and any future high-performance callers can hand in an output buffer and avoid the extra result allocation layer.

This matters because `wma` is exactly the kind of indicator where the math is cheap enough that API overhead becomes visible.

### 3. Crossover writes `1.0` / `0.0` directly

In [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/math/mod.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/math/mod.rs), `crossover` now writes its output directly instead of routing through `bool_to_real(...)`.

This is a small change, not a major redesign. The point is simply to keep the inner loop as thin as possible once it is already on the right batch path.

## What The Benchmark Said

Using the unified `release` compare tool:

- `wma batch 4096`: moved to parity with C
- `wma batch 65536`: also moved into the parity band

`crossover` did not show a strong, stable win. That is useful in itself: not every micro-change matters, and benchmark noise is real. The important win in this commit is `wma`.

## Rust Knowledge Nugget 1: `usize` arithmetic can underflow before you expect

During this change, one version of the code used:

```rust
index - period + 1
```

That looks harmless, but `usize` is unsigned, and subtraction is evaluated left to right. So if `index == 0` and `period == 1`, the `index - period` part can underflow before the `+ 1` happens.

The safe version here was:

```rust
index + 1 - period
```

Same math on paper, different behavior for unsigned integers.

This is a very common beginner trap in Rust because array indices use `usize`, and `usize` does not represent negative numbers.

## Rust Knowledge Nugget 2: stream and batch are different performance shapes

It is tempting to write one stream implementation and then reuse it for batch mode. That is often great for correctness and code reuse early on.

But performance-sensitive code teaches a different lesson:

- stream mode is about stateful incremental updates
- batch mode is about thin loops over contiguous slices

If batch calls stream, you often inherit:

- extra state objects
- more branching
- more pushes into temporary vectors
- less obvious optimization opportunities

So a good Rust design for performance-sensitive libraries is often:

- ergonomic high-level API
- explicit stream API
- separate thin batch kernel for hot paths

That is more code, but it preserves both clarity and speed.

## Why This Matters For Human + AI Collaboration

This is a good example of why benchmark-guided optimization is better than random cleanup:

- we picked a concrete hotspot
- we matched the Rust path to the C execution model
- we measured again
- we kept the explanation close to the code

That makes the change easier for a human newcomer to trust, extend, and review later.
