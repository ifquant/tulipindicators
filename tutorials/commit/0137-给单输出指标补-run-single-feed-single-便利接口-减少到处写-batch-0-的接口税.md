# 给单输出指标补 `run_single` / `feed_single` 便利接口，减少到处写 `batch[0]` 的接口税

这次收的是一个纯接口层的噪声问题。

库里的绝大多数指标其实都只有一个输出序列，但 batch API 一直统一返回 `Vec<Vec<Real>>`。这样做有它的理由，因为像 `macd`、`dm`、`stoch` 这种多输出指标确实需要统一形状；问题在于，单输出指标的调用方几乎总要写一遍：

```rust
let batch = Rsi.run(&[&input], &[14.0])?;
let values = &batch[0];
```

这段代码不难，但会反复出现，而且容易把“单输出”这个更强的语义藏掉。

## 这次怎么做

给 trait 默认补两个便利方法：

- `Indicator::run_single(...) -> Result<Vec<Real>, IndicatorError>`
- `IndicatorStream::feed_single(...) -> Result<Vec<Real>, IndicatorError>`

它们只服务单输出指标。

如果一个指标实际上有多个输出，比如 `macd`，这些 helper 会直接返回 `WrongOutputCount { expected: 1, actual: 3 }`，而不是悄悄只取第一个输出。这样能保证接口是省样板，而不是制造歧义。

## 设计取舍

这笔没有改 `run()` / `feed()` 的返回类型，也没有试图优化它们的分配行为。原因很简单：

- 这次目标是把“`batch[0]` 到处出现”的接口税收掉。
- 如果直接改主签名，会把所有多输出指标、trait object 和现有调用方一起打碎。
- 先用默认方法补一个更顺手的单输出入口，兼容性最好。

## 适合什么时候用

- 你明确知道这个指标只有一个输出，例如 `sma`、`ema`、`rsi`、`atr`。
- 你只想拿那一个序列，不想再拆 `Vec<Vec<Real>>`。

## 不适合什么时候用

- 指标天然有多个输出，例如 `macd`、`dm`、`di`、`stoch`。
- 这时还是应该继续用 `run()` / `feed()`，因为多输出本身就是调用语义的一部分。
