use crate::core::indicator::Indicator;
use crate::indicators::indicator::{
    Ad, AdOsc, Adx, Adxr, Ao, Apo, Aroon, AroonOsc, Atr, Bop, Cci, Cmf, Cmo, Copp, Cvi, Di, Dm,
    Dpo, Dx, Emv, Fi, Fisher, Fosc, Kst, Kvo, LinReg, LinRegAngle, LinRegIntercept, LinRegSlope,
    Macd, MacdFix, MarketFi, Mass, Md, Mfi, Mom, Msw, Natr, Nvi, Obv, Pfe, Posc, Ppo, Psar, Pvi,
    Qstick, Rmi, Roc, Rocr, Rocr100, Rsi, Rvi, Smi, StdDev, StdErr, Stoch, StochRsi, Tr, Trix, Tsf,
    Tsi, UltOsc, Var, Vhf, Volatility, Vosc, Wad, WillR,
};
use crate::indicators::math::{
    Beta, Correl, CrossAny, Crossover, Decay, EDecay, Lag, Max, MaxIndex, MidPoint, Min, MinIndex,
    MinMax, MinMaxIndex, Sum,
};
use crate::indicators::overlay::{
    Abands, Alma, AvgPrice, Bbands, Ce, Dc, Dema, Ema, Hma, Ikhts, Kama, Kc, Mama, MedPrice,
    MidPrice, Pbands, Pc, Rmta, Sma, Tema, Trima, TypPrice, Vidya, Vwap, Vwma, WcPrice, Wilders,
    Wma, Zlema,
};
use crate::indicators::simple::{
    Abs, Acos, Add, Asin, Atan, Ceil, Cos, Cosh, Div, Exp, Floor, Ln, Log10, Mul, Round, Sin, Sinh,
    Sqrt, Sub, Tan, Tanh, ToDeg, ToRad, Trunc,
};

