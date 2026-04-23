# 给剩余 grouped indicator 文件补模块说明，而不是塞公式大全

这次补的是几个“大文件”的入口说明。

这些文件不是一个文件一个指标，而是把一组形状相近的指标放在一起：

- `oscillators.rs`
- `price_volume.rs`
- `regression.rs`
- `ht.rs`
- `prices.rs`
- `talib_ma.rs`
- `math/correlation.rs`
- `math/cross.rs`
- `math/decay.rs`
- `math/extrema.rs`

如果没有模块级说明，读者很难判断“这个文件为什么放这么多指标”。

## 文档重点

这批文档主要解释分组逻辑：

- oscillator 组：输入多为一到三列，输出一到两列，很多都有 rolling state。
- price/volume 组：有 stateless 单 bar transform，也有累计或滚动状态。
- regression 组：共用 rolling linear fit，只是输出投影不同。
- Hilbert Transform 组：共用递归状态和固定 warmup。
- prices 组：多数是 open/high/low/close 的直接组合，`midprice` 是 rolling window。
- TA-Lib MA 组：`ma` 做固定 period dispatch，`mavp` 负责变量 period 策略。
- math 组：cross、correlation、decay、extrema 分别说明自己的 rolling/window 形状。

## 两个容易写错的点

第一，`lag` 不保留输入长度。

`decay` 和 `edecay` 会输出和输入等长的序列，但 `lag` 会丢掉前 `period` 个输出。如果文档写成“三者都 preserve input shape”，就是错的。

第二，`mavp` 不是完全被动的 dispatch wrapper。

它除了复用具体 MA 实现，还负责：

- clamp 每根 bar 的 period。
- 对 SMA 走专用 prefix-sum 路径。
- 对非 SMA 缓存重复 period 的整条输出序列。

所以文档要说清楚 `mavp` 自己拥有 variable-period policy。

## 一个小经验

grouped 文件的文档要解释“为什么这些指标在一起”，而不是逐个复述 metadata。

metadata 已经告诉用户输入、参数、输出名。模块文档更应该告诉维护者：这些指标共享什么 helper、什么状态形状、什么 batch/stream 对齐策略。
