# 用更大工作量和中位数采样提升 Benchmark 稳定性

## 背景

前几轮性能对比里，已经出现了一个很明显的问题：

- 同一个指标，前后两次 benchmark 有时会跳得很厉害
- 有些结果看起来不像真实代码退化，更像短跑测量噪声

这类情况如果不先处理，后面的性能优化就会越来越不可靠。

因为你可能会：

- 优化一个其实并不慢的指标
- 错过一个真正稳定偏慢的热点
- 被单次偶然结果带偏判断

所以这次不是直接继续优化指标，而是先提高 benchmark 基础设施本身的稳定性。

## 主要目标

让 C/Rust 共用的 benchmark contract 更接近“稳态测量”：

- 默认数据量更大一些
- 默认迭代轮数更多一些
- 默认每个 case 重复多次
- 最终取中位数，而不是只看一次单跑结果

这样下一步再分析热点，结论才更可信。

## 改动概览

### 1. Rust benchmark 默认工作量提高了

[`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/benchmark.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/benchmark.rs) 现在把默认参数调高了：

- 默认输入规模增加到 `256, 4096, 65536, 262144`
- 默认最小迭代次数从 `8` 提到 `16`
- 默认目标时长从 `200ms` 提到 `1000ms`
- 新增默认重复次数 `3`

这意味着 benchmark 不再那么偏向“快速跑完”，而是更偏向“测得稳一点”。

### 2. Rust benchmark 改成取中位数

以前 Rust 侧每个 case 基本就是一次计时结果。

现在每个 case 会：

- 按当前 `iterations` 跑完整一轮
- 重复 `repeats` 次
- 收集每次总耗时
- 取中位数作为最终 `elapsed`

中位数比平均值更适合这里，因为它对偶发的慢样本更不敏感。

### 3. C contract benchmark 也同步成相同策略

[`/Users/dev/workspace2/hc_apps/tulipindicators/c/benchmark_contract.c`](/Users/dev/workspace2/hc_apps/tulipindicators/c/benchmark_contract.c) 也同步做了同样的事：

- 默认 `min_iterations` 提高
- 默认 `target_ms` 提高
- 新增 `TI_BENCH_REPEATS`
- 每个 case 采样多次并取中位数

这很关键，因为如果只改 Rust 不改 C，对比口径还是会失衡。

### 4. 把新的稳定性调参入口写进协作规则

[`/Users/dev/workspace2/hc_apps/tulipindicators/AGENTS.md`](/Users/dev/workspace2/hc_apps/tulipindicators/AGENTS.md) 现在也明确写了：

- 如需提高 benchmark 稳定性，优先调大
  - `TI_BENCH_TARGET_MS`
  - `TI_BENCH_MIN_ITERATIONS`
  - `TI_BENCH_REPEATS`

这样以后不只是当前上下文知道，后续 AI 或人也能直接沿用。

## 关键知识

### 为什么这里用“中位数”而不是“平均值”

假设你重复跑 3 次，结果是：

- 100ms
- 101ms
- 160ms

如果取平均值，会被那次异常慢样本明显拉高。

但如果取中位数，结果还是：

- 101ms

这通常更接近“机器这次正常工作的典型速度”。

对 benchmark 来说，这往往比平均值更适合做回归判断。

### 为什么“加大数据量和轮数”能减小噪声

如果一次 benchmark 跑得太短：

- 调度抖动
- cache 状态
- 后台进程
- 一次偶然的系统中断

这些外部因素占比就会很高。

而当输入更多、总时长更长时，真正的内核成本会占更大比例，测量自然更稳定。

这也是为什么短输入常常更容易看起来“忽快忽慢”。

## 补充知识

### Rust / 性能知识点 1

性能分析里，先提高测量质量，往往比先改代码更重要。

如果 benchmark 不稳，后面的“优化”很容易变成：

- 优化随机波动

而不是：

- 优化真实热点

### Rust / 性能知识点 2

“默认值”本身也是性能基础设施的一部分。

如果默认 benchmark 太轻，团队就会反复得到低可信度结果。

把默认值调到更稳的区间，可以减少很多无效讨论和错误优化方向。

## 验证

- `cargo fmt --all` (`PASS`)
- `cargo test` (`PASS`)
- `cargo clippy --all-targets --all-features` (`PASS`)
- `TI_BENCH_SIZES=4096,65536 TI_BENCH_TARGET_MS=300 TI_BENCH_REPEATS=3 cargo run --release --bin indicator-bench-compare` (`PASS`)

## 未覆盖项

- 这次只提高了 benchmark 稳定性，没有直接优化任何具体指标
- 结果虽然更稳了，但仍然可能需要后续再增加 repeats 或做专门热点复测
