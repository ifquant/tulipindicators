pub mod benchmark;
pub mod core;
pub mod indicators;
pub mod registry;

pub use crate::core::error::IndicatorError;
pub use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
pub use crate::core::types::{IndicatorCategory, Real};
pub use crate::registry::{all, find, ATR, BBANDS, DEMA, EMA, MACD, RSI, SMA, STOCH, TEMA, TRIX};
