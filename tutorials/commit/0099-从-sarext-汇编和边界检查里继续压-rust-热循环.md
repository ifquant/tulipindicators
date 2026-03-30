# 0099 从 `sarext` 汇编和边界检查里继续压 Rust 热循环

这次不是改 `sarext` 的算法语义，而是继续按“先看机器码，再改 Rust 热循环”的方式压性能。前一版 `sarext` 的 focused benchmark 里，Rust 相对 C 还在 `1.28x ~ 1.49x`。把 Rust 版热循环摊开之后，能看到两个明显差异：一是递推更新还是普通 `fmul + fadd`；二是每次写输出都带索引边界检查。C 没有这两层税，所以这次就只针对这两点下刀。

Changes:
- 把 `sarext` 的四处 `sar += af * (ep - sar)` 递推更新改成 `mul_add`，让编译器更容易生成接近 C 的 fused multiply-add 热循环
- 把 `output[out_index]` 改成 `output.iter_mut()` 驱动的逐槽写出，去掉 steady-state 循环里的重复边界检查
- 保持 `sarext` 的 TA-Lib 语义不变，包括默认方向推断、反转偏移和空头输出为负值

Verification:
- `cargo test --test talib_missing_parity` (PASS)
- `cargo clippy --all-targets --all-features` (PASS)
- `TI_BENCH_INDICATORS=sarext TI_BENCH_SIZES=4096,65536 TI_BENCH_TARGET_MS=180 TI_BENCH_CALIBRATION_MS=50 TI_BENCH_REPEATS=11 cargo run --release -q --bin indicator-bench-compare` (PASS)

Not included:
- 这次没有补更激进的 `unsafe` 指针版 kernel，仍然保持安全 Rust 写法
- 既有的 `candles.c/h`、`dx.rs`、`rsi.rs`、`simple/mod.rs` 工作树改动没有混进这次提交

这次很适合记一个 Rust 性能细节：`slice[index]` 在热循环里不一定总能被编译器完全消掉边界检查，尤其是循环里分支比较多的时候。把“写第几个输出”改成 `iter_mut()` 驱动，常常能让编译器更容易证明“每次拿到的都是合法元素”，于是机器码更接近 C 那种直接往结果数组推进的形状。

另一个知识点是：`mul_add` 并不是“看到乘加就无脑替换”。它更像是在告诉编译器“这里希望是一条 fused multiply-add 指令”。前面 `ema/rsi/macd` 那批收益已经说明，只有当 C 侧也确实在相同的递推位置生成了 `fmadd` 时，这种改法才通常值得保留。`sarext` 这次就属于这种情况。
