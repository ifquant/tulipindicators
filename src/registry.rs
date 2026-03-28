use crate::core::indicator::Indicator;
use crate::indicators::overlay::{Ema, Sma};

pub static EMA: Ema = Ema;
pub static SMA: Sma = Sma;

pub fn all() -> [&'static dyn Indicator; 2] {
    [&EMA, &SMA]
}

pub fn find(name: &str) -> Option<&'static dyn Indicator> {
    match name {
        "ema" => Some(&EMA),
        "sma" => Some(&SMA),
        _ => None,
    }
}
