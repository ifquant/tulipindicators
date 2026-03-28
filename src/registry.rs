use crate::core::indicator::Indicator;
use crate::indicators::indicator::{Adx, Adxr, Atr, Di, Dm, Dx, Macd, Rsi, Stoch, StochRsi, Trix};
use crate::indicators::overlay::{Bbands, Dema, Ema, Sma, Tema, Wilders};

pub static ADX: Adx = Adx;
pub static ADXR: Adxr = Adxr;
pub static ATR: Atr = Atr;
pub static BBANDS: Bbands = Bbands;
pub static DEMA: Dema = Dema;
pub static DI: Di = Di;
pub static DM: Dm = Dm;
pub static DX: Dx = Dx;
pub static EMA: Ema = Ema;
pub static MACD: Macd = Macd;
pub static RSI: Rsi = Rsi;
pub static SMA: Sma = Sma;
pub static STOCH: Stoch = Stoch;
pub static STOCHRSI: StochRsi = StochRsi;
pub static TEMA: Tema = Tema;
pub static TRIX: Trix = Trix;
pub static WILDERS: Wilders = Wilders;

pub fn all() -> [&'static dyn Indicator; 17] {
    [
        &ADX, &ADXR, &ATR, &BBANDS, &DEMA, &DI, &DM, &DX, &EMA, &MACD, &RSI, &SMA, &STOCH,
        &STOCHRSI, &TEMA, &TRIX, &WILDERS,
    ]
}

pub fn find(name: &str) -> Option<&'static dyn Indicator> {
    match name {
        "adx" => Some(&ADX),
        "adxr" => Some(&ADXR),
        "atr" => Some(&ATR),
        "bbands" => Some(&BBANDS),
        "dema" => Some(&DEMA),
        "di" => Some(&DI),
        "dm" => Some(&DM),
        "dx" => Some(&DX),
        "ema" => Some(&EMA),
        "macd" => Some(&MACD),
        "rsi" => Some(&RSI),
        "sma" => Some(&SMA),
        "stoch" => Some(&STOCH),
        "stochrsi" => Some(&STOCHRSI),
        "tema" => Some(&TEMA),
        "trix" => Some(&TRIX),
        "wilders" => Some(&WILDERS),
        _ => None,
    }
}
