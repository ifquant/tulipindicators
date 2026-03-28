# 把 DPO 彻底拉平，并把 KVO 从 stream 批处理里拆出来

这次处理的是两个仍然明显偏慢的 batch 热点：`dpo` 和 `kvo`。它们有一个共同问题：Rust 之前的 batch 路径都还是“创建 stream 状态对象，再整段 feed 输入”。这种写法对功能迁移很方便，但对性能非常不友好，因为 batch 原本应该是一条很薄、很直接的循环。

## 这次改了什么

### `dpo`

在 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/dpo.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/dpo.rs) 里：

- 新增了直接 batch 内核 `run_dpo_batch(...)`
- batch `run()` 不再走 `stream.feed(...)`
- 补了 `run_in_place(...)`
- stream 路径也补了 `feed_in_place(...)`

`DPO` 的 C 实现其实非常直接：

1. 先算一个长度为 `period` 的窗口和
2. 再输出 `input[i - back] - average`

Rust 以前没有利用这个特点，而是先把它包成了 `VecDeque` 状态机。现在 batch 直接走和 C 很接近的滑窗和更新方式。

### `kvo`

在 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/kvo.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/kvo.rs) 里：

- 新增了直接 batch 内核 `run_kvo_batch(...)`
- batch `run()` 和 `run_in_place()` 不再绕 stream
- stream 路径补了 `feed_in_place(...)`

`KVO` 比 `DPO` 复杂得多，因为它内部还有：

- `trend` 切换
- `cm` 累加
- `vf` 计算
- 短长两条 EMA

所以这次虽然把 batch 路径切薄了，但它不一定能像 `DPO` 一样一下子完全追平。

## 这次的真实效果

`DPO` 这次是非常成功的：

- `dpo batch 4096`: `0.477x`
- `dpo batch 65536`: `0.515x`

也就是说，Rust 现在明显快于 C。

`KVO` 则是“方向对了，但还没收口”：

- `kvo batch 4096`: `1.442x`
- `kvo batch 65536`: `1.434x`

这说明现在剩下的差距，已经不主要是“batch 走 stream”这层税了，而更像是 `KVO` 自身状态更新和 EMA 组合部分还有优化空间。

## 新手 Rust 知识 1：为什么 `run_in_place` 对复杂指标也有价值

很多人会误以为只有特别简单的指标才值得做 `run_in_place`。其实不是。

对于 `KVO` 这种复杂指标，公式本身确实比 `DPO` 重很多，但这不代表接口层开销就可以忽略。只要 batch 路径还在：

- 分配输出 `Vec`
- `push`
- 返回 owned 结果
- 再让 benchmark 或调用方复制

这些成本就始终存在。

所以 `run_in_place` 的价值不是“让复杂指标一定快过 C”，而是先把不该存在的接口税切掉。切掉以后，你才能更准确地看到剩下到底是谁慢。

## 新手 Rust 知识 2：性能优化里“阶段性胜利”也值得单独提交

这次 `DPO` 完全收掉了，但 `KVO` 还没追平。很多新手会觉得：

“那是不是应该继续改到两个都完美了再提交？”

实际工程里，这往往不是好主意。原因是：

- `DPO` 这笔已经是确定收益
- `KVO` 目前也已经完成了架构层面的正确切换
- 如果强行再把后续更深的 `KVO` 优化塞进同一笔，提交就会开始混杂

更好的做法是：

1. 先把“已确定成立的那一层优化”提交掉
2. 再针对 `KVO` 剩下的真正热点继续深挖

这样人和 AI 回头看历史时，能很清楚地知道：

- 哪一笔是在拆 API 层慢路径
- 哪一笔是在抠更深的状态更新成本

## 这次最值得记住的点

性能优化不是只有“追平”才算成功。

只要你把一个真实的慢路径拆掉，并且能明确说明“剩下的慢已经换成另一类问题”，这就是高价值进展。
