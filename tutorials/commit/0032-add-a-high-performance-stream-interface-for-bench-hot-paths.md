# Add A High-Performance Stream Interface For Bench Hot Paths

## Background

After several batch optimizations, the biggest remaining regressions were no longer in the batch path. They were in stream benchmarks, especially:

- `sma`
- `atr`

The key reason was architectural, not mathematical.

On the C side, stream execution already looks like a low-level API:

- the caller provides output buffers
- the stream writes directly into them
- there is no per-chunk result allocation

On the Rust side, the stream trait still returned `Vec<Vec<Real>>` from `feed()`. That is ergonomic, but it means benchmarking a stream in chunks also measures:

- per-chunk vector allocation
- per-chunk outer `Vec` construction
- per-chunk copying into benchmark-owned sinks

That is exactly the kind of fixed overhead that makes stream code look much worse than the underlying math really is.

## Main Goal

Keep the ergonomic stream API, but add a parallel high-performance path that lets performance-sensitive callers supply output buffers directly.

This mirrors the same design decision we already made for batch mode:

- easy high-level API stays
- explicit low-overhead path is available when hot loops need it

## What Changed

### 1. `IndicatorStream` now has `feed_in_place`

In [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/core/indicator.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/core/indicator.rs), the stream trait now includes:

- `feed(...)` for ergonomic callers
- `feed_in_place(...)` for performance-sensitive callers

The default implementation is still safe and simple:

- call `feed()`
- validate output slice count
- copy results into caller-provided buffers

That means existing streams do not break, and performance-sensitive streams can override the method one by one.

### 2. The benchmark now prefers the in-place stream path

In [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/benchmark.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/benchmark.rs), stream benchmarking now preallocates per-chunk output buffers and calls `feed_in_place(...)`.

This matters because it removes a large amount of benchmark-only allocation churn. The benchmark now measures the stream state machine more directly instead of repeatedly measuring result container construction.

### 3. `sma` and `atr` now override `feed_in_place`

In:

- [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/sma.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/sma.rs)
- [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/atr.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/atr.rs)

the stream hot paths now write directly into caller-provided slices.

That avoids building `Vec<Vec<Real>>` for each benchmark chunk while keeping the public `feed()` behavior intact.

## Performance Outcome

This change produced a large structural win.

Before this change, `sma` and `atr` stream benchmarks were heavily dominated by allocation overhead. After the change:

- `sma stream` moved from a large regression to near parity at smaller sizes
- `atr stream` improved substantially, though it still trails the C implementation

So the pattern is:

- the architecture fix clearly mattered
- `sma` got most of the benefit right away
- `atr` still has some state-machine cost left to squeeze

## Rust Knowledge Nugget 1: ergonomic APIs and fast APIs can coexist

A common beginner instinct is to think an API must be either:

- clean and high-level
- or fast and low-level

In systems code, the better answer is often both.

That is what happened here:

- `feed()` remains the simple API
- `feed_in_place()` becomes the hot-path API

The trick is not to force every caller onto the low-level interface. Instead, expose it where it matters and keep the default path pleasant.

## Rust Knowledge Nugget 2: benchmark what your API shape actually costs

It is easy to say “the algorithm is slow” when a benchmark regresses.

But this commit shows a more important lesson:

- sometimes the math is fine
- the allocation pattern around the math is the real problem

If your benchmark repeatedly creates result containers in a tight loop, then the benchmark is partly measuring container churn, not just indicator logic.

That is not fake work. It is real API cost. But you need to know which layer you are optimizing:

- math kernel
- state machine
- allocation pattern
- caller contract

## Why This Matters For Human + AI Collaboration

This is the kind of change that is easy for an AI to miss if it only stares at formulas.

The real win came from stepping back and asking:

> “Is the stream benchmark measuring indicator math, or is it mostly measuring result allocation?”

Once that question was asked, the code change became much more obvious.

That is exactly the kind of reasoning trail worth preserving in a tutorial, because it helps the next human or AI attack the next hotspot from the right layer.
