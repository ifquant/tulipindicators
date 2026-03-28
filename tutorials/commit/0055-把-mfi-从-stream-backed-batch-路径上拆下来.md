# 把 `mfi` 从 stream-backed batch 路径上拆下来

这次是一笔很标准、也很干净的性能优化：

- 先通过 focused benchmark 确认 `mfi` 是真热点
- 再检查实现，发现它的 batch 路径还在复用 stream
- 最后把 batch 拆成 direct kernel

结果非常直接：`mfi` 不只是回到阈值内，而是两组输入规模都明显快于 C。

## 这次改了什么

在 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/mfi.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/mfi.rs) 里：

- `Mfi::run()` 不再走 `MfiStream::feed()`
- 新增 `run_in_place(...)`
- 新增 `run_mfi_batch(...)`

`mfi` 的数学结构其实很适合 direct batch：

1. 先算当前 bar 的典型价格 `typ`
2. 用 `typ * volume` 得到资金流 `bar`
3. 根据 `typ` 和前一根 `typ` 的大小关系，分别把 `bar` 放到 `up` 或 `down`
4. 用滑窗累计和算 `MFI`

这和 C 版本是完全同一思路，只是 Rust 这里保留了：

- 契约检查
- `run_in_place` 入口
- 和 stream 版本并存的接口分层

## 中间踩到的坑

这次不是一把就对。  
第一次改完以后，`golden_indicators` 立刻把我拦下来了：

- batch 输出起点和 C 不一致
- stream/batch 也因此失配

根因是一个很典型的滑窗 off-by-one：

- 我一开始按 `output.iter_mut().zip(1..high.len())` 去写
- 但真正应该开始写输出的点是 `index >= period`

所以修复方式不是改 benchmark，而是把 batch 输出逻辑严格对齐到 C 的输出起点。  
这也是为什么在性能优化里，`golden test` 和 `parity test` 一定要跟着跑。

## 这次 benchmark 结果

我最终保留并验证的是：

```bash
TI_BENCH_INDICATORS=mfi \
TI_BENCH_SIZES=4096,65536 \
TI_BENCH_TARGET_MS=120 \
TI_BENCH_CALIBRATION_MS=30 \
TI_BENCH_REPEATS=5 \
cargo run --release --bin indicator-bench-compare
```

结果：

- `mfi 4096`: `0.667x`
- `mfi 65536`: `0.587x`

也就是说：

- `4096` 下已经明显快于 C
- `65536` 下也明显快于 C

这类结果就属于非常理想的“低垂果实”：

- 先前的慢，主要不是公式问题
- 而是 batch 借道 stream 带来的额外成本

## 给 Rust 新手的两个知识点

### 1. 性能优化里，正确性回归是最正常的事情之一

很多新手会把“性能优化后测试挂了”理解成失败。  
其实不是。

更真实的工程过程通常是：

1. 先改热路径
2. 再发现某个边界起点或窗口对齐错了
3. 再靠测试把语义拉回正确位置

这次 `mfi` 就是这样。  
关键不是“第一次必须写对”，而是：

- 测试能及时抓住
- 你能把错误收敛回和 C 完全一致

### 2. stream 和 batch 可以共享公式，但不该默认共享执行路径

这条经验前面已经出现很多次了，`mfi` 又重复证明了一遍。

可以共享的东西：

- 指标定义
- 数学关系
- 状态含义

不一定该共享的东西：

- 具体的热路径实现

因为 stream 的职责是：

- 处理增量输入
- 维护跨 chunk 状态

而 batch 的职责是：

- 用已知的完整输入一次性吐出结果

如果 batch 直接借道 stream，轻量和中等复杂度指标都很容易被这层抽象拖慢。

## 这一步之后的意义

`mfi` 这次是一个很好的样板：

- focused benchmark 先定位
- 代码审查确认它还是 stream-backed batch
- 先改 direct kernel
- 测试抓出 off-by-one
- 修回正确性
- 最终再看 benchmark 收益

这种闭环比“只看一个好看的倍率”更重要。  
因为它说明：这不是偶然快了一次，而是一笔正确性和性能都站住了的优化。
