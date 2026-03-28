# Add A Default-On Switch For Rust Clippy

## Background

After making `cargo clippy` usable in the Rust crate, the next practical problem was workflow control.

Sometimes you want the full Rust check stack:

- format check
- tests
- clippy

But sometimes you need a temporary escape hatch, for example:

- you are iterating on a performance spike
- you only want a quick correctness pass
- you want to keep the default strict path, but still have a documented way to bypass clippy intentionally

That is a workflow problem, not a compiler problem.

## Main Goal

Add one standard Rust check entrypoint with a simple switch:

- default behavior: run clippy
- opt-out behavior: allow clippy to be skipped explicitly

This keeps the default bar high without making every temporary exception ad hoc.

## What Changed

### 1. Added `make rust-check`

[`/Users/dev/workspace2/hc_apps/tulipindicators/Makefile`](/Users/dev/workspace2/hc_apps/tulipindicators/Makefile) now includes:

- `make rust-check`

It runs:

- `cargo fmt --all --check`
- `cargo test`
- `cargo clippy --all-targets --all-features`

So there is now one canonical Rust validation command.

### 2. Added a switch to disable clippy explicitly

The same Makefile now supports:

- default: `TI_ENABLE_CLIPPY=1`
- opt-out: `TI_ENABLE_CLIPPY=0 make rust-check`

That means clippy is still on by default, but the bypass is explicit, documented, and easy to audit.

### 3. Documented the rule in `AGENTS.md`

[`/Users/dev/workspace2/hc_apps/tulipindicators/AGENTS.md`](/Users/dev/workspace2/hc_apps/tulipindicators/AGENTS.md) now explains:

- the standard Rust check entrypoint is `make rust-check`
- clippy is enabled by default
- `TI_ENABLE_CLIPPY=0` is the supported temporary override

That matters because human and AI contributors now have the same documented switch.

## Why default-on is the right default

The important design choice here is not “can clippy be skipped?”

It is:

> “What happens when nobody says anything?”

The right answer is:

- checks stay strict by default
- skipping clippy must be intentional

That prevents a temporary exception from quietly becoming the normal workflow.

## Rust Knowledge Nugget 1: good workflow switches are explicit, not magical

A bad switch is one that hides in some undocumented local setup.

A good switch is one that:

- has a visible name
- has a clear default
- is easy to override
- is easy to describe in docs and commit messages

`TI_ENABLE_CLIPPY=0 make rust-check` is good because a reviewer can immediately understand what happened.

## Rust Knowledge Nugget 2: “default strict, explicit escape hatch” is often the best team pattern

This pattern shows up everywhere in engineering:

- strict CI with a manual override
- safe API with an explicit lower-level path
- lint on by default, opt-out only when necessary

Why is it useful?

Because it makes the normal path reliable while keeping room for exceptional work.

That is better than either extreme:

- “no escape hatch at all”
- “everything is optional all the time”

## Why This Matters For Human + AI Collaboration

This repository already relies on AI to move quickly, so the workflow has to help humans keep track of what standards were applied.

With this change:

- the default Rust validation path is obvious
- clippy skipping is possible
- but skipping leaves a visible trace in the command itself

That is exactly the kind of small process design that makes a fast-moving repo easier to manage.
