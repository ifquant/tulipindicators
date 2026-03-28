# 把 AD 从 stream-backed batch 路径上拆下来

这次提交只做一件小而值的事：把 `ad` 的批处理路径从 `stream.feed()` 拆出来，改成直接 batch kernel。

## 为什么先做 `ad`

前一轮 focused benchmark 里，`ad` 不是最夸张的热点，但它有两个特点很适合优先处理：

1. 公式很薄，只是一个累积循环
2. Rust 版当时还在 batch 里复用 `AdStream`

这种指标通常很适合“花很小的改动，收一笔很确定的性能收益”。

这次 focused benchmark 的结果也证明了这一点：

- `ad 4096`: `0.914x`
- `ad 65536`: `0.787x`

也就是说，拆完以后 Rust 已经追平并在大输入上反超。

## 这次到底改了什么

改动主要在 [`rust/src/indicators/indicator/price_volume.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/price_volume.rs)：

1. `Ad::run()` 不再创建 `AdStream`
2. 新增 `run_in_place()`，让高性能路径可以直接写调用方提供的输出缓冲区
3. 提取了 `run_ad_batch(...)`，把批处理逻辑写成和 C 版一致的直接累积循环

这个批处理内核的逻辑很简单：

- 先维护一个累计和 `sum`
- 每根 bar 计算一次 `(2 * close - low - high) / (high - low) * volume`
- 累加到 `sum`
- 直接写到输出里

它的重点不在“写得多抽象”，而在“尽量少让 batch 走一圈 stream 状态机”。

## 为什么 `adosc` 这次没有一起交

这次顺手试了 `adosc`，但没有一起提交。

原因很简单：

- `4096` 已经回到阈值附近
- `65536` 还没有稳定压到目标内

这说明它不像 `ad` 这样是一笔“拆掉 stream-backed batch 就立刻见效”的干净收益。  
如果把这种半成品和 `ad` 混在一起提交，commit 历史就会变脏，也不利于后面继续定位真问题。

## 给 Rust 新手的两个知识点

### 1. “高性能接口”不等于“unsafe 接口”

这次新增的 `run_in_place()` 仍然是安全接口，它只是把“由函数内部自己分配输出 `Vec`”改成“调用方提供输出缓冲区”。

这类接口常见于性能敏感库，因为它能明显减少：

- 分配
- 包装
- owned 返回值的额外成本

但它仍然可以保留：

- 参数检查
- 输出长度检查
- 正常的 `Result` 错误返回

### 2. 批处理和流式实现可以共用语义，但不一定该共用执行路径

很多新手会觉得：

> 既然公式一样，batch 直接复用 stream 不就好了？

功能上通常没问题，但性能上往往会吃亏。因为 stream 路径会天然带着：

- 状态推进
- 分块输出
- 额外的分配或包装

如果指标本身只是一个很薄的循环，这些额外层就会变成主要成本。

## 这次的结论

`ad` 这次是一个很典型的“小手术高回报”案例：

- 不需要改算法
- 不需要动 stream 语义
- 只是把 batch 从 stream-backed 路径上拆下来

然后性能就明显回来了。

这也说明，继续做性能优化时，一个很有效的策略是：

> 先找“公式薄、还在走 stream-backed batch”的指标，优先处理这些低垂果实。
