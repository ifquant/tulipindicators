# 把稳定指标里残留的 stream-backed batch 路径继续收口成统一模板

## 背景

前面的功能补齐和性能优化已经把很多指标拉到了比较统一的形态：

- `run()`
- `run_in_place()`
- direct batch kernel
- stream 保留给真正的流式调用

但仓库里仍然有一批稳定指标保留着旧路径：`run()` 里直接 new 一个 stream，然后调用 `stream.feed()`。这种写法在功能上是对的，但工程上会留下两个问题：

1. 同一家族的代码形状不一致。
2. 后续看 benchmark 时，很难一眼判断某个 batch 路径是不是还带着历史结构税。

这次收口的目的不是再做一次大重构，而是把最适合下手、风险最低的一批稳定指标继续拉回到当前统一模板里。

## 主要目标

这次只做三件事：

1. 给 `vhf`、`mass`、`aroon`、`aroonosc` 补 direct batch kernel。
2. 给它们补 `run_in_place()`，让 batch benchmark 和调用方都能走统一高性能路径。
3. 把这套“指标模板收口规则”沉淀成仓库长期文档，而不是只留在 commit 历史里。

`psar` 也做了实验，但 focused benchmark 明显回归，所以没有保留在这次提交里。

## 改动概览

### `vhf`

原来：

- `run()` 直接 `VhfStream::new(...).feed(...)`

现在：

- 新增 `run_vhf_batch(...)`
- `run()` 直接分配输出并调用 batch kernel
- `run_in_place()` 直接写调用方提供的输出切片

这个 kernel 仍然保留原先的数学语义：

- `VecDeque` 维护 `period + 1` 个样本
- `MonotonicQueue` 维护窗口最大值和最小值
- `change_sum` 跟 stream 一样按滑窗更新

也就是说，这次改的是代码组织，不是数值定义。

### `mass`

原来：

- batch 路径还是 `MassStream`

现在：

- 新增 `run_mass_batch(...)`
- 保留和 stream 同样的两层 EMA 语义
- `run_in_place()` 可以直接写输出切片

这里还顺手把第二层 EMA 的递推写成了 `mul_add` 形状，和仓库里其他 EMA 家族保持一致。

### `aroon / aroonosc`

原来：

- 两者都依赖各自 stream 来做 batch

现在：

- `run_aroon_batch(...)` 同时写 `aroon_down` 和 `aroon_up`
- `run_aroonosc_batch(...)` 单独写 oscillator 输出
- 两条 batch kernel 都直接复用 `MonotonicQueue`

这一步的价值在于把“多输出窗口指标”的模板也往统一方向拉了一步：以后遇到同类指标，更容易判断应该怎样组织 batch 和 `run_in_place()`。

## 关键知识

## 什么叫“收口”

这里的“收口”不是重写算法，而是把旧实现整理到当前推荐模板里，让仓库整体更像一个系统，而不是一堆单点优化的叠加。

这次之后，这几个指标的 batch 路径都遵守同一套规则：

- `run()` 只是 owned 输出壳
- 真实 batch 工作放在 `run_<indicator>_batch(...)`
- `run_in_place()` 不再回退到 `run()`

### 为什么 `psar` 没一起保留

`psar` 属于 branch-heavy state machine。

它和 `vhf`、`mass`、`aroon` 不一样，不是把 batch 从 stream 拉平就一定会更快。focused benchmark 说明，这次直接改成 batch kernel 之后反而更慢，所以这条线暂时保留旧路径更合理。

这也是一个很重要的经验：

- “统一模板”是默认方向
- 但不是所有指标都值得强行收口
- 对分支很重的状态机指标，要让 benchmark 先说话

## 验证

- `cargo fmt --all` (`PASS`)
- `cargo test --test stable_parity --test golden_indicators` (`PASS`)
- `cargo clippy --all-targets --all-features` (`PASS`)
- `TI_BENCH_INDICATORS=vhf,mass,aroon,aroonosc,psar TI_BENCH_SIZES=4096,65536 TI_BENCH_TARGET_MS=120 TI_BENCH_CALIBRATION_MS=30 TI_BENCH_REPEATS=5 cargo run --release -q --bin indicator-bench-compare` (`PASS`)

focused benchmark 里保留下来的结果：

- `vhf 4096 = 0.769x`
- `vhf 65536 = 0.607x`
- `mass 4096 = 0.484x`
- `mass 65536 = 0.492x`
- `aroon 4096 = 0.595x`
- `aroon 65536 = 0.863x`
- `aroonosc 4096 = 0.625x`
- `aroonosc 65536 = 0.844x`

这些都说明收口后 batch 路径已经不只是“更统一”，而且性能上也没有吃亏。

## 未覆盖项

- `psar` 的 direct batch 实验已经做过，但这次没有保留，因为 focused benchmark 回归明显。
- 这次没有继续收口 beta 指标里残留的 `stream.feed()` 路径，只处理了稳定指标里最适合先动的一批。
- 这次也没有统一 stream 自身的 `feed_in_place()` 覆盖率；目前重点还是先把 batch 主路径拉到统一模板上。
