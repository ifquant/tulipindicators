//! Rust bindings for Tulip Indicators.
//!
//! The public API is split into three layers:
//!
//! - batch indicators through [`Indicator`]
//! - stream indicators through [`IndicatorStream`]
//! - bounded incremental state through [`IndicatorState`] and [`DynamicIndicatorState`]
//!
//! Batch calls are the release-facing default for offline analysis and benchmarks.
//! Use [`Indicator::run`] for owned output, [`Indicator::run_single`] for single-output
//! indicators, and [`Indicator::run_in_place`] when you want caller-owned buffers.
//!
//! Stream and state APIs are for row-by-row or sample-by-sample processing. State
//! history is bounded, and [`IndicatorState::history_capacity`] must be greater than
//! zero. If you do not need indexed history, prefer [`IndicatorStream`] over extra
//! state.
//!
//! For deeper guides and registry details, see `tutorials/indicator-api.md`,
//! `tutorials/state-api.md`, `tutorials/indicator-reference.md`, and the focused
//! examples under `examples/`.
//!
//! # Batch example
//!
//! ```
//! use tulipindicators::{Indicator, Real, Rsi};
//!
//! let closes: Vec<Real> = vec![100.0, 101.0, 102.0, 101.5, 103.0];
//! let values = Rsi.run_single(&[&closes], &[3.0])?;
//! assert!(!values.is_empty());
//! # Ok::<(), tulipindicators::IndicatorError>(())
//! ```
//!
//! # Incremental example
//!
//! ```
//! use tulipindicators::{IndicatorState, Real, Rsi};
//!
//! let closes: Vec<Real> = vec![100.0, 101.0, 102.0, 101.5, 103.0];
//! let mut state = Rsi::state(&[3.0], 8)?;
//! let seeded = state.seed(&closes)?;
//! assert!(seeded <= closes.len());
//! assert_eq!(state.latest(), state.get(0));
//! # Ok::<(), tulipindicators::IndicatorError>(())
//! ```
//!
//! # Stream example
//!
//! ```
//! use tulipindicators::{Indicator, IndicatorStream, Real, Sma};
//!
//! let closes: Vec<Real> = vec![100.0, 101.0, 102.0, 101.5, 103.0];
//! let mut stream = Sma
//!     .create_stream(&[3.0])?
//!     .expect("SMA supports incremental streaming");
//! let values = stream.feed_single(&[&closes])?;
//! assert!(!values.is_empty());
//! # Ok::<(), tulipindicators::IndicatorError>(())
//! ```
//!
//! # Dynamic state example
//!
//! ```
//! use tulipindicators::{DynamicIndicatorState, IndicatorStateFactory, Real, RSI};
//!
//! let closes: Vec<Real> = vec![100.0, 101.0, 102.0, 101.5, 103.0];
//! let mut state = DynamicIndicatorState::from_name("rsi", &[3.0], 8)?;
//! state.seed_columns(&[&closes])?;
//! let _ = state.update(&[103.5])?;
//!
//! let mut typed = RSI.dynamic_state(&[3.0], 8)?;
//! typed.seed_columns(&[&closes])?;
//! let _ = typed.update(&[103.5])?;
//! assert_eq!(typed.latest_ref(), state.latest_ref());
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
