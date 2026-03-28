# 让 benchmark 同时和 C 以及 Rust 自己的历史最好结果比较

这次改动的背景很直接：只看 `C vs Rust` 还不够。你会知道 Rust 现在有没有比 C 快，但你不知道这次改动到底是让 Rust 进步了、退步了，还是只是和上次差不多。对于持续性能优化来说，这会让“朝哪个方向继续推”变得很模糊。

这次的主要目标，是把现有的 `indicator-bench-compare` 扩成两层比较：

1. `C vs Rust`
2. `Rust 当前结果 vs Rust 历史最好结果`

这样以后每次跑 benchmark，都能同时回答两个问题：

- 现在和 C 比怎么样？
- 现在和 Rust 自己过去最好的一次比怎么样？

## 这次做了什么

现在运行：

```bash
cargo run --release --bin indicator-bench-compare
```

会多生成几份文件：

- `target/indicator-bench/rust-stable-latest.tsv`
  当前这次 Rust benchmark 的原始结果
- `target/indicator-bench/rust-best.tsv`
  每个 `(indicator, mode, input_len)` 历史上最好的 Rust 结果
- `target/indicator-bench/rust-self-compare-latest.tsv`
  当前结果和历史最好结果的逐项对比
- `target/indicator-bench/rust-self-compare-latest.md`
  给人看的 Markdown 报告

这次实现里有一个关键设计：`rust-best.tsv` 不是手工维护的，而是每次跑 benchmark 时自动读取旧基线、和当前结果合并，然后保留更快的那一条。

也就是说：

- 如果这次更快，就刷新历史最好
- 如果这次更慢，也不会把历史最好覆盖掉

这样你就有了一条“Rust 自己的性能上限轨迹”。

## 为什么这比只看 C 对比更有用

有时候 Rust 还没有超过 C，但已经比自己上周快了很多。

如果你只看 `C vs Rust`，你会觉得“还是慢”。  
如果你再看 `Rust vs self-best`，你能知道：

- 这次到底有没有继续进步
- 这次是不是把之前的优化吃回去了
- 这次是不是刷新了某些指标的最佳成绩

这对做长期性能迭代很重要，因为性能优化常常不是一刀砍翻，而是很多次小幅收紧。

## 新手知识点 1：`Result<Vec<Vec<Real>>, IndicatorError>` 为什么会带来性能压力

Rust 里的：

```rust
Result<Vec<Vec<Real>>, IndicatorError>
```

表示“成功时返回拥有所有权的输出数据，失败时返回错误”。

这里的 `Vec<Vec<Real>>` 不是引用，而是真正返回一块新分配的数据。直观含义是：

- 外层 `Vec` 需要分配
- 每个输出通道的内层 `Vec` 也常常需要分配
- 返回时把这整块数据的所有权交给调用方

这对易用 API 很合理，但对轻量指标来说，分配和拷贝本身就可能比指标计算更贵。

这也是为什么后来库里要补：

- 高层易用接口：`run(...)`
- 高性能接口：`run_in_place(...)`

## 新手知识点 2：为什么“历史最好”不是简单平均，而是按键值逐项保存

这里保存历史最好时，不是把所有 benchmark 混成一个总分，而是按：

- `indicator`
- `mode`
- `input_len`

这三个维度分别保存最优 `ns/input`。

原因是不同指标、不同模式、不同输入规模的性能形态完全不一样。  
如果只保存一个总平均值，你会丢失真正有用的信息。

例如：

- `sma batch 4096`
- `sma stream 65536`

虽然都叫 `sma`，但它们其实是两种不同的性能问题。

## 这次最重要的设计经验

做性能基础设施时，不要只问“我现在比别人快吗”，还要问“我有没有比自己过去更快”。

前者决定竞争力，后者决定优化工作是不是在持续产生真实积累。
