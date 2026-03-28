# 0028 Optimize lightweight batch indicators with in-place loops

## 背景

前一轮 benchmark 已经证明，Rust 指标库的很多性能问题不是算法本身，而是 API 层的固定开销。最典型的受害者就是价格合成和轻量 price/volume 指标：它们每个样本只做一两次浮点运算，但 Rust 版本之前还在走 `collect()`、`Vec<Vec<_>>` 包装，甚至有些 batch 路径直接复用了 stream。

## 主要目标

把一批“计算很轻、最容易暴露 API 税”的指标接到真正的高性能 batch 路径上：

- `avgprice`
- `medprice`
- `typprice`
- `wcprice`
- `marketfi`
- `tr`
- `bop`
- `lag`

## 改动概览

- 给价格合成层新增了统一的 in-place helper，让 `avgprice / medprice / typprice / wcprice` 都能直接写入调用方提供的输出缓冲区
- `marketfi`、`tr`、`bop` 的 batch 实现不再先构造 stream 再 `feed()`，而是改成直接循环
- `lag` 新增了 `run_in_place`，避免 benchmark 热路径走 `to_vec()` 再包 `Vec<Vec<_>>`
- 保留原来的高层 `run()` 和 stream API，所以语义和调用方式没有被破坏，只是给性能敏感路径补了更薄的一层

## 关键知识

这次最重要的经验是：对于超轻指标，性能瓶颈经常不在公式，而在“公式外面包了多少层”。

C 版本的这些指标通常就是：

```c
for (i = 0; i < size; ++i) {
    output[i] = ...
}
```

如果 Rust 版本是：

- `zip(...).map(...).collect()`
- 再 `Ok(vec![output])`
- 或者 batch 先 new 一个 stream 再 `feed()`

那么 benchmark 测出来的就不只是公式，而是“公式 + 抽象层税 + 分配税”。

## 补充知识

1. Rust 新手很容易把“迭代器风格更优雅”误解成“性能一定更好”。实际上在热路径上，显式 `for` 循环配合调用方提供的输出 buffer，经常更接近 C 的性能模型，也更容易让你确认到底花销在哪。

2. 做性能优化时，先挑“最简单却最慢”的热点，往往回报最高。因为这类代码最能暴露 API、分配和数据搬运问题，而不是把你带进复杂算法的细节里。

## 验证

- `cargo fmt --all` (`PASS`)
- `cargo test` (`PASS`)
- `TI_BENCH_SIZES=256,4096 TI_BENCH_TARGET_MS=20 cargo run --release --bin indicator-bench-compare` (`PASS`)

## 未覆盖项

- 这次主要优化的是轻量 batch 指标，复杂窗口类和 EMA 级联类还没有系统接入 `run_in_place`
- stream 热点仍然没动，`sma stream`、`atr stream` 这类回归还在
