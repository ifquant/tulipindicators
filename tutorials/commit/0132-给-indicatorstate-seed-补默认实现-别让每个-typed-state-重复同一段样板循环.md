这次继续收的是 typed state 层的重复样板。

之前几乎每个 typed state 都自己写了一份一模一样的：

- 遍历输入
- 调 `update(...)`
- 统计产出个数

所以现在把它提到了：

- `rust/src/state.rs`
- `IndicatorState::seed`

默认实现上。

新的默认实现带了一个明确边界：

```rust
where
    Self::Input: Copy
```

这正好适配当前 typed state 这批输入形状：

- `Real`
- `(Real, Real)`
- `(Real, Real, Real)`

所以这次能安全删掉一大批重复 `seed`：

- `EmaState`
- `WildersState`
- `SmaState`
- `RsiState`
- `MacdState`
- `PpoState`
- `AtrState`
- `NatrState`
- `StochState`
- `DmState`
- `DxState`
- `DiState`
- `AdxState`
- `AdxrState`

这次刻意没动两件事：

1. `DynamicIndicatorState`

它的 `seed_columns/seed_rows` 语义更复杂，不是 typed state 默认实现能覆盖的。

2. `IndicatorState` 的返回类型和历史访问模型

这次只收 `seed` 样板，不顺手改 trait 其它设计，避免把“重复消除”和“接口设计调整”混在一笔里。

一个实现层面的经验：

当多个实现都在重复“调用另一个核心方法”的固定外壳时，通常应该优先把外壳收回 trait 默认实现。  
这样每个具体类型就只需要保留真正有差异的部分：

- `update`
- `latest/get`
- `reset`
