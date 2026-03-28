# 背景

前面我们一直在优化 `ema`、`macd`、`zlema`、`wilders` 这一组指标，但经常遇到一个问题：

- 有些改动对 `C vs Rust` 看起来有帮助
- 但对 Rust 自己的历史最好结果却会回退
- 很难判断慢的是公共接口、还是指标内核本体

这时候继续盲改热循环，收益会越来越差。比继续猜更重要的，是先把 benchmark 的尺子拆细。

# 这次做了什么

这次没有再去重写指标实现，而是给 benchmark 增加了一层“函数级拆分”：

- 继续保留原来的 `run_in_place` 基准
- 额外增加 `kernel-only` 基准
- 先只覆盖最值得分析的四个指标：
  - `ema`
  - `wilders`
  - `zlema`
  - `macd`

现在 compare 工具除了原来的三类报告，还会额外生成：

- `rust-kernel-probes-latest.tsv`
- `rust-kernel-split-latest.tsv`
- `rust-kernel-split-latest.md`

其中最重要的是 `rust-kernel-split-latest.md`，它直接告诉我们：

- `run_in_place ns/input`
- `kernel ns/input`
- `ratio to kernel`

也就是“公共接口包装层”相对“纯内核”到底额外多了多少成本。

# 这次得到的结论

这次 focused run 用的是：

```bash
TI_BENCH_INDICATORS=ema,macd,zlema,wilders TI_BENCH_SIZES=4096,65536 TI_BENCH_TARGET_MS=120 TI_BENCH_CALIBRATION_MS=30 TI_BENCH_REPEATS=5 cargo run --release --bin indicator-bench-compare
```

最有价值的不是某个具体数字，而是“问题被分层看清楚了”：

- `wilders 65536`
  - `run_in_place / kernel = 1.274`
  - 这说明它确实有明显入口税，值得继续看公共契约检查和输出包装

- `ema / macd / zlema`
  - 大多接近 `1.0`
  - 说明它们现在不主要是 `run_in_place` 包装层的问题
  - 更像是内核本体、编译器 codegen，或者 benchmark 波动

这比之前只看 `C vs Rust` 有用得多，因为我们终于能回答：

- 哪些指标该继续压 API 层
- 哪些指标不该再把时间花在 `run_in_place` 外壳上

# 为什么这比继续手改循环更重要

如果一个指标：

- `run_in_place / kernel` 很高

那说明继续压公共接口是有希望的。

如果一个指标：

- `run_in_place / kernel` 已经接近 `1.0`

那继续改入口检查几乎不会有大收益，应该换方向。

这次就是一个典型例子：我们把“猜测”变成了“有证据的分流”。

# 给 Rust 新手的两个知识点

## 1. `run_in_place` 变快，不代表内核一定快

很多新手会觉得：

- “我把 `Vec` 分配去掉了，性能就应该好了”

这只对一部分指标成立。

如果一个指标的大头其实是：

- 多次 EMA 递推
- 多路状态更新
- 复杂窗口逻辑

那你把外层包装去掉，最多只能拿回一部分时间。

所以性能优化不能只看“少分配”，还要看“核心数学循环到底占多少”。

## 2. benchmark 最怕把不同层级的成本混在一起

如果一个 benchmark 同时测了：

- 参数检查
- 输出 buffer 检查
- 数据组织
- 真正的算法内核

那你最后得到的慢点，往往很难解释。

一个更好的办法是分层：

1. 测总成本
2. 测纯内核
3. 做比例

这样你才知道下一步该优化哪一层。

# 后续建议

现在更合理的下一步是：

1. 对 `wilders` 继续看 `run_in_place` 包装层
2. 对 `ema / macd / zlema` 转去看内核 codegen 或 benchmark 噪声
3. 不要再把这几类问题混成同一种热点来打
