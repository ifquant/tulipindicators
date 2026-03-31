# 把统一状态接口扩成全指标可用的动态层，并给无 stream 指标补 batch fallback

## 背景

上一笔提交已经证明了一个方向是可行的：

- 不动现有高性能层
- 在外面额外包一层统一的状态接口
- 先拿 `rsi` 和 `dm` 这种典型指标做原型

但那个版本还有一个明显边界：它只给少数指标提供了 typed 状态封装，还不能说“整个库都有统一的状态接口”。

如果要把这件事真正做完整，不能继续一个指标一个指标补 typed wrapper。那样会非常慢，而且会让接口层自己变成新的维护负担。

更合理的做法是：

- 保留 typed 原型，继续作为更友好的示例
- 再补一层 **动态状态接口**
- 让所有指标都能以统一方式使用

## 主要目标

这次提交的目标有两个：

1. 把统一状态接口从“少数指标原型”扩成“全库可用的动态层”
2. 对没有 `create_stream(...)` 的指标，补一个内部 batch fallback，这样接口能力不会断层

换句话说，这次追求的是：

- API 覆盖完整
- 实现边界清晰
- 但仍然 **不动现有高性能层**

## 改动概览

### 1. 新增 `DynamicIndicatorState`

核心改动在：

- `rust/src/state.rs`

这次新增了 `DynamicIndicatorState`，它不要求调用者提前知道具体 typed 状态对象，只需要：

- 指标名或指标对象
- options
- history capacity

就能构造统一状态实例。

它提供的能力包括：

- `seed_columns(...)`
- `seed_rows(...)`
- `update(...)`
- `latest()`
- `get(index_from_latest)`
- `reset()`

这样库的状态接口终于不再只停留在少数原型上，而是形成了统一入口。

### 2. 对有 stream 的指标，继续复用 stream backend

如果指标本身已经实现了 `create_stream(...)`，动态状态层不会重写它的内部逻辑，而是直接复用现有 stream/state 机理。

这符合这轮一直坚持的边界：

- 高性能层不动
- 已经存在的状态推进逻辑不重写
- 新层只负责把使用方式统一起来

### 3. 对没有 stream 的指标，新增 batch fallback backend

这是这次最关键的一步。

对于没有 `create_stream(...)` 的指标，`DynamicIndicatorState` 会自动退到 batch fallback：

- 内部累积输入历史
- 每次 `seed` 或 `update` 时调用已有 `run(...)`
- 从 batch 输出里提取最新一行
- 再写入统一的 ring history

这条路径不追求成为高性能增量实现，它的目标是：

- 保证接口完整
- 不让“没有 stream”变成“不能 state/update”

也就是说，这次把“统一状态接口”从“能力不完整”推进到了“对全库都可用”。

### 4. 继续保留 typed `RsiState / DmState`

这次没有删掉上一笔提交里补的 typed 原型。

原因很简单：

- `DynamicIndicatorState` 解决的是“全覆盖”
- typed `RsiState / DmState` 解决的是“更自然的静态类型使用”

这两层并不冲突，反而互补：

- 想快速覆盖全指标时，用动态层
- 想在核心业务里获得更自然类型时，用 typed 原型

### 5. 新增 batch fallback 测试

测试文件：

- `rust/tests/state_api.rs`

除了上一笔已有的 `rsi` / `dm` 状态测试，这次还新增了 `ma` 的动态状态测试。

`ma` 是一个很合适的例子：

- 它没有 `create_stream(...)`
- 但它又是典型真实指标，不是人为造出来的测试桩

这条测试证明了：

- 动态状态接口确实不是“只有 stream 指标能用”
- batch fallback 的行为和 batch 结果保持一致

## 关键知识

## 为什么这次要接受 batch fallback

如果严格要求“所有指标都必须先有 stream 状态机，才能有统一状态接口”，那这个任务会拖得很长，而且会倒逼我们去大规模改指标实现。

这和当前原则冲突：

- 不动高性能层
- 不为了接口好看去重写内核

batch fallback 的价值就在于：

- 先把接口能力补齐
- 再把性能敏感的状态化实现逐步替换成真正的 stream/backend

所以它不是终局性能方案，而是完整 API 的必要桥梁。

## 为什么动态层和 typed 层要同时保留

如果只有 typed 层：

- 要给很多指标一个个写 wrapper
- 覆盖会很慢

如果只有动态层：

- 某些核心业务场景又不够顺手

所以这次的分层是：

- typed 层：少量高价值原型
- dynamic 层：全覆盖统一入口

这比只押一边更稳。

## 为什么这次仍然没有碰高性能层

因为这次做的是“接口完整性”，不是“重新设计指标实现”。

无论是 stream backend 还是 batch fallback，底下都只是在复用已有：

- `create_stream(...)`
- `run(...)`
- 既有 batch kernel

也就是说，状态接口这一层到这里仍然是“加法”，不是“替换”。

## 验证

- `cargo fmt --all` (`PASS`)
- `cargo test --test state_api --test stable_parity --test golden_indicators` (`PASS`)
- `cargo clippy --all-targets --all-features` (`PASS`)

## 未覆盖项

- 这次虽然把统一状态接口扩成了全指标可用，但 batch fallback 路径不以增量性能为目标
- 这次没有给所有指标补 typed `FooState`，仍然只保留 `rsi` 和 `dm` 原型
- 这次没有设计更高级的 builder 语法，例如 `with_history(...)` 这种更链式的构造方式
- 这次没有废弃旧的 `stream` 概念；它仍然作为内部实现手段存在
