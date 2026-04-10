# 给动态状态接口补 `UnknownIndicator` 错误，并让 `reset` 返回 `Result`，避免硬 panic

这次收的是两个接口语义问题。

第一，`DynamicIndicatorState::from_name` 里如果名字找不到，之前返回的是 `InternalInvariant`。这不对，因为名字来自调用方输入，不是库内部状态损坏。现在补了 `IndicatorError::UnknownIndicator { name }`，把“用户传了未知指标名”和“库内部不变量被打破”区分开。

第二，`DynamicIndicatorState::reset` 之前内部会重新 `create_stream`，但直接 `expect("indicator state reset should not fail after construction")`。这等于把一个本来可以传播的错误，硬变成库内 panic 点。现在把 backend 重建收成一个可复用的 fallible helper，`new` 和 `reset` 都走它，`reset` 也正式返回 `Result<(), IndicatorError>`。

## 为什么这笔改动值当

- 错误语义更干净：未知指标名就是 lookup error，不再伪装成内部 invariant。
- 动态状态 API 更体面：调用方可以决定怎么处理 reset 失败，而不是被库直接打断。
- 构造和重置共用一套 backend 初始化逻辑，后续再改 stream/batch 选择时不会出现两套分叉。

## 改动点

1. 在 `core/error.rs` 里新增 `UnknownIndicator`，并补上 `Display` 输出。
2. 在 `state.rs` 里提取 `DynamicIndicatorState::build_backend(...) -> Result<...>`。
3. `DynamicIndicatorState::new` 改走 `build_backend`。
4. `DynamicIndicatorState::from_name` 找不到指标时改返回 `UnknownIndicator`。
5. `DynamicIndicatorState::reset` 改成 `Result<(), IndicatorError>`。
6. `benchmark.rs` 里按名称筛 benchmark 时，也改用 `UnknownIndicator`。
7. `state_api.rs` 里动态 state 的 reset 测试改成显式 `expect(...)`。

## 这次没有动什么

- typed `IndicatorState::reset(&mut self)` 还是保持无返回值，它的语义仍然是本地状态清零，不涉及 fallible backend 重建。
- `RingHistory<T>` 仍然返回克隆值，没有在这一笔里切到借用返回。
