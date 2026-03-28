# Move NATR Onto The In-Place ATR Family Path

## Background

`natr` is a good example of a performance bug that hides in plain sight.

Its batch implementation was already direct and mathematically fine. But it still lacked the lower-level `run_in_place` path, which meant the benchmark could not use the faster contract we had already introduced for performance-sensitive indicators.

So even though the formula looked reasonable, the execution path was still paying avoidable API overhead.

## Main Goal

Bring `natr` onto the same performance contract as the rest of the optimized ATR family code:

- batch callers should be able to provide output buffers directly
- stream callers should also have an in-place option
- the public high-level API should stay unchanged

## What Changed

### 1. `natr` now implements `run_in_place`

In [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/natr.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/natr.rs), batch execution can now write directly into caller-provided output buffers.

That removes one more case where the benchmark had to fall back to the allocation-heavy default path.

### 2. `NatrStream` now implements `feed_in_place`

The same file now also adds an in-place stream path for `natr`, so the stream contract is aligned with the rest of the performance-sensitive interface layer.

This does not magically make `atr` or the whole ATR family fast, but it removes one more avoidable layer of overhead.

## Performance Outcome

After this change:

- `natr` is still slower than C
- but it is no longer paying the “missing in-place path” tax

That is an important distinction.

Now the remaining gap is much more likely to be about:

- the math and state update shape
- not just the outer API contract

In performance work, this is real progress because it narrows the problem.

## Rust Knowledge Nugget 1: a correct direct formula is not enough if it still uses the slow contract

Beginners often think:

> “If the loop is already direct, then the performance path must already be good.”

But that is only true if the surrounding API contract is also direct.

If the benchmark still has to:

- allocate result containers
- return owned vectors
- copy outputs back out

then the indicator can still look slower than it should.

So always ask both questions:

- is the math kernel direct?
- is the caller contract also direct?

## Rust Knowledge Nugget 2: optimization often proceeds by removing one class of overhead at a time

This commit did not make `natr` faster than C. That is fine.

What it did do was remove one class of overhead cleanly:

- fallback-to-owned-output overhead

Once that is gone, the next benchmark becomes easier to interpret.

This is a good beginner lesson in systems work:

- first remove the obvious architectural tax
- then measure again
- then decide whether the remaining gap is worth deeper algorithmic tuning

## Why This Matters For Human + AI Collaboration

This kind of change is easy to dismiss if you only look at the final ratio and ask, “Did it beat C yet?”

But for collaboration, it is important to record the narrower truth:

- the indicator now uses the right performance contract
- the remaining slowdown is more meaningful than before

That gives the next human or AI a cleaner starting point for the next pass.
