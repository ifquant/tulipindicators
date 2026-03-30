# 补齐 TA-Lib Hilbert Transform 家族，并接进本地 TA-Lib oracle parity 和 benchmark

这次补的是一整批共享内核的 `HT_*` 指标，而不是 6 个彼此独立的小函数：

- `ht_dcperiod`
- `ht_dcphase`
- `ht_phasor`
- `ht_sine`
- `ht_trendline`
- `ht_trendmode`

真正重要的工程判断是：不能把它们拆成 6 份各写一遍。TA-Lib 本地源码里，这一组共享同一套 Hilbert 状态机，只有最后输出投影不同。如果照着“每个函数自己复制一份”去写，后面一旦要修 lookback、预热、sine/phasor 的边界，就会变成 6 处一起改。

## 这次做了什么

### 1. 先抽共享 Hilbert 核心，再挂 6 个 wrapper

Rust 侧新增了 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/ht.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/ht.rs)，把 HT 家族拆成两类共享 batch kernel：

- 短 lookback 核心：`ht_dcperiod`、`ht_phasor`
- 长 lookback 核心：`ht_dcphase`、`ht_sine`、`ht_trendline`、`ht_trendmode`

C 侧对应新增了 [`/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/ht.c`](/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/ht.c)，保持同样的共享结构。

这样做的好处是：

- lookback 规则只定义一处
- WMA 预热和 Hilbert 历史状态只维护一套
- 最后的“输出是什么”单独放在 wrapper 里

这比“复制 TA-Lib 六份源码”更像 Tulip 的风格。

### 2. parity 不只对 Tulip C，还直接对本地 TA-Lib

这次新增了本地 TA-Lib oracle：

- [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/support/talib_missing_oracle.c`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/support/talib_missing_oracle.c)

然后在 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/talib_missing_parity.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/talib_missing_parity.rs) 里补了一条专门的 `ht_family_matches_local_talib_default_semantics()`。

这条测试现在会同时确认：

- Rust 输出和 Tulip C 输出一致
- Rust 输出和本机安装的 TA-Lib 输出一致
- Tulip C 输出和 TA-Lib 输出一致

这比只做 “Rust vs Tulip C” 更强，因为它能避免我们把同一个错误在 C 和 Rust 里一起复制出来。

### 3. smoke fixture 也补齐了

如果只补 parity，不补 C smoke 的 fixture，后面 `make smoke` 还是会一直提示：

- `WARNING: no test for ht_dcperiod`
- `WARNING: no test for ht_dcphase`
- ...

所以这次也把 `ht_*` 的基线补进了：

- [`/Users/dev/workspace2/hc_apps/tulipindicators/c/tests/extra.txt`](/Users/dev/workspace2/hc_apps/tulipindicators/c/tests/extra.txt)

这样 C 侧的老测试体系也能直接覆盖这批新指标。

## 性能结果

这次 focused benchmark 结果是：

- `ht_dcperiod`: `4096 = 1.368x`, `65536 = 1.404x`
- `ht_dcphase`: `4096 = 1.107x`, `65536 = 1.144x`
- `ht_phasor`: `4096 = 1.530x`, `65536 = 1.549x`
- `ht_sine`: `4096 = 0.967x`, `65536 = 1.011x`
- `ht_trendline`: `4096 = 1.473x`, `65536 = 1.561x`
- `ht_trendmode`: `4096 = 1.133x`, `65536 = 1.142x`

结论要讲清楚：

- 功能和语义这次已经对齐 TA-Lib
- benchmark 也已经接进 C/Rust 对比
- 但性能还没有整体追平，尤其 `ht_dcperiod`、`ht_phasor`、`ht_trendline`

这不是这次实现失败，而是下一轮优化的起点。

## 这次顺手补的两个新手知识点

### 1. `std::slice::from_ref(&x)` 是什么

这次 `clippy` 提醒我：

- `&[input.clone()]`

这种写法会无意义地 clone 一次 `Vec`，只是为了凑一个 `&[Vec<_>]`。

更好的写法是：

```rust
std::slice::from_ref(&input)
```

它的意思是：

- 不复制 `input`
- 只临时把“一个值的引用”包装成“长度为 1 的切片引用”

这类小工具在写测试时很常见，也很值得记住。

### 2. 共享内核和多个 wrapper 的关系

像这次 `HT_*` 这种家族，一个很典型的 Rust 设计技巧是：

- 共享状态机放在内部 helper
- 对外暴露多个实现同一 trait 的 wrapper

也就是：

- 核心代码只写一份
- metadata、输出通道数量、最终投影逻辑由各个 wrapper 决定

这比“每个指标抄一份大函数”更不容易漂，也更适合后面继续做性能优化。

## 后续最自然的下一步

本地 TA-Lib 清单里现在真正还没补齐的，只剩：

- `imi`
- `sarext`

如果先走功能补齐，下一步就是这两个。  
如果先走性能，则最值得从 `ht_dcperiod / ht_phasor / ht_trendline` 开始做 focused 汇编对照。
