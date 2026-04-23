# 明确拒绝 state `history_capacity = 0`，别再让 `RingHistory` 偷偷把 `0` 夹成 `1`

code review 里提到一个接口语义问题：`RingHistory::new(0)` 之前会内部变成容量 `1`。

这虽然避免了内部 ring buffer 出错，但对调用方来说是不透明的。用户传了 `0`，很可能是在表达“不想保留历史”。结果库却静默保留了 1 条历史，这不是一个好接口。

## 这次的决策

明确拒绝 `history_capacity = 0`。

原因是 state API 的核心语义就是：

- 增量更新
- 固定容量历史
- `latest()`
- `get(index_from_latest)`

如果容量是 0，`latest()` 和 `is_ready()` 的语义会变得很奇怪：指标可能已经产出，但 state 永远没有 latest。真正想要“只更新、不保留历史”的用户，应该使用更底层的 stream API，而不是把 state API 的历史模型掰弯。

## 改动点

1. `RingHistory::new(capacity)` 不再 `capacity.max(1)`。
2. 新增 `validate_history_capacity(...)`，统一返回：

```rust
IndicatorError::InvalidOption {
    option: "history_capacity",
    reason: "must be greater than zero",
    ...
}
```

3. 所有公开 typed state 构造函数都接入这个校验。
4. `DynamicIndicatorState::new/from_name` 也接入同一校验。
5. README 和 state API 文档明确说明容量必须至少为 `1`。

## 为什么不支持真正 zero-history mode

zero-history 是另一个接口，而不是当前 state API 的一个小参数。

如果后面真要支持，可以单独设计：

- `FooUpdater`
- no-history dynamic updater
- 或者直接推荐 `IndicatorStream::feed_in_place`

但这不应该偷偷藏在 `history_capacity = 0` 里。
