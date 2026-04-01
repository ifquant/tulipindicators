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
    Atr, AtrState, Dm, DmState, Macd, MacdState, Rsi, RsiState,
};
pub use crate::indicators::overlay::{Ema, EmaState, Sma, SmaState};
pub use crate::registry::{all, find, ATR, BBANDS, DEMA, EMA, MACD, RSI, SMA, STOCH, TEMA, TRIX};
pub use crate::state::{DynamicIndicatorState, IndicatorState, IndicatorStateFactory};
