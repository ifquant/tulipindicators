use crate::core::indicator::Indicator;
use crate::indicators::indicator::{
    Adx, Adxr, Apo, Atr, Di, Dm, Dx, Macd, Mom, Natr, Ppo, Rsi, StdDev, StdErr, Stoch, StochRsi,
    Trix, Var, WillR,
};
use crate::indicators::math::{CrossAny, Crossover, Decay, EDecay, Lag, Max, Min, Sum};
use crate::indicators::overlay::{
    AvgPrice, Bbands, Dema, Ema, Hma, Kama, MedPrice, Sma, Tema, Trima, TypPrice, Vidya, Vwma,
    WcPrice, Wilders, Wma, Zlema,
};
use crate::indicators::simple::{
    Abs, Acos, Add, Asin, Atan, Ceil, Cos, Cosh, Div, Exp, Floor, Ln, Log10, Mul, Round, Sin, Sinh,
    Sqrt, Sub, Tan, Tanh, ToDeg, ToRad, Trunc,
};

pub static ABS: Abs = Abs;
pub static ACOS: Acos = Acos;
pub static ADX: Adx = Adx;
pub static ADXR: Adxr = Adxr;
pub static ADD: Add = Add;
pub static APO: Apo = Apo;
pub static ASIN: Asin = Asin;
pub static ATR: Atr = Atr;
pub static ATAN: Atan = Atan;
pub static AVGPRICE: AvgPrice = AvgPrice;
pub static BBANDS: Bbands = Bbands;
pub static CEIL: Ceil = Ceil;
pub static COS: Cos = Cos;
pub static COSH: Cosh = Cosh;
pub static CROSSANY: CrossAny = CrossAny;
pub static CROSSOVER: Crossover = Crossover;
pub static DECAY: Decay = Decay;
pub static DEMA: Dema = Dema;
pub static DI: Di = Di;
pub static DIV: Div = Div;
pub static DM: Dm = Dm;
pub static DX: Dx = Dx;
pub static EDECAY: EDecay = EDecay;
pub static EMA: Ema = Ema;
pub static EXP: Exp = Exp;
pub static FLOOR: Floor = Floor;
pub static HMA: Hma = Hma;
pub static KAMA: Kama = Kama;
pub static LAG: Lag = Lag;
pub static LN: Ln = Ln;
pub static LOG10: Log10 = Log10;
pub static MACD: Macd = Macd;
pub static MAX: Max = Max;
pub static MEDPRICE: MedPrice = MedPrice;
pub static MIN: Min = Min;
pub static MOM: Mom = Mom;
pub static MUL: Mul = Mul;
pub static NATR: Natr = Natr;
pub static PPO: Ppo = Ppo;
pub static RSI: Rsi = Rsi;
pub static ROUND: Round = Round;
pub static SMA: Sma = Sma;
pub static SIN: Sin = Sin;
pub static SINH: Sinh = Sinh;
pub static STDERR: StdErr = StdErr;
pub static STDDEV: StdDev = StdDev;
pub static STOCH: Stoch = Stoch;
pub static STOCHRSI: StochRsi = StochRsi;
pub static SQRT: Sqrt = Sqrt;
pub static SUB: Sub = Sub;
pub static SUM: Sum = Sum;
pub static TAN: Tan = Tan;
pub static TANH: Tanh = Tanh;
pub static TEMA: Tema = Tema;
pub static TODEG: ToDeg = ToDeg;
pub static TORAD: ToRad = ToRad;
pub static TRIMA: Trima = Trima;
pub static TRIX: Trix = Trix;
pub static TRUNC: Trunc = Trunc;
pub static TYPPRICE: TypPrice = TypPrice;
pub static VAR: Var = Var;
pub static VIDYA: Vidya = Vidya;
pub static VWMA: Vwma = Vwma;
pub static WCPRICE: WcPrice = WcPrice;
pub static WILDERS: Wilders = Wilders;
pub static WILLR: WillR = WillR;
pub static WMA: Wma = Wma;
pub static ZLEMA: Zlema = Zlema;

