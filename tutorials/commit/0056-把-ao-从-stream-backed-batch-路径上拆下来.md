# 把 AO 从 stream-backed batch 路径上拆下来

这次提交只做一件事：把 `ao` 的批处理路径从 `stream.feed()` 拆出来，改成直接批处理内核。

## 背景

之前 `ao` 的 Rust `run()` 实现，和不少早期迁移过来的指标一样，直接复用了 stream 状态机。这样做的好处是迁移快、语义集中，但坏处也很明显：

- batch 路径会带上 stream 的状态推进成本
- benchmark 会测到额外的分配和包装开销
- 对 `ao` 这种公式本来就很薄的指标，这些开销会特别显眼

focused benchmark 已经说明了问题：

- 修改前：`4096 = 2.792x`，`65536 = 1.395x`
- 修改后：`4096 = 0.562x`，`65536 = 0.471x`

也就是说，这不是“Rust 天生慢”，而是“路径选错了”。

## 这次改了什么

这次在 [`rust/src/indicators/indicator/ao.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/ao.rs) 里做了三件事：

1. `run()` 不再新建 `AoStream` 再 `feed(inputs)`，而是直接走 `run_ao_batch(...)`
2. 补了 `run_in_place(...)`，让 benchmark 和性能敏感调用方都能走低开销接口
3. 把 C 版的双滑窗思路直接翻成 Rust 薄循环：
   - 一个 `34` 窗口
   - 一个 `5` 窗口
   - 先预热
   - 再边前进边减旧值

这里的设计重点不是“写得更 Rust 花哨”，而是尽量贴近 C 的性能模型。

## 中途踩到的坑

第一次写 `run_in_place()` 时，我把 `ensure_output_len(...)` 的参数传反了，把 `33` 这个 lookback 错传到了 `output_index` 位置。

结果 benchmark 不是给出性能数字，而是先抛了：

```text
OutputTooSmall { indicator: "ao", output_index: 33, expected: 4096, actual: 4063 }
```

这个错误其实很有价值，它说明高性能接口也不能只盯着“快不快”，还得先把输出契约对齐。

修正后：

- `expected` 改成 `high.len().saturating_sub(33)`
- `output_index` 正确填回 `0`

然后 benchmark 才能正常跑。

## 给 Rust 新手的两个知识点

### 1. `Result<Vec<Vec<Real>>, IndicatorError>` 真的是“返回整块拥有所有权的数据”

它不是数组引用，也不是借用切片。

这意味着：

- 外层 `Vec` 要分配
- 每个输出通道的内层 `Vec` 也通常要分配
- 返回时所有权会交给调用方

这种接口很适合“先把功能用起来”，但对超轻指标不够极致，所以我们才要同时保留 `run_in_place(...)` 这层高性能接口。

### 2. 性能优化最容易先炸出来的是“契约 bug”，不是速度 bug

很多人一提性能优化，就只盯着 `ns/input`。

但真实工程里，更常见的顺序是：

1. 先把高层路径改成薄路径
2. 结果先触发输出长度、起始偏移、warmup、lookback 这类契约错误
3. 修完这些，性能数字才有意义

所以“测试先失败”往往不是坏事，而是在提醒你：实现虽然更快了，但还没真正对齐旧语义。

## 这次的结论

`ao` 是一个很标准的案例：

- 它慢，不是因为公式复杂
- 也不是因为 Rust 算不快
- 而是因为 batch 错走了 stream 路径

把路径纠正之后，它立刻从明显回归变成了明显领先。

这也再次说明一个原则：

> 对性能敏感的 Rust 指标库，高层易用 API 可以保留，但 batch 热路径必须有一层贴近 C 风格的直接内核。
