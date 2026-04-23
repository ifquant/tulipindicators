//! Candlestick pattern helpers and registry data.

use crate::core::error::IndicatorError;
use crate::core::types::Real;

/// Bitset type used to combine candlestick pattern matches.
pub type CandleSet = u64;

/// No candlestick patterns matched.
pub const TC_NONE: CandleSet = 0;
/// All supported candlestick patterns.
pub const TC_ALL: CandleSet = (1u64 << 26) - 1;
pub const TC_ABANDONED_BABY_BEAR: CandleSet = 1u64 << 0;
pub const TC_ABANDONED_BABY_BULL: CandleSet = 1u64 << 1;
pub const TC_BIG_BLACK_CANDLE: CandleSet = 1u64 << 2;
pub const TC_BIG_WHITE_CANDLE: CandleSet = 1u64 << 3;
pub const TC_BLACK_MARUBOZU: CandleSet = 1u64 << 4;
pub const TC_DOJI: CandleSet = 1u64 << 5;
pub const TC_DRAGONFLY_DOJI: CandleSet = 1u64 << 6;
pub const TC_ENGULFING_BEAR: CandleSet = 1u64 << 7;
pub const TC_ENGULFING_BULL: CandleSet = 1u64 << 8;
pub const TC_EVENING_DOJI_STAR: CandleSet = 1u64 << 9;
pub const TC_EVENING_STAR: CandleSet = 1u64 << 10;
pub const TC_FOUR_PRICE_DOJI: CandleSet = 1u64 << 11;
pub const TC_GRAVESTONE_DOJI: CandleSet = 1u64 << 12;
pub const TC_HAMMER: CandleSet = 1u64 << 13;
pub const TC_HANGING_MAN: CandleSet = 1u64 << 14;
pub const TC_INVERTED_HAMMER: CandleSet = 1u64 << 15;
pub const TC_LONG_LEGGED_DOJI: CandleSet = 1u64 << 16;
pub const TC_MARUBOZU: CandleSet = 1u64 << 17;
pub const TC_MORNING_DOJI_STAR: CandleSet = 1u64 << 18;
pub const TC_MORNING_STAR: CandleSet = 1u64 << 19;
pub const TC_SHOOTING_STAR: CandleSet = 1u64 << 20;
pub const TC_SPINNING_TOP: CandleSet = 1u64 << 21;
pub const TC_STAR: CandleSet = 1u64 << 22;
pub const TC_THREE_BLACK_CROWS: CandleSet = 1u64 << 23;
pub const TC_THREE_WHITE_SOLDIERS: CandleSet = 1u64 << 24;
pub const TC_WHITE_MARUBOZU: CandleSet = 1u64 << 25;

/// Thresholds used when evaluating candle pattern rules.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CandleConfig {
    /// Number of samples used when deriving body and wick averages.
    pub period: usize,
    /// Threshold for candles with effectively no body.
    pub body_none: Real,
    /// Threshold for short candle bodies.
    pub body_short: Real,
    /// Threshold for long candle bodies.
    pub body_long: Real,
    /// Threshold for wicks that are treated as negligible.
    pub wick_none: Real,
    /// Threshold for wicks that are treated as long.
    pub wick_long: Real,
    /// Threshold for values that are considered near another reference price.
    pub near: Real,
}

impl Default for CandleConfig {
    fn default() -> Self {
        Self {
            period: 10,
            body_none: 0.05,
            body_short: 0.5,
            body_long: 1.4,
            wick_none: 0.05,
            wick_long: 0.6,
            near: 0.3,
        }
    }
}

/// One candle pattern hit at a specific index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CandleHit {
    /// Zero-based input index where the pattern matched.
    pub index: usize,
    /// Bitset of matching patterns at this index.
    pub patterns: CandleSet,
}

/// Collected candlestick matches for a run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandleResult {
    hits: Vec<CandleHit>,
    pattern_count: usize,
}

impl CandleResult {
    /// Create an empty result set.
    pub fn new() -> Self {
        Self {
            hits: Vec::new(),
            pattern_count: 0,
        }
    }

    /// Return the number of matched indices.
    pub fn count(&self) -> usize {
        self.hits.len()
    }

    /// Return the total number of pattern hits, including repeated hits at one index.
    pub fn pattern_count(&self) -> usize {
        self.pattern_count
    }

    /// Return the hit at `index` in match order.
    pub fn get(&self, index: usize) -> Option<CandleHit> {
        self.hits.get(index).copied()
    }

