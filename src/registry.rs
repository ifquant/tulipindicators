use crate::core::indicator::Indicator;
use crate::indicators::indicator::{
    Adx, Adxr, Atr, Di, Dm, Dx, Macd, Rsi, StdDev, StdErr, Stoch, StochRsi, Trix, Var,
};
use crate::indicators::overlay::{Bbands, Dema, Ema, Hma, Kama, Sma, Tema, Vidya, Wilders, Wma};

pub static ADX: Adx = Adx;
pub static ADXR: Adxr = Adxr;
pub static ATR: Atr = Atr;
pub static BBANDS: Bbands = Bbands;
pub static DEMA: Dema = Dema;
pub static DI: Di = Di;
pub static DM: Dm = Dm;
pub static DX: Dx = Dx;
pub static EMA: Ema = Ema;
pub static HMA: Hma = Hma;
pub static KAMA: Kama = Kama;
pub static MACD: Macd = Macd;
pub static RSI: Rsi = Rsi;
pub static SMA: Sma = Sma;
pub static STDERR: StdErr = StdErr;
pub static STDDEV: StdDev = StdDev;
pub static STOCH: Stoch = Stoch;
pub static STOCHRSI: StochRsi = StochRsi;
pub static TEMA: Tema = Tema;
pub static TRIX: Trix = Trix;
pub static VAR: Var = Var;
pub static VIDYA: Vidya = Vidya;
pub static WILDERS: Wilders = Wilders;
pub static WMA: Wma = Wma;

pub fn all() -> [&'static dyn Indicator; 24] {
    [
        &ADX, &ADXR, &ATR, &BBANDS, &DEMA, &DI, &DM, &DX, &EMA, &HMA, &KAMA, &MACD, &RSI, &SMA,
        &STDERR, &STDDEV, &STOCH, &STOCHRSI, &TEMA, &TRIX, &VAR, &VIDYA, &WILDERS, &WMA,
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
        "hma" => Some(&HMA),
        "kama" => Some(&KAMA),
        "macd" => Some(&MACD),
        "rsi" => Some(&RSI),
        "sma" => Some(&SMA),
        "stderr" => Some(&STDERR),
        "stddev" => Some(&STDDEV),
        "stoch" => Some(&STOCH),
        "stochrsi" => Some(&STOCHRSI),
        "tema" => Some(&TEMA),
        "trix" => Some(&TRIX),
        "var" => Some(&VAR),
        "vidya" => Some(&VIDYA),
        "wilders" => Some(&WILDERS),
        "wma" => Some(&WMA),
        _ => None,
    }
}
