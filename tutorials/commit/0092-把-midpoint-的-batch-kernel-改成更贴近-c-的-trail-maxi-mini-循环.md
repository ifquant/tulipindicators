# 把 midpoint 的 batch kernel 改成更贴近 C 的 trail/maxi/mini 循环

这次修的是 `midpoint`。它之前功能是对的，但 Rust 的 batch 路径一直在用两条 `MonotonicQueue`；C 版则不是队列风格，而是经典的 `trail / maxi / mini / max / min` 滑窗循环。对照后可以看出，Rust 这边虽然写法更统一，但对这个指标来说，批处理内核并没有必要带两条通用队列。

## 这次改了什么

我只改了 batch 内核，没有动 stream：

- 把 `run_midpoint_batch(...)` 从双 `MonotonicQueue` 改成更贴近 C 的 `trail / maxi / mini` 扫描逻辑
- 保留 `MidPointStream` 的队列实现，因为 stream 路径本来就是另一种状态建模
- 把首窗的 sentinel 语义也对齐到了 C：`maxi = -1`、`mini = -1` 对应 Rust 里的 `isize` sentinel

这样做之后，Rust batch 热循环不再为这个指标引入通用队列维护成本。

## 为什么要特别强调 sentinel

这次中途踩了一个很典型的坑：我一开始把 `maxi` / `mini` 直接初始化成 `0`，结果 `talib_missing_parity` 立刻失败。原因是 C 版靠 `-1` 这个 sentinel 强制第一窗完整重扫，而不是复用一个已经存在的窗口索引。

这类问题很适合新手记住：

- `usize` 不能表达 `-1`
- 如果原始 C 算法明显依赖负值 sentinel，Rust 往往要改成 `isize`、`Option<usize>`，或者额外的布尔状态
- 不要因为“Rust 不喜欢负下标”就把 sentinel 语义偷偷改成 0，那往往会直接改坏算法

## 结果

focused benchmark 结果：

- `midpoint 4096 = 0.954x`
- `midpoint 65536 = 0.904x`

也就是 batch 两档都已经快于 C。

## 这次顺手能学到的 Rust/性能知识

### 1. 通用数据结构不一定适合所有热点

`MonotonicQueue` 很好用，也适合很多窗口类指标。  
但如果某个 C 指标本来就有一条更轻、更直接的专用循环，那在 Rust 里继续硬套通用结构，可能反而让 batch 热路径更厚。

### 2. batch 和 stream 可以故意用不同内核

这次没有强迫 `midpoint` 的 batch 和 stream 一定共用同一套内部结构。  
性能敏感代码里，这很正常：

- batch 追求一次性吞吐
- stream 追求逐点状态推进

两条路径外部语义一致，不代表内部一定要一模一样。

## 还没做的事

- `midpoint` 的 stream 路径这次没有改
- 这次只处理了 `midpoint`，`midprice` 之类其它价格/窗口指标要不要进一步按 C 风格专门化，还要看后续 benchmark
