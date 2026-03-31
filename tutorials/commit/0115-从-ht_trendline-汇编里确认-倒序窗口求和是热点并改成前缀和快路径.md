# 从 `ht_trendline` 汇编里确认“倒序窗口求和”是热点，并改成前缀和快路径

这次不是继续去拆 `ht_*` 的共享 Hilbert 主链，而是只盯住 `ht_trendline` 自己那段最可疑的热循环。

前面做过几轮对照后，我们已经知道：

- `ht_trendline` 比 C 慢，而且是稳定慢，不是 benchmark 噪声。
- 它不像 `ema`、`rsi` 那样，主要问题不是少一个 `fmadd`。
- 之前把 `ht_trendline` 从共享 `run_long_ht_batch(...)` 里拆出来以后，已经拿回了一部分收益，但还没有追平 C。

## 这次先看什么

我先重新看了 Rust 和 C 的 release 汇编。

### C 版的形状

`c/indicators/ht.c` 里的 `run_ht_long(..., HT_LONG_TRENDLINE, ...)` 虽然也复杂，但趋势线那段的“窗口均值”只是长周期状态机中的一个局部循环。

### Rust 版的形状

`rust/src/indicators/indicator/ht.rs` 里的 `run_ht_trendline_batch(...)` 里有这样一段：

```rust
let mut average = 0.0;
let mut idx = today;
for _ in 0..dc_period_int {
    average += input[idx];
    idx -= 1;
}
```

从汇编能直接看到，这段会生成：

- 反向逐元素加载
- 循环计数和边界检查
- 多个 `panic_bounds_check` 备用路径

这说明它虽然逻辑简单，但在 Rust 下确实是一段真实热循环。

## 为什么这次不继续硬拆 Hilbert 主链

前面已经试过：

- 把 `ht_dcperiod` / `ht_phasor` 拆专用短内核
- 把 `ht_trendline` 从共享长内核里拆出来
- 在共享 Hilbert 主链上继续做局部小修

结论是：

- 共享主链本身不是最容易继续拿收益的地方
- `trendline` 自己的窗口均值反而是一个更明确、更独立的热点

所以这次改法是：**不碰 Hilbert 主链，只把“窗口均值”改成前缀和快路径。**

## 具体改法

在 `run_ht_trendline_batch(...)` 开头先构建一条前缀和数组：

```rust
let mut prefix_sum = Vec::with_capacity(input.len() + 1);
prefix_sum.push(0.0);
let mut running_sum = 0.0;
for &value in input {
    running_sum += value;
    prefix_sum.push(running_sum);
}
```

然后每轮不再倒序累加整段窗口，而是直接用：

```rust
(prefix_sum[end] - prefix_sum[start]) / dc_period_int as Real
```

这样做的实际收益是：

- 去掉了那段反向逐元素窗口求和循环
- 去掉了循环里的重复边界检查
- 把每轮求均值从“按窗口长度线性扫描”降成了 `O(1)`

## 为什么这次能站住

focused benchmark 结果：

- `ht_trendline 4096 = 1.388x`
- `ht_trendline 16384 = 1.307x`

对比上一版大约：

- `4096 ≈ 1.53x`
- `16384 ≈ 1.58x`

也就是说这次两档都明显改善，而且长输入改善更明显。

## 这次的边界

这次还没有把 `ht_trendline` 打到 parity。

剩下的差距更像：

- Hilbert 历史状态推进本身
- `atan/sin/cos` 这类调用
- 长周期状态机的整体 codegen

所以这笔提交的意义不是“彻底解决 `ht_*` 性能”，而是：

- 先把 `ht_trendline` 里最明确的一段窗口热点收掉
- 让后面继续分析时，把注意力更集中到 Hilbert 主链本体

## 顺手记一个知识点

### 1. 前缀和是“窗口求和”的典型降复杂度工具

如果你发现代码在“每个位置都重新求一遍一个窗口的和”，就该立刻想到前缀和。

前缀和最常见的用途就是：

- 区间求和
- 滑动均值
- 变量窗口均值

### 2. 汇编对照不只用来找 `fmadd`

前面很多指标的收益来自：

- `fmul + fadd -> fmadd`

但这次 `ht_trendline` 的收益告诉我们：

- 汇编对照还能帮你找到“哪一段循环本身就不该继续存在”

也就是说，汇编分析不只是为了改一条指令，也可以帮助你判断：

- 是该继续抠算术表达式
- 还是该直接换算法结构
