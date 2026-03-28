# 用 `mama` 收尾：补齐全部 Beta 指标

## 背景

到这一步时，Rust 版已经把稳定指标全部迁完，beta 也只剩最后一个 `mama`。

`mama` 和前面的 `ce`、`posc`、`rvi`、`smi` 不一样。  
它不是普通的滚动窗口指标，而是一个带有多条中间状态线的自适应平滑器：

- `smooth`
- `detrender`
- `I1` / `Q1`
- `jI` / `jQ`
- `I2` / `Q2`
- `Re` / `Im`
- `period` / `smoothperiod`
- `phase` / `deltaphase`
- `alpha`
- `mama` / `fama`

所以这次提交的真正意义是：Rust 版不再只是“覆盖简单到中等复杂度指标”，而是把 beta 里最后一个复杂状态机也接进来了。

## 主要目标

这次的目标比较集中：

- 把 `mama` 迁进 Rust core
- 让它进入 registry、benchmark、默认测试参数体系
- 用一条合成输入 smoke test 保证它至少在 Rust 侧可运行、可 stream、可 benchmark

做完之后，Rust 对 C 仓库里的 21 个 beta 指标也算是全部对齐完成。

## 改动概览

- 更新 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/beta_smoothers.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/beta_smoothers.rs)，新增 `mama`
- 更新 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/mod.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/mod.rs) 和 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/registry.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/registry.rs)，把 `mama` 接进 overlay 导出和全局注册表
- 更新 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/benchmark.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/benchmark.rs)，为 `fastlimit` / `slowlimit` 增加默认 benchmark 参数
- 更新 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/golden_indicators.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/golden_indicators.rs)，给 `mama` 增加默认 stream 选项和合成序列 smoke test

## 设计思路

这次没有把 `mama` 拆成很多小 trait 或一堆通用抽象，而是反过来做了一件更务实的事：

- 保留它“多条状态线并行推进”的本质
- 但把 C 的宏式 ring buffer 操作收敛成一个小型 Rust `SmallHistory`

这样做的好处是：

- 迁移时更容易对着 C 逻辑逐步核对
- 代码仍然是 Rust 风格的显式状态，而不是宏和指针杂糅
- 以后如果要优化性能，还能把 `SmallHistory` 换成更紧凑的固定数组实现，不必重写整套指标逻辑

## 对新手有用的 Rust 知识

### 1. 给“当前值/前 N 个值”包一层小 API，能让状态机代码清楚很多

这次最有代表性的写法是 `SmallHistory`：

```rust
fn current(&self) -> Real
fn prev(&self, steps: usize) -> Real
```

这样在写公式时，你看到的是：

```rust
self.price.current()
self.price.prev(1)
self.smooth.prev(6)
```

而不是自己手算索引位置。  
对新手来说，这是一种很重要的设计习惯：

- 不要把“索引计算”散落在公式里
- 把它包成语义明确的小方法

这样你读公式时，注意力就会留在指标逻辑本身，而不是数组下标。

### 2. Rust 里“先写一个足够小的专用结构”通常比“过度泛化”更稳

`mama` 的状态很多，如果一上来就想做一个超级通用的 ring buffer 框架，往往会把迁移节奏拖慢。

这次反而是先写了一个很小的 `SmallHistory`：

- 只做固定容量
- 只支持 `push`
- 只支持 `current` / `prev`

这就是一个很典型的 Rust 工程技巧：

- 先写满足当前问题的最小专用工具
- 等出现第二、第三个相似需求时，再考虑抽象升级

对新手来说，这比一开始就追求“漂亮大抽象”更实用。

## 这次一个很重要的协作经验

`mama` 没有现成 golden 数据。  
这意味着如果你只把代码写进去，不把 benchmark 和默认测试入口补上，它就会变成一种“看起来已经迁了，其实没人真正跑过”的假完成状态。

所以这次除了实现本身，还补了两层护栏：

- 默认参数，让 benchmark 能自动跑到 `mama`
- 合成序列 smoke test，让 `batch` / `stream` 至少在 Rust 内部一致

这对人和 AI 协作很重要：  
不是“代码存在”就算完成，而是“代码已经进入自动验证链路”才算完成。

## 验证

实际跑过：

- `cargo fmt --all`
- `cargo test`
- `cargo run --release --bin indicator-bench`
- `make -B smoke`

结果：

- Rust golden tests 和 `mama` 合成 smoke test 都通过
- benchmark 已包含 `mama`
- C 侧 smoke 继续通过，beta 导出 warning 维持旧行为

## 未覆盖项

- `mama` 目前没有来自旧仓库的正式 golden fixture，现阶段依赖的是 Rust 侧 smoke test 和 benchmark 覆盖
- 这次没有改 C 侧 beta 导出策略，所以 legacy C smoke 仍不会真正注册 `mama`
- `SmallHistory` 目前是偏可读性的实现，不是为极限性能专门手写的固定数组 ring buffer
