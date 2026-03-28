# 第一波 Beta 指标迁移到 Rust

## 背景

稳定指标已经全部对齐以后，仓库里剩下的“还没进 Rust”的主要就是 `c/beta/` 这批实验指标。  
它们在 C 侧默认不会进入稳定导出表，所以 `make smoke` 里一直会出现一串 `Couldn't find indicator ...` 的 warning。

这次先不碰最难的 `mama`、`smi`、`rvi` 这类复杂状态指标，而是优先把更适合批量迁移的一批拿下：

- 通道/带状类：`abands`、`dc`、`kc`、`pbands`、`pc`、`vwap`
- 量能/平滑类：`cmf`、`fi`
- 额外补一批相对独立的 beta：`alma`、`ikhts`、`rmta`、`tsi`

## 主要目标

这次提交的目标不是一次吃掉全部 21 个 beta 指标，而是先把最适合复用现有 Rust 骨架的一批接进来：

- 验证 Rust registry 可以开始容纳 beta 指标
- 让 golden test 和 benchmark 自动覆盖这批新名字
- 把一部分 `extra.txt` 中原本会被跳过的 beta case 变成真实执行

迁完之后，Rust registry 从只覆盖稳定指标，扩大到“稳定指标 + 第一波 beta 指标”，总数来到 `116` 个。

## 改动概览

- 新增 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/beta_channels.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/beta_channels.rs)，实现 `abands`、`dc`、`kc`、`pbands`、`pc`、`vwap`
- 新增 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/beta_smoothers.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/beta_smoothers.rs)，实现 `alma`、`ikhts`、`rmta`
- 新增 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/beta_volume.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/beta_volume.rs)，实现 `cmf`、`fi`
- 新增 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/beta_momentum.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/beta_momentum.rs)，实现 `tsi`
- 更新 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/registry.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/registry.rs)，把这 12 个 beta 指标接进 Rust registry
- 更新 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/benchmark.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/benchmark.rs)，给 `offset`、`sigma`、`beta` 这类 beta 参数补合理默认值，避免 benchmark 因无效参数直接失败

## 关键知识

### 1. 迁 beta 指标时，先挑“能复用现有状态骨架”的那批

Beta 指标不等于都很难。  
真正高效的做法不是按文件顺序迁，而是先找哪些能直接复用现有骨架：

- 通道类可以复用单调队列
- VWAP / CMF 可以复用滚动和
- KC / FI / TSI 可以复用 EMA 型状态推进

这样能先把数量和覆盖率拉起来，同时不把复杂状态机和新数学定义混在一起。

### 2. 把 beta 接进 registry 之前，要先想清楚测试和 benchmark 会不会立刻踩雷

这次如果只把 `alma` 登记进 registry，但不调整 benchmark 默认参数，就会马上失败。  
因为 `alma` 的 `offset` 需要在 `0..=1`，而通用默认值原本会给它塞一个无效数字。

这类问题很典型：  
“实现写对了”不代表“工程已经能稳定接纳这个指标”。

## 补充知识

### 1. 对新手来说，功能迁移不只是写公式，还包括让周边工具知道怎么喂数据

很多人会把迁移理解成“把 C 公式抄成 Rust”。  
但这次更关键的工程工作其实有两块：

- registry 要认识这些名字
- benchmark / stream test 要知道给什么参数

如果这两块没补齐，代码虽然存在，但整个工程仍然不能稳定使用它。

### 2. 做批量迁移时，先选“失败后好定位”的小家族

像 `abands`、`cmf`、`fi`、`vwap` 这种指标，一旦出错，通常能很快定位到：

- 窗口起点
- 滚动和
- 前一根价格引用
- 参数校验

这类指标适合作为 beta 批量迁移的第一波。  
因为你先把这些模式跑通，后面再去碰 `mama`、`smi`、`rvi` 这种更复杂状态机时，基础设施已经更稳了。

## 验证

实际跑过：

- `cargo fmt --all`
- `cargo test`
- `cargo run --release --bin indicator-bench`
- `make -B smoke`

结果：

- Rust golden tests 继续通过
- Rust stream 与 batch 一致性测试继续通过
- benchmark 已覆盖这次新增的 beta 指标
- C 侧 smoke 仍然通过，但它仍会按旧行为对未导出的 C beta 指标打印 warning

## 未覆盖项

- `ce`、`copp`、`kst`、`mama`、`pfe`、`posc`、`rmi`、`rvi`、`smi` 还没迁到 Rust
- `alma` 当前只提供 batch 路径，还没有单独补 Rust stream 实现
- 这次没有改 C 的 beta 导出策略，所以 `make smoke` 里关于 beta 指标的 warning 仍然存在
