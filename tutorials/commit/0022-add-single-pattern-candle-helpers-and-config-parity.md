# 给 Rust candle engine 加上单模式入口和多配置对账

## 这次为什么还要继续改

上一笔我们已经让 Rust 拥有了 candle engine，也把默认配置下的 `candles.txt` 和 C candle engine 对齐了。  
但那还不够，因为真实使用里至少还有两个明显缺口：

- 调用方经常只想跑某一个 candle pattern，而不是整套 `TC_ALL`
- 默认配置对齐了，不代表非默认 `CandleConfig` 也对齐了

所以这次继续补的是：

- 更像库 API 的单模式入口
- 更像回归护栏的多配置 parity

## 这次做了什么

### 1. 新增单模式 candle helper

现在 Rust 这边不只暴露整套扫描：

- [`run_candles`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/candles.rs)

还新增了：

- [`run_candle_pattern`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/candles.rs)
- [`run_candle_named`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/candles.rs)

这样调用方可以直接：

- 按 bit 模式跑
- 按名字跑

不用自己先查 metadata，再手动组 bitset。

### 2. 扩展 C candle oracle，让它支持自定义配置

原来的 C candle oracle 只会跑默认配置。  
这次把它扩展成可以从 stdin 读取：

- `patterns`
- `input_len`
- `body_none`
- `body_short`
- `body_long`
- `wick_none`
- `wick_long`
- `near`

这样 Rust 测试就能真正问：

- “在这些配置下，Rust 和 C 还是不是同一个 engine？”

### 3. 新增多配置 parity

这次在 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/candles_parity.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/candles_parity.rs) 里补了多组 `CandleConfig`，逐组对比：

- Rust `run_candles(TC_ALL, ...)`
- C candle oracle

这样 candle 对齐就不再只覆盖默认参数，而是开始覆盖配置变化带来的行为变化。

### 4. 调整单模式 helper 的语义锚点

这里还顺手暴露了一个重要事实：

- C 的 `tc_run(single_pattern)` 快路径，并不保证永远和 `TC_ALL` 后再取单 bit 完全一致

这个差异在 `dragonfly_doji` 上已经能看到。

所以这次我没有把 Rust 单模式 helper 绑定到那个历史上不稳定的快路径，而是明确让它们对齐：

- Rust 全量扫描结果里的对应单 bit

这是更稳的语义，也更符合“同一个 engine，只是缩小输出范围”的直觉。

## 为什么这个设计更合理

如果把 Rust helper 硬绑到 C 的单模式快路径，就会遇到一个很糟糕的问题：

- 全量扫描是一套语义
- 单模式 helper 又变成另一套语义

这会让 Rust API 本身自相矛盾。

相比之下，把单模式 helper 定义成：

- “全量扫描语义的单 pattern 投影”

会更一致，也更容易解释给使用者。

## 新手知识点 1：API 设计里，最重要的不只是“有没有这个函数”，而是“它的语义锚点是什么”

这次 `run_candle_named` 和 `run_candle_pattern` 的难点，不是写两个包装函数，而是决定它们应该对齐谁。

新手很容易只想：

- “加一个函数就好了”

但真正重要的是：

- 它返回的结果，到底应该和哪套语义一致？

这里有两个候选：

1. 对齐 C 的单模式快路径
2. 对齐全量扫描后取单 bit 的结果

如果你选错锚点，函数虽然能跑，但 API 会变得很难理解。

所以以后你设计类似 helper 时，一个很好用的问题是：

- “这个 helper 是一个新语义，还是已有语义的投影？”

大多数情况下，投影比新语义更稳。

## 新手知识点 2：做 parity test 时，配置空间往往比样例空间更容易漏

很多人做对账测试时，只会想到：

- 多喂几组输入数据

这当然重要，但还不够。  
像 candle engine 这种带阈值配置的逻辑，另一条同样重要的维度是：

- 配置变化后，结果还对不对

因为很多 bug 只会在默认配置之外出现，比如：

- 阈值更紧
- 阈值更松
- `near` 和 wick/body 比例改变

如果只测默认配置，你可能会以为 engine 很稳，但其实只是默认值碰巧没踩到问题。

## 这次之后的状态

现在 Rust candle 这条线相比上一笔又更完整了一层：

- 默认配置 parity：有
- `candles.txt` 命名期望：有
- 多配置 parity：有
- 单模式 helper：有

这意味着 candle engine 已经不只是“能跑”，而是开始拥有可用 API 和更像样的回归护栏了。
