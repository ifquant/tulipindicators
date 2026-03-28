# 让 Rust 测试至少和 C 测试一样强

## 这次改动的背景

前面我们已经把 C 的指标实现迁到了 Rust，也已经给 beta 指标补了一条 C/Rust 对账链。  
但如果目标是“Rust 版本真正接棒 C 版本”，那测试标准就不能只是“Rust 自己能过”。

更严格的问题是：

- Rust 的测试强度，是否至少和 C 当前的 `smoke.c` 一样？
- C 已经在验证的东西，Rust 有没有漏掉？

如果答案是否定的，那么即使 Rust 代码已经迁完，测试也还是偏弱的。

## 这次主要补了什么

### 1. 给稳定指标也加上了 C oracle parity

这次新增了稳定指标的 parity 测试：

- [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/stable_parity.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/stable_parity.rs)
- [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/support/stable_oracle.c`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/support/stable_oracle.c)

它的作用是：

- 读取和 C smoke 同一批测试文件
- 对每个稳定指标 case，分别跑 Rust 实现和 C 实现
- 逐点比较输出

这样稳定指标就不只是“Rust 对 fixture 正确”，而是“Rust 对 C 当前实现正确”。

### 2. 把 Rust 的 stream 分块步长对齐到 C smoke

之前 Rust 的 stream 测试只用了几组分块步长，例如 `1, 2, 3, 5, 7, 64`。  
C 的 `smoke.c` 用得更完整：

- `1, 2, 3, 4, 5, 7, 13, 100, maxarray`

这次把 Rust 的步长集合也对齐到了这个级别。  
这样可以覆盖更多现实中的分块喂数方式，尤其是：

- 很小块
- 中等块
- 几乎一次喂完的大块

这些情况在流式指标里经常能暴露不同的边界问题。

### 3. 抽出共享的 golden case 解析

这次还把测试文件解析逻辑抽到了：

- [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/support/golden_cases.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/support/golden_cases.rs)

好处是：

- `golden_indicators.rs` 和 `stable_parity.rs` 现在共用一套测试输入解析逻辑
- 避免一边修了解析规则，另一边忘了同步

这种共享基础设施在测试里尤其重要，因为测试代码分叉之后，最容易出现“看起来都在测同一批数据，其实解析方式已经不一样了”的隐性问题。

## 这次之后，Rust 测试相比以前更强在哪里

现在 Rust 这边至少有这几层：

1. 现有 golden data 对账
2. Rust batch 和 Rust stream 的分块一致性测试
3. beta 指标对 C batch 的 parity
4. 稳定指标对 C batch 的 parity

这比之前明显更接近 C smoke 的保证范围。

要注意，这里说“至少和 C 测试一样强”，不是说 Rust 要机械复制 C 的每一行测试代码，而是说：

- C 已经验证的语义，不应该在 Rust 里变弱

如果 C 已经拿一批官方测试数据验证了某个指标，那 Rust 也应该有同级别甚至更强的验证手段。

## 为什么这对协作重要

人和 AI 协作时，最危险的不是“明显写错”，而是“测试看起来很多，但覆盖点不对”。

例如以前 Rust 已经有：

- golden tests
- stream tests

但如果这些测试没有直接锚定 C 语义，那它们仍然可能一起偏掉。  
一旦 batch 和 stream 都在 Rust 里沿着同一个错误实现前进，就会出现：

- Rust batch = Rust stream
- 但 Rust != C

parity test 的价值就在这里：

- 它提供了一个外部锚点

这样 AI 改实现时，不会只是在 Rust 自己的小世界里自洽。

## 新手知识点 1：测试“复用数据”不等于“复用语义”

这是这次最值得记住的一点。

很多人会以为：

- Rust 读了和 C 一样的测试文件
- 所以 Rust 测试就已经和 C 对齐了

其实不一定。

因为“同一份数据”只能说明输入样例一致，不能说明：

- 调用路径一致
- 输出解释一致
- 边界处理一致

真正的语义对齐，往往还需要一个外部 oracle，也就是这次的 C helper。

所以一个很实用的判断标准是：

- 你是在验证“我的代码能跑出一个结果”
- 还是在验证“我的代码跑出的结果，和被接管的旧系统一致”

这两个层级差很多。

## 新手知识点 2：流式测试里，“步长集合”本身就是设计的一部分

很多新手写 stream 测试时，只会测两种：

- 一次一个
- 一次全喂完

这不够。

因为真正容易出 bug 的，往往是中间那些不规则分块：

- `4`
- `7`
- `13`
- `100`

这些数字本身没有神奇之处，关键是它们会穿过不同的窗口边界、初始化边界、输出开始边界。

所以测试步长不是随便写几个数字就行，它实际上是在决定：

- 你有没有真的测试“同一个 stream 状态机会不会因为喂数节奏不同而改变结果”

如果你以后自己写类似测试，至少要覆盖：

1. 极小步长
2. 几种不整齐的中间步长
3. 比输入长度更大的步长

这样才比较像真正的流式稳健性测试。

## 这次改动后的状态

现在可以更有底气地说：

- Rust 测试不再只是“看起来不少”
- 而是开始系统性地向 C 的测试强度对齐

后面如果继续加强，最自然的方向有两个：

- 让稳定指标 parity 也覆盖更多 C 内部路径信息
- 视情况决定是否把某些 C 的 reference/stream 历史差异显式标注成已知行为，而不是隐含在代码里
