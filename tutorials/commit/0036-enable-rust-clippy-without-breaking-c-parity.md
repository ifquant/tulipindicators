# Enable Rust Clippy Without Breaking C Parity

## Background

The Rust crate had already grown enough that it needed a standard static check, but `cargo clippy` was not actually usable as a routine command yet.

There were two separate problems:

1. some real clippy errors blocked the command entirely
2. a naive fix for one of those errors changed numeric behavior and broke parity with the C implementation

That second point is the important one. In a migration project like this, “make the linter happy” is not allowed to silently move the math.

## Main Goal

Make the Rust project support `cargo clippy` as a normal development command while preserving the existing C/Rust numeric alignment.

That means:

- `cargo clippy` should run successfully
- `cargo test` should still pass
- clippy-driven cleanups must not change indicator outputs unless that change is explicitly intended

## What Changed

### 1. Fixed the clippy blockers

This commit removes the issues that caused `cargo clippy --all-targets --all-features` to fail:

- unnecessary `.into_iter()` calls in the benchmark entrypoints
- needless `Ok(...?)` wrappers in a few Rust indicator paths
- approximate constant errors in the oscillator code

These were low-risk structural cleanups, not behavior changes.

### 2. Preserved legacy numeric behavior where it matters

The most important fix was in [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/oscillators.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/oscillators.rs).

Replacing the historical `3.1415926` constant with `std::f64::consts::PI` made clippy happy, but it also caused a tiny drift in `msw`, which was enough to fail the C parity test.

So the final fix was:

- keep the legacy constant value for compatibility
- scope `#[allow(clippy::approx_constant)]` tightly to that constant

That is the right tradeoff for a migration project: be explicit about the exception instead of pretending the math is interchangeable.

### 3. Documented clippy as a standard repo command

[`/Users/dev/workspace2/hc_apps/tulipindicators/AGENTS.md`](/Users/dev/workspace2/hc_apps/tulipindicators/AGENTS.md) now includes:

- `cargo clippy --all-targets --all-features` in the command list
- guidance to run it when Rust structure, constants, error handling, or public interfaces change

So clippy is now part of the documented workflow, not just an ad hoc command.

## What “support clippy” means here

It does **not** mean every current warning is fixed.

It means:

- the project can run clippy successfully
- the command is documented
- the remaining output is a warning backlog, not a hard blocker

That is the correct first milestone. Clearing the whole warning backlog is a separate cleanup project.

## Rust Knowledge Nugget 1: static analysis is not automatically “safe” for numeric migration work

Beginners often assume:

> “If the linter suggests it, it must be better.”

That is often true for readability and safety, but not automatically true for numeric compatibility.

In this project, replacing an approximate constant with the canonical standard-library constant looked cleaner, but it changed outputs enough to break parity tests.

So the right workflow is:

1. apply the cleanup idea
2. run tests
3. if parity breaks, keep the compatible behavior and document the exception explicitly

## Rust Knowledge Nugget 2: a local `allow` is often better than a global downgrade

When a lint conflicts with intentional project behavior, the best response is usually:

- local exception
- narrow scope
- clear reason

That is better than:

- globally disabling the lint
- or changing behavior just to silence the tool

Here, the local `allow(clippy::approx_constant)` documents exactly where compatibility beats stylistic cleanup.

## Why This Matters For Human + AI Collaboration

This is exactly the kind of repo hygiene work that helps humans keep up with AI changes:

- there is now a documented static check
- the check actually runs
- the exception to the check is explicit and justified

That gives future contributors a much clearer rule:

- use clippy normally
- but do not trade away numeric parity casually in order to satisfy a linter