    /// Return the pattern bitset at a source index.
    pub fn at(&self, index: usize) -> CandleSet {
        self.hits
            .binary_search_by_key(&index, |hit| hit.index)
            .ok()
            .map(|position| self.hits[position].patterns)
            .unwrap_or(TC_NONE)
    }

    /// Iterate over matched indices in order.
    pub fn iter(&self) -> impl Iterator<Item = CandleHit> + '_ {
        self.hits.iter().copied()
    }

    fn add(&mut self, index: usize, pattern: CandleSet) {
        self.pattern_count += 1;
        if let Some(last) = self.hits.last_mut() {
            if last.index == index {
                last.patterns |= pattern;
                return;
            }
        }
        self.hits.push(CandleHit {
            index,
            patterns: pattern,
        });
    }
}

impl Default for CandleResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Metadata describing a named candlestick pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CandleInfo {
    /// Lowercase registry name.
    pub name: &'static str,
    /// Human-readable name.
    pub full_name: &'static str,
    /// Bitset value used to represent the pattern.
    pub pattern: CandleSet,
}

const CANDLES: [CandleInfo; 26] = [
    CandleInfo {
        name: "abandoned_baby_bear",
        full_name: "Abandoned Baby Bear",
        pattern: TC_ABANDONED_BABY_BEAR,
    },
    CandleInfo {
        name: "abandoned_baby_bull",
        full_name: "Abandoned Baby Bull",
        pattern: TC_ABANDONED_BABY_BULL,
    },
    CandleInfo {
        name: "big_black_candle",
        full_name: "Big Black Candle",
        pattern: TC_BIG_BLACK_CANDLE,
    },
    CandleInfo {
        name: "big_white_candle",
        full_name: "Big White Candle",
        pattern: TC_BIG_WHITE_CANDLE,
    },
    CandleInfo {
        name: "black_marubozu",
        full_name: "Black Marubozu",
        pattern: TC_BLACK_MARUBOZU,
    },
    CandleInfo {
        name: "doji",
        full_name: "Doji",
        pattern: TC_DOJI,
    },
    CandleInfo {
        name: "dragonfly_doji",
        full_name: "Dragonfly Doji",
        pattern: TC_DRAGONFLY_DOJI,
    },
    CandleInfo {
        name: "engulfing_bear",
        full_name: "Engulfing Bear",
        pattern: TC_ENGULFING_BEAR,
    },
    CandleInfo {
        name: "engulfing_bull",
        full_name: "Engulfing Bull",
        pattern: TC_ENGULFING_BULL,
    },
    CandleInfo {
        name: "evening_doji_star",
        full_name: "Evening Doji Star",
        pattern: TC_EVENING_DOJI_STAR,
    },
    CandleInfo {
        name: "evening_star",
        full_name: "Evening Star",
        pattern: TC_EVENING_STAR,
    },
    CandleInfo {
        name: "four_price_doji",
        full_name: "Four Price Doji",
        pattern: TC_FOUR_PRICE_DOJI,
    },
    CandleInfo {
        name: "gravestone_doji",
        full_name: "Gravestone Doji",
        pattern: TC_GRAVESTONE_DOJI,
    },
    CandleInfo {
        name: "hammer",
        full_name: "Hammer",
        pattern: TC_HAMMER,
    },
    CandleInfo {
        name: "hanging_man",
        full_name: "Hanging Man",
        pattern: TC_HANGING_MAN,
    },
    CandleInfo {
        name: "inverted_hammer",
        full_name: "Inverted Hammer",
        pattern: TC_INVERTED_HAMMER,
    },
    CandleInfo {
        name: "long_legged_doji",
        full_name: "Long Legged Doji",
        pattern: TC_LONG_LEGGED_DOJI,
    },
    CandleInfo {
        name: "marubozu",
        full_name: "Marubozu",
        pattern: TC_MARUBOZU,
    },
    CandleInfo {
        name: "morning_doji_star",
        full_name: "Morning Doji Star",
        pattern: TC_MORNING_DOJI_STAR,
    },
    CandleInfo {
        name: "morning_star",
        full_name: "Morning Star",
        pattern: TC_MORNING_STAR,
    },
    CandleInfo {
        name: "shooting_star",
        full_name: "Shooting Star",
        pattern: TC_SHOOTING_STAR,
    },
    CandleInfo {
        name: "spinning_top",
        full_name: "Spinning Top",
        pattern: TC_SPINNING_TOP,
    },
    CandleInfo {
        name: "star",
        full_name: "Star",
        pattern: TC_STAR,
    },
    CandleInfo {
        name: "three_black_crows",
        full_name: "Three Black Crows",
        pattern: TC_THREE_BLACK_CROWS,
    },
    CandleInfo {
        name: "three_white_soldiers",
        full_name: "Three White Soldiers",
        pattern: TC_THREE_WHITE_SOLDIERS,
    },
    CandleInfo {
        name: "white_marubozu",
        full_name: "White Marubozu",
        pattern: TC_WHITE_MARUBOZU,
    },
];

