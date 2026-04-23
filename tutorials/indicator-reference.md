# Indicator Reference

This is a release reference for every indicator currently returned by
`registry::all()`. The table was derived from runtime metadata and checked in as
a release snapshot, with a small manually maintained column for typed-state
availability.

The authoritative source of truth is the indicator metadata in code:

- `IndicatorMetadata`
- `registry::find(...)`
- `registry::all()`

## How To Read The Table

- `Inputs` lists the aligned input series in call order.
- `Options` lists indicator options in the same order accepted by the API.
- `Outputs` lists the output series in returned order.
- `Typed state` says whether the crate exports a dedicated `FooState` wrapper.

## Reference Table

| Indicator | Category | Inputs | Options | Outputs | Typed state |
| --- | --- | --- | --- | --- | --- |
| `abs` | `Simple` | `real` | - | `abs` | no |
| `acos` | `Simple` | `real` | - | `acos` | no |
| `abands` | `Overlay` | `high`, `low`, `close` | `period` | `lower_band`, `upper_band`, `middle_point` | no |
| `alma` | `Overlay` | `real` | `period`, `offset`, `sigma` | `alma` | no |
| `ad` | `Indicator` | `high`, `low`, `close`, `volume` | - | `ad` | no |
| `adosc` | `Indicator` | `high`, `low`, `close`, `volume` | `short_period`, `long_period` | `adosc` | no |
| `add` | `Simple` | `real`, `real` | - | `add` | no |
| `adx` | `Indicator` | `high`, `low` | `period` | `adx` | yes |
| `adxr` | `Indicator` | `high`, `low` | `period` | `adxr` | yes |
| `ao` | `Indicator` | `high`, `low` | - | `ao` | no |
| `apo` | `Indicator` | `real` | `short_period`, `long_period` | `apo` | no |
| `aroon` | `Indicator` | `high`, `low` | `period` | `aroon_down`, `aroon_up` | no |
| `aroonosc` | `Indicator` | `high`, `low` | `period` | `aroonosc` | no |
| `asin` | `Simple` | `real` | - | `asin` | no |
| `atr` | `Indicator` | `high`, `low`, `close` | `period` | `atr` | yes |
| `atan` | `Simple` | `real` | - | `atan` | no |
| `avgprice` | `Overlay` | `open`, `high`, `low`, `close` | - | `avgprice` | no |
| `bbands` | `Overlay` | `real` | `period`, `stddev` | `bbands_lower`, `bbands_middle`, `bbands_upper` | no |
| `beta` | `Math` | `real`, `real` | `period` | `beta` | no |
| `bop` | `Indicator` | `open`, `high`, `low`, `close` | - | `bop` | no |
| `ce` | `Overlay` | `high`, `low`, `close` | `period`, `coef` | `ce_high`, `ce_low` | no |
| `cci` | `Indicator` | `high`, `low`, `close` | `period` | `cci` | no |
| `ceil` | `Simple` | `real` | - | `ceil` | no |
| `cmf` | `Indicator` | `high`, `low`, `close`, `volume` | `period` | `cmf` | no |
| `cmo` | `Indicator` | `real` | `period` | `cmo` | no |
| `copp` | `Indicator` | `real` | `short_roc_period`, `long_roc_period`, `wma_period` | `copp` | no |
| `cos` | `Simple` | `real` | - | `cos` | no |
| `cosh` | `Simple` | `real` | - | `cosh` | no |
| `correl` | `Math` | `real`, `real` | `period` | `correl` | no |
| `crossany` | `Math` | `real`, `real` | - | `crossany` | no |
| `crossover` | `Math` | `real`, `real` | - | `crossover` | no |
| `cvi` | `Indicator` | `high`, `low` | `period` | `cvi` | no |
| `decay` | `Math` | `real` | `period` | `decay` | no |
| `dema` | `Overlay` | `real` | `period` | `dema` | no |
| `di` | `Indicator` | `high`, `low`, `close` | `period` | `plus_di`, `minus_di` | yes |
| `div` | `Simple` | `real`, `real` | - | `div` | no |
| `dc` | `Overlay` | `real` | `period` | `dc_lower`, `dc_upper` | no |
| `dm` | `Indicator` | `high`, `low` | `period` | `plus_dm`, `minus_dm` | yes |
| `dpo` | `Indicator` | `real` | `period` | `dpo` | no |
| `dx` | `Indicator` | `high`, `low` | `period` | `dx` | yes |
| `edecay` | `Math` | `real` | `period` | `edecay` | no |
| `ema` | `Overlay` | `real` | `period` | `ema` | yes |
| `emv` | `Indicator` | `high`, `low`, `volume` | - | `emv` | no |
| `exp` | `Simple` | `real` | - | `exp` | no |
| `fi` | `Indicator` | `close`, `volume` | `period` | `fi` | no |
| `fisher` | `Indicator` | `high`, `low` | `period` | `fisher`, `signal` | no |
| `floor` | `Simple` | `real` | - | `floor` | no |
| `fosc` | `Indicator` | `real` | `period` | `fosc` | no |
| `hma` | `Overlay` | `real` | `period` | `hma` | no |
| `ht_dcperiod` | `Indicator` | `real` | - | `ht_dcperiod` | no |
| `ht_dcphase` | `Indicator` | `real` | - | `ht_dcphase` | no |
| `ht_phasor` | `Indicator` | `real` | - | `inphase`, `quadrature` | no |
| `ht_sine` | `Indicator` | `real` | - | `sine`, `lead_sine` | no |
| `ht_trendline` | `Indicator` | `real` | - | `ht_trendline` | no |
| `ht_trendmode` | `Indicator` | `real` | - | `ht_trendmode` | no |
| `imi` | `Indicator` | `open`, `close` | `period` | `imi` | no |
| `ikhts` | `Overlay` | `high`, `low` | `period` | `ikhts` | no |
| `kama` | `Overlay` | `real` | `period` | `kama` | no |
| `kc` | `Overlay` | `high`, `low`, `close` | `period`, `multiple` | `kc_lower`, `kc_middle`, `kc_upper` | no |
| `kvo` | `Indicator` | `high`, `low`, `close`, `volume` | `short_period`, `long_period` | `kvo` | no |
| `lag` | `Math` | `real` | `period` | `lag` | no |
| `linreg` | `Overlay` | `real` | `period` | `linreg` | no |
| `linearregangle` | `Indicator` | `real` | `period` | `linearregangle` | no |
| `linregintercept` | `Overlay` | `real` | `period` | `linregintercept` | no |
| `linregslope` | `Indicator` | `real` | `period` | `linregslope` | no |
| `ln` | `Simple` | `real` | - | `ln` | no |
| `log10` | `Simple` | `real` | - | `log10` | no |
| `macd` | `Indicator` | `real` | `short_period`, `long_period`, `signal_period` | `macd`, `macd_signal`, `macd_histogram` | yes |
| `macdext` | `Indicator` | `real` | `fast_period`, `fast_ma_type`, `slow_period`, `slow_ma_type`, `signal_period`, `signal_ma_type` | `macd`, `macd_signal`, `macd_histogram` | no |
| `macdfix` | `Indicator` | `real` | `signal_period` | `macd`, `macd_signal`, `macd_histogram` | no |
| `ma` | `Overlay` | `real` | `period`, `ma_type` | `ma` | no |
| `mama` | `Overlay` | `real` | `fastlimit`, `slowlimit` | `mama`, `fama` | no |
| `mavp` | `Overlay` | `real`, `periods` | `min_period`, `max_period`, `ma_type` | `mavp` | no |
| `marketfi` | `Indicator` | `high`, `low`, `volume` | - | `marketfi` | no |
| `mass` | `Indicator` | `high`, `low` | `period` | `mass` | no |
| `max` | `Math` | `real` | `period` | `max` | no |
| `maxindex` | `Math` | `real` | `period` | `maxindex` | no |
| `md` | `Indicator` | `real` | `period` | `md` | no |
| `medprice` | `Overlay` | `high`, `low` | - | `medprice` | no |
| `midpoint` | `Math` | `real` | `period` | `midpoint` | no |
| `midprice` | `Overlay` | `high`, `low` | `period` | `midprice` | no |
| `mfi` | `Indicator` | `high`, `low`, `close`, `volume` | `period` | `mfi` | no |
| `min` | `Math` | `real` | `period` | `min` | no |
| `minindex` | `Math` | `real` | `period` | `minindex` | no |
| `minmax` | `Math` | `real` | `period` | `min`, `max` | no |
| `minmaxindex` | `Math` | `real` | `period` | `minindex`, `maxindex` | no |
| `mom` | `Indicator` | `real` | `period` | `mom` | no |
| `msw` | `Indicator` | `real` | `period` | `sine`, `lead` | no |
| `mul` | `Simple` | `real`, `real` | - | `mul` | no |
| `natr` | `Indicator` | `high`, `low`, `close` | `period` | `natr` | yes |
| `nvi` | `Indicator` | `close`, `volume` | - | `nvi` | no |
| `obv` | `Indicator` | `close`, `volume` | - | `obv` | no |
| `pfe` | `Indicator` | `real` | `period`, `ema_period` | `pfe` | no |
| `posc` | `Indicator` | `high`, `low`, `close` | `period`, `ema_period` | `posc` | no |
| `ppo` | `Indicator` | `real` | `short_period`, `long_period` | `ppo` | yes |
| `pbands` | `Overlay` | `high`, `low`, `close` | `period` | `pbands_lower`, `pbands_upper` | no |
| `pc` | `Overlay` | `high`, `low` | `period` | `pc_low`, `pc_high` | no |
| `psar` | `Indicator` | `high`, `low` | `acceleration_factor_step`, `acceleration_factor_maximum` | `psar` | no |
| `pvi` | `Indicator` | `close`, `volume` | - | `pvi` | no |
| `qstick` | `Indicator` | `open`, `close` | `period` | `qstick` | no |
| `rmi` | `Indicator` | `real` | `period`, `lookback_period` | `rmi` | no |
| `roc` | `Indicator` | `real` | `period` | `roc` | no |
| `rocr` | `Indicator` | `real` | `period` | `rocr` | no |
| `rocr100` | `Indicator` | `real` | `period` | `rocr100` | no |
| `rmta` | `Overlay` | `real` | `period`, `beta` | `rmta` | no |
| `rsi` | `Indicator` | `real` | `period` | `rsi` | yes |
| `rvi` | `Indicator` | `real` | `sma_period`, `stddev_period` | `rvi` | no |
| `round` | `Simple` | `real` | - | `round` | no |
| `sarext` | `Overlay` | `high`, `low` | `start_value`, `offset_on_reverse`, `acceleration_init_long`, `acceleration_long`, `acceleration_max_long`, `acceleration_init_short`, `acceleration_short`, `acceleration_max_short` | `sarext` | no |
| `sin` | `Simple` | `real` | - | `sin` | no |
| `sinh` | `Simple` | `real` | - | `sinh` | no |
| `sma` | `Overlay` | `real` | `period` | `sma` | yes |
| `smi` | `Indicator` | `high`, `low`, `close` | `q_period`, `r_period`, `s_period` | `smi` | no |
| `sqrt` | `Simple` | `real` | - | `sqrt` | no |
| `stderr` | `Indicator` | `real` | `period` | `stderr` | no |
| `stddev` | `Indicator` | `real` | `period` | `stddev` | no |
| `stoch` | `Indicator` | `high`, `low`, `close` | `k_period`, `k_slowing_period`, `d_period` | `stoch_k`, `stoch_d` | yes |
| `stochrsi` | `Indicator` | `real` | `period` | `stochrsi` | no |
| `sub` | `Simple` | `real`, `real` | - | `sub` | no |
| `sum` | `Math` | `real` | `period` | `sum` | no |
| `tan` | `Simple` | `real` | - | `tan` | no |
| `tanh` | `Simple` | `real` | - | `tanh` | no |
| `t3` | `Overlay` | `real` | `period`, `vfactor` | `t3` | no |
| `tema` | `Overlay` | `real` | `period` | `tema` | no |
| `todeg` | `Simple` | `real` | - | `degrees` | no |
| `torad` | `Simple` | `real` | - | `radians` | no |
| `tr` | `Indicator` | `high`, `low`, `close` | - | `tr` | no |
| `trima` | `Overlay` | `real` | `period` | `trima` | no |
| `trix` | `Indicator` | `real` | `period` | `trix` | no |
| `tsi` | `Indicator` | `real` | `y_period`, `z_period` | `tsi` | no |
| `trunc` | `Simple` | `real` | - | `trunc` | no |
| `tsf` | `Overlay` | `real` | `period` | `tsf` | no |
| `kst` | `Indicator` | `real` | `roc1_period`, `roc2_period`, `roc3_period`, `roc4_period`, `ma1_period`, `ma2_period`, `ma3_period`, `ma4_period` | `kst`, `signal` | no |
| `typprice` | `Overlay` | `high`, `low`, `close` | - | `typprice` | no |
| `ultosc` | `Indicator` | `high`, `low`, `close` | `short_period`, `medium_period`, `long_period` | `ultosc` | no |
| `var` | `Indicator` | `real` | `period` | `var` | no |
| `vhf` | `Indicator` | `real` | `period` | `vhf` | no |
| `vidya` | `Overlay` | `real` | `short_period`, `long_period`, `alpha` | `vidya` | no |
| `volatility` | `Indicator` | `real` | `period` | `volatility` | no |
| `vosc` | `Indicator` | `volume` | `short_period`, `long_period` | `vosc` | no |
| `vwap` | `Overlay` | `high`, `low`, `close`, `volume` | `period` | `vwap` | no |
| `vwma` | `Overlay` | `close`, `volume` | `period` | `vwma` | no |
| `wad` | `Indicator` | `high`, `low`, `close` | - | `wad` | no |
| `wcprice` | `Overlay` | `high`, `low`, `close` | - | `wcprice` | no |
| `wilders` | `Overlay` | `real` | `period` | `wilders` | yes |
| `willr` | `Indicator` | `high`, `low`, `close` | `period` | `willr` | no |
| `wma` | `Overlay` | `real` | `period` | `wma` | no |
| `zlema` | `Overlay` | `real` | `period` | `zlema` | no |

## Notes

- The reference table mirrors `registry::all()` at the time this document was
  updated.
- The registry metadata still drives the real API shape, so `registry::find`
  and `IndicatorMetadata` should be used for code generation or dynamic lookup.
- Typed state availability is limited to the indicators that currently export a
  dedicated wrapper; the rest still work through batch, stream, or dynamic
  state.
