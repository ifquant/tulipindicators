# 从 NATR 汇编里确认：收益较小的 `fmadd` 优化更容易被基准噪声误判

这次的重点不是“终于把一个大热点打掉”，而是把一个很容易误判的性能 case 讲清楚。

前一轮里，我把 `natr` 的递推更新改成了 `mul_add`，和 `ema`、`atr` 一样希望它能更接近 C 的 `fmadd` 热循环。但当时和 `ppo` 一起跑 focused benchmark 时，结果是：

- `natr 4096` 明显变差
- `natr 65536` 看起来又还可以

所以当时我把它整笔回退了，没有硬塞进提交历史。

这次重新从汇编和单指标 benchmark 复核后，结论变成：

- `natr` 这笔其实是有效的
- 之前“没站住”的主要原因不是写法错了
- 而是它的收益幅度比 `ema/atr` 更小，更容易被 benchmark 噪声淹掉

## 先看 C 和 Rust 的热循环差别

`natr` 的 steady-state 循环大致是：

```rust
let tr = true_range(...);
value = (tr - value) * per + value;
output = 100.0 * value / close[index];
```

这和 `atr` 很像，但多了一步：

- 每个样本都要再做一次 `100 * value / close[index]`

这点非常关键。

### C 这边的形状

C 的 `natr` 循环里，递推更新本身还是典型的 EMA/Wilder 形状：

- `fsub`
- `fmadd`
- 然后再做归一化除法

也就是说，C 的 fused multiply-add 只优化了其中一小段递推链。

### Rust 这边原始形状

Rust 原来对应位置更像：

- `fsub`
- `fmul`
- `fadd`
- 再做后面的 `fdiv`

所以从指令形状看，理论上确实值得把那一段改成：

```rust
value = (tr - value).mul_add(per, value);
```

## 为什么这次和 `ema/atr` 不一样

对于 `ema` 这种指标，热循环本体几乎就是递推更新本身，所以一旦 `fmul + fadd` 变成 `fmadd`，收益会很直接。

但 `natr` 不一样：

1. 前面要算 `true_range`
2. 中间只有一处递推可以融合
3. 后面还要做一次除法归一化

所以它虽然也能吃到 `fmadd`，但**整体收益占比更小**。

这意味着：

- 同样的写法改动
- 在 `ema` 上会很明显
- 在 `natr` 上可能只拿回一点点

而“一点点”正是最容易被 benchmark 波动盖住的。

## 这次为什么重新判断为有效

我把 `natr` 单独拿出来再跑 focused benchmark，结果变成：

- `natr 4096 = 0.978x`
- `natr 65536 = 0.971x`

也就是：

- 两档都已经不慢于 C
- 并且已经进入 Rust 自己的 self-best 区间

这和上一次“和 `ppo` 一起跑”得到的结论是相反的。

差异最关键的线索在报告里的波动指标上：

- 之前那次 `natr 4096` 的 Rust `cv` 很高
- 这说明那次读数更像被噪声放大了

所以这次真正学到的不是“`mul_add` 总会赢”，而是：

> 有些优化本身是对的，但收益比较小；如果 benchmark 稳定性不够，就会被误判成无效甚至负优化。

## 这次实际保留的代码

最后保留的是：

```rust
value = (tr - value).mul_add(per, value);
```

包括：

- batch `run()`
- batch `run_in_place()`
- stream `feed_in_place()`

也就是把 `natr` 里 ATR 风格的递推更新统一改成了 `mul_add`。

## 这次可以学到的两点

### 1. 性能优化里，“收益大小”决定了你需要多稳的 benchmark

如果一个改动能带来 `20% ~ 30%` 的收益，哪怕 benchmark 有点抖，也通常还能看出来。

但如果收益只有几个点，或者只是把某段循环从“略慢”拉到“略快”，那 benchmark 的：

- repeats
- calibration
- 单指标隔离
- `cv`

都会直接影响结论。

这就是为什么我们这次不能只看第一次跑出来的表。

### 2. 从汇编回推优化时，要先判断“它在总成本里占多大比例”

看到 C 有 `fmadd`、Rust 没有，并不自动等于“这就是主要瓶颈”。

要先问：

- 这条 fused 链是不是热循环的大头？
- 还是只是总循环里的一小段？

`ema` 属于前者，`natr` 更接近后者。

所以：

- `ema` 上 `mul_add` 会很亮眼
- `natr` 上 `mul_add` 会更容易被其他开销和 benchmark 噪声稀释

## 这次留下的结论

- `natr` 的 `mul_add` 方向本身是对的
- 前一次“没站住”主要不是因为写法错误，而是 benchmark 误判
- 做性能工程时，**小收益优化最怕噪声，不怕代码复杂**
