# 给首批 TA-Lib 缺失指标补上 benchmark 覆盖和 C/Rust 性能对比

这次提交不是继续补功能，而是把上一批新指标的性能验证链补完整。

上一笔里我们先做了语义映射，再补了第一批真缺失指标：

- `linearregangle`
- `midpoint`
- `midprice`
- `rocr100`

但是只做功能正确性还不够。对这种已经同时存在 C 实现和 Rust 实现的指标，还应该补两类检查：

1. 它们是否真的进入了 benchmark 系统，而不是只在 registry 里“存在”。
2. 在 release 编译下，Rust 和 C 的性能差异大概是什么样。

这次就是把这两件事补上。

## 这次做了什么

### 1. 增加 benchmark 覆盖测试

新增测试文件：

- [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/talib_missing_benchmark.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/talib_missing_benchmark.rs)

并在 [`/Users/dev/workspace2/hc_apps/tulipindicators/Cargo.toml`](/Users/dev/workspace2/hc_apps/tulipindicators/Cargo.toml) 里注册成单独的 integration test。

这个测试做了两件事：

- 直接调用 Rust 的 `run_named_benchmarks(...)`
- 构建并调用 C 的 `benchmark_contract`

然后确认这四个指标在两边都真的能跑出 batch benchmark 结果，并且都能得到合法的 `ns/input`。

注意，这个测试**不是**性能门槛测试。它不去断言“Rust 一定快于 C”或者“差异必须小于某个倍数”，因为这种断言很容易被机器负载和短时间波动干扰。它只保证：

- benchmark 覆盖真实存在
- C/Rust 结果都可比较

这属于“先把测量链打通”的工作。

### 2. 实际跑一轮 release compare

除了 `cargo test` 里的覆盖测试，我还实际跑了 release 的对比命令：

```bash
TI_BENCH_INDICATORS=linearregangle,midpoint,midprice,rocr100 \
TI_BENCH_SIZES=4096,65536 \
TI_BENCH_TARGET_MS=120 \
TI_BENCH_CALIBRATION_MS=30 \
TI_BENCH_REPEATS=5 \
cargo run --release -q --bin indicator-bench-compare
```

这轮结果说明：

- `midprice`：Rust 明显快于 C
- `rocr100`：Rust 和 C 基本持平，4096 还略快
- `midpoint`：Rust 略慢，但还在可接受区间附近
- `linearregangle`：Rust 明显慢于 C，是这一批里真正需要后续继续优化的点

按这次结果：

- `linearregangle`
  - `4096 = 1.855x`
  - `65536 = 1.965x`
- `midpoint`
  - `4096 = 1.127x`
  - `65536 = 1.173x`
- `midprice`
  - `4096 = 0.790x`
  - `65536 = 0.377x`
- `rocr100`
  - `4096 = 0.824x`
  - `65536 = 1.003x`

所以这次性能侧最有价值的结论是：

- `midprice` / `rocr100` 不需要现在继续追
- `midpoint` 可以以后再看
- `linearregangle` 是下一批里最值得单独做汇编/内核分析的点

## 为什么要把“覆盖测试”和“真实性能结果”分开

这是做人机协作时很重要的一个习惯。

如果把“能不能 bench”与“bench 出来快不快”混成一个测试，CI 会很容易因为环境抖动而红掉，结果让人和 AI 都不信 benchmark。

更稳的做法是分两层：

1. 覆盖测试
   保证 benchmark 系统真的覆盖到了这个指标。

2. 实际 compare 结果
   作为性能分析结论写进教程、报告或 benchmark 产物。

这样一来：

- 测试负责“有没有测到”
- 基准负责“测出来怎么样”

这两个职责就不会互相污染。

## Rust 新手知识点 1：integration test 不等于 unit test

这次新增的是 `rust/tests/*.rs` 下的 integration test。

它和写在模块里的 `#[cfg(test)]` unit test 有一个很重要的区别：

- unit test 更适合测单个函数、单个模块的内部逻辑
- integration test 更适合从外部把整个 crate 当作一个库来用

这里我们要验证的是：

- Rust registry 能否找到指标
- benchmark API 能否跑起来
- 外部 C benchmark 程序能否对接上

这就很适合 integration test，而不是写成某个 benchmark 模块里的小单测。

## Rust 新手知识点 2：性能测试不应该轻易写死阈值

很多人第一次写 benchmark 测试时，容易直接写：

```rust
assert!(rust_ratio < 1.2);
```

这看起来很直观，但在真实机器上很脆弱，因为：

- CPU 频率会波动
- 同机其它任务会抢资源
- benchmark 自己的 calibration 也有误差

更稳的做法是：

- 在测试里断言“这个 benchmark 链条存在、能跑、值合法”
- 把实际倍率记录到报告或教程里
- 真要加阈值时，再挑那些已经非常稳定、已经观察很多轮的数据点

这比一开始就把所有新指标都绑死在一个倍率阈值上更可靠。

