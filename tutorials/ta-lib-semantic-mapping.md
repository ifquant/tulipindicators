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
- `ma`
- `mavp`
- `macdext`
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

## 第二批实现

第二批优先补窗口索引家族：

- `maxindex`
- `minindex`
- `minmax`
- `minmaxindex`

这一批的特点是：
- 和现有 `max` / `min` / `midpoint` 共用同一类滑窗极值语义
- 比 `beta` / `correl` 更容易先接进 Tulip 现有 math 指标体系
- 可以同时补齐 C、Rust、parity 和 benchmark，而不用先引入新的统计类型系统

## 第三批实现

第三批补的是双输入统计窗口家族：

- `beta`
- `correl`

这一批的特点是：
- 都是固定窗口上的双输入统计量，不需要引入 TA-Lib 风格的可变 MA 类型系统
- 可以直接沿用 Tulip 现有 `period` lookback 和 batch/in-place 风格
- 但 `beta` 的语义必须按 TA-Lib 源码而不是按注释想当然地实现：它实际输出的是“第二路收益率对第一路收益率”的回归斜率

## 第四批实现

第四批补的是：

- `macdfix`

这一批的特点是：

- 算法本体并不缺，缺的是 TA-Lib 风格的固定 `12/26` API 壳
- 最适合直接复用现有 `macd` 内核，而不是复制一份三层 EMA 算法
- parity 最应该盯的是它是否和 `macd(12, 26, signal_period)` 一致

## 第五批实现

第五批补的是：

- `ma`
- `mavp`
- `t3`
- `macdext`

这一批的特点是：

- 不是各自独立的一堆指标，而是一整组 TA-Lib 风格的均线类型接口层
- `ma` / `mavp` / `macdext` 都依赖同一套 `MAType` 语义，所以应作为一批一起落
- `t3` 是这一批里唯一真正缺少的均线内核，其余更多是对现有 Tulip kernel 的重新编排

## 当前仍未补齐

完成前五批之后，当前还明确保留为缺失的是：

- `sarext`
- `ht_*` Hilbert Transform 家族
