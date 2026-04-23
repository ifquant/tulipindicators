# 给高频 typed state 指标补源码文档，说明输入输出和 warmup 边界

这次文档切片覆盖最常用、也最容易被用户直接实例化 state 的指标。

这些指标不只是 batch 函数，它们还有 `Foo::state(...)` 和 typed `FooState`，所以源码文档需要同时回答两类问题：

- batch 调用时输入列、参数、输出列是什么。
- 增量调用时 `update(...)` 的输入和输出类型是什么，什么时候开始产出。

## 覆盖的指标

本批覆盖：

- RSI、ATR、NATR
- DM、DX、DI、ADX、ADXR
- MACD、PPO、STOCH
- EMA、SMA、WILDERS

它们都是当前 state API 示例和测试里高频出现的指标，也是用户最可能先看的源码入口。

## 文档写法

每个文件的顶部增加模块级说明，主要写：

- 输入 shape，例如 `real`、`high/low`、`high/low/close`。
- option shape，例如 `period` 或 `short_period/long_period/signal_period`。
- output shape，例如单输出 `rsi`，或多输出 `macd/macd_signal/macd_histogram`。
- lookback/warmup 边界，例如 `period - 1`、`(period - 1) * 2`。
- typed state 的输入输出类型，例如 `Real -> Real` 或 `(Real, Real, Real) -> (Real, Real)`。

对性能敏感路径，只补已经存在的实现意图，不改算法。例如 MACD/PPO/ADX 这些注释只说明“batch path 和 stream path 保持一致、避免中间 series”，不重新解释公式。

## 为什么不写公式大全

公式大全会让源码变重，而且容易和实现漂移。

这批文档的目标是使用和维护入口：用户知道怎么调用，维护者知道 warmup 和 state 边界在哪里。具体公式如果以后要补，应该放在更系统的 indicator reference，而不是塞进每个 hot kernel 前面。

## 一个小经验

rustdoc 最好放在属性前面：

```rust
/// 指标入口说明。
#[derive(Debug, Clone, Copy)]
pub struct Rsi;
```

放在 `#[derive]` 后面也能编译，但 review 时不符合常见阅读顺序，后续统一按属性前置。
