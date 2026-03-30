## 为什么要改

`psar` 前面已经确认不是简单的 wrapper 问题，而是一个典型的强分支状态机难例。  
这类指标和 `ema`、`rsi` 不一样，不能只靠补一个 `run_in_place()` 或 `mul_add()` 就自然追平 C。

这次真正有效的点，是把 Rust steady-state 里最容易拖慢 codegen 的两部分先收紧：

- `prev2_high / prev2_low` 不再用 `Option<Real>`
- `long / short` 两套 clamp 和 extreme 更新逻辑拆成更扁平的 helper

这样做的目标不是“让代码更抽象”，而是让编译器更容易把热循环压成接近 C 的控制流。

## 改了什么

这次只改了：

- `rust/src/indicators/indicator/psar.rs`

具体变化：

1. `prev2_high / prev2_low` 从 `Option<Real>` 改成固定字段，再加一个 `has_prev2: bool`
2. `sar` 递推改成 `mul_add` 形状：

```rust
self.sar = (self.extreme - self.sar).mul_add(self.accel, self.sar);
```

3. 把 long/short 两套逻辑拆成：
   - `advance_long(...)`
   - `advance_short(...)`

这样 steady-state 主函数里只保留：

- 递推
- 选择 long/short 分支
- reversal 判断
- 滚动更新历史值

## 为什么这次比“大状态机重构”更有效

前一版我试过更激进的状态重构：

- 把初始化阶段和稳态阶段收成一套 phase machine
- 再配 direct batch kernel

那版虽然最后能把 parity 修对，但 benchmark 明显更差。  
原因是它虽然“结构更统一”，但对 `psar` 这种强分支状态机来说，反而让热循环更厚了。

这次保留原来的整体行为形状，只把 steady-state 里最容易影响 codegen 的点收紧，所以风险更小，也更容易站住。

## 结果

我实际跑了：

```bash
cargo fmt --all
cargo test --test stable_parity --test golden_indicators
cargo clippy --all-targets --all-features
TI_BENCH_INDICATORS=psar TI_BENCH_SIZES=4096,65536 TI_BENCH_TARGET_MS=180 TI_BENCH_CALIBRATION_MS=50 TI_BENCH_REPEATS=11 cargo run --release -q --bin indicator-bench-compare
TI_BENCH_INDICATORS=psar TI_BENCH_SIZES=65536 TI_BENCH_TARGET_MS=300 TI_BENCH_CALIBRATION_MS=80 TI_BENCH_REPEATS=15 cargo run --release -q --bin indicator-bench-compare
```

focused 结果：

- 常规 focused:
  - `psar 4096 = 0.990x`
  - `psar 65536 = 1.285x`
- 更重的 65536 复核:
  - `psar 65536 = 0.949x`

这说明两件事：

1. 方向是对的，这次收口至少把 `psar` 从明显回退拉回到了 parity 附近
2. `psar` 的测量对参数比较敏感，所以 branch-heavy 指标最好再用更重的 focused benchmark 复核一次

## 给新手的两个提醒

### 1. `Option<T>` 不一定慢，但在热循环里可能让状态机更难优化

`Option<Real>` 本身不是原罪。  
但像 `psar` 这种每一步都要走 long/short 判断、reversal 判断、历史值 clamp 的状态机里，再把历史值表示成 `Option<Real>`，就很容易把 codegen 变复杂。

### 2. 不是所有性能优化都应该是“大重构”

这次一个很实用的经验是：

- 大重构不一定更快
- 有时更有效的是只把 steady-state 热循环里最影响编译器的几个点收紧

对 branch-heavy 指标尤其如此。先做小而硬的收口，通常比先重写整套状态机更稳。
