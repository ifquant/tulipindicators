# 让 benchmark 报告自己暴露抖动，而不是只给一个比值

## 这次改动的背景

上一笔我们已经修了一个关键问题：  
benchmark 不再只按 `input_len` 猜迭代数，而是会先按真实耗时校准。

但这还不够。因为就算校准逻辑变对了，如果报告最后只给你一个：

- `ns/input`
- `ratio`

你还是不知道这个结果稳不稳。

比如一个 `1.03x` 的结果，可能真的很稳；  
也可能是：

- 最快一轮很快
- 最慢一轮很慢
- 中位数刚好落在中间

如果看不到 sample 分布，你就很难判断这个结果能不能信。

这次的目标就是把这个“可见性缺口”补上：

- 让 benchmark 报告自己告诉我们，它到底稳不稳

## 这次做了什么

### 1. 默认 `repeats` 从 3 提到 5

这一步很朴素，但很有用。  
以前默认只做 3 次 sample，偶发抖动太容易直接进入中位数。  
现在默认做 5 次，虽然不是终极解法，但已经能明显减少“偶发一次就把整个结果带偏”的概率。

相关默认值改在：

- [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/benchmark.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/benchmark.rs)
- [`/Users/dev/workspace2/hc_apps/tulipindicators/c/benchmark_contract.c`](/Users/dev/workspace2/hc_apps/tulipindicators/c/benchmark_contract.c)

### 2. Rust benchmark 结果里新增了校准信息和 sample 分布

现在 Rust 的 `BenchmarkResult` 不再只有：

- `iterations`
- `elapsed`
- `ns_per_input`

还新增了：

- `calibration_runs`
- `calibration_ms`
- `sample_min`
- `sample_max`

中位数 sample 仍然保留在原来的 `elapsed` 字段里。

这样一个点的完整信息就变成了：

- 为了校准，先跑了多少轮
- 校准阶段用了多久
- 正式 sample 最快 / 中位 / 最慢各是多少

这一步改在：

- [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/benchmark.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/benchmark.rs)

### 3. C benchmark contract 输出也对齐了同样的 schema

如果只有 Rust 报 sample 分布，C 不报，那 compare 层还是没法公平对照。  
所以 C 侧也同步扩了输出格式，现在也会输出：

- `calibration_runs`
- `calibration_ms`
- `sample_min_ms`
- `sample_median_ms`
- `sample_max_ms`

相关改动在：

- [`/Users/dev/workspace2/hc_apps/tulipindicators/c/benchmark_contract.c`](/Users/dev/workspace2/hc_apps/tulipindicators/c/benchmark_contract.c)

### 4. compare 报告现在会直接展示两边的 sample 分布

这一步是最关键的可观测性升级。  
现在 compare 的 Markdown 表里不再只有：

- `c ns/input`
- `rust ns/input`
- `ratio`

还会直接显示：

- `c sample ms (min/med/max)`
- `rust sample ms (min/med/max)`

也就是说，看到一个异常点时，你可以马上判断：

- 是真的持续性慢
- 还是某一边 sample 分布明显散掉了

这一步改在：

- [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/bin/indicator-bench-compare.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/bin/indicator-bench-compare.rs)

## 这次验证里最有价值的收获

这次的重点不是“某个指标又快了多少”，而是 benchmark 终于开始说人话了。

比如现在一条记录会像这样：

- `c sample ms (min/med/max): 324 / 364 / 487`
- `rust sample ms (min/med/max): 319 / 337 / 426`

你一眼就能看出来：

- 这条记录有没有明显抖动
- 抖动来自哪一边
- 中位数是不是可信

这比之前只给一个 `ratio` 强太多了，因为之前的情况是：

- 结果坏了
- 你还不知道是 benchmark 坏了，还是实现坏了

现在这层判断至少已经能在报告里直接做了。

## 给 Rust 新手的两个知识点

### 知识点 1：统计结果不是只有“平均值”和“中位数”

很多人写 benchmark 报告时，最后只留一个数字。  
但做性能排查时，`min / median / max` 往往比单个中位数更有解释力。

原因很简单：

- `median` 告诉你典型情况
- `min/max` 告诉你抖动范围

如果一个 benchmark 的 `median` 看起来没问题，但 `max` 离谱地高，那就要怀疑：

- 系统噪声
- 资源竞争
- 测量窗口太短

### 知识点 2：数据结构里多存一点“解释性字段”，比事后猜原因强得多

这次我们给 `BenchmarkResult` 增加了：

- `calibration_runs`
- `calibration_ms`
- `sample_min`
- `sample_max`

这类字段不是为了“好看”，而是为了让结果可解释。  
这在性能工具、监控工具、日志工具里都很常见：

- 先把解释所需的上下文一起存下来
- 后面分析就不需要反复回放和猜测

## 这次验证怎么做的

- `cargo fmt --all`
- `cargo test`
- `cargo clippy --all-targets --all-features`
- `TI_BENCH_INDICATORS=bbands,adxr,sma,md TI_BENCH_SIZES=4096,65536 TI_BENCH_TARGET_MS=300 TI_BENCH_CALIBRATION_MS=50 cargo run --release --bin indicator-bench-compare`

## 下一步怎么用这套新报告

现在下一轮排查就会更直接：

1. 先看 `ratio`
2. 再看两边 `sample min/med/max`
3. 如果某边 sample 分布很散，先怀疑 benchmark 运行环境
4. 如果 sample 分布很稳，但 ratio 还是高，再怀疑实现本身

也就是说，benchmark 现在终于不只是“给结论”，而是开始“给证据”了。  
这会让后面的热点优化更像工程判断，而不是猜谜。
