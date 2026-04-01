# 继续给 Wilders、NATR、PPO、STOCH 补上 typed state，让状态接口覆盖更多高频指标

## 背景

前一笔已经把一批最常用的状态接口补成了 typed 形式：

- `EmaState`
- `SmaState`
- `AtrState`
- `MacdState`

再加上更早的：

- `RsiState`
- `DmState`

库里的 typed 状态层已经开始有规模，但还没有覆盖到另一批也很常见的指标：

- Wilder 类平滑
- 归一化 ATR
- 双 EMA 差值类指标
- 双输出振荡器

这次就是把这几类代表继续补上，让 typed 状态接口不只停留在“均线和最基础指标”这一层。

## 主要目标

这次继续扩 typed state，但仍然守住同一个边界：

- 不动已有高性能层
- 不重写 batch kernel
- 只复用现有 stream/state 逻辑，在外层补 typed 包装

本次覆盖：

- `Wilders`
- `Natr`
- `Ppo`
- `Stoch`

## 改动概览

### 1. 给 Wilders 补了 `WildersState`

文件：

- `rust/src/indicators/overlay/wilders.rs`

这里最自然，因为 `WildersStream` 本来就已经有 `feed_sample(...)`。  
所以这次只是在它外面包了一层：

- `WildersState`
- `RingHistory<Real>`

属于“低风险高复用”的典型 typed state。

### 2. 给 NATR 补了 `NatrState`

文件：

- `rust/src/indicators/indicator/natr.rs`

`NATR` 的特点是：

- 三输入 `(high, low, close)`
- 有 warmup
- 输出还要做一次归一化

这次做法是先给 `NatrStream` 提炼 `update_one(high, low, close)`，然后在外层包：

- `NatrState`

这样 typed 状态层就又补了一类“三输入单输出指标”。

### 3. 给 PPO 补了 `PpoState`

文件：

- `rust/src/indicators/indicator/ppo.rs`

`PPO` 和 `MACD` 是一家人，但比 `MACD` 更简单：

- 单输出
- 双 EMA 差值归一化

这次也是先提炼 `update_one(sample)`，再加 `PpoState`。  
这样 typed 状态层对“单输入、lookback 1、递推型振荡器”也有了覆盖。

### 4. 给 STOCH 补了 `StochState`

文件：

- `rust/src/indicators/indicator/stoch.rs`

`STOCH` 是这批里最复杂的一个：

- 三输入 `(high, low, close)`
- 双输出 `(stoch_k, stoch_d)`
- 有多阶段 warmup
- 内部有最大最小队列和两层均值

这次没有去碰 batch kernel，而是：

- 从 `StochStream` 提炼 `update_one(high, low, close)`
- 在外层包 `StochState`
- 用 `RingHistory<(Real, Real)>` 保存双输出结果

这很重要，因为它证明 typed 状态层已经能覆盖“较复杂的多输入双输出状态机”。

### 5. 补导出

修改文件：

- `rust/src/indicators/overlay/mod.rs`
- `rust/src/indicators/indicator/mod.rs`
- `rust/src/lib.rs`

现在这些新的 typed state 也都可以直接从 crate 根拿到。

### 6. 补测试

文件：

- `rust/tests/state_api.rs`

这次新增了四组 typed state 对 batch 的一致性测试：

- `wilders`
- `natr`
- `ppo`
- `stoch`

也就是说，状态接口测试现在已经覆盖到了：

- 单输入、零 lookback
- 单输入、有 lookback
- 双输入、双输出
- 三输入、单输出
- 三输入、双输出
- 三输出递推指标

typed 状态层的代表性已经明显更强了。

## 关键知识

## 为什么这次还要继续做 typed state，而不是只保留 DynamicIndicatorState

`DynamicIndicatorState` 已经解决了“全覆盖”的问题，但它返回的是：

- `Vec<Real>`

这对通用性很好，但对高频业务代码来说，还是不够顺手。

typed state 的价值就在于：

- 输入类型更明确
- 输出结构更明确
- 不需要每次去记 `values[0]`、`values[1]`

所以这条线不是和 dynamic 状态层竞争，而是继续在它之上做“高频指标优化体验”。

## 为什么 STOCH 值得单独做

如果一个状态接口体系只能覆盖：

- 单输入
- 单输出
- 简单均线

那它还不算真正成熟。

`StochState` 这次的意义就在于，它证明了这套 typed 状态层已经能支撑：

- 多输入
- 双输出
- 多阶段 warmup
- 比较复杂的内部状态机

这比再补一个简单均线更有代表性。

## 验证

- `cargo fmt --all` (`PASS`)
- `cargo test --test state_api --test stable_parity --test golden_indicators` (`PASS`)
- `cargo clippy --all-targets --all-features` (`PASS`)

## 未覆盖项

- 这次继续扩了一批高频 typed state，但仍然没有把所有 stream-backed 指标都补成 typed `FooState`
- `MacdFix`、`StochRsi` 等相邻指标这次没有顺手补 typed state
- 顶层文档和 README 里的状态接口示例仍然还没同步，当前主要依赖测试和提交教程说明用法
