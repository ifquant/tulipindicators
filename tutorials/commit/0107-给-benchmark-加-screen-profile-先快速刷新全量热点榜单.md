# 给 benchmark 加 `screen` profile，先快速刷新全量热点榜单

这次改的不是某个指标，而是 benchmark 的使用方式。

前面继续补 TA-Lib 指标、继续做 C/Rust 性能对比之后，全量 `indicator-bench-compare` 已经越来越重。问题不是 benchmark 不正确，而是“想先刷新一版热点榜单”时，没必要直接用研究型口径硬跑完整套。

## 主要目标

- 给 benchmark 增加一档专门用来“快速筛热点”的 profile
- 让 C contract 和 Rust benchmark 都能一起吃这档 profile
- 以后先刷新热点榜单时，不必手写一串很短的环境变量

## 改动概览

- Rust 的 `BenchmarkConfig::from_env()` 新增 `TI_BENCH_PROFILE=screen`
- C 的 `benchmark_contract` 也一起识别同一个 profile
- `screen` 档会把默认参数切到更轻的组合：
  - `min_iterations = 4`
  - `target_ms = 20`
  - `calibration_ms = 5`
  - `repeats = 1`
- 更新了 `AGENTS.md`，把“快速全量筛热点”的命令写成固定入口

## 为什么这次值得单独提交

之前的问题是：全量 compare 已经重到不适合频繁刷新，而 focused benchmark 又不能替代全量筛查。  
所以需要两档口径：

- `screen`：先快速看谁慢
- `research`：再重跑复核真热点

这不是“偷懒跑短 benchmark”，而是把“筛查”和“复核”分成两个明确阶段。

## 关键知识

### 1. benchmark 不是越重越好

如果你的目的只是找前 10 热点，过重的 benchmark 会拖慢反馈循环。  
这时候更好的办法是：

- 先用轻口径快速筛
- 再拿 focused benchmark 去复核

### 2. profile 要同时作用在 C 和 Rust 两边

如果只有 Rust 侧切轻口径，而 C contract 还是重口径，那 compare 还是会卡在 C 侧，整体体验不会变好。

## 验证

- `cargo fmt --all`
- `cargo test --test beta_parity --test golden_indicators`
- `cargo clippy --all-targets --all-features`
- `TI_BENCH_PROFILE=screen TI_BENCH_SIZES=4096,65536 cargo run --release -q --bin indicator-bench-compare`

## 未覆盖项

- 这次没有改变 benchmark 的数学口径，只是增加了快速筛查档
- `screen` 适合先筛热点，不适合作为最终性能结论的唯一依据