/// Return every supported candlestick pattern.
pub fn all_candles() -> &'static [CandleInfo] {
    &CANDLES
}

/// Return the number of supported candlestick patterns.
pub fn candle_count() -> usize {
    CANDLES.len()
}

/// Look up a candle pattern by its lowercase registry name.
pub fn find_candle(name: &str) -> Option<&'static CandleInfo> {
    CANDLES
        .binary_search_by_key(&name, |info| info.name)
        .ok()
        .map(|index| &CANDLES[index])
}

/// Look up a candle pattern by bitset value.
pub fn get_candle_info(pattern: CandleSet) -> Option<&'static CandleInfo> {
    let lowest_bit = pattern & (!pattern + 1);
    CANDLES
        .binary_search_by_key(&lowest_bit, |info| info.pattern)
        .ok()
        .map(|index| &CANDLES[index])
}

/// Run all supported candlestick pattern checks over the provided OHLC data.
///
/// `inputs` must contain four aligned slices in open, high, low, close order.
/// `patterns` is a bitset of the candle families to evaluate.
pub fn run_candles(
    patterns: CandleSet,
    inputs: &[&[Real]],
    config: &CandleConfig,
) -> Result<CandleResult, IndicatorError> {
    if inputs.len() != 4 {
        return Err(IndicatorError::WrongInputCount {
            indicator: "candle",
            expected: 4,
            actual: inputs.len(),
        });
    }

    let size = inputs[0].len();
    for (index, input) in inputs.iter().enumerate().skip(1) {
        if input.len() != size {
            return Err(IndicatorError::InputLengthMismatch {
                indicator: "candle",
                expected: size,
                actual: input.len(),
                input_index: index,
            });
        }
    }

    if config.period == 0 {
        return Err(IndicatorError::InvalidOption {
            indicator: "candle",
            option: "period",
            value: 0.0,
            reason: "period must be at least 1",
        });
    }

    let mut result = CandleResult::new();
    if size < config.period {
        return Ok(result);
    }

    let open = inputs[0];
    let high = inputs[1];
    let low = inputs[2];
    let close = inputs[3];

    let mut avg_body_sum = 0.0;
    let mut avg_total_sum = 0.0;
    for i in 0..config.period {
        avg_body_sum += body(open, close, i);
        avg_total_sum += total(high, low, i);
    }

    let div = 1.0 / config.period as Real;

    for i in config.period..size {
        let top_value = top(open, close, i);
        let bottom_value = bottom(open, close, i);
        let body_value = body(open, close, i);
        let total_value = total(high, low, i);
        let upper = high[i] - top_value;
        let lower = bottom_value - low[i];
        let avg_body = avg_body_sum * div;
        let avg_total = avg_total_sum * div;

        let opt_body_none = config.body_none * avg_total;
        let opt_body_short = config.body_short * avg_body;
        let opt_body_long = config.body_long * avg_body;
        let opt_wick_none = config.wick_none * avg_total;
        let opt_wick_long = config.wick_long * avg_body;
        let opt_near = config.near * avg_total;

        let black = open[i] > close[i];
        let white = open[i] < close[i];
        let body_none = body_value < opt_body_none;
        let body_short = body_value < opt_body_short;
        let body_long = body_value > opt_body_long;
        let wick_upper_none = upper < opt_wick_none;
        let wick_upper_long = upper > opt_wick_long;
        let wick_upper_longer_than_body = upper > body_value;
        let wick_lower_none = lower < opt_wick_none;
        let wick_lower_long = lower > opt_wick_long;
        let wick_lower_longer_than_body = lower > body_value;

        let gap_up = i >= 1 && bottom_value >= top(open, close, i - 1);
        let gap_down = i >= 1 && top_value <= bottom(open, close, i - 1);
        let star_up = i >= 1 && low[i] >= high[i - 1];
        let star_down = i >= 1 && high[i] <= low[i - 1];
        let star = i >= 1 && (star_up || star_down);
        let open_inside = |index: usize| {
            open[index] < top(open, close, index - 1)
                && open[index] > bottom(open, close, index - 1)
        };

        if patterns & TC_ABANDONED_BABY_BEAR != 0
            && i >= 2
            && is_white(open, close, i - 2)
            && body(open, close, i - 2) > opt_body_long
            && (low[i - 1] >= high[i - 2])
            && body(open, close, i - 1) < opt_body_none
            && black
            && star_down
            && close[i] <= close[i - 2]
        {
            result.add(i, TC_ABANDONED_BABY_BEAR);
        }
        if patterns & TC_ABANDONED_BABY_BULL != 0
            && i >= 2
            && is_black(open, close, i - 2)
            && body(open, close, i - 2) > opt_body_long
            && (high[i - 1] <= low[i - 2])
            && body(open, close, i - 1) < opt_body_none
            && white
            && star_up
            && close[i] >= close[i - 2]
        {
            result.add(i, TC_ABANDONED_BABY_BULL);
        }
        if patterns & TC_BIG_BLACK_CANDLE != 0 && black && body_long {
            result.add(i, TC_BIG_BLACK_CANDLE);
        }
        if patterns & TC_BIG_WHITE_CANDLE != 0 && white && body_long {
            result.add(i, TC_BIG_WHITE_CANDLE);
        }
        if patterns & TC_BLACK_MARUBOZU != 0
            && body_long
            && wick_upper_none
            && wick_lower_none
            && black
        {
            result.add(i, TC_BLACK_MARUBOZU);
        }
        if patterns & TC_DOJI != 0 && body_none {
            result.add(i, TC_DOJI);
        }
        if patterns & TC_DRAGONFLY_DOJI != 0 && body_none && wick_upper_none && wick_lower_long {
            result.add(i, TC_DRAGONFLY_DOJI);
        }
        if patterns & TC_ENGULFING_BEAR != 0
            && i >= 1
            && black
            && is_white(open, close, i - 1)
            && close[i] < open[i - 1]
            && open[i] > close[i - 1]
        {
            result.add(i, TC_ENGULFING_BEAR);
        }
        if patterns & TC_ENGULFING_BULL != 0
            && i >= 1
            && white
            && is_black(open, close, i - 1)
            && close[i] > open[i - 1]
            && open[i] < close[i - 1]
        {
            result.add(i, TC_ENGULFING_BULL);
        }
        if patterns & TC_EVENING_DOJI_STAR != 0
            && i >= 2
            && is_white(open, close, i - 2)
            && body(open, close, i - 2) > opt_body_long
            && gap_up_prev(open, close, i - 1)
            && body(open, close, i - 1) < opt_body_none
            && black
            && gap_down
            && close[i] <= close[i - 2]
        {
            result.add(i, TC_EVENING_DOJI_STAR);
        }
        if patterns & TC_EVENING_STAR != 0
            && i >= 2
            && is_white(open, close, i - 2)
            && body(open, close, i - 2) > opt_body_long
            && gap_up_prev(open, close, i - 1)
            && body(open, close, i - 1) < opt_body_short
            && black
            && gap_down
            && close[i] <= close[i - 2]
        {
            result.add(i, TC_EVENING_STAR);
        }
        if patterns & TC_FOUR_PRICE_DOJI != 0 && body_none && wick_upper_none && wick_lower_none {
            result.add(i, TC_FOUR_PRICE_DOJI);
        }
        if patterns & TC_GRAVESTONE_DOJI != 0 && body_none && wick_upper_long && wick_lower_none {
            result.add(i, TC_GRAVESTONE_DOJI);
        }
        if patterns & TC_HAMMER != 0
            && i >= 1
            && body_short
            && wick_upper_none
            && wick_lower_long
            && bottom_value <= low[i - 1] + opt_near
        {
            result.add(i, TC_HAMMER);
        }
        if patterns & TC_HANGING_MAN != 0
            && i >= 1
            && body_short
            && wick_upper_none
            && wick_lower_long
            && bottom_value >= high[i - 1] - opt_near
        {
            result.add(i, TC_HANGING_MAN);
        }
        if patterns & TC_INVERTED_HAMMER != 0
            && i >= 1
            && body_short
            && wick_upper_long
            && wick_lower_none
            && gap_down
        {
            result.add(i, TC_INVERTED_HAMMER);
        }
        if patterns & TC_LONG_LEGGED_DOJI != 0 && body_none && wick_upper_long && wick_lower_long {
            result.add(i, TC_LONG_LEGGED_DOJI);
        }
        if patterns & TC_MARUBOZU != 0 && body_long && wick_upper_none && wick_lower_none {
            result.add(i, TC_MARUBOZU);
        }
        if patterns & TC_MORNING_DOJI_STAR != 0
            && i >= 2
            && is_black(open, close, i - 2)
            && body(open, close, i - 2) > opt_body_long
            && gap_down_prev(open, close, i - 1)
            && body(open, close, i - 1) < opt_body_none
            && white
            && gap_up
            && close[i] >= close[i - 2]
        {
            result.add(i, TC_MORNING_DOJI_STAR);
        }
        if patterns & TC_MORNING_STAR != 0
            && i >= 2
            && is_black(open, close, i - 2)
            && body(open, close, i - 2) > opt_body_long
            && gap_down_prev(open, close, i - 1)
            && body(open, close, i - 1) < opt_body_short
            && white
            && gap_up
            && close[i] >= close[i - 2]
        {
            result.add(i, TC_MORNING_STAR);
        }
        if patterns & TC_SHOOTING_STAR != 0
            && i >= 1
            && body_short
            && wick_upper_long
            && wick_lower_none
            && gap_up
        {
            result.add(i, TC_SHOOTING_STAR);
        }
        if patterns & TC_SPINNING_TOP != 0
            && body_short
            && wick_upper_longer_than_body
            && wick_lower_longer_than_body
        {
            result.add(i, TC_SPINNING_TOP);
        }
        if patterns & TC_STAR != 0 && star {
            result.add(i, TC_STAR);
        }
        if patterns & TC_THREE_BLACK_CROWS != 0
            && i >= 2
            && is_black(open, close, i - 2)
            && is_black(open, close, i - 1)
            && black
            && open_inside(i - 1)
            && open_inside(i)
        {
            result.add(i, TC_THREE_BLACK_CROWS);
        }
        if patterns & TC_THREE_WHITE_SOLDIERS != 0
            && i >= 2
            && is_white(open, close, i - 2)
            && is_white(open, close, i - 1)
            && white
            && open_inside(i - 1)
            && open_inside(i)
        {
            result.add(i, TC_THREE_WHITE_SOLDIERS);
        }
        if patterns & TC_WHITE_MARUBOZU != 0
            && body_long
            && wick_upper_none
            && wick_lower_none
            && white
        {
            result.add(i, TC_WHITE_MARUBOZU);
        }

        avg_body_sum += body_value;
        avg_body_sum -= body(open, close, i - config.period);
        avg_total_sum += total_value;
        avg_total_sum -= total(high, low, i - config.period);
    }

    Ok(result)
}

