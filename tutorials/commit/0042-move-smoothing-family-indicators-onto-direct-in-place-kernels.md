# 把平滑家族指标迁到直接的 in-place kernel

## 背景

在基准稳定性提高之后，`ema`、`wilders`、`atr` 这一组仍然反复偏慢。

它们有一个共同点：

- 都属于“平滑家族”或直接依赖平滑逻辑
- 算法本身并不复杂
- 但 Rust 侧之前还没有把 batch 路径完全压到“直接写输出缓冲区”的形式

这类热点很适合优先处理，因为一旦只靠高层 `Vec<Vec<_>>` 返回值或者 stream 回退路径，固定开销就会在 benchmark 里持续放大。

## 主要目标

把 `ema`、`wilders`、`atr` 这三类指标的热路径进一步压薄：

- batch 直接写调用方提供的输出切片
- 尽量避免 batch 路径依赖 stream 风格接口
- 保持现有 C/Rust 结果完全对齐

## 改动概览

### 1. 给 `ema` 增加真正的 `run_in_place`

[`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/ema.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/ema.rs) 现在不只是有高层 `run()`，还补了：

- `run_in_place`
- `EmaStream::feed_in_place`

这样 benchmark 和性能敏感调用方都可以直接走预分配输出缓冲区路径。

### 2. 给 `wilders` 增加直接 batch kernel

[`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/wilders.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/wilders.rs) 之前的 batch 是通过 `WildersStream` 间接完成的。

现在新增了：

- `run_wilders_batch`
- `run_in_place`
- `WildersStream::feed_in_place`

这是这次最有价值的变化，因为 `wilders` 本身就是一个基础平滑算子，性能回落会连带影响很多派生指标的判断。

### 3. 给 `atr` 增加直接 batch kernel

[`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/atr.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/atr.rs) 现在新增了：

- `run_atr_batch`
- `run_in_place`

这样 `atr` 的 batch 路径也不需要再依赖临时向量堆积结果后再复制出去。

## 关键知识

### 为什么这次要优先处理 `run_in_place`

在这个仓库里，性能敏感 batch 路径已经有一个很明确的经验：

- 先把结果直接写进调用方提供的缓冲区
- 再考虑更细的循环级优化

原因很简单：

- 如果 API 层每次都先分配，再包装，再复制
- 那么很多“算法不复杂”的指标也会被框架开销拖慢

所以 `run_in_place` 不是锦上添花，而是性能层的基础设施。

### 为什么 `wilders` 的收益比 `atr` 更明显

`wilders` 更像是一个“纯平滑核心”。

它的路径更短，去掉高层分配和间接层之后，收益就更直接。

而 `atr` 除了平滑本身，还要先算 `true range`，所以：

- 即使把输出路径压薄了
- 仍然还有更多内层成本留在里面

这就是为什么这次 `wilders` 回落更明显，而 `atr` 只是部分改善。

## 补充知识

### Rust / 性能知识点 1

不是所有“看起来差不多”的指标，优化收益都一样。

如果两个指标都用相同的高层 API，但其中一个还有额外前置计算，那么：

- 去掉 API 税
- 对“更薄”的那个指标帮助通常更大

所以要看清楚：

- 当前瓶颈是 API 层
- 还是算法层

### Rust / 性能知识点 2

`stream` 接口和 `batch` 接口在语义上相关，但不代表实现上应该互相代替。

一个常见的坏味道是：

- 为了少写代码，让 batch 借道 stream

这样往往会把状态机开销、额外分支、输出收集开销一起带进 batch 热路径。

更好的做法通常是：

- batch 有自己的 direct kernel
- stream 有自己的增量状态逻辑

共享的是数学语义，而不是强行共享整条执行路径。

## 验证

- `cargo fmt --all` (`PASS`)
- `cargo test` (`PASS`)
- `cargo clippy --all-targets --all-features` (`PASS`)
- `TI_BENCH_SIZES=4096,65536 TI_BENCH_TARGET_MS=300 TI_BENCH_REPEATS=3 cargo run --release --bin indicator-bench-compare` (`PASS`)

## 未覆盖项

- `atr stream` 仍然明显慢于 C，这次没有继续深入它的状态路径
- `ema` 虽然补了 direct in-place，但整体 benchmark 改善还不大，后续仍需要继续分析它在派生指标里的成本
- `wilders` 虽然明显改善，但还没有完全追平 C
