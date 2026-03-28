# 完成全部稳定 C 指标到 Rust 的首轮对齐

## 背景

前几批迁移已经把均线、趋势、统计窗口和中等复杂度振荡器迁进了 Rust，  
但仓库里还剩最后一批稳定 C 指标没有对齐。

这些剩余指标有一个共同点：它们单个看不一定最复杂，但类型很杂。

- 有价格和成交量派生指标，比如 `ad`、`adosc`、`obv`、`pvi`
- 有线性回归家族，比如 `linreg`、`linregintercept`、`linregslope`、`tsf`、`fosc`
- 有一批振荡器和窗口派生指标，比如 `cci`、`cmo`、`fisher`、`msw`、`qstick`

如果不把这一批系统地收口，Rust 侧虽然已经能跑很多指标，但还不能说“稳定指标已经对齐”。

## 主要目标

这次提交的目标很直接：

- 把剩余的稳定 C 指标全部迁进 Rust
- 让 Rust registry 和 C 的稳定指标集合在数量上对齐到 `104 / 104`
- 保持 batch、stream、golden data 和 benchmark 基础设施继续共用同一套框架

换句话说，这次不是再证明“Rust 可以实现复杂指标”，而是把“还有哪些稳定指标没进 Rust”这个尾巴收掉。

## 改动概览

- 新增价格/成交量家族模块，收口 `ad`、`adosc`、`bop`、`marketfi`、`nvi`、`obv`、`pvi`、`tr`、`vosc`、`wad`
- 新增线性回归家族模块，收口 `linreg`、`linregintercept`、`linregslope`、`tsf`、`fosc`
- 新增振荡器家族模块，收口 `cci`、`cmo`、`cvi`、`fisher`、`md`、`msw`、`qstick`
- 更新 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/registry.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/registry.rs)，把 Rust 稳定指标数量补齐到 `104 / 104`
- 修正线性回归家族的参数解析入口，让错误信息按各自指标名返回，而不是都显示成 `linreg`

## 关键知识

### 1. 批量迁移时，按“指标家族”分模块比按文件名平铺更稳

最后这一批指标的难点不是某一个公式特别难，而是种类很多。

如果继续一个文件塞一个指标，会很快出现两个问题：

- 共用状态结构被复制多份
- review 时不容易看出“这一批到底在补哪一类能力”

这次改成三组：

- `price_volume.rs`
- `regression.rs`
- `oscillators.rs`

这样做的好处是：

- 共用 helper 可以放在同一个模块内，减少复制
- 同类指标能一起校对 lookback、窗口边界和状态推进
- 提交信息和教程也更容易讲清楚

### 2. “全部对齐”不只是 registry 数量一样，还要保持验证链不掉

只是把名字登记进 registry 还不够。  
要真正在工程上完成对齐，至少还要保证这三条链继续工作：

- golden data 对齐
- stream 和 batch 结果一致
- benchmark 能自动发现新指标

这也是为什么这次虽然是“最后一批收尾”，还是继续跑了 `cargo test`、`indicator-bench` 和 `make smoke`。

## 补充知识

### 1. 对新手来说，先找“共享骨架”，再填公式

做指标迁移时，最容易犯的错是直接盯着公式抄。

更高效的顺序通常是：

1. 先找这类指标共享的状态骨架
2. 再确定 lookback 和首个输出位置
3. 最后才填每步更新公式

比如这次：

- 价格/成交量指标共享“上一根 close / 上一根 volume / 累计量”
- 回归家族共享“窗口内 `y_sum` / `xy_sum`”
- 振荡器共享“滚动窗口、极值队列、延迟值”

这样能减少“公式看起来对，但状态推进错了”的问题。

### 2. 调试这类迁移时，先用第一处 mismatch 缩小状态范围

新手经常会看到一串测试失败，然后从头重新推整条公式。  
这通常很慢。

更实用的做法是：

- 先看第一个 mismatch 在哪个索引
- 再看这个索引是否正好落在 lookback 结束、窗口滑动、趋势切换或初始化结束附近
- 只检查这一小段状态转移

技术指标里大量 bug 都不是“公式完全不会”，而是：

- 少减了一次旧值
- 多推进了一次 cursor
- 在错误的 bar 上开始输出

## 验证

实际跑过：

- `cargo fmt --all`
- `cargo test`
- `cargo run --release --bin indicator-bench`
- `make -B smoke`

结果：

- Rust 稳定指标 registry 已补齐到 `104 / 104`
- Rust golden tests 继续通过
- Rust stream 与 batch 一致性测试继续通过
- benchmark 已自动覆盖新增指标
- C 侧 smoke 仍然完整通过

## 未覆盖项

- `beta` 指标仍未迁到 Rust
- 还没有建立 C / Rust 同指标并排性能阈值和回归告警
- 这次没有顺手继续抽更大的跨家族公共状态层，而是优先把稳定指标对齐完成
