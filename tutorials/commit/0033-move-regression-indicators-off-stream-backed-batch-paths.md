# Move Regression Indicators Off Stream-Backed Batch Paths

## Background

After the earlier performance passes, one ugly cluster still remained near the top of the regression table:

- `linreg`
- `linregintercept`
- `linregslope`
- `tsf`
- `fosc`

These indicators all belong to the same family. That matters, because a shared structural bug often means a shared optimization opportunity.

The real issue here was not the linear regression math itself. The problem was that Rust batch mode was still using stream-oriented machinery, while the C implementation runs a direct regression loop over the slice.

So even though the formulas matched, the execution model did not.

## Main Goal

Replace stream-backed batch execution for the regression family with a shared direct batch kernel that mirrors the C implementation more closely.

That gives us three benefits at once:

- less overhead
- easier reasoning about performance
- one optimization pass fixes several indicators together

## What Changed

### 1. Added a shared regression batch kernel

In [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/regression.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/regression.rs), the regression-family indicators now share a direct batch helper:

- maintain running `y_sum`
- maintain running weighted `xy_sum`
- compute slope and intercept directly
- write outputs straight into the provided slice

This mirrors the C macro in `trend.h` much more closely than the old stream-backed path.

### 2. `linreg`, `linregintercept`, `linregslope`, and `tsf` now use direct batch + `run_in_place`

These indicators no longer construct a stream object just to compute batch outputs.

That is the real performance win in this commit: batch execution now behaves like batch execution.

### 3. `fosc` now builds on the same direct regression core

`fosc` still derives from the time-series forecast logic, but it now gets that forecast from the same shared batch kernel instead of paying for a stream-backed path first.

That is a good example of performance work that also improves structural consistency.

## Performance Outcome

On the `release` C-vs-Rust benchmark compare:

- `linreg` moved into parity or better
- `linregintercept` moved into parity or better
- `linregslope` moved clearly ahead of C in the tested runs
- `tsf` moved into parity
- `fosc` improved dramatically on smaller inputs and much more modestly on large inputs

The important result is not one single headline number. It is that an entire regression family stopped paying the stream-backed batch tax.

## Rust Knowledge Nugget 1: shared math families are optimization gold mines

A beginner often looks at indicators one by one:

- “optimize this file”
- “then optimize that file”

But a better systems mindset is to look for families that share one mathematical core.

Here, five indicators all depended on the same regression state. Once that was visible, the optimization target stopped being “five separate indicators” and became:

> “build one good batch kernel for the regression family.”

That kind of refactoring is often much higher leverage than chasing one function at a time.

## Rust Knowledge Nugget 2: direct batch kernels are often simpler than stream-backed batch code

It may sound backwards, but for performance-sensitive code the “more direct” solution is often also the simpler one.

Why?

Because the batch kernel can focus on exactly one shape:

- one pass over a contiguous slice
- one rolling update per sample
- one output write per finished window

Stream-backed batch code has to carry extra ideas:

- partial progress
- chunk boundaries
- persistent state transitions

Those are necessary for streaming, but they are extra baggage for batch mode.

## Why This Matters For Human + AI Collaboration

This commit is a strong example of a good performance debugging pattern:

1. identify the hotspot family, not just one indicator
2. compare Rust structure to the C execution model
3. move batch mode onto a shared direct kernel
4. measure the whole family again

That gives a human reviewer a much clearer story than a pile of unrelated micro-optimizations, and it gives the next AI a better foundation for the next round.
