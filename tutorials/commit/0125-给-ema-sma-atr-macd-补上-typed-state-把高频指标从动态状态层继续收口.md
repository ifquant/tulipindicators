# 给 EMA、SMA、ATR、MACD 补上 typed state，把高频指标从动态状态层继续收口

## 背景

前两笔和状态接口有关的提交，已经把统一状态层搭起来了：

- 先有 `rsi`、`dm` 的 typed 原型
- 再有对全指标可用的 `DynamicIndicatorState`
- 还补了统一工厂入口

到那个阶段，库在“能力覆盖”上已经够用了，但在“静态类型友好度”上还不平衡。

如果调用方操作的是高频常用指标，最好还是能直接拿到：

- `EmaState`
- `SmaState`
- `AtrState`
- `MacdState`

而不是先退回动态层，再自己拆 `Vec<Real>` 或多输出数组。

## 主要目标

这次的目标不是扩接口能力，而是把一批最常用的指标从动态状态层继续收口成 typed state。

选择标准很简单：

- 使用频率高
- 已经有稳定 stream/state 内核
- 可以直接复用现有增量逻辑

这次覆盖：

- `ema`
- `sma`
- `atr`
- `macd`

## 改动概览

### 1. 给 EMA 补了 `EmaState`

文件：

- `rust/src/indicators/overlay/ema.rs`

这次没有重写 EMA 的批处理核，而是：

- 从 `EmaStream` 提炼出 `update_one(sample)`
- 在外层包一个 `EmaState`
- 用 `RingHistory<Real>` 保存最近输出

因为 `EMA` 没有 lookback，状态层也比较直接：

- 每次 update 都会产出一个值
- 历史缓存会把全部产出按 ring buffer 保存

### 2. 给 SMA 补了 `SmaState`

文件：

- `rust/src/indicators/overlay/sma.rs`

`SMA` 和 `EMA` 的区别在于它有 warmup 阶段。

这次做法仍然是：

- 从 `SmaStream` 提炼 `update_one(sample)`
- `SmaState` 只在真正产出均线值时写入 history

这样状态接口的语义和 batch 输出长度保持一致，不会把 warmup 阶段误当成有效输出。

### 3. 给 ATR 补了 `AtrState`

文件：

- `rust/src/indicators/indicator/atr.rs`

`ATR` 是这批里最值得注意的一个，因为它不是单输入，而是：

- `(high, low, close)`

而且它也有 warmup 阶段。

这次处理方式是：

- 从 `AtrStream` 提炼 `update_one(high, low, close)`
- 在外层加 `AtrState`
- 历史层只保存真正产出的 ATR 值

这证明 typed state 不只是适合最简单的单输入指标。

### 4. 给 MACD 补了 `MacdState`

文件：

- `rust/src/indicators/indicator/macd.rs`

`MACD` 这批里最重要的地方是多输出。

输出是三路：

- `macd`
- `signal`
- `histogram`

这次没有再拆成三套状态，而是：

- 继续复用现有 `MacdStream`
- 提炼 `update_one(sample)`
- 在 `MacdState` 里用 `RingHistory<(Real, Real, Real)>` 保存三元组输出

这样 typed state 这一层也覆盖了多输出递推指标。

### 5. 把这些 typed state 导出到更自然的位置

修改文件：

- `rust/src/indicators/overlay/mod.rs`
- `rust/src/indicators/indicator/mod.rs`
- `rust/src/lib.rs`

这样外部调用方可以直接拿到：

- `EmaState`
- `SmaState`
- `AtrState`
- `MacdState`

而不用深入内部模块树。

### 6. 补了对应测试

文件：

- `rust/tests/state_api.rs`

这次把状态接口测试扩成了更完整的一组：

- `ema`：单输入、零 lookback
- `sma`：单输入、有 lookback
- `atr`：三输入、单输出
- `macd`：单输入、三输出

测试核心仍然是同一个原则：

- `seed(...)` 结果和 batch 对齐
- `latest()/get(index)` 读到的结果和 batch 对齐

## 关键知识

## 为什么这次仍然没有动高性能层

因为 typed state 这件事要做对，关键不是“再实现一份指标”，而是：

- 直接复用现有 stream/state 机理
- 只在外层加更自然的 typed 壳

这可以把风险压到最低。

如果为了 typed state 再去碰 batch kernel 或 benchmark 路径，就违反了前面已经明确的工程边界。

## 为什么高频指标优先做 typed state

动态状态层适合“全覆盖”，但它的输出是：

- `Vec<Real>`

对于常用指标来说，这层还是偏动态。

而 high-frequency / high-usage 指标更适合：

- `Option<Real>`
- `Option<(Real, Real)>`
- `Option<(Real, Real, Real)>`

这种 typed 输出更容易直接被业务代码消费，也更不容易写错索引。

## 为什么 MACD 值得做成三元组 typed state

如果只给 `MACD` 暴露动态层，调用方每次都要记：

- `values[0]` 是 macd
- `values[1]` 是 signal
- `values[2]` 是 histogram

这对使用者不友好，也容易出错。

所以 `MacdState` 的意义不只是“类型更漂亮”，而是把多输出指标的语义显式化。

## 验证

- `cargo fmt --all` (`PASS`)
- `cargo test --test state_api --test stable_parity --test golden_indicators` (`PASS`)
- `cargo clippy --all-targets --all-features` (`PASS`)

## 未覆盖项

- 这次只挑了一批高频指标做 typed state，还没有把所有 stream-backed 指标都补成 typed `FooState`
- `MacdFix` 这次没有额外补 typed state，仍然可走动态状态层
- 这次没有补更完整的顶层示例文档，typed state 的使用方式仍主要体现在测试和提交教程里
