# 批量迁移规则型指标

## 这次改动想解决什么问题

前几次迁移已经把 `sma`、`ema`、`macd`、`stoch`、`adx`、`wma` 这类“骨架型”和“复杂型”指标搬到了 Rust 里，但总覆盖率还是不高。

真正拖慢整体对齐进度的，不一定是最复杂的指标，而是大量规则非常清楚、却还没开始迁的基础指标，例如：

- 一元数学变换：`abs`、`sin`、`ln`
- 二元向量运算：`add`、`sub`、`mul`、`div`
- 简单价格合成：`avgprice`、`typprice`
- 窗口基础算子：`sum`、`max`、`min`
- 简单状态机：`lag`、`mom`、`crossany`

如果这些一直不迁，Rust 侧就会长期停留在“有代表性，但不完整”的状态。

## 这次的策略

这次没有继续追一个特别复杂的单点指标，而是换成“批量清扫规则型指标”的策略。

核心思路是：

1. 把可模板化的一元/二元向量指标成组迁掉。
2. 把输入结构固定、公式直接的价格合成指标一并迁掉。
3. 把窗口与状态机类指标迁掉，同时补齐 stream 路径。
4. 调整测试默认参数，让无参数指标的 stream 也真正被测到。

这是一个很适合人机协作的切法，因为它不是追求“最炫的算法”，而是追求“最大化清理剩余面积”。

## 这次迁移了哪些指标

### Simple 类

- `abs`
- `acos`
- `asin`
- `atan`
- `ceil`
- `cos`
- `cosh`
- `exp`
- `floor`
- `ln`
- `log10`
- `round`
- `sin`
- `sinh`
- `sqrt`
- `tan`
- `tanh`
- `todeg`
- `torad`
- `trunc`
- `add`
- `sub`
- `mul`
- `div`

### Overlay 类

- `avgprice`
- `medprice`
- `typprice`
- `wcprice`

### Math / Indicator 类

- `crossany`
- `crossover`
- `decay`
- `edecay`
- `lag`
- `sum`
- `max`
- `min`
- `mom`

## 这次在结构上做了什么

为了不把规则型指标继续塞进旧模块，这次顺手把 Rust 目录结构往更清晰的方向推了一步：

- 新增 `rust/src/indicators/simple/`
- 新增 `rust/src/indicators/math/`
- 在 `overlay/` 里新增价格合成模块
- 在 `indicator/` 里补了 `mom`

这一步很重要，因为它把“迁移复杂指标”和“迁移规则型指标”拆成了不同的代码组织方式，后面继续扩展时不容易乱。

## 为什么要补 stream

这次不是只补 batch 版本，而是大部分新指标都直接带上了 stream 实现。

原因很简单：

- 这些指标的 stream 路径本来就不复杂
- 如果现在不补，后面又要回头再做一轮重复工作
- 现有测试框架已经能验证 stream 和 batch 一致性，应该趁这次一起吃掉

对于 `lag`、`mom`、`crossany`、`sum`、`max`、`min` 这种强状态型指标，stream 其实比 batch 更能暴露状态边界是否正确。

## 这次还修了什么测试问题

之前 `golden_indicators.rs` 在试探 stream 能力时，默认会给未知指标传 `vec![5.0]` 作为 options。

这对有一个周期参数的指标没问题，但对 `abs`、`avgprice`、`crossany` 这种“没有 options”的指标，会让 `create_stream()` 因参数数量不对而直接跳过。

这次把默认参数逻辑改成了按 metadata 的 `option_names` 长度生成，这样：

- 无参数指标也会进入 stream 测试
- 单参数指标继续用 `5.0`
- 多参数指标保留专门默认值

这一步的价值不在于“代码多漂亮”，而在于测试覆盖终于和 registry 的真实能力一致了。

## 结果怎么样

这次提交后，Rust registry 从 `24 / 104` 提升到了 `61 / 104`。

这意味着 Rust 侧已经不只是“有一些代表指标”，而是已经覆盖了很大一批基础能力：

- 复杂均线骨架
- 趋势方向家族
- 统计窗口
- 一元/二元向量运算
- 价格合成
- 基础窗口和交叉判断

也就是说，后面剩下的工作，更多会集中在中复杂指标和少数特殊实现，而不是海量简单空白。

## 这次验证了什么

实际跑过：

- `cargo fmt --all`
- `cargo test`
- `cargo run --release --bin indicator-bench`
- `make -B smoke`

这几项一起说明：

- 新增指标能通过现有 golden data
- 新增 stream 路径和 batch 路径一致
- benchmark 会自动把新注册指标纳入性能报表
- 现有 C 测试链没有被 Rust 侧改动破坏

## 这次没有做什么

- 没有继续迁 `apo`、`ppo`、`natr`、`willr`、`vwma`、`trima`、`zlema` 这类下一批中等复杂指标
- 没有补 C/Rust 对照 benchmark
- 没有引入自动统计“已迁移比例”的脚本
- 没有处理 C 构建链里现存的老 warning

## 新手应该怎么理解这次提交

如果你是第一次看这个仓库，可以把这次提交理解成：

- 前几次是在搭 Rust 指标库的骨架
- 这一次是在大面积填基础砖块

它不一定是最难的一次提交，但它非常关键，因为它把 Rust 迁移从“示范性质”推进到了“开始形成规模”。  
