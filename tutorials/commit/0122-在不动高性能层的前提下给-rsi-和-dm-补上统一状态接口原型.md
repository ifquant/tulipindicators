# 在不动高性能层的前提下，给 RSI 和 DM 补上统一状态接口原型

## 背景

前面的性能优化已经把 `run(...)`、`run_in_place(...)` 和很多专用 batch kernel 打磨得比较细了。这一层是 benchmark 和高性能调用的基础，已经形成了比较稳定的工程约束：不要为了易用接口去打乱热路径。

但真实使用里，技术指标并不总是“拿一整段历史数据一次性跑完”。更常见的场景是：

- 先用一段历史数据把指标热起来
- 后面每来一条新数据，就增量更新一次
- 有时还要按索引取最近第 1 条、第 2 条、第 N 条结果

原来的 `stream` 能表达一部分这个需求，但它的接口形状并不够自然，而且容易把“内部状态机实现”直接暴露成“对外主接口概念”。

这次的目标，就是在 **完全不动现有高性能层** 的前提下，先给库补一个统一的状态接口原型。

## 主要目标

这次提交要解决的是“如何更方便地实时使用指标”，不是“如何再做一次性能优化”。

具体目标有四个：

1. 保留现有 `run(...)` / `run_in_place(...)` / batch kernel 不变
2. 新增统一的状态接口层，支持 `seed + update + get`
3. 支持固定容量历史缓存，缓存满了后用 ring buffer 覆盖最旧值，而不是整体搬移
4. 先拿 `rsi` 和 `dm` 做两个不同输入形状的原型，验证接口是否顺手

## 改动概览

### 1. 新增统一状态接口模块

新增文件：

- `rust/src/state.rs`

这里定义了两个核心组件：

- `IndicatorState` trait
- `RingHistory<T>` 固定容量环形历史缓存

`IndicatorState` 统一了这几个能力：

- `seed(&mut self, input: &[Input])`
- `update(&mut self, input: Input)`
- `latest()`
- `get(index_from_latest)`
- `len()`
- `history_capacity()`
- `reset()`

这层是“对外易用状态接口”，不是对原有 batch 层的替代。

### 2. 历史缓存使用 ring buffer，不做整体前移

这次明确没有采用“缓存满了整体左移”的写法。

原因很简单：

- 整体搬移会把每次 `update` 变成 `O(n)`
- 对实时指标场景不划算

这次的 `RingHistory<T>` 是固定容量覆盖式 ring buffer：

- 未满时直接 `push`
- 满了以后覆盖最旧元素
- 通过索引映射实现 `get(0)` 取最新、`get(1)` 取上一个

这让状态接口的历史读取能力不会明显拖慢增量更新路径。

### 3. 给 RSI 补了状态化原型

修改文件：

- `rust/src/indicators/indicator/rsi.rs`

这次没有重写 `RSI` 的计算逻辑，而是做了两件很克制的事情：

1. 给 `RsiStream` 提炼出 `update_one(sample)`，让单条推进逻辑可以被复用
2. 在外层包一个 `RsiState`

`RsiState` 内部只维护：

- `period`
- 原有 `RsiStream`
- `RingHistory<Real>`

所以这层是“包裹现有状态机”，不是另起炉灶写第二份指标实现。

### 4. 给 DM 补了状态化原型

修改文件：

- `rust/src/indicators/indicator/dm.rs`

`DM` 的输入形状和 `RSI` 不一样，它是双输入 `(high, low)`。这正好可以验证统一状态接口是否真的通用。

这次同样没有动 `DM` 的 batch kernel，只是：

1. 给 `DmStream` 提炼出 `update_one(high, low)`
2. 在外层包一个 `DmState`
3. 用 `RingHistory<(Real, Real)>` 保存最近输出

这样就证明了同一个状态接口既能覆盖单输入指标，也能覆盖多输入指标。

### 5. 在 crate 根导出状态接口和两个原型

修改文件：

- `rust/src/lib.rs`
- `rust/src/indicators/indicator/mod.rs`

这样外部使用者可以直接拿到：

- `IndicatorState`
- `RsiState`
- `DmState`

而不需要自己钻到内部模块里找。

### 6. 新增状态接口测试

新增文件：

- `rust/tests/state_api.rs`

这里重点验证的不是“性能”，而是“状态接口和批处理结果是否一致”。

测试做了两类检查：

- `seed(...)` 后的输出数量是否和 batch 一致
- `latest()` / `get(index)` 读到的结果，是否和 batch 输出对齐

测试里还顺手修正了一个现实问题：状态接口和 batch 接口在浮点末位上可能出现极小误差，所以这次改成了带容差的比较，而不是强行要求 bitwise 完全相等。

## 关键知识

## 为什么这次不直接把 `stream` 改成主接口

因为 `stream` 这个词更像“内部实现手段”，而不是“对外易用能力”。

对使用者来说，更直观的其实是：

- 先 `seed`
- 再 `update`
- 需要时 `latest/get`

所以这次的方向不是强化 `stream` 概念，而是在它外面补一层更统一的状态接口。

## 为什么必须强调“不动高性能层”

因为这个库已经有很多性能敏感路径：

- `run_in_place(...)`
- 专用 batch kernel
- 递推型指标的 `mul_add/fmadd` 收口

这些路径已经被 benchmark 和汇编分析打磨过。如果为了“好用”去改它们，很容易把前面的性能收益打掉。

所以这次的工程原则是：

- 高性能层继续保持原状
- 新状态接口只做“上层包装”
- 不把两层搅在一起

## 为什么 `get(index)` 要按“离最新值的偏移”来定义

因为实时指标场景更关心：

- 最新值
- 上一个值
- 最近 N 个值

所以这里的语义是：

- `get(0)`：最新
- `get(1)`：上一个
- `get(2)`：上上一个

这比“按绝对写入位置索引”更符合实时使用直觉。

## 验证

- `cargo fmt --all` (`PASS`)
- `cargo test --test state_api --test stable_parity --test golden_indicators` (`PASS`)
- `cargo clippy --all-targets --all-features` (`PASS`)

## 未覆盖项

- 这次只给 `rsi` 和 `dm` 做了状态接口原型，其他指标还没有统一接入
- 这次没有给状态接口补通用 builder，例如 `with_history(...)` 之类的更顺手构造方式
- 这次没有尝试把旧的 `stream` API 隐藏或废弃，只是先增加一层更自然的状态接口
- 这次没有给状态接口做性能 benchmark；它的目标是易用性和统一性，不是替代已有高性能层