pub static ABS: Abs = Abs;
pub static ACOS: Acos = Acos;
pub static ABANDS: Abands = Abands;
pub static ALMA: Alma = Alma;
pub static AD: Ad = Ad;
pub static ADOSC: AdOsc = AdOsc;
pub static ADX: Adx = Adx;
pub static ADXR: Adxr = Adxr;
pub static ADD: Add = Add;
pub static AO: Ao = Ao;
pub static APO: Apo = Apo;
pub static AROON: Aroon = Aroon;
pub static AROONOSC: AroonOsc = AroonOsc;
pub static ASIN: Asin = Asin;
pub static ATR: Atr = Atr;
pub static ATAN: Atan = Atan;
pub static AVGPRICE: AvgPrice = AvgPrice;
pub static BBANDS: Bbands = Bbands;
pub static BETA: Beta = Beta;
pub static BOP: Bop = Bop;
pub static CE: Ce = Ce;
pub static CCI: Cci = Cci;
pub static CEIL: Ceil = Ceil;
pub static CMF: Cmf = Cmf;
pub static CMO: Cmo = Cmo;
pub static COPP: Copp = Copp;
pub static COS: Cos = Cos;
pub static COSH: Cosh = Cosh;
pub static CORREL: Correl = Correl;
pub static CROSSANY: CrossAny = CrossAny;
pub static CROSSOVER: Crossover = Crossover;
pub static CVI: Cvi = Cvi;
pub static DECAY: Decay = Decay;
pub static DEMA: Dema = Dema;
pub static DI: Di = Di;
pub static DIV: Div = Div;
pub static DM: Dm = Dm;
pub static DPO: Dpo = Dpo;
pub static DX: Dx = Dx;
pub static DC: Dc = Dc;
pub static EDECAY: EDecay = EDecay;
pub static EMA: Ema = Ema;
pub static EMV: Emv = Emv;
pub static EXP: Exp = Exp;
pub static FI: Fi = Fi;
pub static FISHER: Fisher = Fisher;
pub static FLOOR: Floor = Floor;
pub static FOSC: Fosc = Fosc;
pub static HMA: Hma = Hma;
pub static IKHTS: Ikhts = Ikhts;
pub static KAMA: Kama = Kama;
pub static KC: Kc = Kc;
pub static KVO: Kvo = Kvo;
pub static LAG: Lag = Lag;
pub static LINREG: LinReg = LinReg;
pub static LINEARREGANGLE: LinRegAngle = LinRegAngle;
pub static LINREGINTERCEPT: LinRegIntercept = LinRegIntercept;
pub static LINREGSLOPE: LinRegSlope = LinRegSlope;
pub static LN: Ln = Ln;
pub static LOG10: Log10 = Log10;
pub static MACD: Macd = Macd;
pub static MACDFIX: MacdFix = MacdFix;
pub static MAMA: Mama = Mama;
pub static MARKETFI: MarketFi = MarketFi;
pub static MASS: Mass = Mass;
pub static MAX: Max = Max;
pub static MAXINDEX: MaxIndex = MaxIndex;
pub static MD: Md = Md;
pub static MEDPRICE: MedPrice = MedPrice;
pub static MIDPOINT: MidPoint = MidPoint;
pub static MIDPRICE: MidPrice = MidPrice;
pub static MFI: Mfi = Mfi;
pub static MIN: Min = Min;
pub static MININDEX: MinIndex = MinIndex;
pub static MINMAX: MinMax = MinMax;
pub static MINMAXINDEX: MinMaxIndex = MinMaxIndex;
pub static MOM: Mom = Mom;
pub static MSW: Msw = Msw;
pub static MUL: Mul = Mul;
pub static NATR: Natr = Natr;
pub static NVI: Nvi = Nvi;
pub static OBV: Obv = Obv;
pub static PFE: Pfe = Pfe;
pub static POSC: Posc = Posc;
pub static PPO: Ppo = Ppo;
pub static PBANDS: Pbands = Pbands;
pub static PC: Pc = Pc;
pub static PSAR: Psar = Psar;
pub static PVI: Pvi = Pvi;
pub static QSTICK: Qstick = Qstick;
pub static RMI: Rmi = Rmi;
pub static ROC: Roc = Roc;
pub static ROCR: Rocr = Rocr;
pub static ROCR100: Rocr100 = Rocr100;
pub static RMTA: Rmta = Rmta;
pub static RSI: Rsi = Rsi;
pub static RVI: Rvi = Rvi;
pub static ROUND: Round = Round;
pub static SMA: Sma = Sma;
pub static SIN: Sin = Sin;
pub static SINH: Sinh = Sinh;
pub static SMI: Smi = Smi;
pub static SQRT: Sqrt = Sqrt;
pub static STDERR: StdErr = StdErr;
pub static STDDEV: StdDev = StdDev;
pub static STOCH: Stoch = Stoch;
pub static STOCHRSI: StochRsi = StochRsi;
pub static SUB: Sub = Sub;
pub static SUM: Sum = Sum;
pub static TAN: Tan = Tan;
pub static TANH: Tanh = Tanh;
pub static TEMA: Tema = Tema;
pub static TODEG: ToDeg = ToDeg;
pub static TORAD: ToRad = ToRad;
pub static TR: Tr = Tr;
pub static TRIMA: Trima = Trima;
pub static TRIX: Trix = Trix;
pub static TSI: Tsi = Tsi;
pub static TRUNC: Trunc = Trunc;
pub static TSF: Tsf = Tsf;
pub static KST: Kst = Kst;
pub static TYPPRICE: TypPrice = TypPrice;
pub static ULTOSC: UltOsc = UltOsc;
pub static VAR: Var = Var;
pub static VHF: Vhf = Vhf;
pub static VIDYA: Vidya = Vidya;
pub static VOLATILITY: Volatility = Volatility;
pub static VOSC: Vosc = Vosc;
pub static VWAP: Vwap = Vwap;
pub static VWMA: Vwma = Vwma;
pub static WAD: Wad = Wad;
pub static WCPRICE: WcPrice = WcPrice;
pub static WILDERS: Wilders = Wilders;
pub static WILLR: WillR = WillR;
pub static WMA: Wma = Wma;
pub static ZLEMA: Zlema = Zlema;

