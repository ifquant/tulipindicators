# 给 Rust 指标库补一层高性能 batch 接口

## 这次为什么不是继续优化单个指标

前面我们已经确认，很多最差回归点并不是“数学算法写坏了”，而是：

- Rust 的统一高层 API 太贵
- 对于非常简单的指标，框架开销比计算本身还重

最典型的是：

- `roc`
- `rocr`
- `mom`
- `abs` / `floor` / `ceil` / `todeg` / `torad`

这些指标在 C 里基本就是一层 `for` 循环。  
如果继续只盯着单个指标做零碎优化，最后会变成：

- 指标改了很多
- 根因没动

所以这次先做的是架构修正：补一层真正面向性能的 batch 接口。

## 这次做了什么

### 1. 在现有 `Indicator` 抽象上新增 `run_in_place`

核心修改在 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/core/indicator.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/core/indicator.rs)。

现在 `Indicator` 不只提供：

- `run(...) -> Result<Vec<Vec<Real>>, _>`

还新增了：

- `run_in_place(inputs, options, outputs) -> Result<usize, _>`

它的意义是：

- 调用者自己提供输出缓冲区
- 指标直接把结果写进去
- 返回本次写了多少有效输出

这更接近 C 的性能模型。

### 2. 先给最差的一批指标接上高性能实现

这次优先接的是：

- [`roc.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/roc.rs)
- [`rocr.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/rocr.rs)
- [`mom.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/mom.rs)
- [`simple/mod.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/simple/mod.rs)

这里做了两件很关键的事：

- `roc` / `rocr` 的 batch 路径不再绕到 stream 实现
- unary / binary simple 指标不再只能走 `map(...).collect()` 那条高层分配路径

### 3. benchmark batch 路径改为优先走高性能接口

[`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/benchmark.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/benchmark.rs) 现在在 batch benchmark 里会：

- 先算 lookback
- 预分配输出缓冲区
- 调用 `run_in_place`

这样 benchmark 测到的就更接近真实 batch kernel，而不是“每次顺手测了一堆 `Vec<Vec<_>>` 分配成本”。

### 4. 补了输出缓冲区校验错误

这次还在 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/core/error.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/core/error.rs) 加了两类错误：

- `WrongOutputCount`
- `OutputTooSmall`

因为一旦允许调用者传入输出缓冲区，就必须把“缓冲区数量不对”和“长度不够”明确建模，不然这层接口就不稳。

## 这次带来的直接效果

这一轮不是只“感觉会更快”，而是已经在 benchmark 上看到明显回收：

- `roc batch 4096`：从 `20x+` 级回归，回落到接近可接受区间
- `rocr batch 4096`：也从 `20x+` 级别掉回接近持平
- `mom batch 4096`：已经基本回到接近持平
- `abs` 这种 simple unary，在小输入下甚至已经能跑赢 C

这说明我们之前的判断是对的：

- 问题主要不是公式
- 而是 API / 分配 / 抽象层的固定开销

## 为什么这次的设计比“再写一个新 trait”更合适

这里我没有再新开一个完全独立的 `FastIndicator` trait。  
原因是如果那样做，很快就会出现两套 registry、两套查找入口、两套实现职责。

这次的做法是：

- 保留现有 `Indicator`
- 在它上面加一个高性能 batch 能力
- 默认实现仍可回退到旧 `run`

这样好处是：

- 兼容现有调用方
- 不需要重做 registry
- 可以按热点指标逐步接入
- benchmark 也能渐进切换

## 新手知识点 1：高性能接口通常意味着“调用者承担更多责任”

高层接口喜欢自己分配、自己返回、自己包装，写起来方便。  
高性能接口通常相反，它会要求调用者：

- 先准备好输出缓冲区
- 保证缓冲区数量正确
- 保证长度足够

这不是“接口变差了”，而是职责重新分配了：

- 易用性下降一点
- 但额外开销和控制权都回到调用者手里

这在数值计算、图形、音视频、数据库内核里都很常见。

## 新手知识点 2：不要让 batch 逻辑默认复用 stream 逻辑

很多新手看到 stream 已经能跑，就会很自然地写：

- batch = new stream + feed whole input

这样当然能复用代码，但性能经常会变差，因为 stream 天然带着：

- 状态对象
- 环形缓冲
- 每次 feed 的边界逻辑
- 更细粒度的输出组织

这次 `roc` / `rocr` 就是很典型的例子。  
所以经验是：

- 如果 batch 是主要吞吐路径
- 就给 batch 单独留一条最直接的实现

复用不是越多越好，复用到把性能模型也绑死就过头了。

## 这次之后的状态

现在 Rust 这边终于开始有了明确分层：

- 高层易用接口：`run`
- 高性能 batch 接口：`run_in_place`
- stream 接口：`create_stream` / `feed`

这还不是终点，但它已经把“性能优化”从零碎修补推进到了一个可持续的架构方向上。
