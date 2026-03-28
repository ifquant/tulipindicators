# 继续按汇编差异收 `atr`，并只保留有效的共享 `EmaState` 优化

这次改动继续沿着“先看汇编，再改 Rust 热循环”的路线推进，但有一个很重要的工程判断：不是所有看起来相似的 `mul_add` 改写都值得留下。我们先同时试了 `atr`、`dema`、`tema`，最后只保留了真正站稳收益的那部分。

## 背景

前几轮已经证明了一件事：很多递推型指标在 C 里会被编译成 `fmadd`，而 Rust 默认写法常常只会生成 `fmul + fadd`。当指标热点主要落在这种递推更新上时，把表达式改写成 `mul_add` 往往能把热循环拉回更接近 C 的机器码。

`atr` 的 steady-state 更新式正好属于这种模式：

```rust
value = (tr - value) * per + value;
```

它和前面的 `ema`、`rsi`、`macd` 很像，本质上都是“当前值朝新样本方向收敛”的递推。

## 这次改了什么

### 1. 共享的 `EmaState::feed()` 改成 `mul_add`

文件：
- `/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/shared.rs`

原来：

```rust
(sample - current) * self.multiplier + current
```

现在：

```rust
(sample - current).mul_add(self.multiplier, current)
```

这不是只影响某一个指标，而是影响所有复用 `EmaState` 的 stream 和级联 EMA 状态机。它的意义在于，把“共享状态层”也往 `fmadd` 方向推，而不是只在单个指标里零散修。

### 2. `atr` 的 batch 和 stream 递推更新改成 `mul_add`

文件：
- `/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/atr.rs`

原来：

```rust
last = (tr - last) * per + last;
value = (tr - value) * per + value;
```

现在：

```rust
last = (tr - last).mul_add(per, last);
value = (tr - value).mul_add(per, value);
```

这一步的目的很直接：让 `atr` 的 steady-state 热循环更像 C 里的 fused multiply-add 递推。

### 3. `dema/tema` 的 `mul_add` 实验被撤回了

我一开始也试了把：
- `/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/dema.rs`
- `/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/tema.rs`

里的 batch EMA 链一起改成 `mul_add`，但 focused benchmark 明确显示它们更差，所以全部回退了。

这点很重要：**“汇编思路正确”不等于“同一写法对所有指标都更快”**。能留下来的优化，必须靠实测结果说话。

## 为什么这次只提交 `atr`

因为 `atr` 的收益是稳定的，而 `dema/tema` 不是。

保留后的 focused benchmark：

- `atr 4096`
  - batch: `0.967x`
  - stream: `1.014x`
- `atr 65536`
  - batch: `0.986x`
  - stream: `0.967x`

这说明：
- `atr` batch 已经从之前的明显回归回到 parity
- `atr` stream 也回到了接近或优于 C 的区间

而 `dema/tema` 在回退后仍然慢：
- `dema 4096 = 1.535x`
- `dema 65536 = 1.585x`
- `tema 4096 = 1.501x`
- `tema 65536 = 1.531x`

所以工程上正确的做法不是“把一组改动一起带走”，而是**只提交经过 benchmark 证明有效的那一部分**。

## 给 Rust 新手的两个知识点

### 知识点 1：`mul_add` 不只是语法糖

Rust 里的：

```rust
a.mul_add(b, c)
```

语义上等价于：

```rust
a * b + c
```

但编译器更容易把它降成硬件的 fused multiply-add 指令。在数值递推热点里，这种写法经常不只是“更优雅”，而是真正会改变机器码。

### 知识点 2：共享状态层的优化会扩散到很多指标

`EmaState` 这种小状态机，如果被很多指标复用，那么你只改一处：

- stream EMA
- DEMA/TEMA 的 stream 链
- MACD/KVO 一类复用 EMA 状态的路径

都可能一起受影响。

这就是为什么性能优化不能只盯单文件。有时候最值钱的点不是具体某个指标文件，而是它背后共享的状态抽象。

## 这次更大的方法论

这次最值得记住的不是 `atr` 本身，而是筛选方式：

1. 先用汇编差异找到可能的模式
2. 再做 focused benchmark
3. 只保留真正收益成立的改动
4. 把“看起来也许合理，但实测更差”的改动全部回掉

这比“感觉不错就一起提交”更慢一点，但长期会让性能历史更可信。
