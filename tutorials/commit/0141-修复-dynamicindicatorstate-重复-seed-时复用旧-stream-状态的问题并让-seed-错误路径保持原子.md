# 修复 `DynamicIndicatorState` 重复 seed 时复用旧 stream 状态的问题，并让 seed 错误路径保持原子

这次是 code review 抓出来的一个真实正确性问题。

`DynamicIndicatorState::seed_columns(...)` 和 `seed_rows(...)` 的语义应该是“用一批历史数据重新初始化状态”。但旧实现对 stream-backed 指标只清掉了可见 history，没有重建内部 stream backend。

这会导致一种隐蔽错误：

1. 用第一批数据 seed 一个 `rsi` dynamic state
2. 再用第二批数据 seed 同一个 state
3. history 看起来被清了，但内部 `RsiStream` 的平滑状态、progress、last input 仍然可能来自第一批数据

结果就是 `latest()` 看似来自第二批 seed，后续 `update(...)` 却接着旧 stream 状态继续算，输出会悄悄偏掉。

## 这次怎么修

现在 public seed 流程改成两阶段：

1. 先校验输入形状
2. 创建一个新的临时 `DynamicIndicatorState`
3. 在临时 state 上完成 seed
4. 全部成功后再 `*self = seeded`

这样有两个好处：

- 重复 seed 一定从干净 backend 开始，不会复用旧 stream accumulator
- 如果 seed 过程中失败，原来的 state 不会被半清空、半更新

`seed_rows(...)` 也补了完整预校验：所有 row 形状都先检查完，再进入临时 state seed。

## 为什么不用简单 `reset()`

简单 `reset()` 可以解决“旧 stream 状态污染”的问题，但它还有一个弱点：如果后续 seed 失败，调用方手里的 state 已经被清掉了。

这次用临时 state staged commit，是更稳的接口语义：成功才替换，失败不破坏旧状态。

## 新增测试

新增了动态 `rsi` 的重复 seed 回归测试：

- 先用第一批输入 seed
- 再用第二批输入重新 seed
- 确认输出等于第二批输入的 batch `run_single`
- 再用 `seed_rows` 重复一次，确认 row seed 也会重建 stream backend

## 没有顺手做的事

reviewer 还指出 `RingHistory::new(0)` 当前会把容量静默夹到 `1`。这属于公开语义选择问题，会影响 typed state 和 dynamic state 的共同历史模型。

这次没有把 zero-capacity 语义混进来，后面应该单独决定：

- 是显式拒绝 `0`
- 还是支持真正的 zero-history mode
