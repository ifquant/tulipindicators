# TA-Lib 与 Tulip 的语义映射

这份表不是“名字对名字”的清单，而是“语义对语义”的清单。

目标有两个：
- 避免把已经存在、只是名称不同的指标又实现一遍
- 把真正缺失的指标单独挑出来，再按 Tulip 风格补齐

## 已有语义映射

这些指标在 Tulip 里已经有等价或近等价实现，不应重复发明：

| TA-Lib | Tulip | 说明 |
| --- | --- | --- |
| `ROCP` | `roc` | Tulip 的 `roc` 输出 `(x - lag) / lag`，语义上对应 TA-Lib `ROCP` |
| `ROCR` | `rocr` | 同为 ratio 形式 |
| `TRANGE` | `tr` | 同为 true range |
| `WCLPRICE` | `wcprice` | 同为 weighted close price |
| `MEDPRICE` | `medprice` | 同为单根 bar 的 `(high + low) / 2` |
| `TYPPRICE` | `typprice` | 同为 `(high + low + close) / 3` |
| `AVGPRICE` | `avgprice` | 同为 `(open + high + low + close) / 4` |
| `SAR` | `psar` | 同为标准抛物线 SAR |
| `LINEARREG` | `linreg` | 同为线性回归投影值 |
| `LINEARREG_SLOPE` | `linregslope` | 同为回归斜率 |
| `LINEARREG_INTERCEPT` | `linregintercept` | 同为回归截距 |
| `TSF` | `tsf` | 同为 time series forecast |
| `PLUS_DM` / `MINUS_DM` | `dm` | Tulip 用双输出合并成一个指标 |
| `PLUS_DI` / `MINUS_DI` | `di` | Tulip 用双输出合并成一个指标 |

## 名字相近但语义不同

这些名字容易误判，不能简单按“看起来像”处理：

| TA-Lib | 容易混淆为 | 实际差异 |
| --- | --- | --- |
| `MIDPOINT` | `medprice` / `midprice` | `MIDPOINT` 是单输入窗口 `(max + min) / 2`，不是单根 bar 平均价 |
| `MIDPRICE` | `medprice` | `MIDPRICE` 是窗口内 `(highest high + lowest low) / 2`，不是单根 `(high + low) / 2` |
| `ROCR100` | `rocr` | `ROCR100` 是 `100 * x / lag`，`rocr` 只是 `x / lag` |
| `CMO` | `cmo` | TA-Lib 的 `CMO` 与 Tulip `cmo` 平滑定义不同，`c/benchmark.c` 里已有说明 |

## 当前确认的真缺失

这些指标在当前 Tulip stable 线里没有等价实现：

- `linearregangle`
- `midpoint`
- `midprice`
- `rocr100`
- `maxindex`
- `minindex`
- `minmax`
- `minmaxindex`
- `beta`
- `correl`
- `ma`
- `mavp`
- `macdext`
- `macdfix`
- `sarext`
- `t3`
- `ht_*` Hilbert Transform 家族

## 第一批实现

这一批优先补“最适合 Tulip 现有风格”的四个：

- `linearregangle`
- `midpoint`
- `midprice`
- `rocr100`

它们的共同特点是：
- 可以直接复用现有 `linreg` / `linregslope` / `max` / `min` / `rocr` 的实现模式
- 不需要先引入 TA-Lib 风格的可变 MA 类型系统
- 语义明确，不会和现有指标重叠
