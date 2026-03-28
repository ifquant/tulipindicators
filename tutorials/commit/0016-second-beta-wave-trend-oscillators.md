# 第二波 Beta 指标：趋势与动量家族

## 背景

第一波 beta 迁移先拿下了通道、量能和平滑类。  
那一批解决的是“先把好迁、能复用现有骨架的 beta 指标接进 Rust”。

接下来最自然的一步，就是把剩下 beta 里那批中等复杂度、但结构仍然比较清晰的趋势/动量指标继续拿下：

- `copp`
- `kst`
- `pfe`
- `rmi`

这一批的共同点是：它们都不是单一窗口平均，而是“多个派生量再叠一层平滑”的模式。

## 主要目标

这次提交的目标是继续削减 beta 缺口，但仍然避免一下子跳到最难的自适应类和复杂状态机。

具体来说，这次要验证三类模式都已经能在 Rust core 里稳定表达：

- ROC/WMA 叠加类
- 多组 ROC + EMA 聚合类
- 带 lookback 的动量平滑类

迁完之后，Rust beta 覆盖继续往前推进，只剩最后 5 个更难的 beta 指标未迁。

## 改动概览

- 新增 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/beta_trend.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/beta_trend.rs)，实现 `copp`、`kst`、`pfe`、`rmi`
- 更新 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/mod.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/mod.rs) 和 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/registry.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/registry.rs)，把这 4 个 beta 接进 registry
- 更新 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/golden_indicators.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/golden_indicators.rs)，为 `kst` 增加默认 stream 参数，避免测试辅助层因为 8 个参数而失效
- 更新 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/benchmark.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/benchmark.rs)，给 `roc1_period` 到 `roc4_period`、`ma1_period` 到 `ma4_period` 补合理默认值，让全量 benchmark 能跑通

## 关键知识

### 1. 复杂指标常常不是“一个大公式”，而是几层小状态叠起来

这一批 4 个指标的实现方式很能说明这一点：

- `copp`：两组 ROC 先合成，再喂进 WMA
- `kst`：四组 ROC 各自 EMA，再做加权和，再做 signal EMA
- `pfe`：先做分段路径长度比值，再 EMA
- `rmi`：先做 lookback 动量，再分别平滑 gains / losses

如果你直接把它们看成“一个超长公式”，就很难维护。  
拆成状态层之后，反而比较清楚。

### 2. 默认参数本身也是基础设施的一部分

这次最典型的问题不是实现公式错，而是 benchmark 默认参数把 `kst` 的四个 ROC period 全喂成了同一个值，直接违反了：

`roc1 < roc2 < roc3 < roc4`

这说明一个很常见但容易忽略的事实：

- registry 扩容
- benchmark 扩容
- 测试辅助默认参数

这些都属于“实现新指标”这件事的一部分，而不是附属小事。

## 补充知识

### 1. 对新手来说，遇到多层指标时，先分清“输入层”“平滑层”“输出层”

可以把这类指标拆成三问：

1. 原始输入是什么
2. 中间状态怎么平滑
3. 最终输出是哪个状态的组合

比如 `kst`：

1. 输入层：四组不同 period 的 ROC
2. 平滑层：四组 EMA
3. 输出层：加权和，再做 signal EMA

这样比盯着源码一行一行追更容易建立模型。

### 2. 当 benchmark 失败时，先怀疑“参数约束”而不是公式

这次 benchmark 的第一次失败，不是实现错，而是参数生成器给 `kst` 生成了非法选项。  
这是一种很实用的排查顺序：

- 先看是不是参数不满足约束
- 再看是不是输出数量和 lookback 有问题
- 最后才回头怀疑数学实现

这样能避免大量无效调试。

## 验证

实际跑过：

- `cargo fmt --all`
- `cargo test`
- `cargo run --release --bin indicator-bench`
- `make -B smoke`

结果：

- Rust golden tests 继续通过
- Rust stream 与 batch 一致性测试继续通过
- benchmark 已覆盖 `copp`、`kst`、`pfe`、`rmi`
- C 侧 smoke 继续通过，beta warning 仍保持旧行为

## 未覆盖项

- 还剩 `ce`、`mama`、`posc`、`rvi`、`smi` 这 5 个 beta 指标未迁到 Rust
- 这次没有改 C 的 beta 导出策略，所以 `make smoke` 仍然只把 beta 当 warning 显示
- `kst` 目前对齐的是现有 C 行为，不是重新设计后的“更合理数学定义”
