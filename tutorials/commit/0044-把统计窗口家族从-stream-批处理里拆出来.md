# 把统计窗口家族从 stream 批处理里拆出来

这次优化的目标很明确：`stddev`、`stderr`、`var` 这三个指标在 benchmark 里一直是稳定热点，而且它们其实是同一类问题。它们的公式不复杂，C 版本就是很薄的滑动窗口循环；但 Rust 版本之前的 batch 路径却是“先 new 一个 stream，再把整段输入喂进去”。这会把本来不该出现在 batch 热路径里的状态机和结果收集开销一起带进去。

所以这次不是只修一个指标，而是把这整个“统计窗口家族”的 batch 路径一起拉平。

## 这次改了什么

先在 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/shared.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/shared.rs) 里增加了一个共享的 `rolling_variance_batch(...)` helper。它做的事情非常接近 C：

- 先算出首个窗口的 `sum` 和 `sum2`
- 直接写出第一个输出
- 后续每前进一步，就做一次
  - `sum += new - old`
  - `sum2 += new^2 - old^2`
  - 重新算 `variance`

然后在下面三个指标里都接上真正的 batch 内核和 `run_in_place`：

- [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/stddev.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/stddev.rs)
- [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/stderr.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/stderr.rs)
- [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/var.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/var.rs)

与此同时，stream 路径也补上了 `feed_in_place`，这样 benchmark 在测 stream 时不会再退回到“先 push 到 `Vec`，再 copy 到输出切片”的慢路径。

## 为什么这次优化效果特别大

这组三个指标之前慢，不是因为 Rust 算得慢，而是因为 Rust 在 batch 路径上多绕了一大圈。

原来大概是这样：

1. `run()`
2. 创建 stream 状态对象
3. 一点点 `feed(...)`
4. `push` 到 `Vec`
5. 最后再返回结果

而 C 做的是：

1. 直接滑窗循环
2. 直接写输出

这类“窗口内只有几个标量更新”的指标，非常怕框架层绕远路。因为公式本身太轻了，任何额外对象、分配、push、状态机分支，都会被 benchmark 放大。

所以这次的结果非常明显：

- `stddev batch 4096`: `0.445x`
- `stderr batch 4096`: `0.445x`
- `var batch 4096`: `0.439x`
- `stddev batch 65536`: `0.493x`
- `stderr batch 65536`: `0.506x`
- `var batch 65536`: `0.440x`

也就是说，这组三个热点现在都已经明显快于 C。

## 新手 Rust 知识 1：`run_in_place` 的意义到底是什么

很多新手第一次看这类性能接口时，会问：

“为什么不直接 `return Vec<f64>`，这样不是更 Rust 吗？”

高层 API 当然可以这么做，但热路径不适合只剩这一层。原因是：

- `Vec` 需要分配
- 返回值往往还要再包一层
- benchmark 或调用方可能只是想把结果写进已有缓冲区

`run_in_place` 的意思就是：

- 调用方把输出缓冲区先准备好
- 指标只负责往里写
- 返回真正写了多少个值

这类接口看起来没那么“优雅”，但在性能敏感场景下非常有价值。它不是为了取代高层接口，而是为了给热路径留一条薄通道。

## 新手 Rust 知识 2：为什么共享 helper 比复制三份循环更好

这次我没有给 `stddev / stderr / var` 各自复制一份几乎一样的滑窗循环，而是抽了一个共享的 `rolling_variance_batch(...)`。

这么做的原因不是“为了抽象而抽象”，而是这三个指标的差异非常小：

- `var`: 直接输出 `variance`
- `stddev`: 输出 `sqrt(variance)`
- `stderr`: 输出 `sqrt(variance) * scale`

也就是说，真正不同的只是“最后怎么映射这个 variance”。这种情况下，把公共滑窗逻辑收成一个 helper，反而更容易保证：

- 三个指标的行为一致
- 将来如果要继续调这段热路径，只改一个地方
- 不会出现一个指标修了，另外两个还停留在旧实现上的情况

## 这次最值得记住的点

优化一组指标时，先找“它们是不是同一条慢路径”。

如果答案是“是”，那最赚的做法通常不是逐个修，而是把那条共享的慢路径直接拆掉。