pub fn all() -> [&'static dyn Indicator; 136] {
    [
        &ABS,
        &ACOS,
        &ABANDS,
        &ALMA,
        &AD,
        &ADOSC,
        &ADD,
        &ADX,
        &ADXR,
        &AO,
        &APO,
        &AROON,
        &AROONOSC,
        &ASIN,
        &ATR,
        &ATAN,
        &AVGPRICE,
        &BBANDS,
        &BETA,
        &BOP,
        &CE,
        &CCI,
        &CEIL,
        &CMF,
        &CMO,
        &COPP,
        &COS,
        &COSH,
        &CORREL,
        &CROSSANY,
        &CROSSOVER,
        &CVI,
        &DECAY,
        &DEMA,
        &DI,
        &DIV,
        &DC,
        &DM,
        &DPO,
        &DX,
        &EDECAY,
        &EMA,
        &EMV,
        &EXP,
        &FI,
        &FISHER,
        &FLOOR,
        &FOSC,
        &HMA,
        &IKHTS,
        &KAMA,
        &KC,
        &KVO,
        &LAG,
        &LINREG,
        &LINEARREGANGLE,
        &LINREGINTERCEPT,
        &LINREGSLOPE,
        &LN,
        &LOG10,
        &MACD,
        &MACDFIX,
        &MAMA,
        &MARKETFI,
        &MASS,
        &MAX,
        &MAXINDEX,
        &MD,
        &MEDPRICE,
        &MIDPOINT,
        &MIDPRICE,
        &MFI,
        &MIN,
        &MININDEX,
        &MINMAX,
        &MINMAXINDEX,
        &MOM,
        &MSW,
        &MUL,
        &NATR,
        &NVI,
        &OBV,
        &PFE,
        &POSC,
        &PPO,
        &PBANDS,
        &PC,
        &PSAR,
        &PVI,
        &QSTICK,
        &RMI,
        &ROC,
        &ROCR,
        &ROCR100,
        &RMTA,
        &RSI,
        &RVI,
        &ROUND,
        &SIN,
        &SINH,
        &SMA,
        &SMI,
        &SQRT,
        &STDERR,
        &STDDEV,
        &STOCH,
        &STOCHRSI,
        &SUB,
        &SUM,
        &TAN,
        &TANH,
        &TEMA,
        &TODEG,
        &TORAD,
        &TR,
        &TRIMA,
        &TRIX,
        &TSI,
        &TRUNC,
        &TSF,
        &KST,
        &TYPPRICE,
        &ULTOSC,
        &VAR,
        &VHF,
        &VIDYA,
        &VOLATILITY,
        &VOSC,
        &VWAP,
        &VWMA,
        &WAD,
        &WCPRICE,
        &WILDERS,
        &WILLR,
        &WMA,
        &ZLEMA,
    ]
}

