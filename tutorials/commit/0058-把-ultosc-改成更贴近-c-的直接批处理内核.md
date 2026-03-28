# 把 ULTOSC 改成更贴近 C 的直接批处理内核

这次提交处理的是 `ultosc`。

它之前的 Rust batch 路径功能上没有问题，但执行方式更像“拿 stream 状态机硬跑完整个输入”，而 C 版是一个更贴近批处理思维的实现：

- 两个长窗口环形缓存
- 长窗口和自己维护和
- 短/中窗口借同一个长窗口做扣减

这类指标很适合“用 direct batch kernel 重新表达同一语义”，因为热点不在公式本身，而在状态推进方式。

## 为什么这次值得做

在更稳的 focused benchmark 里，`ultosc` 留下了一个比较清晰的信号：

- `4096` 不一定总是慢
- 但 `65536` 这种长输入下，Rust 还会稳定落后

这类现象通常说明：

- 不是纯粹的固定开销问题
- 更像每一步状态更新方式本身比 C 重

改完之后结果就很干净了：

- `ultosc 4096`: `0.598x`
- `ultosc 65536`: `0.728x`

也就是说，这次不是“勉强压进 1.2x”，而是直接跑到明显更快。

## 这次改了什么

改动在 [`rust/src/indicators/indicator/ultosc.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/ultosc.rs)：

1. `run()` 不再走 `UltOscStream::feed()`
2. 新增 `run_in_place()`
3. 新增 `run_ultosc_batch(...)`

新的 batch 内核大意是：

- 每步先计算 `bp` 和 `range`
- 用两个长度为 `long_period` 的数组保存历史值
- `bp_long_sum / r_long_sum` 直接靠环形覆盖维护
- `bp_short_sum / bp_medium_sum / r_short_sum / r_medium_sum` 用索引回退做 piggy-back 扣减

这个写法和 C 的思路非常接近，所以很适合拿来回收 batch 热路径损耗。

## 为什么这次比只用 `RingSum` 更快

之前 stream 版里，每一步都要推进 6 个 `RingSum`：

- `bp_short`
- `bp_medium`
- `bp_long`
- `r_short`
- `r_medium`
- `r_long`

这在语义上很直观，但对批处理来说不够薄。

新的 direct batch kernel 把它压缩成：

- 两个真实的长窗口 ring buffer
- 四个 piggy-back 累积和

这样每步的更新更像 C，也更少抽象层。

## 给 Rust 新手的两个知识点

### 1. “环形缓冲区”不一定非要用 `VecDeque`

很多新手一看到滑窗，就会先想到 `VecDeque`。

但性能敏感代码里，最常见的做法其实是：

- 一个固定长度数组或 `Vec`
- 一个手动维护的 `ring_index`

原因很简单：

- 内存布局更简单
- 覆盖旧值更直接
- 更容易贴近 C 的性能模型

### 2. batch 优化时，最值钱的是“共享窗口”，不是“把所有窗口都各自对象化”

这里短、中、长三个周期如果都各自维护完整 ring buffer，逻辑也能写通，但状态更新会更重。

C 版高效的关键点是：

> 只维护一套长窗口缓存，短/中窗口从里面扣减出自己的和

这个技巧很适合初学者记住，因为它体现的不是 Rust 语法，而是性能设计思路：

- 不重复维护能复用的状态
- 让多个窗口共享同一份底层数据

## 这次的结论

`ultosc` 这笔很典型地说明：

- 同一个指标，公式没变
- 测试也一直是对的
- 但 batch 路径的状态表达方式不同，性能就会差很多

当我们把 Rust 写法从“方便复用 stream”改成“贴近 C 的 direct batch kernel”后，性能差距就真正收回来了。
