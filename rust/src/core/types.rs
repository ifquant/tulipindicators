//! Core numeric and classification types shared by the public API.

/// Floating-point type used by indicator inputs, options, and outputs.
pub type Real = f64;

/// Broad indicator family used in metadata and registry lookups.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndicatorCategory {
    /// Indicators that sit on top of the price series, such as moving averages and bands.
    Overlay,
    /// Directional, momentum, volatility, and trend indicators.
    Indicator,
    /// Pure math transforms that operate on one or more numeric series.
    Math,
    /// Single-series transforms with minimal input structure.
    Simple,
    /// Indicators that compare or combine multiple series.
    Comparative,
}
