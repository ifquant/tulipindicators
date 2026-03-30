# 从 `pvi` 汇编里把 `fmul + fadd` 递推链改成和 C 一致的 `fmadd` 形状

这次修的是 `pvi` 的性能，不是功能。

`pvi` 之前已经是 direct batch kernel，没有再走 `stream-backed batch`。但 focused benchmark 还是稳定慢于 C：

- `4096 = 1.544x`
- `65536 = 2.010x`

这种形状说明问题不在 wrapper，而在热循环本体。

## 先看 C 和 Rust 的汇编差异

C 版 `ti_pvi` 的热循环很薄，关键更新是：

```c
pvi += ((close[i] - close[i-1]) / close[i-1]) * pvi;
```

在这台机器上，C 编译出来是：

- `fsub`
- `fdiv`
- `fmadd`

也就是把：

```text
ratio * pvi + pvi
```

融合成了一条 `fmadd`。

Rust 原来的写法也是数学上等价的：

```rust
pvi += ratio * pvi;
```

但汇编还是：

- `fsub`
- `fdiv`
- `fmul`
- `fadd`

多了一条指令链。

## 这次怎么改

把 Rust 的递推更新显式改成：

```rust
let ratio = ...;
pvi = ratio.mul_add(pvi, pvi);
```

同样的改法也补到了 stream 里的 `PviStream::feed()`。

这样做的目的不是“代码更优雅”，而是明确告诉编译器：

- 我想要的就是 fused multiply-add 形状

## 为什么这招在这里有效

因为 `pvi` 的循环本身非常薄：

- 一个 volume 比较
- 一个 price ratio
- 一个累计更新

这种指标里，只要 C 已经吃到 `fmadd`，Rust 还停在 `fmul + fadd`，差距就会比较稳定地显出来。

## 结果

focused benchmark 结果：

- `pvi 4096 = 1.073x`
- `pvi 65536 = 1.129x`

也就是两档都回到了阈值内。

## 这次得到的经验

### 1. 不是所有慢点都先查 wrapper

`pvi` 这次说明：

- batch 路径已经是 direct kernel
- 但仍然可能因为热循环少一条 fused 指令而稳定慢于 C

### 2. `mul_add` 最适合这种“递推值乘再加自己”的链

像：

```text
state = ratio * state + state
```

这类表达式是最典型的 `fmadd` 候选。

### 3. 先看汇编，再决定改法

如果不先看汇编，很容易误判成：

- 还要继续拆 wrapper
- 或还要继续改 batch 路径

但这次真正的问题，已经只剩 codegen 形状了。
