# 背景

这次继续沿着前几笔性能优化的主线往下走：先跑一轮 focused benchmark，确认 `cvi` 单独跑时仍然稳定慢于 C；再直接对比 C 和 Rust 的汇编热循环，看差异到底在算法、包装层，还是指令形状。

`cvi` 的 C 版本已经不是“结构完全不同”的那类问题了。Rust 这边已经有 direct batch kernel、`run_in_place`，而且环形 lag 的组织方式也和 C 比较接近。所以这次真正值得看的，是 steady-state 递推在机器码上为什么还慢。

# 主要目标

把 `cvi` 的 EMA 风格递推更新写成更接近 C 编译结果的形状，让 Rust 也更容易生成 `fmadd`，而不是继续停留在 `fmul + fadd`。

# 这次做了什么

- 对了 C 的 [`c/indicators/cvi.c`](/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/cvi.c) 和 Rust 的 [`rust/src/indicators/indicator/oscillators.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/oscillators.rs)。
- 观察到 C 的热循环里已经是：
  - `fsub`
  - `fmadd`
  - lag 读取 / 写回
- Rust 的 `run_cvi_batch(...)` 和 `CviStream::feed(...)` 还在用普通表达式：
  - `((range) - val) * per + val`
- 把这两处递推都改成 `mul_add`：
  - `((range) - val).mul_add(per, val)`

# 为什么这次有效

`cvi` 的递推本质上和 `ema` 是同类问题：数学上等价，不代表编译器一定会生成同样的指令。

这次的经验是：
- 如果 C 已经编成 `fmadd`
- Rust 还停在 `fmul + fadd`
- 那么把 Rust 写法改成 `mul_add`，往往能更直接地把“你真正想要的是 fused multiply-add”告诉编译器

这不是为了代码“看起来更高级”，而是为了让热循环的真实机器码更接近 C。

# 验证结果

focused benchmark:

- `cvi 4096 = 0.968x`
- `cvi 65536 = 0.966x`

这说明 `cvi` 已经从之前大约 `1.39x` 的回归，回到略快于 C 的区间。

# 给 Rust 新手的两个知识点

## 1. `mul_add` 不是“语法糖”，它可能影响最终指令

`a * b + c` 和 `a.mul_add(b, c)` 数学上很像，但编译器看到的信息并不完全一样。

在支持 FMA 的平台上，`mul_add` 更容易对应到一条 fused multiply-add 指令。对性能敏感的递推核，这种差异是真实存在的。

## 2. “零成本抽象”不是自动发生的，要看最终汇编

Rust 很强，但“语义等价”不等于“机器码等价”。

这次 `cvi` 的教训是：
- 代码结构已经很像 C
- benchmark 还是慢
- 真正的答案藏在汇编里

所以做性能优化时，不能只盯源码表面，还要回到最终指令层确认热循环到底长成了什么样。
