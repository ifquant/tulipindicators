//! Rust bindings and stateful APIs for Tulip Indicators.
//!
//! The crate keeps the existing high-performance batch layer:
//!
//! - [`Indicator::run`]
//! - [`Indicator::run_in_place`]
//!
//! On top of that batch layer, it also exposes a stateful incremental API:
//!
//! - typed state wrappers such as [`RsiState`] and [`MacdState`]
//! - runtime-selected [`DynamicIndicatorState`]
//! - fixed-capacity history access through [`IndicatorState::latest`] and
//!   [`IndicatorState::get`]
//!
//! For a longer guide, see the repository document:
//! `tutorials/state-api.md`.
//!
//! # Typed state example
//!
//! ```
//! use tulipindicators::{IndicatorState, Real, Rsi};
//!
//! let closes: Vec<Real> = vec![
//!     100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 103.5, 105.0, 106.0, 105.5,
//! ];
//! let mut state = Rsi::state(&[3.0], 16)?;
//!
//! let produced = state.seed(&closes)?;
//! assert!(produced <= closes.len());
//!
//! let latest = state.update(106.5);
//! assert_eq!(latest, state.latest());
//! let _previous = state.get(1);
//! # Ok::<(), tulipindicators::IndicatorError>(())
//! ```
//!
//! # Dynamic state example
//!
//! ```
//! use tulipindicators::{DynamicIndicatorState, IndicatorStateFactory, Real, RSI};
//!
//! let closes: Vec<Real> = vec![
//!     100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 103.5, 105.0, 106.0, 105.5,
//! ];
//!
//! let mut by_name = DynamicIndicatorState::from_name("rsi", &[3.0], 16)?;
//! by_name.seed_columns(&[&closes])?;
//! let next = [106.5];
//! let _ = by_name.update(&next)?;
//!
//! let mut by_factory = RSI.dynamic_state(&[3.0], 16)?;
//! by_factory.seed_columns(&[&closes])?;
//! let _ = by_factory.update(&next)?;
//! assert_eq!(by_factory.latest(), by_name.latest());
//! # Ok::<(), tulipindicators::IndicatorError>(())
//! ```
//!
pub mod benchmark;
pub mod candles;
pub mod core;
pub mod indicators;
pub mod registry;
pub mod state;

pub use crate::candles::{
    all_candles, candle_count, find_candle, get_candle_info, run_candle_named, run_candle_pattern,
    run_candles, CandleConfig, CandleHit, CandleInfo, CandleResult, CandleSet,
    TC_ABANDONED_BABY_BEAR, TC_ABANDONED_BABY_BULL, TC_ALL, TC_BIG_BLACK_CANDLE,
    TC_BIG_WHITE_CANDLE, TC_BLACK_MARUBOZU, TC_DOJI, TC_DRAGONFLY_DOJI, TC_ENGULFING_BEAR,
    TC_ENGULFING_BULL, TC_EVENING_DOJI_STAR, TC_EVENING_STAR, TC_FOUR_PRICE_DOJI,
    TC_GRAVESTONE_DOJI, TC_HAMMER, TC_HANGING_MAN, TC_INVERTED_HAMMER, TC_LONG_LEGGED_DOJI,
    TC_MARUBOZU, TC_MORNING_DOJI_STAR, TC_MORNING_STAR, TC_NONE, TC_SHOOTING_STAR, TC_SPINNING_TOP,
    TC_STAR, TC_THREE_BLACK_CROWS, TC_THREE_WHITE_SOLDIERS, TC_WHITE_MARUBOZU,
};
pub use crate::core::error::IndicatorError;
pub use crate::core::indicator::{Indicator, IndicatorMetadata, IndicatorStream};
pub use crate::core::types::{IndicatorCategory, Real};
pub use crate::indicators::indicator::{
    Adx, AdxState, Adxr, AdxrState, Atr, AtrState, Di, DiState, Dm, DmState, Dx, DxState, Macd,
    MacdState, Natr, NatrState, Ppo, PpoState, Rsi, RsiState, Stoch, StochState,
};
pub use crate::indicators::overlay::{Ema, EmaState, Sma, SmaState, Wilders, WildersState};
pub use crate::registry::{all, find, ATR, BBANDS, DEMA, EMA, MACD, RSI, SMA, STOCH, TEMA, TRIX};
pub use crate::state::{DynamicIndicatorState, IndicatorState, IndicatorStateFactory};
