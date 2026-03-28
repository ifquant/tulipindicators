# 把 Rust candle 测试补到 period 和 metadata 都对齐 C

## 这次改动的背景

前一轮我们已经让 Rust candle engine 在默认配置下和 C engine 对齐，也补了单模式 helper。  
但如果目标是“Rust 测试不能比 C 弱”，还差两块没有完全落地：

- `CandleConfig.period` 还没有进入 parity 测试
- Rust 还没有直接验证自己的 candle metadata 是否和 C 的 `tc_candles` 表完全一致

这两个缺口都不大，但都属于会在后面慢慢积累漂移的地方。  
所以这次不是去设计新 API，而是继续把“对齐”做实。

## 这次做了什么

### 1. 把 `period` 加进 C/Rust candle parity

现在 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/candles_parity.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/candles_parity.rs) 里的配置组不再只有默认 `period: 10`，还显式覆盖了非默认 `period`。

这很重要，因为 candle engine 不是纯函数表，它内部有滚动窗口平均值：

- `avg_body`
- `avg_total`

而 `period` 直接决定这两个阈值参考系。  
如果这个维度不测，默认配置对齐也不能说明实现真的稳定。

### 2. 让 C oracle 能吐出 metadata，而不是只吐运行结果

这次扩展了 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/support/candle_oracle.c`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/support/candle_oracle.c)：

- 普通模式继续输出逐 bar 的 candle set
- `--metadata` 模式输出 C 侧的 candle 名称、全名和 pattern bit

有了这条能力，Rust 测试就不只是对行为对账，还能对“表结构”对账：

- 名字有没有漂移
- `full_name` 有没有写错
- bit pattern 有没有对错位
- `candle_count` 和 `TC_ALL` 的 bit 数是不是一致

### 3. 补了一个 `shooting_star` 的定向 case

`c/tests/candles.txt` 没有给 `shooting_star` 提供正例，所以只靠老 fixture，虽然 parity 已经存在，但这个 pattern 的正命中行为并没有被单独钉住。

这次补了一个合成 case，直接验证：

- Rust 全量 candle engine 会命中 `shooting_star`
- Rust 单 pattern helper 的结果等于“全量扫描后取单 bit”
- 同一组输入在 C 全量 engine 下投影出来的 `shooting_star` 结果一致

这样这个 pattern 不再只是存在于 metadata 里，而是真正被执行路径覆盖到了。

## 这次顺手发现了什么

这次其实还暴露出一个很典型的“测试基础设施 bug”：

- C oracle 原来用 `tc_result_at` 逐位置读取命中
- 但在只有一个命中的情况下，这条 C API 会把结果读丢

这不是 Rust engine 的错误，而是 oracle 读取方式不稳。  
所以这次把 oracle 改成基于 `tc_result_get` 展开命中表，再恢复成逐 bar 输出。

这类修正很重要，因为：

- 如果 oracle 本身会吞结果
- parity test 就可能把“对齐问题”误判成“Rust 错了”

## 新手知识点 1：滑动窗口参数本身就是语义，不只是性能参数

新手常常会把 `period` 看成“窗口多大而已”。  
但在这种 pattern engine 里，`period` 不只是性能或平滑程度参数，它直接决定分类阈值：

- 什么叫 `body_short`
- 什么叫 `body_long`
- 什么叫 `wick_none`

也就是说，`period` 一变，判定语义就变了。  
所以只测默认 `period`，并不等于实现真的对齐。

以后你看到任何依赖滚动平均、窗口统计、平滑基线的逻辑时，都可以先问一句：

- “这个窗口参数是不是其实在改语义？”

大多数时候，答案是“是”。

## 新手知识点 2：测试 oracle 也可能有 bug，不能把它当神谕

做 C/Rust 对账时，很容易默认：

- C 是老实现
- 所以只要结果不同，就是 Rust 有 bug

但真实工程里，测试基准本身也可能写错，或者 API 有历史坑。  
这次 `tc_result_at` 的单命中读取问题就是典型例子。

一个更稳的思路是：

1. 先确认“真正要对齐的语义”是什么
2. 再确认你用来读取这个语义的工具是不是可靠

否则你会花很多时间在修一个其实不存在的 Rust bug。

## 这次之后，candle 这条线到了什么程度

现在 Rust candle 测试已经覆盖：

- `candles.txt` 的命名期望
- 默认配置下的 C/Rust parity
- 非默认 `period` 和阈值配置的 C/Rust parity
- candle metadata 与 C `tc_candles` 表的精确对账
- `shooting_star` 的定向正例

这意味着 candle 这条线已经不只是“能跑”，而是把容易漂移的结构层和配置层也钉住了。
