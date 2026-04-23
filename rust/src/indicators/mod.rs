//! Indicator module map for the Rust port.
//!
//! The public layout mirrors the indicator families used by the crate:
//! - `indicator` covers the named oscillator and indicator set.
//! - `overlay` covers moving averages and other series that sit on top of price.
//! - `math` covers vector utilities and rolling math helpers.
//! - `simple` covers per-sample transforms with no lookback.
//!
//! Naming stays close to the underlying indicator families. Module names group
//! related indicators by behavior, while the exported type names keep the short
//! indicator abbreviations that callers expect.

pub mod indicator;
pub mod math;
pub mod overlay;
mod shared;
pub mod simple;
