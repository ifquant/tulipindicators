# 启动 Rust candle engine，并先和 C 对齐

## 这次改动的目标

前面我们已经把指标迁到了 Rust，也把指标测试和 C 的强度对齐了。  
但蜡烛图 engine 还是一个明显缺口：

- C 里有完整的 `tc_run`
- Rust 里还没有 candle engine
- 所以 Rust 测试再强，也还不能覆盖 `candles.txt`

这次改动的目标不是“讨论以后怎么做”，而是直接让 Rust 第一次真正拥有 candle engine，并且一上来就接到 C parity。

## 这次做了什么

### 1. 新增 Rust candle core

新增了 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/candles.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/candles.rs)，里面先把 candle engine 的最小核心搬了过去：

- `CandleSet`
- 26 个 pattern bit 常量
- `CandleConfig`
- `CandleHit`
- `CandleResult`
- `CandleInfo`
- `find_candle` / `get_candle_info`
- `run_candles`

这意味着 Rust 这边不再只是“知道 candles 存在”，而是真的能按默认配置跑完整蜡烛图扫描。

### 2. 直接把 26 个模式条件迁进 Rust

这里没有走“先迁 2 个 pattern 试试看”的路线，而是把 C 里的那套统一窗口逻辑和 26 个 pattern 条件一起搬过来了。

这样做的原因很简单：

- candle engine 的复杂度不在单个 pattern，而在共享的窗口统计
- 一旦共享统计搭好，26 个 pattern 大多只是条件表达式

所以这里整包迁移，反而比拆得太碎更稳。

### 3. 新增 `candles.txt` 解析器和 C parity 测试

为了不让 Rust candle engine 变成“看起来实现了，但没人知道是不是对的”，这次还补了：

- [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/support/candle_cases.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/support/candle_cases.rs)
- [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/support/candle_oracle.c`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/support/candle_oracle.c)
- [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/candles_parity.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/candles_parity.rs)

它们做两件事：

1. 让 Rust 直接验证 `c/tests/candles.txt` 里的命名期望
2. 让 Rust 逐 bar 对比 C candle engine 输出的 pattern bitset

也就是说，现在 candle 这条线已经不是“只有 C smoke 在测”了。

## 为什么这一步很关键

到这一步为止，Rust 和 C 的对齐已经分成两块：

- 指标：已有 parity
- candles：现在也有 parity

这件事的真正价值不是“测试数量更多”，而是：

- Rust 已经开始接管 C 的两个核心语义面

如果没有 candle engine，这个仓库始终会留下一块“Rust 还没碰到的核心功能”。现在这个空洞已经开始被填上了。

## 新手知识点 1：bitset 很适合表示“同一根 bar 命中了哪些模式”

这次 candle engine 用的是 `u64` bitset，而不是 `Vec<String>` 或 `Vec<Enum>`。

原因很现实：

- 一根 bar 可以同时命中多个 pattern
- pattern 总数固定，而且只有 26 个

这种情况下，用 bitset 有几个直接好处：

- 内存紧凑
- 合并模式很快，直接按位或
- 和 C 原版语义天然一致

如果你是 Rust 新手，可以先这样理解：

- `1 << n` 表示第 `n` 个模式
- `a | b` 表示把两个模式集合合并
- `set & pattern != 0` 表示检查某个模式是否命中

这是一种非常常见、也很实用的数据表达方式。

## 新手知识点 2：有些迁移任务，整包迁比“每次搬一点点”更稳

很多人一开始会觉得：

- 先迁一个 pattern
- 再迁第二个
- 最安全

但这并不总对。

这次 candle engine 就是一个典型反例。

因为它的 26 个 pattern 并不是 26 套完全独立逻辑，而是共享：

- 同一组输入
- 同一套平均窗口
- 同一组宏含义
- 同一个结果容器

在这种结构下，先把共享骨架搭对，再把条件整体搬过来，通常比碎片化迁移更不容易出现“局部看起来没问题，整体语义却不断漂移”的情况。

一个实用判断标准是：

- 如果复杂度主要在共享框架，倾向整包迁移
- 如果复杂度主要在独立子模块，倾向分批迁移

## 这次之后的状态

现在可以更准确地说：

- Rust 已经有 candle engine 了
- Rust candle 测试已经接上 `candles.txt`
- Rust candle 输出已经能和 C candle engine 逐 bar 对账

后面如果继续推进，最自然的方向是：

- 把 candle API 再整理得更像 Rust 风格
- 根据需要补更多 config 维度的测试
- 决定是否把 candle engine 进一步拆成更清晰的内部模块
