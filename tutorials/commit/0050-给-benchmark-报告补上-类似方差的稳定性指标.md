# 给 benchmark 报告补上“类似方差”的稳定性指标

这次提交不是去优化某一个指标本身，而是继续修 benchmark 的“观察能力”。

前一轮我们已经让报告输出了 `sample_min / sample_median / sample_max`，这比只看一个 `ratio` 强很多，因为你至少能知道一组样本里有没有明显抖动。但这还不够。原因很简单：当你看到两组数据时，光靠最小值和最大值，你很难判断“这一组到底稳不稳”，更难比较两个指标之间谁的波动更严重。

所以这次我们继续往前走一步：给 C 和 Rust 两边的 benchmark 结果都补上两个稳定性指标：

- `sample_stddev_ms`
- `sample_cv`

这样以后看报告时，不只是知道“快还是慢”，还能知道“这个数字到底稳不稳，能不能信”。

## 这次改了什么

### 1. Rust benchmark 结果补上标准差和变异系数

在 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/benchmark.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/benchmark.rs) 里：

- `BenchmarkResult` 新增了 `sample_stddev_ms`
- `BenchmarkResult` 新增了 `sample_cv`
- 新增 `stddev_duration_ms(samples)` 计算样本标准差
- 新增 `coefficient_of_variation(stddev_ms, median_ms)` 计算变异系数

这里的思路是：

- `stddev` 告诉你样本围绕平均值有多分散
- `cv = stddev / median` 告诉你“相对于本次测量本身的体量，它抖了多少”

`cv` 很适合 benchmark，因为不同指标本来运行时长差很多。  
例如：

- 一个指标中位数是 `300ms`，标准差 `15ms`
- 另一个指标中位数是 `20ms`，标准差也 `15ms`

这两个标准差一样，但稳定性完全不是一回事。前者 `cv = 0.05`，后者 `cv = 0.75`。第二个显然抖得厉害得多。

### 2. C benchmark contract 也输出同样字段

在 [`/Users/dev/workspace2/hc_apps/tulipindicators/c/benchmark_contract.c`](/Users/dev/workspace2/hc_apps/tulipindicators/c/benchmark_contract.c) 里：

- `bench_result` 结构体新增 `sample_stddev_ms`
- `bench_result` 结构体新增 `sample_cv`
- 新增 `stddev_double()` 计算 C 侧样本标准差
- batch 和 stream 路径都统一计算这两个字段
- TSV 输出格式也同步扩展

这样做很重要，因为如果只有 Rust 一边输出稳定性指标，compare 报告就没法做真正对称的对比。

### 3. compare 报告把两边的稳定性一起摊开

在 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/bin/indicator-bench-compare.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/bin/indicator-bench-compare.rs) 里：

- 解析 C TSV 时，新增读取 `sample_stddev_ms` 和 `sample_cv`
- compare 行结构里补上 C/Rust 两边各自的 `stddev/cv`
- Markdown 报告把原来的
  - `min/med/max`
  扩展成
  - `min/med/max/stddev/cv`

所以现在看报告时，你能同时看到：

- 速度比值
- C 这边稳不稳
- Rust 这边稳不稳

这就能帮助我们分辨两种情况：

1. 真的慢
2. 不是慢，是这一组 benchmark 抖得厉害

## 为什么这一步值得单独做

因为 benchmark 最怕的不是“数字难看”，而是“数字看起来很精确，但其实不可信”。

之前我们已经定位到一个经典问题：用 `input_len` 猜 `iterations`，会导致很多 sample 只跑几毫秒。那种情况下，系统调度、CPU 温度、cache 状态、后台进程，都会把结果搅得很厉害。

现在虽然已经改成了按真实耗时校准迭代数，但我们仍然需要一个办法，让报告自己暴露“这一组到底稳不稳”。标准差和变异系数就是为这个目的服务的。

## 这次跑了什么验证

- `cargo fmt --all`
- `cargo test`
- `cargo clippy --all-targets --all-features`
- `TI_BENCH_INDICATORS=bbands,adxr,sma,md TI_BENCH_SIZES=4096,65536 TI_BENCH_TARGET_MS=300 TI_BENCH_CALIBRATION_MS=50 cargo run --release --bin indicator-bench-compare`

这次 focused benchmark 的输出已经能直接看到类似下面的信息：

- `adxr batch 4096`
  - C: `293.588/320.952/348.799/20.321/0.063`
  - Rust: `140.417/154.011/193.485/20.286/0.132`
- `bbands batch 4096`
  - C: `232.068/255.642/302.224/24.123/0.094`
  - Rust: `236.647/267.571/385.187/61.996/0.232`

这里你就能一眼看出：`bbands` 这一组里 Rust 的 `cv` 明显更高，说明它虽然倍率还在阈值内，但稳定性比 C 差，后续需要继续关注。

## 给 Rust 新手的两个知识点

### 1. 标准差不等于“稳定性已经可比”

很多新手会以为：有了 `stddev` 就够了。其实不够。

标准差是绝对量。它回答的是：

- “这一组样本平均会偏离均值多少毫秒”

但 benchmark 经常要比较“不同量级”的东西。  
这时候你需要一个归一化指标，也就是 `cv`。

`cv` 回答的是：

- “相对于这次测量本身的体量，抖动占了多大比例”

所以 `cv` 通常更适合做 benchmark 稳定性排序。

### 2. 先让观测系统诚实，再谈优化

性能优化里最常见的误区之一，是看到一个难看的倍率就直接冲去改实现。

这很危险，因为你可能优化的是噪声，不是真问题。

更好的顺序是：

1. 先确认 benchmark 口径合理
2. 再确认样本时长足够
3. 再观察波动范围
4. 最后才决定要不要动实现

这次加 `stddev/cv`，本质上就是在让 benchmark 报告更诚实。  
只有先把“尺子”修好，后面追 `1.2x` 以内这个目标才有意义。

## 还没做什么

这次还没有做这些事：

- 还没有对异常 sample 做自动剔除
- 还没有把 compare 变成“单指标单进程”隔离跑
- 还没有基于 `cv` 自动给结果打“稳定/不稳定”标签

所以这一步更像是：先把 benchmark 的抖动显式量化出来，为下一轮继续诊断打基础。
