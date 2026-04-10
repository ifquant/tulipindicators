# 把测试里的单输出调用点逐步改成 `run_single`，先从 `state_api` 开始收掉 `batch[0]` 噪声

上一笔只是把 `run_single` / `feed_single` 这个新入口补出来。那还不够，因为如果仓库里的调用点还都继续写：

```rust
let batch = Rsi.run(&[&input], &options)?;
let expected = &batch[0];
```

那这个 helper 只是“存在”，并没有真的把代码变干净。

所以这次开始做第二步：从测试和文档里最稳定、最容易确认语义的地方，逐步把单输出指标改成直接用 `run_single`。

## 为什么先改测试

测试最适合做第一批迁移，原因有三个：

1. 语义清楚。我们通常明确知道这里测的是 `rsi`、`ema`、`atr` 这类单输出指标。
2. 风险低。改的是断言准备代码，不是业务 kernel。
3. 能反向校验 helper 是否真的顺手。如果连测试都不愿意用它，说明接口设计还不够自然。

## 这次改了什么

主要收的是 `rust/tests/state_api.rs` 里的单输出指标：

- `rsi`
- `ma`
- `ema`
- `sma`
- `atr`
- `wilders`
- `natr`
- `ppo`
- `dx`
- `adx`
- `adxr`

这些地方现在直接拿：

```rust
let expected = Rsi.run_single(&[&input], &options)?;
```

而不是先拿一个 `Vec<Vec<Real>>` 再索引 `[0]`。

像下面这些仍然保留 `batch[0]`：

- `dm`
- `di`
- `macd`
- `stoch`
- `mama`
- `single_output_api` 里故意拿 `batch[0]` 做 helper 对照的测试

原因也很简单：它们要么本来就是多输出，要么测试目的就是证明 `run_single` 和老路径第一输出完全一致。

## 结果

- `state_api.rs` 的断言准备代码更直白了。
- 单输出与多输出的语义边界更清楚了。
- 后面继续收调用点时，也有了明确规则：只有语义确定的单输出 case 才换 `run_single`。
