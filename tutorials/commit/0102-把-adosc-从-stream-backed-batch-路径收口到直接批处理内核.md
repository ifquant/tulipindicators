# 把 adosc 从 stream-backed batch 路径收口到直接批处理内核

## 背景

前面继续收口稳定指标时，我们已经把一批典型的旧 batch 形状整理到了统一模板里。  
但 `adosc` 还保留着很典型的旧路径：

- `run()` 直接 `AdOscStream::new(...).feed(...)`
- 没有自己的 `run_in_place()`

这类实现有一个很常见的问题：功能和 stream 是对的，但 batch benchmark 测到的不是“真正的 batch kernel”，而是：

1. `run()`
2. `stream.feed()`
3. 分配 owned `Vec`
4. 默认 trait `run_in_place()` 再把结果 copy 到输出切片

所以在继续优化之前，先要搞清楚：`adosc` 到底是热循环慢，还是 batch 入口走错路径。

## 主要目标

这次只做一件事：

- 给 `adosc` 补 direct batch kernel 和 `run_in_place()`，让 batch 主路径不再依赖 `AdOscStream::feed()`

## 改动概览

### 新增 `run_adosc_batch(...)`

新的 batch kernel 直接对齐 C 的结构：

- 遍历 `high / low / close / volume`
- 维护累计 `sum`
- 分别推进 `short_ema` 和 `long_ema`
- 从 `long_period` 开始写出 `short_ema - long_ema`

这条路径更贴近 C 的 [`adosc.c`](/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/adosc.c)：

- 一个 batch 循环
- 两条 EMA 递推
- 一路输出

### `run()` 和 `run_in_place()` 都走 batch kernel

现在：

- `run()` 只负责分配 owned 输出
- `run_in_place()` 直接调用 `run_adosc_batch(...)`

stream 仍然保留，用于真正的流式调用；但 batch 不再默认借道 stream。

## 关键知识

### 这次为什么能证明“是 wrapper 问题”

这次改动前，focused benchmark 大致是：

- `4096 ≈ 1.10x`
- `65536 ≈ 1.21x`

而且 Rust 代码里明明还能看到：

- 递推逻辑并不复杂
- stream 内核本身已经很接近 C

这类形状往往意味着：

- 内核不是主要问题
- batch 入口还带着额外包装层

这次改成 direct batch kernel 后，结果直接跳到：

- `4096 = 0.389x`
- `65536 = 0.502x`

这就是很典型的“路径问题一旦去掉，性能就明显回收”的 case。

### 为什么先不去看汇编

只有在 direct batch kernel 已经到位后，汇编分析才更值得做。  
如果 batch 入口还带着旧路径，先去看汇编很容易把 wrapper 税和内核税混在一起。

所以这次的顺序是：

1. 先去掉 wrapper / batch 入口问题
2. 再决定是否还要看内核和汇编

而 `adosc` 的结果说明，第一步就已经把主要问题解决了。

## 验证

- `cargo fmt --all` (`PASS`)
- `cargo test --test stable_parity --test golden_indicators` (`PASS`)
- `cargo clippy --all-targets --all-features` (`PASS`)
- `TI_BENCH_INDICATORS=adosc TI_BENCH_SIZES=4096,65536 TI_BENCH_TARGET_MS=180 TI_BENCH_CALIBRATION_MS=50 TI_BENCH_REPEATS=11 cargo run --release -q --bin indicator-bench-compare` (`PASS`)

focused benchmark：

- `adosc 4096 = 0.389x`
- `adosc 65536 = 0.502x`

## 未覆盖项

- 这次没有继续给 `adosc` 做 kernel split，因为 direct batch kernel 已经证明主要问题在 wrapper / 路径层。
- stream 本身的实现还保留原状，没有进一步重写它的内部组织。
- 这次也没有顺手处理 `psar` 这类 branch-heavy 指标，因为它们不是同一种问题。
