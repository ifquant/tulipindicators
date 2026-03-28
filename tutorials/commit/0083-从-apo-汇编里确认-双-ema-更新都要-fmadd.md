# 从 `apo` 汇编里确认：双 EMA 更新都要 `fmadd`

这次的 `apo` 是一个很干净、也很适合作为新手模板的例子。

它的公式本身并不复杂：

```text
APO = short_ema - long_ema
```

所以真正的热循环其实只有三步：

1. 更新 `short_ema`
2. 更新 `long_ema`
3. 两者相减写出

也正因为它够简单，汇编上的差异特别容易看清楚。

## 先看 C 做了什么

文件：
- `/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/apo.c`

C 的核心更新是：

```c
short_ema = (input[i]-short_ema) * short_per + short_ema;
long_ema = (input[i]-long_ema) * long_per + long_ema;
```

在汇编里，这两条更新都已经变成了 `fmadd`：

```text
fmadd short_ema, delta_short, short_per, short_ema
fmadd long_ema,  delta_long,  long_per,  long_ema
```

也就是说，C 对这两个 EMA 递推都已经吃到了 fused multiply-add。

## Rust 原来差在哪里

文件：
- `/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/apo.rs`

原来 batch kernel 还是：

```rust
short_ema = (sample - short_ema) * short_per + short_ema;
long_ema = (sample - long_ema) * long_per + long_ema;
```

虽然数学上完全对，但在这台机器上，Rust 这两条表达式在 batch 路径里还没有稳定落成和 C 一样的 `fmadd` 形状。

## 这次怎么改

直接把两条递推都改成显式 `mul_add`：

```rust
short_ema = (sample - short_ema).mul_add(short_per, short_ema);
long_ema = (sample - long_ema).mul_add(long_per, long_ema);
```

这一步的目标非常单纯：

- 不改算法
- 不改接口
- 不改输出语义
- 只把热循环的乘加形状明确告诉编译器

## 为什么这次收益这么稳定

focused benchmark：

- 改之前
  - `apo 4096 = 1.165x`
  - `apo 65536 = 1.407x`
- 改之后
  - `apo 4096 = 0.970x`
  - `apo 65536 = 0.943x`

这说明：

- `4096` 从回归回到 parity
- `65536` 直接从明显慢于 C 回到略快于 C

而且 `apo` 的路径很简单，所以这笔收益几乎可以直接解释为：

**把两条 EMA 更新都改成更接近 C 的 fused multiply-add 形状。**

## 给 Rust 新手的两个知识点

### 知识点 1：同类递推要一起看，不要只修一条

`apo` 里有两条 EMA：

- `short_ema`
- `long_ema`

如果只修一条，另一条还是原来的 `fmul + fadd`，整体收益就会被吃掉一半。  
这类“成对、成组的递推链”在性能分析时要一起看。

### 知识点 2：指标很简单时，codegen 差异更容易显现

`apo` 没有复杂窗口，也没有很多分支，它几乎就是两条递推更新加一次减法。  
越是这种简单指标，越能暴露：

- C 已经 fused 了
- Rust 还没 fused

这种差异。

也就是说，简单指标不只是“容易优化”，它们还特别适合作为汇编学习样板。

## 这次的核心经验

如果一个指标本质上就是“几条并列的 EMA 递推链 + 简单组合输出”，那排查顺序应该是：

1. 先看 C 汇编是不是每条链都已经是 `fmadd`
2. 再看 Rust 是否只有一部分 fused，或者一条都没 fused
3. 把每一条递推都显式改成 `mul_add`

`apo` 这次就是最标准的案例：  
**不是只有一条 EMA 需要修，而是两条都要一起修。**