pub fn find(name: &str) -> Option<&'static dyn Indicator> {
    match name {
        "abs" => Some(&ABS),
        "acos" => Some(&ACOS),
        "abands" => Some(&ABANDS),
        "alma" => Some(&ALMA),
        "ad" => Some(&AD),
        "adosc" => Some(&ADOSC),
        "add" => Some(&ADD),
        "adx" => Some(&ADX),
        "adxr" => Some(&ADXR),
        "ao" => Some(&AO),
        "apo" => Some(&APO),
        "aroon" => Some(&AROON),
        "aroonosc" => Some(&AROONOSC),
        "asin" => Some(&ASIN),
        "atr" => Some(&ATR),
        "atan" => Some(&ATAN),
        "avgprice" => Some(&AVGPRICE),
        "bbands" => Some(&BBANDS),
        "beta" => Some(&BETA),
        "bop" => Some(&BOP),
        "ce" => Some(&CE),
        "cci" => Some(&CCI),
        "ceil" => Some(&CEIL),
        "cmf" => Some(&CMF),
        "cmo" => Some(&CMO),
        "copp" => Some(&COPP),
        "cos" => Some(&COS),
        "cosh" => Some(&COSH),
        "correl" => Some(&CORREL),
        "crossany" => Some(&CROSSANY),
        "crossover" => Some(&CROSSOVER),
        "cvi" => Some(&CVI),
        "decay" => Some(&DECAY),
        "dema" => Some(&DEMA),
        "dc" => Some(&DC),
        "di" => Some(&DI),
        "div" => Some(&DIV),
        "dm" => Some(&DM),
        "dpo" => Some(&DPO),
        "dx" => Some(&DX),
        "edecay" => Some(&EDECAY),
        "ema" => Some(&EMA),
        "emv" => Some(&EMV),
        "exp" => Some(&EXP),
        "fi" => Some(&FI),
        "fisher" => Some(&FISHER),
        "floor" => Some(&FLOOR),
        "fosc" => Some(&FOSC),
        "hma" => Some(&HMA),
        "ikhts" => Some(&IKHTS),
        "kama" => Some(&KAMA),
        "kc" => Some(&KC),
        "kvo" => Some(&KVO),
        "lag" => Some(&LAG),
        "linreg" => Some(&LINREG),
        "linearregangle" => Some(&LINEARREGANGLE),
        "linregintercept" => Some(&LINREGINTERCEPT),
        "linregslope" => Some(&LINREGSLOPE),
        "ln" => Some(&LN),
        "log10" => Some(&LOG10),
        "macd" => Some(&MACD),
        "macdfix" => Some(&MACDFIX),
        "mama" => Some(&MAMA),
        "marketfi" => Some(&MARKETFI),
        "mass" => Some(&MASS),
        "max" => Some(&MAX),
        "maxindex" => Some(&MAXINDEX),
        "md" => Some(&MD),
        "medprice" => Some(&MEDPRICE),
        "midpoint" => Some(&MIDPOINT),
        "midprice" => Some(&MIDPRICE),
        "mfi" => Some(&MFI),
        "min" => Some(&MIN),
        "minindex" => Some(&MININDEX),
        "minmax" => Some(&MINMAX),
        "minmaxindex" => Some(&MINMAXINDEX),
        "mom" => Some(&MOM),
        "msw" => Some(&MSW),
        "mul" => Some(&MUL),
        "natr" => Some(&NATR),
        "nvi" => Some(&NVI),
        "obv" => Some(&OBV),
        "pfe" => Some(&PFE),
        "posc" => Some(&POSC),
        "ppo" => Some(&PPO),
        "pbands" => Some(&PBANDS),
        "pc" => Some(&PC),
        "psar" => Some(&PSAR),
        "pvi" => Some(&PVI),
        "qstick" => Some(&QSTICK),
        "rmi" => Some(&RMI),
        "roc" => Some(&ROC),
        "rocr" => Some(&ROCR),
        "rocr100" => Some(&ROCR100),
        "rmta" => Some(&RMTA),
        "rsi" => Some(&RSI),
        "rvi" => Some(&RVI),
        "round" => Some(&ROUND),
        "sma" => Some(&SMA),
        "sin" => Some(&SIN),
        "sinh" => Some(&SINH),
        "smi" => Some(&SMI),
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
        "tr" => Some(&TR),
        "trima" => Some(&TRIMA),
        "trix" => Some(&TRIX),
        "tsi" => Some(&TSI),
        "trunc" => Some(&TRUNC),
        "tsf" => Some(&TSF),
        "kst" => Some(&KST),
        "typprice" => Some(&TYPPRICE),
        "ultosc" => Some(&ULTOSC),
        "var" => Some(&VAR),
        "vhf" => Some(&VHF),
        "vidya" => Some(&VIDYA),
        "volatility" => Some(&VOLATILITY),
        "vosc" => Some(&VOSC),
        "vwap" => Some(&VWAP),
        "vwma" => Some(&VWMA),
        "wad" => Some(&WAD),
        "wcprice" => Some(&WCPRICE),
        "wilders" => Some(&WILDERS),
        "willr" => Some(&WILLR),
        "wma" => Some(&WMA),
        "zlema" => Some(&ZLEMA),
        _ => None,
    }
}
