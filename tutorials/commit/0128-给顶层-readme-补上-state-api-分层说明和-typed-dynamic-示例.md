这次没有继续补实现，而是把已经完成的状态接口写进了顶层 README。

主要补了三件事：

1. 说明 batch 层和 state 层的分工

- `run(...)`
- `run_in_place(...)`
- typed `FooState`
- `DynamicIndicatorState`

这样外部使用者第一次打开仓库时，就能知道：

- 离线整段算，用 batch
- 极致 batch 性能，用 `run_in_place`
- 增量更新和历史访问，用 state

2. 给出 typed state 示例

示例里直接演示：

- `Rsi::state(...)`
- `seed(...)`
- `update(...)`
- `latest()`
- `get(index_from_latest)`

这样读者不需要先翻测试文件，README 就能直接说明这种接口的意图。

3. 给出 dynamic state 示例

README 里同时放了：

- `DynamicIndicatorState::from_name(...)`
- `RSI.dynamic_state(...)`

这样就把“按名字动态选择指标”和“用静态句柄构造状态对象”两种入口都讲清楚了。

一个对新人有帮助的点：

统一状态接口不是为了替代高性能层，而是为了补齐另一种使用模式：

- 历史数据先热起来
- 后面一条条更新
- 还要按索引取最近结果

所以最重要的不是把所有内部实现改成一种样子，而是把对外使用分层说清楚。
