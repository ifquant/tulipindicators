# 别再用输入长度猜迭代数了：要按真实耗时校准 benchmark

## 这次改动的背景

这一轮我们追性能时，发现了一个很不正常的现象：

- 同一个指标，单独跑时结果正常
- 混在一组指标里跑时，倍率会偶发性暴涨
- 再跑一遍，又可能恢复正常

一开始很容易怀疑是：

- 数据喂错了
- Rust 和 C 用了不同输入
- 某个指标实现有隐藏 bug

但继续核对后发现，这些都不是主因。  
真正的问题更经典，也更隐蔽：**benchmark 自己在用错误的方法决定迭代次数**。

## 根因是什么

旧逻辑是按 `input_len` 估算迭代数，也就是：

```text
iterations ≈ 常数 / input_len
```

这听起来合理，但对指标库其实很不靠谱。因为两个指标就算输入长度一样，单次运行耗时也可能完全不是一个量级：

- `abs` 这种几乎就是一层薄循环
- `msw`、`fisher` 这种会重很多

如果只按输入长度估算：

- 轻指标的单次 sample 可能只有几毫秒，甚至更低
- 这时系统调度、cache、CPU 频率波动，都会把 `ns/input` 放大成假异常

我们后来量出来的现象正好符合这一点：

- 单指标隔离跑更稳定
- 混合跑偶尔会出现整组同方向抖动
- 说明不是某个公式错了，而是测量窗口太短

## 这次做了什么

### 1. Rust benchmark 改成“先校准，再正式测”

现在 Rust 这边不再根据 `input_len` 猜测迭代数，而是：

1. 先做一次或多次 warmup
2. 真的测一下这个指标当前要花多少时间
3. 如果还没达到最小校准时长，就把运行次数翻倍继续测
4. 最后根据真实耗时反推正式 benchmark 的 `iterations`

这样做的结果是：

- `bbands`、`sma`、`md`、`adxr` 这类之前会偶发乱跳的指标，结果明显稳定很多
- benchmark 不再是“拿 1ms 短跑去装精确测量”

相关代码在：

- [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/benchmark.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/benchmark.rs)

### 2. C benchmark contract 也改成同样的校准逻辑

如果只改 Rust，不改 C，比较口径还是不公平。  
所以 C 侧 [`/Users/dev/workspace2/hc_apps/tulipindicators/c/benchmark_contract.c`](/Users/dev/workspace2/hc_apps/tulipindicators/c/benchmark_contract.c) 也做了同样的事情：

- batch benchmark 先校准
- stream benchmark 也先校准
- 再根据真实耗时决定正式测量次数

这一步很重要，因为 compare 工具最终要比较的是两边的 **同类测量**，不是“一边是真实时长校准，一边还是猜的”。

### 3. 新增了校准时长开关

这次还加了一个新的 benchmark 参数：

- `TI_BENCH_CALIBRATION_MS`

它的作用是指定：

- 进入正式 sample 之前，最少要先累计跑多久，才认为“这个指标的单次耗时估计得差不多了”

默认值现在是 `50ms`。  
如果你要做更严肃的性能对比，可以把它调高。

## 这次验证里观察到什么

这次最有价值的观察，不是某个指标具体快了多少，而是 benchmark 行为更像“可信工具”了：

- 在单指标隔离跑里，`bbands` 结果已经明显稳定
- 在小范围混合跑里，绝大多数样本也回到了合理区间

但这次也保留了一个诚实结论：

- run-level 的偶发干扰还没完全消失
- 也就是整轮 benchmark 偶尔还是会遇到系统级噪声

这说明：

- **主要错误口径已经修了**
- 但如果要做最终的严格性能门槛，后面还应该继续做更强的隔离，例如单指标单进程

## 给 Rust 新手的两个知识点

### 知识点 1：性能 benchmark 里，“校准”比“公式算得漂亮”更重要

很多新手第一次写 benchmark，会很自然地觉得：

- 输入大一点就多跑几次
- 输入小一点就少跑几次

但真正可靠的 benchmark，不应该靠输入长度猜，而应该靠**真实跑出来的耗时**来决定后续测量量级。

这是一个很通用的经验：

- 先 warmup
- 再校准
- 最后正式采样

### 知识点 2：`div_ceil` 是表达“向上取整除法”的标准写法

这次 Rust 里有一个很典型的小点：

```rust
target_ns.saturating_mul(runs as u128).div_ceil(elapsed_ns)
```

它表达的是：

- “我要至少跑到目标时间，所以除法结果要向上取整”

以前很多人会手写：

```rust
(a + b - 1) / b
```

这个写法能用，但可读性差，还容易写错。  
现在标准库已经有 `div_ceil`，看到这种需求就优先用它。

## 这次验证怎么做的

- `cargo fmt --all`
- `cargo test`
- `cargo clippy --all-targets --all-features`
- `TI_BENCH_INDICATORS=bbands,adxr,sma,md TI_BENCH_SIZES=4096,65536 TI_BENCH_TARGET_MS=300 TI_BENCH_CALIBRATION_MS=50 TI_BENCH_REPEATS=3 cargo run --release --bin indicator-bench-compare`
- 额外做了多轮 compare 重复实验，确认单指标隔离跑和小范围混合跑的稳定性差异

## 这笔提交的意义

这笔不是“又修了一个指标”，而是把性能工作里的**尺子**先修正了一次。  
如果尺子是歪的，后面所有“优化”和“回归”判断都会被带偏。

下一步如果继续做，我会优先沿着这个方向推进：

- 继续分析整轮运行级别的剩余抖动
- 考虑把 compare 再推进到“单指标单进程”隔离
- 然后再用更可信的 benchmark 结果去继续清理剩下的真实热点
