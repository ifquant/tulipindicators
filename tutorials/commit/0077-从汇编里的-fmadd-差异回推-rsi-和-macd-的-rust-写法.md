# 从汇编里的 `fmadd` 差异回推 `RSI` 和 `MACD` 的 Rust 写法

这次不是先“猜测哪里慢”，而是先去看 C 和 Rust 的汇编，再倒推应该怎样写 Rust 才更接近 C 的机器码。

## 背景

前面我们已经在 `EMA` 上验证过一条有效方法：

- C 的热循环里有 `fmadd`
- Rust 原来是 `fmul + fadd`
- 把 Rust 的递推更新改成 `mul_add(...)`
- benchmark 和汇编都一起改善

这次继续用同一条方法看 `RSI` 和 `MACD`。

## 先看出了什么汇编差异

### `RSI`

C 版 [`/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/rsi.c`](/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/rsi.c) 的平滑更新本质是：

```c
smooth_up = (upward - smooth_up) * per + smooth_up;
smooth_down = (downward - smooth_down) * per + smooth_down;
```

在汇编里，它会落成两次 `fmadd`。

Rust 版 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/rsi.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/rsi.rs) 原来对应的是普通：

```rust
smooth_up = (upward - smooth_up) * per + smooth_up;
smooth_down = (downward - smooth_down) * per + smooth_down;
```

之前的 codegen 更像：

```text
fsub
fmul
fadd
```

### `MACD`

C 版 [`/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/macd.c`](/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/macd.c) 有三条递推链：

- `short_ema`
- `long_ema`
- `signal_ema`

这三条在汇编里都能看到 `fmadd` 形状。

Rust 版 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/macd.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/macd.rs) 原来写法虽然数学等价，但也更容易落成 `fmul + fadd`。

## 这次怎么修

很直接，不改算法，只改热循环表达式：

### `RSI`

把：

```rust
(upward - smooth_up) * per + smooth_up
```

改成：

```rust
(upward - smooth_up).mul_add(per, smooth_up)
```

`smooth_down` 同理。

### `MACD`

把这三条都改成 `mul_add`：

- `short_ema`
- `long_ema`
- `signal_ema`

也就是把“乘后再加”的表达式都明确写成 fused multiply-add 形状。

## 结果

focused benchmark：

- `macd 4096 = 1.008x`
- `macd 65536 = 0.969x`
- `rsi 4096 = 1.050x`
- `rsi 65536 = 1.023x`

这次的意义非常明确：

- `MACD` 从明显慢于 C 回到 parity，并且长输入已经略快
- `RSI` 也从明显慢于 C 回到 parity 区间

这说明这次抓到的是“真指令级差异”，不是 benchmark 噪声。

## 给 Rust 新手的 2 个知识点

### 1. “数学等价”不等于“机器码等价”

这两种写法数学上是一样的：

```rust
(a - b) * c + b
```

和

```rust
(a - b).mul_add(c, b)
```

但它们不一定生成同样的机器码。

在性能敏感的浮点递推里，第二种更容易让编译器直接生出 `fmadd`。

### 2. 性能优化最有效的证据往往不是源码，而是汇编模式

这次最重要的不是“我感觉 `mul_add` 可能更快”，而是：

- 先看 C 的汇编里哪里出现了 `fmadd`
- 再看 Rust 对应位置为什么没有
- 最后只改那条表达式

这比盲目重写循环、乱加 `unsafe`、或者乱调编译参数更可控。

## 这次留下的边界

- 这次只修了 `RSI` 和 `MACD` 的递推更新式
- 还没有重新逐段检查它们的完整汇编是否每条关键递推都已经压成最理想形状
- `KVO`、`Wilders`、`ZLEMA` 这些还要继续按同样方法做

## 一句话总结

这次再次验证了一条很实用的性能规则：

> 如果 C 的热循环能落成 `fmadd`，Rust 侧就值得优先检查是不是还停留在 `fmul + fadd`，然后用 `mul_add` 去逼近它。
