## 背景

之前这套 benchmark 每次都会重新做一次动态校准。它的优点是自动，但缺点也明显：

- 每次跑出来的 iterations 不完全一样
- 想做“前后两次严格对比”时，尺子本身会变
- 默认尺寸里还混着 `256`、`65536`、`262144`，更适合探索，不适合把日常比较收口成固定口径

这次我们把思路改成：

- 先用 C contract 为每个指标、每种模式、每个输入规模生成一份固定 iterations 基线
- 后续 compare 默认都读取这份基线
- 默认只测 `4096` 和 `16384`
- 固定基线的目标时长默认是 `4s`，如果要更重，可以调 `TI_BENCH_FIXED_TARGET_MS`

这样做的重点不是“绝对精确”，而是让后面的 benchmark 口径稳定下来。

## 这次改了什么

### 1. Rust benchmark config 增加了固定 iterations 概念

在 `rust/src/benchmark.rs` 里增加了：

- `fixed_iterations_path`
- `fixed_target_duration`
- `fixed_calibration_duration`
- `use_fixed_iterations`

同时加了 `FixedIterationRow`、TSV 的读写函数，以及：

- `BenchmarkConfig::without_fixed_iterations()`
- `BenchmarkConfig::with_fixed_iterations_rows(...)`
- `BenchmarkConfig::fixed_iterations_for(...)`

这样 batch / stream benchmark 在正式跑之前，会先查：

- 这条 `(indicator, mode, input_len)` 有没有锁定好的 iterations

如果有，就直接用，不再重新校准；如果没有，才回退到旧的动态校准逻辑。

### 2. compare 增加了“只生成固定基线”的入口

在 `rust/src/bin/indicator-bench-compare.rs` 里加了：

- `prepare_fixed_iterations(...)`
- `build_fixed_iteration_rows(...)`

并支持：

- `TI_BENCH_REFRESH_FIXED_ITERATIONS=1`
- `TI_BENCH_GENERATE_FIXED_ITERATIONS_ONLY=1`

这两个环境变量配合起来，可以只做一件事：

- 跑 C contract 的 calibration
- 生成 `benchmarks/fixed-iterations.tsv`
- 然后退出

这样就不会因为“想更新基线”而顺手把一整轮 full compare 也跑完。

### 3. 固定基线现在只用 C 生成

这次我们没有再把 C 和 Rust 的 iterations 混在一起算。

基线文件现在只有四列：

- `indicator`
- `mode`
- `input_len`
- `iterations`

它的语义很直接：

- 这份固定 iterations 只由 C contract 生成
- Rust 和 C 后续 compare 都共享这同一份 iterations 文件

这样做的好处是：

- 基线来源更单纯
- 规则更容易理解
- 以后看 benchmark 历史时，不会再问“这份固定 iterations 到底是按哪边算出来的”

### 4. C contract 也学会读取同一份固定文件

在 `c/benchmark_contract.c` 里加了：

- fixed iterations TSV 的加载逻辑
- `(indicator, mode, input_len)` 的查找逻辑

如果命中了固定 iterations：

- `calibration_runs = 0`
- `calibration_ms = 0`
- 直接进入正式样本测量

也就是说，C 和 Rust 终于共用同一把固定尺子了。

### 5. 默认 benchmark 尺寸收口到 `4096` 和 `16384`

现在 C 和 Rust 的默认 benchmark 尺寸都改成了：

- `4096`
- `16384`

这是从“探索型 benchmark”往“长期比较型 benchmark”靠的一步。  
`65536` 不是不能测了，而是从默认口径里拿掉了；需要时仍然可以用 `TI_BENCH_SIZES` 显式加回来。

## 为什么默认目标时长改成 `4s`

用户后面补了一条很关键的要求：

- 固定基线的目标时长应该是可调参数
- 默认不要太重，先用 `4s`

这很合理。

如果默认就是 `10s`：

- 单个指标会更稳
- 但 full compare 的总时间会非常长

而 `4s` 的折中更适合日常开发：

- 已经比之前的短样本稳定很多
- 但还没重到让日常回归不可用

如果真要做研究型复核，仍然可以显式拉高：

```bash
TI_BENCH_REFRESH_FIXED_ITERATIONS=1 \
TI_BENCH_FIXED_TARGET_MS=10000 \
TI_BENCH_GENERATE_FIXED_ITERATIONS_ONLY=1 \
cargo run --release --bin indicator-bench-compare
```

## 这次踩到的一个 C 解析坑

给 C contract 接 TSV 文件时，最开始它一直报：

- `failed to load benchmark config`

根因不是大问题，而是一个很典型的文本解析坑：

- header 行没有被正确跳过
- `atoi("input_len")` 变成了 `0`
- 于是整份配置被判成无效

后来把 header 判断改成：

- 用完整 header 字符串的 `strlen(...)` 做前缀比较

这个问题就消掉了。

这里给 Rust 新手一个顺手的提醒：

- 不只是 Rust 会有“输入格式契约”问题
- 这种跨语言 benchmark 基础设施里，最脆弱的地方往往反而是 TSV/CSV 这类小解析逻辑

## 现在怎么用

### 1. 只刷新固定 iterations 基线

```bash
TI_BENCH_REFRESH_FIXED_ITERATIONS=1 \
TI_BENCH_GENERATE_FIXED_ITERATIONS_ONLY=1 \
cargo run --release --bin indicator-bench-compare
```

### 2. 用现成基线跑 compare

```bash
cargo run --release --bin indicator-bench-compare
```

### 3. 临时改固定基线目标时长

```bash
TI_BENCH_REFRESH_FIXED_ITERATIONS=1 \
TI_BENCH_FIXED_TARGET_MS=10000 \
TI_BENCH_GENERATE_FIXED_ITERATIONS_ONLY=1 \
cargo run --release --bin indicator-bench-compare
```

## 这次没有做的事

- 没有把 kernel probe 也改成固定 iterations；这次先收口主 compare 口径
- 没有强制要求所有 benchmark 都必须存在 fixed row；目前缺失时仍允许回退到动态校准
- 没有把 `screen` profile 和固定基线完全合并；它仍然保留快速筛热点的职责
