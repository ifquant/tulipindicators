# 迁移 PSAR、Ultimate Oscillator、MFI 和 KVO

## 背景

上一批我们把剩余的中等复杂度指标又清掉了一部分，Rust 对齐率来到了 `78 / 104`。  
这之后最明显的空白，就集中在几类“状态更强、输入更多、公式更像交易规则而不是简单窗口”的指标上。

这一批选的是：

- `psar`
- `ultosc`
- `mfi`
- `kvo`

它们很适合作为下一步，因为四个指标分别覆盖了四种常见难点：

- 趋势切换状态机
- 多窗口组合振荡器
- 资金流方向累计
- 带趋势上下文的 EMA 派生量

## 主要目标

这次提交的目标不是单纯再加 4 个名字，而是验证 Rust 核心已经能稳定承接这几类更复杂的实现模式：

- 明确的状态机切换
- 多输入指标
- 多窗口滚动和的组合输出
- 首根样本和趋势初始化的精确对齐

迁完之后，Rust registry 从 `78 / 104` 提升到 `82 / 104`。

## 改动概览

- 新增 Rust `psar` 实现，补上趋势方向、极值、加速因子和反转处理的完整状态机
- 新增 Rust `ultosc` 实现，用三组买压/真实波幅滚动和组合输出
- 新增 Rust `mfi` 实现，用典型价格和成交量构建正负资金流窗口
- 新增 Rust `kvo` 实现，用趋势方向、累计波幅和双 EMA 重建 Klinger Volume Oscillator
- 把这四个指标接进 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/registry.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/registry.rs)
- 更新 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/golden_indicators.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/golden_indicators.rs) 的默认参数，让 `psar` 和 `ultosc` 的流式一致性测试不会被无效选项跳过

## 关键知识

### 1. `psar` 不是普通窗口指标，而是状态机

`PSAR` 的难点不在均值，而在“趋势方向什么时候翻转”。

它需要持续维护：

- 当前是多头还是空头
- 当前 `sar`
- 当前趋势极值 `extreme`
- 当前加速因子 `accel`
- 前 1 根和前 2 根 bar 的高低点

也就是说，它更像一台小状态机，而不是一个只看固定窗口的统计函数。

### 2. `ultosc` 是三套窗口同时工作的组合指标

`Ultimate Oscillator` 会同时维护短、中、长三组周期：

- `short_period`
- `medium_period`
- `long_period`

然后把三组买压/真实波幅比值按不同权重加总。  
这类指标很适合用来训练“不要只想着一个窗口”的思维。

### 3. `mfi` 的核心不是价格涨跌，而是资金流方向

`MFI` 用的是典型价格：

`(high + low + close) / 3`

然后看这根典型价格相对上一根是上升还是下降，再把：

`typical_price * volume`

算进正资金流或负资金流。  
所以它是“价格方向 + 成交量”结合后的振荡器，不是单纯价格动量。

### 4. `kvo` 的坑在初始化和趋势切换

`KVO` 表面上像双 EMA 差值，但真正麻烦的是它先要构造一个 `vf`，而 `vf` 又依赖：

- 当前趋势方向
- 累计波幅 `cm`
- 前一根 bar 的波幅

这意味着只要初始化细节不对，前几根结果就会整体偏掉。

这次第一次 `cargo test` 失败，最后定位到的就是这里：  
趋势切换时我把 `cm` 重置成了当前 bar 的振幅，但 C 实现实际用的是前一根 bar 的振幅。

## 补充知识

### 1. Rust 里的 `Option` 很适合表达“状态还没初始化”

这次 `psar`、`ultosc`、`mfi`、`kvo` 都需要处理“第一根样本还不够开始计算”的阶段。

对新手来说，`Option<T>` 的一个很实用用法就是：

- `None` 表示状态还不存在
- `Some(value)` 表示状态已经建立

这比用魔法数字或者额外的布尔标记更清晰，也更不容易把“未初始化状态”偷偷当成正常值继续算下去。

### 2. 调试指标时，优先盯住“第一处失败”

这次 `kvo` 的 bug 不是靠肉眼通读整段公式发现的，而是靠 golden test 给出的第一处偏差位置。

这是一种很实用的调试习惯：

- 先看第一个出错索引
- 再问“这个索引附近，哪些状态刚刚初始化或切换？”
- 优先检查首根样本、窗口边界、趋势翻转、上一个值引用

对于技术指标这类状态驱动逻辑，这通常比从最后一个错误反推更快。

## 验证

实际跑过：

- `cargo fmt --all`
- `cargo test`
- `cargo run --release --bin indicator-bench`
- `make -B smoke`

结果：

- Rust batch 结果对齐现有 golden data
- Rust stream 结果和 batch 保持一致
- benchmark 已自动纳入 `psar`、`ultosc`、`mfi`、`kvo`
- C 侧 smoke 仍然完整通过

## 未覆盖项

- 还没有继续迁剩余的稳定复杂指标
- 还没有做 C / Rust 同指标的并排性能对比
- 这次没有把更大范围的指标共享状态再继续抽象成新基础设施，而是优先完成功能对齐
