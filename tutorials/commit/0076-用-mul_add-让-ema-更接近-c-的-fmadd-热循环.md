# 用 `mul_add` 让 `EMA` 更接近 C 的 `fmadd` 热循环

这次改动只做了一件很小但很关键的事：把 Rust `EMA` 内核里的递推更新式改成 `mul_add`。目标不是“代码看起来更高级”，而是让编译器更容易生成和 C 版同样薄的机器指令。

## 背景

前面我们已经确认过：

- `EMA` 的包装层不是主要问题
- 真正的差距在热循环的 codegen 形状
- C 版 `ema` 在 Apple Silicon 上会生成很干净的：

```text
ldr
fsub
fmadd
str
```

而 Rust 之前对应的递推写法：

```rust
value = (sample - value) * multiplier + value;
```

生成的是：

```text
fsub
fmul
fadd
```

也就是说，Rust 之前没有吃到 fused multiply-add。

## 这次改了什么

在 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/ema.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/ema.rs) 里，把 batch 和 stream 两条 `EMA` 热循环都改成：

```rust
value = (sample - value).mul_add(multiplier, value);
```

这样做的目的很直接：

- 不改算法
- 不改公共 API
- 只改热循环表达式
- 让 LLVM 更容易认出这是一个可以 fused 的数值递推

## 结果

focused benchmark：

- `ema batch 4096 = 0.896x`
- `ema batch 65536 = 1.029x`

也就是说，这一版已经稳定回到和 C 持平区间，而且 `4096` 还略快。

更关键的是，重新导出的 Rust 汇编里，`EMA` 热循环已经从之前的：

```text
fsub
fmul
fadd
```

变成了：

```text
fsub
fmadd
str
```

这说明这次优化不是“碰巧 benchmark 好看”，而是机器码形状真的更接近 C 了。

## 给 Rust 新手的 2 个知识点

### 1. `mul_add` 不只是数学库函数，它还是 codegen 提示

很多新手会把 `mul_add` 理解成“更精确一点的乘加函数”。这不完整。

在数值热循环里，`mul_add` 还经常是在告诉编译器：

- 这里我就是想要一个 fused multiply-add 形状
- 如果目标平台支持，尽量直接生出对应指令

所以在性能敏感的浮点递推里，`mul_add` 往往不是“语法花样”，而是 codegen 工具。

### 2. “zero-cost” 不是靠信念，是靠看汇编

Rust 常说 zero-cost abstraction，但这不表示“写什么都自动零成本”。

更准确的理解是：

- 你得把热路径写成编译器容易识别的形状
- 然后用 benchmark 和汇编确认它真的压下去了

这次 `EMA` 就是一个很典型的例子：

- 数学上完全等价
- Rust 写法只改了一点点
- 但最后从 `fmul + fadd` 变成了 `fmadd`

这就是“从汇编反推 Rust 写法”。

## 这次留下的边界

- 这次只特化了 `EMA`
- 还没有把同样的 `mul_add` 思路扩到 `RSI / MACD / Wilders / ZLEMA`
- 它们未必都能像 `EMA` 一样直接受益，还需要逐项验证

## 一句话总结

这次最重要的不是“把 `EMA` 又快了一点”，而是确认了一条以后还能复用的方法：

> 对递推型浮点指标，先看 C 的热循环指令，再把 Rust kernel 写成最容易落成同样机器码的表达式。
