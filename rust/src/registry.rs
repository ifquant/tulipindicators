//! Registry of built-in indicators.

use crate::core::indicator::Indicator;
use crate::indicators::indicator::{
    Ad, AdOsc, Adx, Adxr, Ao, Apo, Aroon, AroonOsc, Atr, Bop, Cci, Cmf, Cmo, Copp, Cvi, Di, Dm,
    Dpo, Dx, Emv, Fi, Fisher, Fosc, HtDcPeriod, HtDcPhase, HtPhasor, HtSine, HtTrendMode,
    HtTrendline, Imi, Kst, Kvo, LinReg, LinRegAngle, LinRegIntercept, LinRegSlope, Macd, MacdExt,
    MacdFix, MarketFi, Mass, Md, Mfi, Mom, Msw, Natr, Nvi, Obv, Pfe, Posc, Ppo, Psar, Pvi, Qstick,
    Rmi, Roc, Rocr, Rocr100, Rsi, Rvi, Sarext, Smi, StdDev, StdErr, Stoch, StochRsi, Tr, Trix, Tsf,
    Tsi, UltOsc, Var, Vhf, Volatility, Vosc, Wad, WillR,
};
use crate::indicators::math::{
    Beta, Correl, CrossAny, Crossover, Decay, EDecay, Lag, Max, MaxIndex, MidPoint, Min, MinIndex,
    MinMax, MinMaxIndex, Sum,
};
use crate::indicators::overlay::{
    Abands, Alma, AvgPrice, Bbands, Ce, Dc, Dema, Ema, Hma, Ikhts, Kama, Kc, Ma, Mama, Mavp,
    MedPrice, MidPrice, Pbands, Pc, Rmta, Sma, Tema, Trima, TypPrice, Vidya, Vwap, Vwma, WcPrice,
    Wilders, Wma, Zlema, T3 as T3Indicator,
};
use crate::indicators::simple::{
    Abs, Acos, Add, Asin, Atan, Ceil, Cos, Cosh, Div, Exp, Floor, Ln, Log10, Mul, Round, Sin, Sinh,
    Sqrt, Sub, Tan, Tanh, ToDeg, ToRad, Trunc,
};

