use crate::core::indicator::Indicator;
use crate::indicators::indicator::{Atr, Macd, Rsi};
use crate::indicators::overlay::{Bbands, Ema, Sma};

pub static ATR: Atr = Atr;
pub static BBANDS: Bbands = Bbands;
pub static EMA: Ema = Ema;
pub static MACD: Macd = Macd;
pub static RSI: Rsi = Rsi;
pub static SMA: Sma = Sma;

pub fn all() -> [&'static dyn Indicator; 6] {
    [&ATR, &BBANDS, &EMA, &MACD, &RSI, &SMA]
}

pub fn find(name: &str) -> Option<&'static dyn Indicator> {
    match name {
        "atr" => Some(&ATR),
        "bbands" => Some(&BBANDS),
        "ema" => Some(&EMA),
        "macd" => Some(&MACD),
        "rsi" => Some(&RSI),
        "sma" => Some(&SMA),
        _ => None,
    }
}
