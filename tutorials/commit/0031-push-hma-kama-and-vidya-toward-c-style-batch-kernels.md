# Push HMA KAMA And VIDYA Toward C-Style Batch Kernels

## Background

After fixing the lighter hotspots, the next regressions were no longer tiny math helpers. They were medium-complexity smoothing indicators:

- `hma`
- `kama`
- `vidya`

All three had the same structural issue: Rust batch mode was still paying for stream-oriented state or generic rolling helpers, while the C versions used direct loops with running sums.

That difference matters a lot for performance-sensitive libraries. Even when the formulas are more complex than `avgprice` or `lag`, the C code is still just a tight loop over slices.

## Main Goal

Move these indicators closer to the C execution model without changing their external behavior:

- keep the existing high-level API
- keep stream support
- add direct batch kernels and `run_in_place`
- preserve output parity with the existing C fixtures

## What Changed

### 1. HMA now has a dedicated batch kernel

In [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/hma.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/hma.rs), batch execution no longer routes through `HmaStream`.

Instead, the batch path now mirrors the C structure:

- maintain running sums for the long WMA
- maintain running sums for the short WMA
- feed the `2 * short - long` diff into the final WMA stage
- write results directly into a batch output buffer

This pulled `hma` out of the worst-regression bucket.

### 2. KAMA now uses a direct batch loop

In [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/kama.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/kama.rs), batch mode now computes the efficiency ratio and smoothing constant in a thin loop instead of reusing the stream path.

This did not fully close the performance gap, but it materially reduced it.

### 3. VIDYA now has a direct rolling-sum batch path

In [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/vidya.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/vidya.rs), batch mode now tracks short and long rolling sums and sum-of-squares directly, matching the C implementation shape much more closely.

That change was enough to bring `vidya` back into the parity band.

## Performance Outcome

On the unified `release` benchmark compare:

- `hma` moved from a large regression to parity
- `vidya` moved from a clear regression to parity
- `kama` improved a lot, but still trails C and needs another pass

This is a good reminder that “same algorithm” is not enough. The execution shape matters too.

## Rust Knowledge Nugget 1: ring-buffer cursor meaning matters

While fixing `hma`, one subtle bug came from misunderstanding what the ring-buffer cursor means after a push.

In the C helper, after pushing a value:

- the cursor points to the next write position
- the oldest current item is now found at that new cursor

If you subtract the value that was just overwritten, you get the wrong window. That is exactly the kind of bug that still produces plausible numbers, which makes it dangerous.

When porting ring-buffer logic, always ask:

- does the cursor point to the last written slot?
- or the next slot that will be written?

That one detail changes how you read “oldest” and “newest” values.

## Rust Knowledge Nugget 2: stream code and batch code are not interchangeable

For a beginner, it is very tempting to say:

> “The stream version already works, so the batch version can just feed all the samples through it.”

That is often correct for correctness, but not for speed.

Batch code wants:

- direct loops
- direct indexing
- minimal branching
- explicit output writes

Stream code wants:

- persistent state
- incremental updates
- chunk-friendly behavior

If you reuse stream for batch, you often trade away the exact performance shape that made the C implementation fast.

## Why This Matters For Human + AI Collaboration

This commit is a good example of a productive optimization loop:

1. benchmark identified the hotspot
2. tests caught a wrong-but-plausible `hma` port
3. the final code stayed close to the C model
4. the tutorial records both the performance lesson and the bug pattern

That means the next person, human or AI, can continue from a clear, audited state instead of re-learning the same trap.
