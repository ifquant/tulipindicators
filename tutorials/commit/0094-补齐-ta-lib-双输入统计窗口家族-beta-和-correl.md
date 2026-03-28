# 补齐 TA-Lib 双输入统计窗口家族：`beta` 和 `correl`

这次补的是 TA-Lib 语义映射里下一批最适合 Tulip 风格落地的真缺失指标：

- `beta`
- `correl`

它们的共同点是：

- 都是双输入、单输出、固定窗口统计量
- 不需要先引入 TA-Lib 那套可变 MA 类型系统
- 可以直接做成 Tulip 现有风格的 batch + `run_in_place`

## 这次改了什么

### 1. 先把 C 版本补齐

新增了：

- `/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/beta.c`
- `/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/correl.c`

然后把它们接进：

- `/Users/dev/workspace2/hc_apps/tulipindicators/c/build.tcl`
- `/Users/dev/workspace2/hc_apps/tulipindicators/c/tests/extra.txt`

这样 `build.tcl` 重新生成后，`indicators.c` / `indicators.h` 就会自动把这两个指标暴露给整个 C 库和 smoke test。

### 2. Rust 版本放进 `math` 族

Rust 这次没有新开一个文件，而是直接放进：

- `/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/math/mod.rs`

原因很简单：

- `beta`
- `correl`

都更像“窗口统计/数学量”，而不是趋势指标或 overlay。

这次给它们补了：

- metadata
- `run(...)`
- `run_in_place(...)`

没有补 stream。先把 C/Rust 功能、parity、benchmark 闭环，是这一步更重要的目标。

### 3. 接进 registry、parity 和 benchmark

同步更新了：

- `/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/registry.rs`
- `/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/talib_missing_parity.rs`
- `/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/talib_missing_benchmark.rs`
- `/Users/dev/workspace2/hc_apps/tulipindicators/tutorials/ta-lib-semantic-mapping.md`

这样现在这两个新指标不仅能调用，也会：

- 对 C 做 parity
- 进入 benchmark 覆盖
- 被语义映射文档正式记录

## 这次最重要的语义坑

`beta` 最容易被注释误导。

TA-Lib 源码的说明文字在讲“market return / security return”，但真正代码里回归斜率是：

- 第一输入先变成 `x`
- 第二输入先变成 `y`
- 最后输出 `y 对 x` 的 slope

也就是说，这次我们不是按“注释直觉”实现，而是按 TA-Lib 真正的 C 源码行为实现。

这就是为什么这类迁移不能只看函数名，也不能只看文档摘要，最后还是得回到源码。

## Rust 新手知识点 1：`run_in_place` 为什么重要

像这种窗口统计指标，如果只留：

```rust
Result<Vec<Vec<Real>>, IndicatorError>
```

那每次调用都会有输出分配和包装成本。

所以现在我们默认做两层：

- 易用接口：`run(...)`
- 高性能接口：`run_in_place(...)`

这样 benchmark 和性能敏感路径就不必先分配一个新的 `Vec<Vec<_>>`。

## Rust 新手知识点 2：窗口统计最常见的提速手法

`correl` 这种指标如果每个窗口都重新扫一遍，复杂度会比较差。

这次实现里用的是更典型的滑窗写法：

- 维护 `sum_x`
- `sum_y`
- `sum_x2`
- `sum_y2`
- `sum_xy`

窗口向前滑一格时：

- 加入新样本贡献
- 减去旧样本贡献

这样每步就是 `O(1)` 更新，而不是每个输出都重新扫一个完整窗口。

## 结果

功能和测试都已经闭环：

- C smoke 通过
- Rust parity 通过
- benchmark 覆盖测试通过

release benchmark 结果：

- `beta`
  - `4096 = 1.008x`
  - `65536 = 1.058x`
- `correl`
  - `4096 = 1.209x`
  - `65536 = 1.189x`

也就是说：

- `beta` 已经基本追平 C
- `correl` 目前还略慢一些，但已经进入 benchmark 管线，后面可以继续按汇编/内核形状去优化

## 这次没有做的事

- 没有给 `beta / correl` 补 stream
- 没有继续优化 `correl` 的 batch kernel 到完全追平 C
- 没有开始 `ma / mavp / macdfix / macdext / t3 / sarext / ht_*`

这次的目标就是先把“真缺失指标 + parity + benchmark”这一整条链做完整。
