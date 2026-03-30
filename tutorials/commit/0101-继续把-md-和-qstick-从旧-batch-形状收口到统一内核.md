# 继续把 md 和 qstick 从旧 batch 形状收口到统一内核

## 背景

上一笔收口把 `vhf`、`mass`、`aroon`、`aroonosc` 拉到了统一模板：

- `run()`
- `run_in_place()`
- `run_<indicator>_batch(...)`

但稳定指标里还剩下一类“半收口”的实现：

- 已经有 `run_in_place()`
- 或 stream 语义本身没问题
- 但是 `run()` 仍然直接走 `stream.feed()`，或者 batch 逻辑还散在方法体里，没有被整理成统一 kernel

`md` 和 `qstick` 正好都属于这类，很适合继续低风险收口。

## 主要目标

这次只做两件事：

1. 给 `md` 提炼出共享 batch kernel，让 `run()` 和 `run_in_place()` 走同一条路径。
2. 给 `qstick` 补 direct batch kernel 和 `run_in_place()`，不再让 batch 默认绕 stream。

## 改动概览

### `md`

原来：

- `run()` 直接 `MdStream::new(...).feed(...)`
- `run_in_place()` 自己内嵌了一份 batch 逻辑

现在：

- 新增 `run_md_batch(...)`
- `run()` 只负责分配 owned 输出
- `run_in_place()` 直接复用同一个 batch kernel

这一步的价值不在于“再造一个算法”，而在于把同一份数学逻辑变成唯一实现来源，减少后续继续抠性能或排查 parity 时的分叉。

### `qstick`

原来：

- `run()` 直接 new `QstickStream`
- 没有 `run_in_place()`

现在：

- 新增 `run_qstick_batch(...)`
- 补了 `run_in_place()`
- `run()` 直接走 batch kernel

`qstick` 的 batch 核心其实很简单：一个固定窗口的 `RingSum`。收口以后，它在工程形态上就和仓库里其他 rolling-window 指标一致了。

## 关键知识

### 什么叫“半收口”

“半收口”不是说实现有 bug，而是说：

- batch 已经有 direct path 的条件
- 但代码组织还没完全统一

这种状态短期不会出错，但长期会增加维护成本：

- 同一个指标的 batch 逻辑分散在多个函数里
- 性能分析时难以快速定位真正的热路径
- 新增同类指标时，新人不容易判断该抄哪一种模板

### 为什么 `md` 值得先整理

`md` 这类指标不属于最容易大幅优化的 low-level kernel，因为它每步都要重新累计窗口内偏差，算法本体就不轻。

但它非常适合做“形态收口”：

- 把 batch 唯一实现抽出来
- 让 `run()` 和 `run_in_place()` 统一
- 以后如果真的要继续抠性能，只要盯一条 kernel 就够了

## 验证

- `cargo fmt --all` (`PASS`)
- `cargo test --test stable_parity --test golden_indicators` (`PASS`)
- `cargo clippy --all-targets --all-features` (`PASS`)
- `TI_BENCH_INDICATORS=md,qstick TI_BENCH_SIZES=4096,65536 TI_BENCH_TARGET_MS=180 TI_BENCH_CALIBRATION_MS=50 TI_BENCH_REPEATS=11 cargo run --release -q --bin indicator-bench-compare` (`PASS`)

focused benchmark 结果：

- `md 4096 = 0.941x`
- `md 65536 = 1.068x`
- `qstick 4096 = 1.087x`
- `qstick 65536 = 1.002x`

也就是说，这次不是只把代码组织整理漂亮了；这两项对 C 也都保持在阈值内。

## 未覆盖项

- 这次没有继续处理 `md` 和 `qstick` 的 stream 自身性能，只收口了 batch 主路径。
- `msw`、`adosc` 等仍残留旧 batch 形状的稳定指标还没纳入这次提交。
- `md` 虽然已经收口，但相对 Rust 自己历史最好结果还略慢，后续如果要继续抠性能，应该直接盯 `run_md_batch(...)`。
