# 继续按 `fmadd` 差异修复 `Wilders` 和 `ZLEMA`

这次继续沿用前面 `EMA / RSI / MACD` 已经验证过的方法：

1. 先看 C 汇编
2. 再看 Rust 汇编
3. 找出热循环里哪些地方 C 已经是 `fmadd`，Rust 还停留在 `fmul + fadd`
4. 只改热循环表达式，不先重构别的结构

## 先看到的汇编差异

### `Wilders`

C 版 [`/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/wilders.c`](/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/wilders.c) 的 steady-state 更新是：

```c
val = (input[i] - val) * per + val;
```

在汇编里已经落成了 `fmadd`。

Rust 版 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/wilders.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/wilders.rs) 原来对应的是普通 `* +`，更容易得到：

```text
fsub
fmul
fadd
```

### `ZLEMA`

C 版 [`/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/zlema.c`](/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/zlema.c) 的更新是：

```c
val = ((c + (c-l)) - val) * per + val;
```

同样能看到 `fmadd`。

Rust 版 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/zlema.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/zlema.rs) 原来在 batch 和 stream 里也还是普通 `* +` 形状。

## 这次怎么修

很直接，把这些递推更新改成 `mul_add`。

### `Wilders`

把：

```rust
(sample - value) * per + value
```

改成：

```rust
(sample - value).mul_add(per, value)
```

并且 stream 里的 `feed_sample()` 也同步改掉。

### `ZLEMA`

把：

```rust
((current + (current - lagged)) - value) * per + value
```

改成：

```rust
((current + (current - lagged)) - value).mul_add(per, value)
```

同时把 `lag == 0` 的退化 EMA 路径也一起改成 `mul_add`。

## 结果

focused benchmark：

- `wilders 4096 = 0.927x`
- `wilders 65536 = 0.962x`
- `zlema 4096 = 0.948x`
- `zlema 65536 = 1.096x`

也就是说：

- `Wilders` 两档都已经稳定回到 parity
- `ZLEMA` 两档也都压进了阈值内，长输入还有一点尾差，但已经不是热点

## 给 Rust 新手的 2 个知识点

### 1. 同一个性能模式，通常会在一家指标里重复出现

这次最值得记住的不是 `Wilders` 或 `ZLEMA` 自己，而是：

- `EMA`
- `RSI`
- `MACD`
- `Wilders`
- `ZLEMA`

这些递推型浮点指标，本质上都在重复“差值 -> 乘系数 -> 加回旧值”这个模式。

所以一旦在其中一个指标里看到了 `fmadd` 差异，通常同一类指标都值得检查。

### 2. 热循环里最值得先改的是“表达式形状”，不是先加 `unsafe`

很多人一想到性能，就会先想：

- 能不能全改成原始指针
- 能不能全去掉边界检查
- 能不能先加很多 `unsafe`

这次再次说明，很多时候第一步更该做的是：

- 把热循环写成编译器更容易识别的数值 kernel 形状

也就是先让：

```rust
a * b + c
```

变成：

```rust
a.mul_add(b, c)
```

这种修改往往比乱加 `unsafe` 更稳、更可复用。

## 这次留下的边界

- 这次只处理了 `Wilders` 和 `ZLEMA`
- `KVO` 这类更复杂的多状态指标，还不能直接套这个模板
- 下一步仍然应该继续从汇编指令差异出发，而不是回到“凭感觉改 Rust 写法”

## 一句话总结

这次再次验证了一条很实用的性能方法：

> 对递推型浮点指标，先找 C 已经生成 `fmadd` 的地方，再把 Rust 的热循环表达式改写成更容易触发 fused multiply-add 的形状。
