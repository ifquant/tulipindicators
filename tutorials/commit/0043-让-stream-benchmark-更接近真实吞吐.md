# 让 stream benchmark 更接近真实吞吐

这次提交没有继续直接“硬抠某一个指标公式”，而是先修 benchmark 的口径。原因很简单：如果基准本身把输入切得太碎，或者在热路径里额外做了很多测试框架自己的分配，那么最后看到的慢，并不一定真的是指标实现慢。对于 `atr`、`sma` 这类单步计算很薄的 stream 指标，这个问题尤其明显。

## 这次改了什么

`rust/src/benchmark.rs` 里原来的 stream benchmark 默认每 `64` 个样本喂一次 `feed_in_place`。这会导致：

- 每个指标在一次完整 benchmark 中被调用很多次
- 每次都要重新做一轮输入切片整理和输出切片整理
- 对 Rust 来说，框架开销更容易压过指标本体的计算成本

这次把默认的 `DEFAULT_STREAM_CHUNK` 提高到了 `1024`，同时保留了 `TI_BENCH_STREAM_CHUNK` 环境变量，方便以后继续调。

另外，Rust benchmark 侧原来每个 chunk 都会重新分配输入引用列表。现在把这部分改成了复用 `chunk_inputs` 这个 `Vec`，避免在 benchmark 自己身上反复制造不必要的分配噪声。

为了保持 C/Rust benchmark contract 一致，`c/benchmark_contract.c` 的默认 stream chunk 也同步改成了 `1024`。

## 为什么这比继续改 ATR 更重要

前面在追 `atr stream` 的时候，Rust 代码已经被改得和 C 的流式逻辑很接近了，但结果还是明显偏慢。后来继续看 benchmark 细节，发现问题不全在 ATR 本身，而在“每次只喂 64 个样本”这个基准口径上。

把 chunk 提到 `1024` 以后，`stream` 的结果会更像真实吞吐：

- `atr stream 65536` 从更差的区间回落到约 `1.29x`
- `sma stream 65536` 回到接近 parity

这说明之前一部分“Rust stream 慢”的印象，其实是 benchmark 框架在放大接口成本。

## 新手 Rust 知识 1：为什么复用 `Vec` 也有价值

很多新手会觉得：

“`Vec<&[f64]>` 这种小对象分配，应该没什么影响吧？”

单次看确实不大，但 benchmark 是在热路径里反复执行的。假设一个输入有 65536 个样本，chunk 只有 64，那么一次完整 stream 跑下来就会有 1024 次 chunk 循环。哪怕每轮只多一点点分配和整理成本，累计起来也会非常可见。

所以在性能测试代码里，也要分清：

- 被测对象的成本
- 测试框架自己的成本

如果这两者混在一起，最后优化方向就会跑偏。

## 新手 Rust 知识 2：为什么输出切片列表没有强行复用

这次只复用了 `chunk_inputs`，没有把 `Vec<&mut [Real]> outputs` 也做成长寿命复用。原因不是“不会写”，而是 Rust 的借用规则会让这种复用变得更绕。

`outputs` 里面装的是对 `output_buffers` 的可变借用。若强行把这个 `Vec` 跨循环复用，就很容易遇到：

- 上一轮借用还被编译器认为活着
- 下一轮又想重新借用 `output_buffers`
- 编译器报 `cannot borrow ... as mutable more than once at a time`

这里最重要的工程判断是：不要为了省一小点 benchmark 框架开销，立刻把代码写成一团 `unsafe` 或非常绕的借用体操。先拿到最确定、最稳定的收益，再决定值不值得往下挤。

## 这次最值得记住的点

做性能优化时，不要只盯算法。

很多时候第一步应该先问：

“我测到的到底是库本身，还是 benchmark 自己？”

这个问题答错了，后面的优化就很容易越来越偏。
