## 背景

这轮 focused 复核里：

- `vosc` 已经基本和 C 持平
- `tanh` 也没有稳定慢于 C
- 真正稳定慢的只有 `fosc`

原始结果大约是：

- `fosc 4096 = 1.231x`
- `fosc 16384 = 1.338x`

## 汇编和结构上看到的问题

这次 `fosc` 的问题不是单个 `fmadd`，而是结构。

Rust 原来的 `Fosc::run()` / `run_in_place()` 走的是：

1. 先调用 `run_regression_batch(...)`
2. 生成整条 `TSF(period + 1)` 序列
3. 再第二遍做：

```rust
100.0 * (sample - previous_tsf) / sample
```

也就是说：

- 先分配一条中间 `regression` 缓冲区
- 再二次遍历

而 C 的 `ti_fosc(...)` 不是这样。它直接在回归主循环里在线产出 FOSC，没有先 materialize 整条 TSF。

这就是这笔性能差距的根因。

## 这次改了什么

直接给 `fosc` 补了专用 batch kernel：

- 新增 `run_fosc_batch(...)`
- 在同一条回归推进循环里：
  - 维护 `x_sum / x2_sum / y_sum / xy_sum`
  - 直接计算前一个窗口的 `tsf`
  - 当前步直接写出 `100 * (sample - tsf) / sample`

所以现在 `fosc` 不再：

- 分配整条 regression 中间数组
- 再跑第二遍输出

## 结果

focused benchmark：

- `fosc 4096 = 0.953x`
- `fosc 16384 = 0.976x`

也就是说，两档都已经跑赢 C 了。

## 经验

这个 case 很典型：

- 如果 Rust 版本是“先生成一条完整中间序列，再做二次派生”
- 而 C 版本是“在线推进并直接产出最终值”

那么真正该补的通常不是局部 `mul_add`，而是：

- 专用 batch kernel
- 去掉中间数组
- 把二次遍历收进同一条主循环

这类结构差异，收益会比继续抠单条指令大得多。  