pub fn all() -> [&'static dyn Indicator; 68] {
    [
        &ABS, &ACOS, &ADD, &ADX, &ADXR, &APO, &ASIN, &ATR, &ATAN, &AVGPRICE, &BBANDS, &CEIL, &COS,
        &COSH, &CROSSANY, &CROSSOVER, &DECAY, &DEMA, &DI, &DIV, &DM, &DX, &EDECAY, &EMA, &EXP,
        &FLOOR, &HMA, &KAMA, &LAG, &LN, &LOG10, &MACD, &MAX, &MEDPRICE, &MIN, &MOM, &MUL, &NATR,
        &PPO, &ROUND, &RSI, &SIN, &SINH, &SMA, &SQRT, &STDERR, &STDDEV, &STOCH, &STOCHRSI, &SUB,
        &SUM, &TAN, &TANH, &TEMA, &TODEG, &TORAD, &TRIMA, &TRIX, &TRUNC, &TYPPRICE, &VAR, &VIDYA,
        &VWMA, &WCPRICE, &WILDERS, &WILLR, &WMA, &ZLEMA,
    ]
}

pub fn find(name: &str) -> Option<&'static dyn Indicator> {
    match name {
        "abs" => Some(&ABS),
        "acos" => Some(&ACOS),
        "add" => Some(&ADD),
        "adx" => Some(&ADX),
        "adxr" => Some(&ADXR),
        "apo" => Some(&APO),
        "asin" => Some(&ASIN),
        "atr" => Some(&ATR),
        "atan" => Some(&ATAN),
        "avgprice" => Some(&AVGPRICE),
        "bbands" => Some(&BBANDS),
        "ceil" => Some(&CEIL),
        "cos" => Some(&COS),
        "cosh" => Some(&COSH),
        "crossany" => Some(&CROSSANY),
        "crossover" => Some(&CROSSOVER),
        "decay" => Some(&DECAY),
        "dema" => Some(&DEMA),
        "di" => Some(&DI),
        "div" => Some(&DIV),
        "dm" => Some(&DM),
        "dx" => Some(&DX),
        "edecay" => Some(&EDECAY),
        "ema" => Some(&EMA),
        "exp" => Some(&EXP),
        "floor" => Some(&FLOOR),
        "hma" => Some(&HMA),
        "kama" => Some(&KAMA),
        "lag" => Some(&LAG),
        "ln" => Some(&LN),
        "log10" => Some(&LOG10),
        "macd" => Some(&MACD),
        "max" => Some(&MAX),
        "medprice" => Some(&MEDPRICE),
        "min" => Some(&MIN),
        "mom" => Some(&MOM),
        "mul" => Some(&MUL),
        "natr" => Some(&NATR),
        "ppo" => Some(&PPO),
        "rsi" => Some(&RSI),
        "round" => Some(&ROUND),
        "sma" => Some(&SMA),
        "sin" => Some(&SIN),
        "sinh" => Some(&SINH),
        "sqrt" => Some(&SQRT),
        "stderr" => Some(&STDERR),
        "stddev" => Some(&STDDEV),
        "stoch" => Some(&STOCH),
        "stochrsi" => Some(&STOCHRSI),
        "sub" => Some(&SUB),
        "sum" => Some(&SUM),
        "tan" => Some(&TAN),
        "tanh" => Some(&TANH),
        "tema" => Some(&TEMA),
        "todeg" => Some(&TODEG),
        "torad" => Some(&TORAD),
        "trima" => Some(&TRIMA),
        "trix" => Some(&TRIX),
        "trunc" => Some(&TRUNC),
        "typprice" => Some(&TYPPRICE),
        "var" => Some(&VAR),
        "vidya" => Some(&VIDYA),
        "vwma" => Some(&VWMA),
        "wcprice" => Some(&WCPRICE),
        "wilders" => Some(&WILDERS),
        "willr" => Some(&WILLR),
        "wma" => Some(&WMA),
        "zlema" => Some(&ZLEMA),
        _ => None,
    }
}
