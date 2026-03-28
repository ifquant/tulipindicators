# 把 `wad` 和 `emv` 从 stream-backed batch 路径上拆下来

这次改动是一次很典型的性能修复：不是改公式，不是改结果，而是把本来应该走“直接批处理”的代码，从错误的“借道 stream 状态机”路径里拆出来。

前面我们已经通过 benchmark 定位到两个非常像“经典热点”的指标：

- `wad`
- `emv`

它们有几个共同特点：

- 公式本身并不复杂
- C 版本实现非常薄
- Rust 版本却明显慢
- 慢得很像“接口层税”，不像算法本体慢

结果一看代码，确实是这样。

## 这次到底改了什么

### 1. `emv` 不再让 batch 借道 stream

在 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/emv.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/emv.rs) 里，之前的 `run()` 是这样工作的：

1. 先创建 `EmvStream`
2. 再调用 `stream.feed(inputs)`
3. `feed()` 里一边维护 `last_midpoint`，一边 `push` 输出

这对 stream 接口当然没问题，但对 batch benchmark 来说，就多了一层完全没必要的状态机和 `Vec` 构造路径。

这次改成了：

- `run()` 直接走 `run_emv_batch(...)`
- 新增 `run_in_place(...)`
- `run_emv_batch(...)` 直接写输出切片

也就是说，batch 终于和 C 一样，回到了“薄循环”模型。

### 2. `wad` 也不再让 batch 借道 stream

在 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/price_volume.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/price_volume.rs) 里，`Wad` 之前也是：

- `run()` 里先 `WadStream::new(...)`
- 再 `feed(inputs)`

但 C 版本 [`/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/wad.c`](/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/wad.c) 本来就是一层非常直接的累计循环。

所以这次也做了同样的处理：

- `run()` 改成直接走 `run_wad_batch(...)`
- 新增 `run_in_place(...)`
- 内核直接按 `previous_close` 和累计 `sum` 写出结果

这样一来，`wad` 的 batch 路径终于和它的数学结构匹配了。

## 为什么这一步值钱

因为这两个点都非常适合用来证明一个性能经验：

> 很多 Rust 性能问题，不是“Rust 数学慢”，而是“热路径绕到了错误的抽象层”。

`wad` 和 `emv` 都不是复杂指标。  
如果这种指标都能慢出 1.8x 到 2.1x，那第一怀疑对象就不该是公式，而应该是：

- 是否复用了 stream
- 是否多做了动态分配
- 是否多走了一层高层返回包装

这次 benchmark 结果也正好验证了这个判断。

## 这次 benchmark 的结果

我跑的是：

```bash
TI_BENCH_INDICATORS=wad,emv \
TI_BENCH_SIZES=4096,65536 \
TI_BENCH_TARGET_MS=120 \
TI_BENCH_CALIBRATION_MS=30 \
TI_BENCH_REPEATS=5 \
cargo run --release --bin indicator-bench-compare
```

结果是：

- `emv batch 4096`: `2.183x -> 0.746x`
- `emv batch 65536`: `2.106x -> 0.759x`
- `wad batch 4096`: `1.850x -> 1.091x`
- `wad batch 65536`: `1.698x -> 0.924x`

这说明：

- `emv` 的慢几乎完全是路径问题，拆掉 stream-backed batch 后直接反超
- `wad` 也基本被拉平，已经回到阈值内

## 给 Rust 新手的两个知识点

### 1. 同一个状态逻辑，不等于同一个最佳 API 路径

很多新手会觉得：

- “反正 batch 和 stream 都是同一个公式，那 batch 直接复用 stream 不就行了吗？”

功能上，这通常能跑通。  
但性能上，这经常是错的。

因为 stream 接口天然要处理：

- 增量状态
- chunk 输入
- 不确定长度的输出收集

而 batch 更适合：

- 一次性拿到完整输入
- 预先知道输出长度
- 直接写切片

所以“复用逻辑”和“复用接口”不是一回事。  
可以复用数学思想，但不一定要复用同一条执行路径。

### 2. `run_in_place` 的意义，不只是少分配一个 `Vec`

很多人第一次看到 `run_in_place`，会以为它的意义只是“少 new 一个容器”。

其实更重要的是：

- 调用方已经知道输出长度
- 指标实现可以直接写目标缓冲区
- 热循环不需要再包一层 owned 结果模型

这对轻量指标尤其重要。  
因为轻量指标本身计算量很小，接口层多绕一圈，比例就会被无限放大。

## 这一步还没做什么

这次只收了 `wad` 和 `emv`，还没有动这些点：

- `wilders`
- `vosc`
- `lag`
- 更大的一批 EMA / smoothing 家族尾差

所以这次不是“性能问题已解决”，而是“先把两个最标准的 stream-backed batch 热点拆平”，给后面继续清热点打基础。
