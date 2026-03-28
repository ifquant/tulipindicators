# 0006 方向性指标家族与派生平滑指标迁移

## 背景

到 `0005` 为止，这个仓库的 Rust 迁移已经覆盖了：

- 常见均线
- `bbands`
- `rsi`
- `macd`
- `dema` / `tema` / `trix`
- `stoch`
- 上述指标对应的主要 stream 路径

但还有一批很重要的稳定指标还留在 C 侧：

- `dm`
- `di`
- `dx`
- `adx`
- `adxr`
- `stochrsi`
- `wilders`

这批指标之所以值得单独拿一笔提交来做，不是因为数量多，而是因为它们代表了三类很关键的实现模式：

1. 方向性趋势指标的共享状态机
2. 在已有指标之上再做派生指标
3. 不同层级的 Wilder 平滑复用

如果这批还不迁，Rust 侧就还缺一块非常核心的“趋势分析和派生振荡器”能力。

## 主要目标

这次提交主要想完成四件事：

1. 把 `dm / di / dx / adx / adxr / stochrsi / wilders` 迁进 Rust registry
2. 给这些指标补上 batch 和 stream 两条路径
3. 把方向性运动、Wilder 平滑、RSI 递推这些共享状态抽出来
4. 让现有 golden test 和 benchmark 自动覆盖这批新指标

这样做以后，后续再迁更多趋势类或派生类指标时，就不需要从零设计状态机了。

## 改动概览

这次新增了 7 个 Rust 指标：

- `wilders`
- `dm`
- `di`
- `dx`
- `adx`
- `adxr`
- `stochrsi`

除了指标本身，更关键的是把共享底层件补出来了：

- `double_input` 校验 helper
- `WildersAverageState`
- `RsiState`
- `DirectionalMovementState`
- `DirectionalIndexState`
- `true_range`
- `directional_movement`
- 通用的单调队列与环形求和容器

这意味着：

- ADX 家族不需要各写一份几乎一样的方向性递推逻辑
- `stochrsi` 不需要重新发明一套 RSI 状态管理
- `wilders` 既能作为独立指标存在，也能作为其它指标的共享平滑模型

## 关键知识

### 1. 为什么 ADX 家族最好一起迁

`dm`、`di`、`dx`、`adx`、`adxr` 不是五个完全独立的算法，它们是一个家族：

- `dm` 先算方向运动量
- `di` 在方向运动量基础上再结合 true range
- `dx` 用正负方向运动的差异构造强度指标
- `adx` 对 `dx` 做 Wilder 平滑
- `adxr` 再把当前 `adx` 和更早的 `adx` 做均值

如果分散迁移，很容易出现：

- 同一套方向运动公式在不同文件里抄了多份
- warm-up 边界不完全一致
- stream 实现和 batch 实现各自长出不同版本

把它们放在同一批里迁移，最大的收益就是可以把公共状态机一次性定稳。

### 2. 为什么 `stochrsi` 不只是“调用 RSI 再调用 STOCH”

从概念上看，`stochrsi` 确实像是：

- 先算 RSI
- 再对 RSI 结果做 stochastic 归一化

但落到实现时，问题会复杂一些：

- RSI 本身有自己的 warm-up
- stochastic 窗口又要在 RSI 输出序列上继续滚动
- stream 模式下，不能每来一个新点就回头重算整段 RSI

所以这次实现里把 RSI 的递推状态抽成了 `RsiState`，让 `stochrsi` 可以直接在增量 RSI 输出上维护自己的 min/max 窗口。

### 3. 为什么 `wilders` 不是可有可无的小指标

很多人在看指标库时会把 `wilders` 当成一个“顺手补一下的均线变体”，但它其实有两个价值：

- 它本身是稳定指标，需要对外提供
- 它也是一类重要平滑方式的最小原型

把 `WildersAverageState` 单独抽出来之后，像 `adx` 这种“先得到一个序列，再做 Wilder 平滑”的逻辑就更容易表达，也更容易保证 batch 和 stream 一致。

### 4. 为什么这批迁移对 benchmark 很重要

这次接入的很多指标都不是简单的一层计算：

- `di` 要维护 true range 和方向运动
- `adx` 要串联方向运动和 Wilder 平滑
- `adxr` 还要额外保留更早的 `adx`
- `stochrsi` 要叠加 RSI 递推和窗口归一化

如果 benchmark 不能自动覆盖它们，后面即使功能对了，也很难知道这些组合型状态机的性能到底如何。

这次迁移完成后，这批指标会自动进入 Rust benchmark 输出，后续优化就有了持续观察面。

## 验证

这次实际运行了：

- `cargo fmt --all`
- `cargo test`
- `cargo run --release --bin indicator-bench`
- `./smoke`

结果上：

- 新增指标全部通过 Rust golden data 对齐
- 新增 stream 路径通过 batch-vs-stream 分块对齐
- benchmark 成功输出这批指标的 batch / stream 性能数据
- 旧 C 测试路径仍然全部通过

## 未覆盖项

这次还没有处理：

- `stddev` / `var` / `stderr` 这类统计窗口指标还没有 Rust 版本
- `kama`、`hma`、`vidya` 等其它复杂平滑类指标还没迁
- benchmark 仍然没有做 C 与 Rust 的直接对比，也没有性能回归阈值

## 对新手的理解建议

可以把这次提交看成“把 Rust 迁移推进到趋势家族和派生指标阶段”。

前几次更多是在建立：

- 批处理骨架
- stream 状态机
- benchmark 基础设施

而这次开始把这些基础件真正用在一整组相互依赖的指标家族上。

如果你以后继续看这个仓库，理解这一批有一个很好的抓手：

- 不要把 `dm`、`di`、`dx`、`adx`、`adxr` 当成五个散点
- 把它们看成“共享方向运动状态，再逐层派生”的一条链

一旦这条链看清楚，后面的很多复杂指标迁移都会更容易读懂。
