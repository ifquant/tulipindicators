# 0029 Use benchmark-guided hotspots before heavy profiling

## 背景

上一轮已经把一批最轻量的 batch 指标接进了高性能 in-place 路径，但 benchmark 仍然显示一些热点没有消失，尤其是：

- `stochrsi`
- `nvi`
- `obv`
- `pvi`
- `crossany`
- `crossover`

这次的目标不是盲目上 profiler，而是先判断这些热点是不是还停留在“batch 没走薄循环”的阶段。如果是，那直接补高性能 batch 路径往往比先做复杂 profile 更划算。

## 主要目标

把这几个仍然明显带着高层开销的指标接上真正的 batch 快路径，并用更稳定的 benchmark 参数复核结果。

## 改动概览

- `stochrsi` 现在有了真正的 batch 实现和 `run_in_place`，不再让 batch 只是调用 stream
- `nvi`、`obv`、`pvi` 的 batch 路径改成直接循环，并新增 `run_in_place`
- `crossany`、`crossover` 新增 `run_in_place`，让 benchmark 可以绕开 `Vec<Vec<_>>` 包装
- 这次还额外用更稳定的 benchmark 口径复核了热点，而不是只看一次短跑结果

## 关键知识

这里最重要的方法论是：

- 如果一个指标的公式本身不复杂，但 benchmark 还是慢，先怀疑“有没有走错 API 层”
- 只有当 batch 已经是薄循环、调用方缓冲区、没有多余分配后，再去考虑更重的 profiler

换句话说，profiler 最有价值的时候，是在你已经把明显的 API 税清掉之后。

## 补充知识

1. Rust 新手容易把“stream 实现已经写好了”当成“batch 可以直接复用”。这是很常见的性能陷阱。stream 关注状态推进，batch 关注吞吐；两者语义相关，但性能模型不一样。

2. 做性能优化时，benchmark 也要讲“测量卫生”。如果一次短跑结果特别反常，不要马上相信它。先换更长的测量窗口，看看趋势是否稳定，再决定要不要做更重的 profiler 分析。

## 验证

- `cargo fmt --all` (`PASS`)
- `cargo test` (`PASS`)
- `TI_BENCH_SIZES=256,4096 TI_BENCH_TARGET_MS=20 cargo run --release --bin indicator-bench-compare` (`PASS`)
- `TI_BENCH_SIZES=4096,65536 TI_BENCH_TARGET_MS=200 cargo run --release --bin indicator-bench-compare` (`PASS`)

## 未覆盖项

- 这次没有给 benchmark 工具补“只跑指定指标”的正式过滤接口
- 更复杂的窗口/平滑类热点还在，比如 `hma`、`kama`、`vidya`、`wma`
