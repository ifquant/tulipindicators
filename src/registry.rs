use crate::core::indicator::Indicator;
use crate::indicators::indicator::{Atr, Macd, Rsi, Stoch, Trix};
use crate::indicators::overlay::{Bbands, Dema, Ema, Sma, Tema};

pub static ATR: Atr = Atr;
pub static BBANDS: Bbands = Bbands;
pub static DEMA: Dema = Dema;
pub static EMA: Ema = Ema;
pub static MACD: Macd = Macd;
pub static RSI: Rsi = Rsi;
pub static SMA: Sma = Sma;
pub static STOCH: Stoch = Stoch;
pub static TEMA: Tema = Tema;
pub static TRIX: Trix = Trix;

pub fn all() -> [&'static dyn Indicator; 10] {
    [
        &ATR, &BBANDS, &DEMA, &EMA, &MACD, &RSI, &SMA, &STOCH, &TEMA, &TRIX,
    ]
}

pub fn find(name: &str) -> Option<&'static dyn Indicator> {
    match name {
        "atr" => Some(&ATR),
        "bbands" => Some(&BBANDS),
        "dema" => Some(&DEMA),
        "ema" => Some(&EMA),
        "macd" => Some(&MACD),
        "rsi" => Some(&RSI),
        "sma" => Some(&SMA),
        "stoch" => Some(&STOCH),
        "tema" => Some(&TEMA),
        "trix" => Some(&TRIX),
        _ => None,
    }
}