/// Static indicator handle for `abs`.
pub static ABS: Abs = Abs;
/// Static indicator handle for `acos`.
pub static ACOS: Acos = Acos;
/// Static indicator handle for `abands`.
pub static ABANDS: Abands = Abands;
/// Static indicator handle for `alma`.
pub static ALMA: Alma = Alma;
/// Static indicator handle for `ad`.
pub static AD: Ad = Ad;
/// Static indicator handle for `adosc`.
pub static ADOSC: AdOsc = AdOsc;
/// Static indicator handle for `adx`.
pub static ADX: Adx = Adx;
/// Static indicator handle for `adxr`.
pub static ADXR: Adxr = Adxr;
/// Static indicator handle for `add`.
pub static ADD: Add = Add;
/// Static indicator handle for `ao`.
pub static AO: Ao = Ao;
/// Static indicator handle for `apo`.
pub static APO: Apo = Apo;
/// Static indicator handle for `aroon`.
pub static AROON: Aroon = Aroon;
/// Static indicator handle for `aroonosc`.
pub static AROONOSC: AroonOsc = AroonOsc;
/// Static indicator handle for `asin`.
pub static ASIN: Asin = Asin;
/// Static indicator handle for `atr`.
pub static ATR: Atr = Atr;
/// Static indicator handle for `atan`.
pub static ATAN: Atan = Atan;
/// Static indicator handle for `avgprice`.
pub static AVGPRICE: AvgPrice = AvgPrice;
/// Static indicator handle for `bbands`.
pub static BBANDS: Bbands = Bbands;
/// Static indicator handle for `beta`.
pub static BETA: Beta = Beta;
/// Static indicator handle for `bop`.
pub static BOP: Bop = Bop;
/// Static indicator handle for `ce`.
pub static CE: Ce = Ce;
/// Static indicator handle for `cci`.
pub static CCI: Cci = Cci;
/// Static indicator handle for `ceil`.
pub static CEIL: Ceil = Ceil;
/// Static indicator handle for `cmf`.
pub static CMF: Cmf = Cmf;
/// Static indicator handle for `cmo`.
pub static CMO: Cmo = Cmo;
/// Static indicator handle for `copp`.
pub static COPP: Copp = Copp;
/// Static indicator handle for `cos`.
pub static COS: Cos = Cos;
/// Static indicator handle for `cosh`.
pub static COSH: Cosh = Cosh;
/// Static indicator handle for `correl`.
pub static CORREL: Correl = Correl;
/// Static indicator handle for `crossany`.
pub static CROSSANY: CrossAny = CrossAny;
/// Static indicator handle for `crossover`.
pub static CROSSOVER: Crossover = Crossover;
/// Static indicator handle for `cvi`.
pub static CVI: Cvi = Cvi;
/// Static indicator handle for `decay`.
pub static DECAY: Decay = Decay;
/// Static indicator handle for `dema`.
pub static DEMA: Dema = Dema;
/// Static indicator handle for `di`.
pub static DI: Di = Di;
/// Static indicator handle for `div`.
pub static DIV: Div = Div;
/// Static indicator handle for `dm`.
pub static DM: Dm = Dm;
/// Static indicator handle for `dpo`.
pub static DPO: Dpo = Dpo;
/// Static indicator handle for `dx`.
pub static DX: Dx = Dx;
/// Static indicator handle for `dc`.
pub static DC: Dc = Dc;
/// Static indicator handle for `edecay`.
pub static EDECAY: EDecay = EDecay;
/// Static indicator handle for `ema`.
pub static EMA: Ema = Ema;
/// Static indicator handle for `emv`.
pub static EMV: Emv = Emv;
/// Static indicator handle for `exp`.
pub static EXP: Exp = Exp;
/// Static indicator handle for `fi`.
pub static FI: Fi = Fi;
/// Static indicator handle for `fisher`.
pub static FISHER: Fisher = Fisher;
/// Static indicator handle for `floor`.
pub static FLOOR: Floor = Floor;
/// Static indicator handle for `fosc`.
pub static FOSC: Fosc = Fosc;
/// Static indicator handle for `hma`.
pub static HMA: Hma = Hma;
/// Static indicator handle for `ht_dcperiod`.
pub static HT_DCPERIOD: HtDcPeriod = HtDcPeriod;
/// Static indicator handle for `ht_dcphase`.
pub static HT_DCPHASE: HtDcPhase = HtDcPhase;
/// Static indicator handle for `ht_phasor`.
pub static HT_PHASOR: HtPhasor = HtPhasor;
/// Static indicator handle for `ht_sine`.
pub static HT_SINE: HtSine = HtSine;
/// Static indicator handle for `ht_trendline`.
pub static HT_TRENDLINE: HtTrendline = HtTrendline;
/// Static indicator handle for `ht_trendmode`.
pub static HT_TRENDMODE: HtTrendMode = HtTrendMode;
/// Static indicator handle for `imi`.
pub static IMI: Imi = Imi;
/// Static indicator handle for `ikhts`.
pub static IKHTS: Ikhts = Ikhts;
/// Static indicator handle for `kama`.
pub static KAMA: Kama = Kama;
/// Static indicator handle for `kc`.
pub static KC: Kc = Kc;
/// Static indicator handle for `kvo`.
pub static KVO: Kvo = Kvo;
/// Static indicator handle for `lag`.
pub static LAG: Lag = Lag;
/// Static indicator handle for `linreg`.
pub static LINREG: LinReg = LinReg;
/// Static indicator handle for `linearregangle`.
pub static LINEARREGANGLE: LinRegAngle = LinRegAngle;
/// Static indicator handle for `linregintercept`.
pub static LINREGINTERCEPT: LinRegIntercept = LinRegIntercept;
/// Static indicator handle for `linregslope`.
pub static LINREGSLOPE: LinRegSlope = LinRegSlope;
/// Static indicator handle for `ln`.
pub static LN: Ln = Ln;
/// Static indicator handle for `log10`.
pub static LOG10: Log10 = Log10;
/// Static indicator handle for `macd`.
pub static MACD: Macd = Macd;
/// Static indicator handle for `macdext`.
pub static MACDEXT: MacdExt = MacdExt;
/// Static indicator handle for `macdfix`.
pub static MACDFIX: MacdFix = MacdFix;
/// Static indicator handle for `ma`.
pub static MA: Ma = Ma;
/// Static indicator handle for `mama`.
pub static MAMA: Mama = Mama;
/// Static indicator handle for `mavp`.
pub static MAVP: Mavp = Mavp;
/// Static indicator handle for `marketfi`.
pub static MARKETFI: MarketFi = MarketFi;
/// Static indicator handle for `mass`.
pub static MASS: Mass = Mass;
/// Static indicator handle for `max`.
pub static MAX: Max = Max;
/// Static indicator handle for `maxindex`.
pub static MAXINDEX: MaxIndex = MaxIndex;
/// Static indicator handle for `md`.
pub static MD: Md = Md;
/// Static indicator handle for `medprice`.
pub static MEDPRICE: MedPrice = MedPrice;
/// Static indicator handle for `midpoint`.
pub static MIDPOINT: MidPoint = MidPoint;
/// Static indicator handle for `midprice`.
pub static MIDPRICE: MidPrice = MidPrice;
/// Static indicator handle for `mfi`.
pub static MFI: Mfi = Mfi;
/// Static indicator handle for `min`.
pub static MIN: Min = Min;
/// Static indicator handle for `minindex`.
pub static MININDEX: MinIndex = MinIndex;
/// Static indicator handle for `minmax`.
pub static MINMAX: MinMax = MinMax;
/// Static indicator handle for `minmaxindex`.
pub static MINMAXINDEX: MinMaxIndex = MinMaxIndex;
/// Static indicator handle for `mom`.
pub static MOM: Mom = Mom;
/// Static indicator handle for `msw`.
pub static MSW: Msw = Msw;
/// Static indicator handle for `mul`.
pub static MUL: Mul = Mul;
/// Static indicator handle for `natr`.
pub static NATR: Natr = Natr;
/// Static indicator handle for `nvi`.
pub static NVI: Nvi = Nvi;
/// Static indicator handle for `obv`.
pub static OBV: Obv = Obv;
/// Static indicator handle for `pfe`.
pub static PFE: Pfe = Pfe;
/// Static indicator handle for `posc`.
pub static POSC: Posc = Posc;
/// Static indicator handle for `ppo`.
pub static PPO: Ppo = Ppo;
/// Static indicator handle for `pbands`.
pub static PBANDS: Pbands = Pbands;
/// Static indicator handle for `pc`.
pub static PC: Pc = Pc;
/// Static indicator handle for `psar`.
pub static PSAR: Psar = Psar;
/// Static indicator handle for `pvi`.
pub static PVI: Pvi = Pvi;
/// Static indicator handle for `qstick`.
pub static QSTICK: Qstick = Qstick;
/// Static indicator handle for `rmi`.
pub static RMI: Rmi = Rmi;
/// Static indicator handle for `roc`.
pub static ROC: Roc = Roc;
/// Static indicator handle for `rocr`.
pub static ROCR: Rocr = Rocr;
/// Static indicator handle for `rocr100`.
pub static ROCR100: Rocr100 = Rocr100;
/// Static indicator handle for `rmta`.
pub static RMTA: Rmta = Rmta;
/// Static indicator handle for `rsi`.
pub static RSI: Rsi = Rsi;
/// Static indicator handle for `rvi`.
pub static RVI: Rvi = Rvi;
/// Static indicator handle for `round`.
pub static ROUND: Round = Round;
/// Static indicator handle for `sarext`.
pub static SAREXT: Sarext = Sarext;
/// Static indicator handle for `sma`.
pub static SMA: Sma = Sma;
/// Static indicator handle for `t3`.
pub static T3: T3Indicator = T3Indicator;
/// Static indicator handle for `sin`.
pub static SIN: Sin = Sin;
/// Static indicator handle for `sinh`.
pub static SINH: Sinh = Sinh;
/// Static indicator handle for `smi`.
pub static SMI: Smi = Smi;
/// Static indicator handle for `sqrt`.
pub static SQRT: Sqrt = Sqrt;
/// Static indicator handle for `stderr`.
pub static STDERR: StdErr = StdErr;
/// Static indicator handle for `stddev`.
pub static STDDEV: StdDev = StdDev;
/// Static indicator handle for `stoch`.
pub static STOCH: Stoch = Stoch;
/// Static indicator handle for `stochrsi`.
pub static STOCHRSI: StochRsi = StochRsi;
/// Static indicator handle for `sub`.
pub static SUB: Sub = Sub;
/// Static indicator handle for `sum`.
pub static SUM: Sum = Sum;
/// Static indicator handle for `tan`.
pub static TAN: Tan = Tan;
/// Static indicator handle for `tanh`.
pub static TANH: Tanh = Tanh;
/// Static indicator handle for `tema`.
pub static TEMA: Tema = Tema;
/// Static indicator handle for `todeg`.
pub static TODEG: ToDeg = ToDeg;
/// Static indicator handle for `torad`.
pub static TORAD: ToRad = ToRad;
/// Static indicator handle for `tr`.
pub static TR: Tr = Tr;
/// Static indicator handle for `trima`.
pub static TRIMA: Trima = Trima;
/// Static indicator handle for `trix`.
pub static TRIX: Trix = Trix;
/// Static indicator handle for `tsi`.
pub static TSI: Tsi = Tsi;
/// Static indicator handle for `trunc`.
pub static TRUNC: Trunc = Trunc;
/// Static indicator handle for `tsf`.
pub static TSF: Tsf = Tsf;
/// Static indicator handle for `kst`.
pub static KST: Kst = Kst;
/// Static indicator handle for `typprice`.
pub static TYPPRICE: TypPrice = TypPrice;
/// Static indicator handle for `ultosc`.
pub static ULTOSC: UltOsc = UltOsc;
/// Static indicator handle for `var`.
pub static VAR: Var = Var;
/// Static indicator handle for `vhf`.
pub static VHF: Vhf = Vhf;
/// Static indicator handle for `vidya`.
pub static VIDYA: Vidya = Vidya;
/// Static indicator handle for `volatility`.
pub static VOLATILITY: Volatility = Volatility;
/// Static indicator handle for `vosc`.
pub static VOSC: Vosc = Vosc;
/// Static indicator handle for `vwap`.
pub static VWAP: Vwap = Vwap;
/// Static indicator handle for `vwma`.
pub static VWMA: Vwma = Vwma;
/// Static indicator handle for `wad`.
pub static WAD: Wad = Wad;
/// Static indicator handle for `wcprice`.
pub static WCPRICE: WcPrice = WcPrice;
/// Static indicator handle for `wilders`.
pub static WILDERS: Wilders = Wilders;
/// Static indicator handle for `willr`.
pub static WILLR: WillR = WillR;
/// Static indicator handle for `wma`.
pub static WMA: Wma = Wma;
/// Static indicator handle for `zlema`.
pub static ZLEMA: Zlema = Zlema;

/// Return every built-in indicator in registry order.
pub fn all() -> [&'static dyn Indicator; 148] {
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
        &HT_DCPERIOD,
        &HT_DCPHASE,
        &HT_PHASOR,
        &HT_SINE,
        &HT_TRENDLINE,
        &HT_TRENDMODE,
        &IMI,
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
        &MACDEXT,
        &MACDFIX,
        &MA,
        &MAMA,
        &MAVP,
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
        &SAREXT,
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
        &T3,
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

/// Look up a built-in indicator by its lowercase registry name.
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
        "ht_dcperiod" => Some(&HT_DCPERIOD),
        "ht_dcphase" => Some(&HT_DCPHASE),
        "ht_phasor" => Some(&HT_PHASOR),
        "ht_sine" => Some(&HT_SINE),
        "ht_trendline" => Some(&HT_TRENDLINE),
        "ht_trendmode" => Some(&HT_TRENDMODE),
        "imi" => Some(&IMI),
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
        "macdext" => Some(&MACDEXT),
        "macdfix" => Some(&MACDFIX),
        "ma" => Some(&MA),
        "mama" => Some(&MAMA),
        "mavp" => Some(&MAVP),
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
        "sarext" => Some(&SAREXT),
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
        "t3" => Some(&T3),
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
