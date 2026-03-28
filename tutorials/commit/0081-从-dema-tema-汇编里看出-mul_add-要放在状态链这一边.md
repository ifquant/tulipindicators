# 从 `dema/tema` 汇编里看出：`mul_add` 要放在状态链这一边

这次是一个很典型、也很适合新手学的性能案例：**数学上等价，不代表生成的机器码一样好。**

前面我们在 `ema/rsi/macd` 上已经证明，`mul_add` 往往能把 Rust 的热循环从：

```rust
fmul + fadd
```

拉成更接近 C 的：

```text
fmadd
```

所以直觉上会想把 `dema/tema` 也全都照着改一遍。但上一次那样改完，focused benchmark 反而更差。问题不在 `mul_add` 本身，而在于：**它被放在了错误的一边。**

## 背景：为什么 `dema/tema` 和 `ema` 不一样

单级 EMA 的递推式是：

```rust
next = (sample - current) * k + current
```

这种写法天然适合：

```rust
(sample - current).mul_add(k, current)
```

因为新的样本和旧状态只参与一条递推链。

但 `dema/tema` 不一样。它们是多级 EMA 级联：

- `dema`
  - `ema1`
  - `ema2`
- `tema`
  - `ema1`
  - `ema2`
  - `ema3`

每一级的输入既依赖当前新值，也依赖上一层已经更新好的状态。这里的关键路径更像“状态链”，而不是“样本链”。

## 上一次为什么会更差

上一次失败的尝试，本质上是这种形状：

```rust
ema = sample.mul_add(per, ema * per1);
ema2 = ema.mul_add(per, ema2 * per1);
ema3 = ema2.mul_add(per, ema3 * per1);
```

这在数学上没错，但它更像是：

- 先把状态项 `ema * per1` 单独算出来
- 再把 `sample * per + carry` 放进 `fmadd`

而从 C 的汇编看，最佳形状正好相反。

## C 汇编真正告诉了我们什么

### `dema`

C 在 `_ti_dema` 里更像这样：

```text
sample_part = sample * per
ema = fmadd(ema, per1, sample_part)

ema_part = ema * per
ema2 = fmadd(ema2, per1, ema_part)
```

也就是：

- 新样本那边先单独乘出来
- `fmadd` 留给“上一层状态 * per1 + sample_part”

### `tema`

`_ti_tema` 也一样，只是多一层：

```text
sample_part = sample * per
ema1 = fmadd(ema1, per1, sample_part)

ema1_part = ema1 * per
ema2 = fmadd(ema2, per1, ema1_part)

ema2_part = ema2 * per
ema3 = fmadd(ema3, per1, ema2_part)
```

这里最关键的洞察是：

**C 把 `fmadd` 用在“状态更新”上，而不是“样本更新”上。**

这就是为什么我们上一次虽然也写了 `mul_add`，但收益反而更差。因为 fused 的方向不对。

## 这次真正保留下来的 Rust 写法

### `dema`

现在改成：

```rust
let sample_part = *sample * per;
ema = ema.mul_add(per1, sample_part);

let ema_part = ema * per;
ema2 = ema2.mul_add(per1, ema_part);
```

### `tema`

现在改成：

```rust
let sample_part = *sample * per;
ema = ema.mul_add(per1, sample_part);

let ema_part = ema * per;
ema2 = ema2.mul_add(per1, ema_part);

let ema2_part = ema2 * per;
ema3 = ema3.mul_add(per1, ema2_part);
```

注意这里并不是“到处乱加 `mul_add`”，而是严格按 C 的指令形状来放。

## 实际收益

focused benchmark：

- `dema`
  - `4096 = 1.080x`
  - `65536 = 0.951x`
- `tema`
  - `4096 = 1.007x`
  - `65536 = 0.975x`

也就是说：

- 这次 `dema/tema` 都已经回到了 parity 区间
- 和上一次失败的尝试相比，差别不在“有没有 `mul_add`”，而在“`mul_add` 放在哪一边”

## 给 Rust 新手的两个知识点

### 知识点 1：代数等价，不代表性能等价

下面两句数学上等价：

```rust
a * x + b
```

```rust
x.mul_add(a, b)
```

但如果你把 `mul_add` 换到另一边，例如：

```rust
y.mul_add(b, a * x)
```

虽然结果可能仍然等价，编译器看到的依赖链已经变了，寄存器分配和指令调度也会跟着变。

### 知识点 2：性能优化时要先找“关键依赖链”

在 `dema/tema` 这种级联递推里，真正敏感的不是“新样本乘一下”，而是“上一层状态如何被更新并传给下一层”。所以 `fmadd` 要优先留给状态链，而不是样本链。

这是一种很常见的底层优化思路：

- 不只是问“哪里有乘加”
- 而是问“哪一条乘加在关键路径上”

## 这次的核心经验

这次真正有价值的不是 `dema/tema` 本身，而是一个可复用的规则：

1. 先看 C 汇编怎么用 `fmadd`
2. 不要只看“有没有 `fmadd`”
3. 要看 `fmadd` 用在了哪一边
4. 在 Rust 里把 `mul_add` 放到相同的依赖链上

这比“看到乘加就机械替换成 `mul_add`”要可靠得多。
