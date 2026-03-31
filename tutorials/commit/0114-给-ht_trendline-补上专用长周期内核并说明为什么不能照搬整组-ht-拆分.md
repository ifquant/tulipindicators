# 给 `ht_trendline` 补上专用长周期内核，并说明为什么不能照搬整组 `ht_*` 拆分

这次改的是 `ht_trendline` 的 Rust batch 路径。

前面的 focused benchmark 已经说明，`ht_*` 这一组是真热点，但把整组共享 kernel 一次性拆成专用版本反而更慢。所以这次不再大拆，而是只对 `ht_trendline` 做一条更贴近 C `HT_LONG_TRENDLINE` 分支的专用长周期内核。

## 这次做了什么

1. 在 `rust/src/indicators/indicator/ht.rs` 里新增 `run_ht_trendline_batch(...)`
2. `HtTrendline::run()` 和 `run_in_place()` 直接走这条专用内核
3. 保留共享 `run_long_ht_batch(...)` 给 `dcphase/sine/trendmode`
4. 删掉共享 `LongHtKind` 里已经不用的 `Trendline` 分支

## 为什么这次只单独打 `ht_trendline`

因为 `ht_*` 这组不是简单的 `mul_add -> fmadd` 问题。

从 C 和 Rust 的汇编对照看：

- 两边都有大量 `fmadd`
- 两边都要调用 `atan/sin/cos`
- 真正的差异更像“共享状态机的机器码形状”

之前把 `ht_dcperiod / ht_phasor / ht_trendline` 一起拆成专用 kernel，反而会让 LLVM 生成更差的代码。  
所以这次只保留最慢的 `ht_trendline` 这一条专用路径，避免把整组一起拆坏。

## 结果

focused benchmark：

- `ht_trendline 4096 = 1.530x`
- `ht_trendline 16384 = 1.577x`

对比上一版 focused：

- `4096`：从约 `1.66x` 收到 `1.53x`
- `16384`：从约 `1.86x` 收到 `1.58x`

也就是说，这刀是有效收益，但还没有回到 parity。

## 这次得到的经验

### 1. 不是所有热点都适合“一次把整组共享 kernel 拆开”

像 `ema/macd/rsi` 那一类，拆开常常有效。  
但 `ht_*` 这种 Hilbert 家族更复杂，整组硬拆会让编译器失去原本还能利用的一些整体优化。

### 2. 这类热点更像“共享大状态机 codegen 问题”

它们的主成本更像：

- 共享模式分发
- 固定 ring/history 状态推进
- `atan/sin/cos` 的调用形状
- 长周期内层循环

而不是单个算术表达式写错了。

## 还没解决什么

- `ht_dcperiod` 和 `ht_phasor` 仍然慢
- `ht_trendline` 也还没有追平 C
- 下一步如果继续，不该再乱拆整组，而是要继续按单个模式去看 steady-state 汇编形状
