[![Build Status](https://travis-ci.com/TulipCharts/tulipindicators.svg?branch=master)](https://travis-ci.com/TulipCharts/tulipindicators)

# Tulip Indicators

## Introduction

Tulip Indicators is a library of technical analysis functions written in ANSI C.

The repository now keeps the ANSI C implementation under `c/` and the in-progress
Rust implementation under `rust/`. Root `make` and `cargo` commands remain as
entry points and delegate to those directories.

Lots of information is available on the website:
[https://tulipindicators.org](https://tulipindicators.org)

Bindings are available for Node.js, Go, Ruby, Python, and others. [See here](https://tulipindicators.org/bindings).

## Features

 - **C99 with no dependencies**.
 - Uses fast algorithms.
 - Easy to use programming interface.
 - Release under LGPL license.


## Building

Building is easy. You only need a decent C compiler. Tulip Indicators has no
other dependencies.

Just download the code and run `make`.

```
git clone https://github.com/TulipCharts/tulipindicators
cd tulipindicators
make
```

You should get a static library at `c/libindicators.a`. You'll need that library
and the header file `c/indicators.h` to use Tulip Indicators in your code.


## Not Building

If you don't want to build the library, you can simply add the
`c/tiamalgamation.c` file to your project, along with `c/indicators.h` and
`c/candles.h`. The amalgamation file contains all of Tulip Indicators - you
don't actually need any of the other source files.

This is the recommended method to import Tulip Indicators into code for
bindings to other languages, since it makes it very easy to update versions.

## Usage

For usage information, please see:
[https://tulipindicators.org/usage](https://tulipindicators.org/usage)

## Rust State API

The Rust implementation keeps the existing high-performance batch layer:

- `run(...)`
- `run_single(...)`
- `run_in_place(...)`

Those paths are still the recommended choice for offline analysis, benchmarking,
and caller-managed output buffers.

For single-output indicators such as `rsi`, `ema`, or `atr`, prefer
`run_single(...)` when you only need that one output series and do not want to
unwrap `Vec<Vec<Real>>` manually.

See also:
- [`tutorials/state-api.md`](tutorials/state-api.md)

On top of that batch layer, the Rust crate now also exposes a stateful API for
incremental usage:

- seed from historical data once
- update with one new sample at a time
- keep a fixed-size history ring for indexed access

The state layer does not shift buffers when history fills up. It uses a fixed
capacity ring buffer and overwrites the oldest values.

State history capacity must be at least `1`. Use the lower-level stream API
when you want incremental calculation without keeping any history.

### Typed State Example

Use a typed state wrapper when you know the indicator type at compile time.

```rust
use tulipindicators::{IndicatorState, Rsi};

let closes = [100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 103.5, 105.0];
let mut rsi = Rsi::state(&[3.0], 32)?;

// Seed from historical data.
let produced = rsi.seed(&closes)?;
assert!(produced <= closes.len());

// Incrementally update with one new value.
let latest = rsi.update(106.0);

// Read the latest and prior outputs.
let current = rsi.latest();
let previous = rsi.get(1);
```

### Batch Single-Output Example

Use `run_single(...)` when the indicator has exactly one output and you want the
simple batch API without `batch[0]` unpacking.

```rust
use tulipindicators::{Indicator, Rsi};

let closes = [100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 103.5, 105.0];
let values = Rsi.run_single(&[&closes], &[3.0])?;

assert!(!values.is_empty());
```

Typed state wrappers are currently available for several high-frequency
indicators, including:

- `RsiState`
- `DmState`
- `DxState`
- `DiState`
- `AdxState`
- `AdxrState`
- `EmaState`
- `SmaState`
- `WildersState`
- `AtrState`
- `NatrState`
- `MacdState`
- `PpoState`
- `StochState`

### Dynamic State Example

Use the dynamic state layer when the indicator is selected by name at runtime.

```rust
use tulipindicators::{DynamicIndicatorState, IndicatorStateFactory, RSI};

let closes = [100.0, 101.0, 102.0, 101.5, 103.0, 104.0, 103.5, 105.0];

// Build by registry name.
let mut by_name = DynamicIndicatorState::from_name("rsi", &[3.0], 32)?;
by_name.seed_columns(&[&closes])?;
let next = by_name.update(&[106.0])?;

// Or build directly from a static indicator handle.
let mut by_factory = RSI.dynamic_state(&[3.0], 32)?;
by_factory.seed_columns(&[&closes])?;
assert_eq!(by_factory.latest(), by_name.latest());
```

For multi-input indicators, `seed_columns(...)` and `update(...)` use one value
per declared input. For example:

- `dm.update(&[high, low])`
- `di.update(&[high, low, close])`
- `stoch.update(&[high, low, close])`

### Choosing Between Batch and State

- Use `run(...)` when you want the simplest batch API.
- Use `run_single(...)` when the indicator has one output and you want that
  single series directly.
- Use `run_in_place(...)` when you want maximum batch performance and control
  over output buffers.
- Use typed `FooState` wrappers when you process one sample at a time and want a
  strongly typed incremental API.
- Use `DynamicIndicatorState` when the indicator is selected dynamically at
  runtime.


## Indicator Listing
```
104 total indicators

Overlay
   avgprice            Average Price
   bbands              Bollinger Bands
   dema                Double Exponential Moving Average
   ema                 Exponential Moving Average
   hma                 Hull Moving Average
   kama                Kaufman Adaptive Moving Average
   linreg              Linear Regression
   medprice            Median Price
   psar                Parabolic SAR
   sma                 Simple Moving Average
   tema                Triple Exponential Moving Average
   trima               Triangular Moving Average
   tsf                 Time Series Forecast
   typprice            Typical Price
   vidya               Variable Index Dynamic Average
   vwma                Volume Weighted Moving Average
   wcprice             Weighted Close Price
   wilders             Wilders Smoothing
   wma                 Weighted Moving Average
   zlema               Zero-Lag Exponential Moving Average

Indicator
   ad                  Accumulation/Distribution Line
   adosc               Accumulation/Distribution Oscillator
   adx                 Average Directional Movement Index
   adxr                Average Directional Movement Rating
   ao                  Awesome Oscillator
   apo                 Absolute Price Oscillator
   aroon               Aroon
   aroonosc            Aroon Oscillator
   atr                 Average True Range
   bop                 Balance of Power
   cci                 Commodity Channel Index
   cmo                 Chande Momentum Oscillator
   cvi                 Chaikins Volatility
   di                  Directional Indicator
   dm                  Directional Movement
   dpo                 Detrended Price Oscillator
   dx                  Directional Movement Index
   emv                 Ease of Movement
   fisher              Fisher Transform
   fosc                Forecast Oscillator
   kvo                 Klinger Volume Oscillator
   linregintercept     Linear Regression Intercept
   linregslope         Linear Regression Slope
   macd                Moving Average Convergence/Divergence
   marketfi            Market Facilitation Index
   mass                Mass Index
   mfi                 Money Flow Index
   mom                 Momentum
   msw                 Mesa Sine Wave
   natr                Normalized Average True Range
   nvi                 Negative Volume Index
   obv                 On Balance Volume
   ppo                 Percentage Price Oscillator
   pvi                 Positive Volume Index
   qstick              Qstick
   roc                 Rate of Change
   rocr                Rate of Change Ratio
   rsi                 Relative Strength Index
   stoch               Stochastic Oscillator
   stochrsi            Stochastic RSI
   tr                  True Range
   trix                Trix
   ultosc              Ultimate Oscillator
   vhf                 Vertical Horizontal Filter
   volatility          Annualized Historical Volatility
   vosc                Volume Oscillator
   wad                 Williams Accumulation/Distribution
   willr               Williams %R

Math
   crossany            Crossany
   crossover           Crossover
   decay               Linear Decay
   edecay              Exponential Decay
   lag                 Lag
   max                 Maximum In Period
   md                  Mean Deviation Over Period
   min                 Minimum In Period
   stddev              Standard Deviation Over Period
   stderr              Standard Error Over Period
   sum                 Sum Over Period
   var                 Variance Over Period

Simple
   abs                 Vector Absolute Value
   acos                Vector Arccosine
   add                 Vector Addition
   asin                Vector Arcsine
   atan                Vector Arctangent
   ceil                Vector Ceiling
   cos                 Vector Cosine
   cosh                Vector Hyperbolic Cosine
   div                 Vector Division
   exp                 Vector Exponential
   floor               Vector Floor
   ln                  Vector Natural Log
   log10               Vector Base-10 Log
   mul                 Vector Multiplication
   round               Vector Round
   sin                 Vector Sine
   sinh                Vector Hyperbolic Sine
   sqrt                Vector Square Root
   sub                 Vector Subtraction
   tan                 Vector Tangent
   tanh                Vector Hyperbolic Tangent
   todeg               Vector Degree Conversion
   torad               Vector Radian Conversion
   trunc               Vector Truncate

```


## Special Thanks

The stochrsi indicator was sponsored by: [Gunthy](https://gunthy.org).

The candle pattern recognition was sponsored by: [Algorum](https://algorumsoftware.com)
