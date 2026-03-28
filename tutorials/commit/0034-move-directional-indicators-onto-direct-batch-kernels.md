# Move Directional Indicators Onto Direct Batch Kernels

## Background

Another remaining hotspot cluster lived in the directional indicator family:

- `dm`
- `di`
- `adx`

These three are closely related. They all depend on directional movement updates over high/low data, and in the case of `di` and `adx`, they build more calculations on top of that same rolling structure.

The Rust version still used state-machine style batch execution for this family. That was correct, but it meant batch mode was carrying more machinery than the C implementation, which uses thin direct loops.

## Main Goal

Move the directional batch paths closer to the C execution model:

- direct batch loops
- direct output writes
- keep stream support unchanged
- improve the whole family together instead of tuning one file at a time

## What Changed

### 1. `dm` now uses a direct batch kernel

In [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/dm.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/dm.rs), batch mode now computes smoothed positive and negative directional movement directly.

That means:

- no batch-time state object setup
- no per-sample `Option` path in the batch loop
- direct writes into the caller’s output buffers

### 2. `di` now uses a direct batch kernel

In [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/di.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/di.rs), batch mode now computes:

- true range smoothing
- directional movement smoothing
- `+DI` / `-DI` outputs

all in one direct loop, much closer to the C implementation shape.

### 3. `adx` now builds from a direct batch loop too

In [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/adx.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/adx.rs), batch mode now accumulates the directional ratio and ADX smoothing directly instead of routing through batch-time state machines.

This produced the strongest immediate win in the family.

## Performance Outcome

On the `release` C-vs-Rust compare:

- `adx` moved from a large regression to parity or faster
- `dm` improved substantially, but still trails C
- `di` improved substantially, but still trails C

That is still a good result. In performance work, it is common for one family member to fall quickly while others need a follow-up pass.

## Rust Knowledge Nugget 1: improving a family does not mean every member lands equally

A beginner may expect that if three indicators share a structure, one optimization pass should improve all three by the same amount.

In real systems work, that is rarely true.

Why?

Because each family member may stack different extra costs on top of the shared core:

- more divisions
- more smoothing
- more outputs
- more branching

So the right question is not:

> “Did every member improve equally?”

It is:

> “Did this pass remove the shared overhead we intended to remove?”

If yes, the pass was successful even if one member still needs more work.

## Rust Knowledge Nugget 2: borrow splitting is a normal Rust skill, not a workaround

This commit also needed a small but classic Rust fix: writing to two output slices at once.

Rust does not let you take overlapping mutable borrows casually. So code like:

```rust
&mut outputs[0]
&mut outputs[1]
```

can fail if Rust cannot prove they are disjoint.

The idiomatic fix is to split the slice first:

```rust
let (left, right) = outputs.split_at_mut(1);
```

That tells Rust the borrows are separate, and now both mutable references are allowed.

This is one of the most useful beginner lessons in Rust: many “borrow checker fights” are really requests to make aliasing structure explicit.

## Why This Matters For Human + AI Collaboration

This was another good example of a productive optimization pattern:

1. find a hotspot family
2. align its batch path with the C execution model
3. measure the whole family again
4. record which members still need follow-up

That gives the next human or AI a clear checkpoint:

- `adx` is largely recovered
- `dm` and `di` are better, but not finished

That is much more useful than pretending the whole family is done.
