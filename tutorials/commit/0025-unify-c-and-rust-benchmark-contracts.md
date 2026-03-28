# 统一 C 和 Rust 的 benchmark contract，让性能对比有意义

## 这次改动的目标

在这次之前，仓库里其实已经同时有：

- C 侧 benchmark
- Rust 侧 benchmark

但它们并不能直接拿来做 C/Rust 性能对比。  
原因不是“没有 benchmark”，而是“没有统一 contract”：

- 输入数据不一定一致
- option 默认值不一定一致
- 迭代次数策略不一定一致
- 输出格式不一致
- Rust 会 benchmark beta 和额外 stream，C 稳定库却没有对应项

如果这些东西不统一，后面的“性能回归”就是伪精确。  
所以这次的目标不是先做 fancy 报表，而是先把 benchmark 口径统一。

## 这次做了什么

### 1. 给 C 新增一条专门用于对齐的 benchmark 入口

新增了 [`/Users/dev/workspace2/hc_apps/tulipindicators/c/benchmark_contract.c`](/Users/dev/workspace2/hc_apps/tulipindicators/c/benchmark_contract.c)。

它不是去复用旧的历史 `benchmark.c` 逻辑，而是单独做一条“对齐用”的 benchmark 路：

- 使用和 Rust 相同的输入生成公式
- 使用和 Rust 相同的 option 默认值选择规则
- 使用和 Rust 相同的 `sizes / target_ms / min_iterations / stream_chunk`
- 输出和 Rust 相同 schema 的 TSV

这条路径的意义是：

- 旧 benchmark 继续保留它自己的用途
- C/Rust 对比则站到一条更干净、可解释的 contract 上

### 2. 让 Rust benchmark 支持按名称只跑共同子集

在 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/benchmark.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/benchmark.rs) 里补了按 indicator 名称过滤的运行入口。

这样 compare 工具就可以：

- 先读取 C 侧稳定指标集合
- 再让 Rust 只跑这批共同可比的指标

这一步很关键，因为 Rust 现在已经比 C 多了 beta 和更多 stream 实现。  
性能对比时，不能让“Rust 多跑了更多东西”污染结论。

### 3. 新增 C/Rust compare 可执行入口

新增了 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/bin/indicator-bench-compare.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/bin/indicator-bench-compare.rs)。

它会做这几件事：

1. 构建 C 的 `benchmark_contract`
2. 跑出 C 的 TSV
3. 用同一份 config 跑 Rust stable benchmark
4. 只按 C 的键集做对表
5. 输出 compare markdown 和 TSV

现在结果会统一写到：

- [`/Users/dev/workspace2/hc_apps/tulipindicators/target/indicator-bench/c-latest.tsv`](/Users/dev/workspace2/hc_apps/tulipindicators/target/indicator-bench/c-latest.tsv)
- [`/Users/dev/workspace2/hc_apps/tulipindicators/target/indicator-bench/rust-stable-latest.tsv`](/Users/dev/workspace2/hc_apps/tulipindicators/target/indicator-bench/rust-stable-latest.tsv)
- [`/Users/dev/workspace2/hc_apps/tulipindicators/target/indicator-bench/compare-latest.tsv`](/Users/dev/workspace2/hc_apps/tulipindicators/target/indicator-bench/compare-latest.tsv)
- [`/Users/dev/workspace2/hc_apps/tulipindicators/target/indicator-bench/compare-latest.md`](/Users/dev/workspace2/hc_apps/tulipindicators/target/indicator-bench/compare-latest.md)

### 4. 把 benchmark 默认输出目录固定到仓库根

这次还顺手修了一个很现实的问题：

- 从仓库根运行时，benchmark 会写到根 `target/`
- 从 `rust/` 子目录运行时，benchmark 会写到 `rust/target/`

这会让基线文件位置飘来飘去。  
现在默认输出目录已经改成基于 `CARGO_MANIFEST_DIR` 的仓库根绝对路径，所以不管从哪跑，报告都落在同一个地方。

### 5. 更新构建和协作文档

这次还补了：

- [`/Users/dev/workspace2/hc_apps/tulipindicators/c/Makefile`](/Users/dev/workspace2/hc_apps/tulipindicators/c/Makefile) 的 `benchmark_contract` 目标和清理逻辑
- [`/Users/dev/workspace2/hc_apps/tulipindicators/Cargo.toml`](/Users/dev/workspace2/hc_apps/tulipindicators/Cargo.toml) 的 compare bin 入口
- [`/Users/dev/workspace2/hc_apps/tulipindicators/AGENTS.md`](/Users/dev/workspace2/hc_apps/tulipindicators/AGENTS.md) 的常用命令和性能验证要求

## 现在性能对比已经到了哪一步

现在已经做到：

- C 和 Rust 用同一 benchmark contract
- 两边输出同 schema
- compare 工具能自动对表
- compare 结果能直接保存成产物
- 可以开始定义“回归阈值”

也就是说，`性能对比` 这件事现在已经从“手工看两个不同脚本的输出”变成了一个正式工具链。

## 新手知识点 1：benchmark 最先要统一的是“workload contract”，不是画图

很多人做性能工作时，第一反应是：

- 先做一个漂亮表格
- 先出一个 dashboard

但真正最重要的是先回答：

- 两边到底是不是在跑同一个东西？

如果 workload 不统一，再漂亮的图都是错的。  
所以 benchmark 的顺序通常应该是：

1. 统一输入和参数
2. 统一迭代策略
3. 统一输出格式
4. 再做 compare / 回归 / 可视化

这次就是按这个顺序落的。

## 新手知识点 2：对比工具应该以“共同可比键集”为准

这次 compare 工具一开始报错，是因为：

- Rust 有更多 stream 行
- C 没有这些对应实现

如果硬要求两边完全同形，工具就会把“Rust 功能更多”误判成“compare 失败”。

更稳的设计是：

- 先找共同可比的键集
- 再只对这部分做比较

这个思路不只适用于 benchmark，对 API diff、数据对账、回归分析也一样常见。

## 这次之后该怎么用

如果只是看 Rust 自己的 benchmark：

```bash
cargo run --release --bin indicator-bench
```

如果要做 C/Rust 对比：

```bash
cargo run --release --bin indicator-bench-compare
```

如果想缩小 workload、加快本地验证：

```bash
TI_BENCH_SIZES=256 TI_BENCH_TARGET_MS=20 cargo run --release --bin indicator-bench-compare
```

这会更快，但它不是最终基线，只适合本地开发验证。
