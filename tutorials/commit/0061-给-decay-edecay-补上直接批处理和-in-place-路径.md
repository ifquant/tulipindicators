# 给 `decay` / `edecay` 补上直接批处理和 `in-place` 路径

## 背景

这一轮的目标不是“随便改几个热点”，而是先按全量 benchmark 的前 10 排行，挑那种原因最明确、收益最稳定的点下手。

`decay` 属于很典型的一类：

- 指标公式本身很简单
- C 版本是一层很薄的顺序循环
- Rust 版本虽然逻辑正确，但 batch 还在走“先分配 `Vec`，再一路 `push`”的较厚路径

这种指标特别适合先优化，因为你很容易判断慢在哪里，也容易确认优化之后有没有真实收益。

## 这次主要做了什么

这次没有改公式语义，只改了 batch 路径的实现形态：

- 给 `decay` 增加了 `run_in_place(...)`
- 给 `edecay` 也一起补了同样的 `run_in_place(...)`
- 抽出了共享的 `run_decay_batch(...)`
- `run(...)` 不再靠 `Vec::with_capacity + push` 一边算一边长，而是直接分配目标长度后顺序写入

这样做的目的很直接：

- 让 benchmark 走到更接近 C 的输出模型
- 减少 batch 路径里的动态增长和额外包装
- 保持高层易用 API 还在，但让热路径能更薄

## 为什么这笔值得单独提交

因为它是“有效收益”而不是“实验中”。

我同一轮还试了 `cvi`，方向是把它从 `stream-backed batch` 改成 direct batch kernel。但 `cvi` 的 focused benchmark 结果不够稳定，特别是 `65536` 档没有稳定进线，所以这次故意没有把它混进提交。

这也是人机协作里很重要的一点：

- 不要因为已经动过代码，就强行把实验塞进 commit
- 只有验证站稳的部分，才值得进历史

## 验证结果

focused benchmark：

- `decay 4096`: `0.491x`
- `decay 65536`: `0.737x`

也就是这两档都已经进入这轮的新目标 `0.8x` 以内。

## 给 Rust 新手的两个知识点

### 1. `Result<Vec<Vec<Real>>, Error>` 是“拥有所有权的返回值”，不是数组引用

像这种接口：

```rust
fn run(&self, ...) -> Result<Vec<Vec<Real>>, IndicatorError>
```

返回的是一整块新分配出来、由调用方接手所有权的数据。

这很好用，但直觉上就比 C 里“调用方先分配好输出缓冲区，我只负责往里写”的模型更重。

所以性能敏感库常见的 Rust 设计不是只保留一种接口，而是分层：

- 高层易用接口：直接返回 `Vec`
- 低层高性能接口：调用方提供输出切片

这次的 `run_in_place(...)` 就属于第二层。

### 2. `Vec::with_capacity(...)` 不等于“已经有可写元素”

很多新手会以为：

```rust
let mut output = Vec::with_capacity(n);
```

之后就能直接 `output[i] = ...`。

其实不行，因为它只是“容量够了”，长度还是 `0`。你只能继续 `push(...)`，或者改成：

```rust
let mut output = vec![0.0; n];
```

后者才是真的创建了 `n` 个可写元素。

在性能敏感代码里，这个区别很重要：

- `with_capacity + push` 适合“长度不确定”
- `vec![0.0; n] + 顺序写入` 更适合“长度确定”的批处理内核

## 这次没有包含什么

- `cvi` 的 direct batch 实验没有稳定达标，所以已撤回，不在本次提交里
- 还没有继续处理 `medprice`、`sub`、`crossover`、`dema`
- 也没有改 stream 路径，这次只收 batch 热点
