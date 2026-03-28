# 从 `kama` 汇编里把两处乘加都改成 `fmadd` 形状

这次是一个非常典型的“不要只修最后一步”的性能案例。

前面看 `kama` 的 C 汇编时，有两个关键观察：

1. `alpha = er * const + const`
2. `kama = kama + sc * (sample - kama)`

这两处在 C 里都已经落成了 `fmadd` 形状。  
而 Rust 原来对应的写法还是普通的：

```rust
let alpha = er * (FAST_PER - SLOW_PER) + SLOW_PER;
let sc = alpha * alpha;
kama += sc * (input[index] - kama);
```

也就是说，Rust 不只是最后一步慢，前面的 `alpha` 组合也还在丢一次 fused multiply-add 的机会。

## 这次改了什么

文件：
- `/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/kama.rs`

### 1. 把 `alpha` 的线性组合改成 `mul_add`

原来：

```rust
let alpha = er * (FAST_PER - SLOW_PER) + SLOW_PER;
```

现在：

```rust
let alpha = er.mul_add(FAST_PER - SLOW_PER, SLOW_PER);
```

这一步的目标是让 Rust 的 codegen 更接近 C 里那条：

```text
fmadd alpha, er, const_delta, const_base
```

### 2. 把 KAMA 的最终递推更新也改成 `mul_add`

原来：

```rust
kama += sc * (input[index] - kama);
```

现在：

```rust
kama = (input[index] - kama).mul_add(sc, kama);
```

这一步和前面的 `ema/rsi/macd` 很像，都是把：

```text
delta * factor + current
```

写成更接近 fused multiply-add 的形式。

### 3. stream 路径也一起对齐

同样的两处改动也补到了 `KamaStream::feed_in_place()`：

- `alpha`
- `next`

这样 batch 和 stream 的热循环就不会出现“一个路径已经修了，另一个路径还停留在旧形状”的分裂。

## 为什么这次收益明显

focused benchmark：

- 改之前
  - `kama 4096 = 1.522x`
  - `kama 65536 = 1.702x`
- 改之后
  - `kama 4096 = 1.075x`
  - `kama 65536 = 1.078x`

也就是说：

- `4096` 这一档改善了大约 `29%`
- `65536` 这一档改善了大约 `37%`

而且这不是只修了一处乘加拿来的收益，而是**两处乘加一起回到了更像 C 的形状**。

## 给 Rust 新手的两个知识点

### 知识点 1：一条公式里可能有不止一个值得 fused 的点

很多人一看到：

```rust
current + factor * delta
```

就会想到 `mul_add`。这没错，但还不够。

如果前面还有：

```rust
er * const_a + const_b
```

那也是乘加，也可能是热点。  
这次 `kama` 的收益就来自两个位置都做了这件事，而不是只修最后一步。

### 知识点 2：性能优化时要从机器码倒推表达式

这次不是先拍脑袋说“这里也许该用 `mul_add`”，而是先看 C 的汇编，确认：

- 它已经有两条 `fmadd`
- Rust 这里只有 `fmul + fadd`

再回到源码里把表达式改成更利于 fused multiply-add 的写法。

这比盲猜更可靠，也更适合做可复用的性能工程。

## 这次的核心经验

如果一个指标的热循环里有多处乘加组合，不要只修最显眼的一处。  
先看 C 的指令形状，再确认：

1. 哪几处都已经是 `fmadd`
2. Rust 哪几处还只是 `fmul + fadd`
3. 然后把这些点一起改成更像 C 的 `mul_add` 形状

`kama` 这次就是一个很干净的例子：  
**真正拉回性能的，不是一条 `mul_add`，而是两条。**