/// Run the candle pattern identified by its bitset value.
pub fn run_candle_pattern(
    pattern: CandleSet,
    inputs: &[&[Real]],
    config: &CandleConfig,
) -> Result<CandleResult, IndicatorError> {
    run_candles(pattern, inputs, config)
}

/// Run the candle pattern identified by its registry name.
pub fn run_candle_named(
    name: &str,
    inputs: &[&[Real]],
    config: &CandleConfig,
) -> Result<CandleResult, IndicatorError> {
    let info = find_candle(name).ok_or(IndicatorError::InvalidOption {
        indicator: "candle",
        option: "pattern",
        value: 0.0,
        reason: "unknown candle pattern name",
    })?;
    run_candles(info.pattern, inputs, config)
}

fn body(open: &[Real], close: &[Real], index: usize) -> Real {
    (open[index] - close[index]).abs()
}

fn total(high: &[Real], low: &[Real], index: usize) -> Real {
    high[index] - low[index]
}

fn top(open: &[Real], close: &[Real], index: usize) -> Real {
    open[index].max(close[index])
}

fn bottom(open: &[Real], close: &[Real], index: usize) -> Real {
    open[index].min(close[index])
}

fn is_black(open: &[Real], close: &[Real], index: usize) -> bool {
    open[index] > close[index]
}

fn is_white(open: &[Real], close: &[Real], index: usize) -> bool {
    open[index] < close[index]
}

fn gap_up_prev(open: &[Real], close: &[Real], index: usize) -> bool {
    bottom(open, close, index) >= top(open, close, index - 1)
}

fn gap_down_prev(open: &[Real], close: &[Real], index: usize) -> bool {
    top(open, close, index) <= bottom(open, close, index - 1)
}
