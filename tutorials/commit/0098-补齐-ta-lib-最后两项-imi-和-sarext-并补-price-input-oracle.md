# 0098 补齐 TA-Lib 最后两项 `imi` 和 `sarext`，并补 `price` input oracle

这次把 TA-Lib 语义映射里最后两个真缺失项补齐了：`imi` 和 `sarext`。其中 `imi` 是一个 `open/close` 双输入窗口指标，本身不复杂；真正麻烦的是 `sarext`，因为它不能偷懒映射到现有 `psar`。`sarext` 多了初始方向、反转偏移、长短分离加速参数，以及“空头输出记为负数”的 TA-Lib 语义。为了让 Rust 侧不只是“本地自洽”，还要能和 TA-Lib 本地 oracle 对账，这次顺手把 oracle 对 `TA_Input_Price` 的支持也补上了。

Changes:
- 在 C 和 Rust 两边新增 `imi`，实现为 `open/close` 差值的滚动上涨/下跌和窗口，并接进 registry、build 生成链和 smoke fixture
- 在 C 和 Rust 两边新增 `sarext`，实现 TA-Lib 风格的 8 参数扩展 SAR，包括默认方向推断、长短分离 AF 和空头负号输出
- 扩展本地 TA-Lib oracle 对 `price` 类型输入的支持，让 `sarext` 和其它 price-bundle 指标也能走 parity 和 benchmark 覆盖

Verification:
- `make -C c libindicators.a` (PASS)
- `make -B -C c smoke` (PASS)
- `cargo test --test talib_missing_parity` (PASS)
- `cargo test --test talib_missing_benchmark` (PASS)
- `cargo test` (PASS)
- `cargo clippy --all-targets --all-features` (PASS)
- `TI_BENCH_INDICATORS=imi,sarext TI_BENCH_SIZES=4096,65536 TI_BENCH_TARGET_MS=120 TI_BENCH_CALIBRATION_MS=30 TI_BENCH_REPEATS=5 cargo run --release -q --bin indicator-bench-compare` (PASS)

Not included:
- `sarext` 这次先按 TA-Lib 语义补齐功能和 parity，还没有单独做汇编级性能优化
- 既有的 `dx`、`rsi`、`simple/mod.rs`、`candles.c/h` 工作树改动没有混进这次提交

这次实现里一个对 Rust 新手很有帮助的点，是 `&[f64]` 这类切片只是一段“借来的连续内存视图”，不是拥有数据的新容器。像 `imi` 这种窗口算法，如果能一直在切片上滚动维护求和，而不是反复创建新的 `Vec`，代码既更快，也更接近 C 那种“在现有数组上推进”的性能模型。

另一个值得记住的是：有些 TA-Lib 指标看起来名字和现有指标很像，但语义差一点就不能直接复用。`sarext` 和 `psar` 就是典型例子。做跨库兼容时，先把“名字相近”和“算法相同”分开，比直接开始写代码更重要；否则很容易做出一个功能上差不多、但永远过不了 parity 的实现。
