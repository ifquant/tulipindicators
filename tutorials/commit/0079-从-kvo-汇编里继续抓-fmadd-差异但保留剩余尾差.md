# 从 `KVO` 汇编里继续抓 `fmadd` 差异，但保留剩余尾差

这次继续沿用“先比汇编，再修热循环”的方法，不过 `KVO` 比前几次更复杂，所以这篇教程也专门讲一下：有些优化是有效的，但不一定一步到位。

## 背景

前面我们已经在这些指标上反复验证过：

- `EMA`
- `RSI`
- `MACD`
- `Wilders`
- `ZLEMA`

它们都有一个共同模式：

- C 的递推更新在汇编里是 `fmadd`
- Rust 有些地方还是 `fmul + fadd`

`KVO` 虽然更复杂，但里面也有两条非常像的递推链：

- `short_ema`
- `long_ema`

所以这次先不碰更复杂的 `vf / trend / cm` 状态逻辑，只看这两条 EMA 更新链。

## 先看到的汇编差异

C 版 [`/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/kvo.c`](/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/kvo.c) 在热循环里，`short_ema` 和 `long_ema` 的更新已经是：

```text
fsub
fmadd
fsub
fmadd
```

Rust 版 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/kvo.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/kvo.rs) 对应位置还是：

```text
fsub
fmul
fadd
fsub
fmul
fadd
```

所以这次方向是非常明确的：

- 不改 `vf` 的数学逻辑
- 不改 `trend/cm` 切换逻辑
- 只改两条 EMA 递推链

## 这次怎么修

把：

```rust
(vf - short_ema) * short_per + short_ema
```

改成：

```rust
(vf - short_ema).mul_add(short_per, short_ema)
```

`long_ema` 同理。  
batch 和 stream 两条路径都一起改。

## 结果

focused benchmark：

- 之前大致是：
  - `kvo 4096 ≈ 1.63x`
  - `kvo 65536 ≈ 1.48x`

- 现在变成：
  - `kvo 4096 = 1.181x`
  - `kvo 65536 = 1.003x`

这说明：

- 长输入已经基本追平
- 小输入也明显改善
- 但 `4096` 这档还留着一点尾差

所以这是一笔**有效收益**，但不是“一刀彻底清零”。

## 给 Rust 新手的 2 个知识点

### 1. 复杂指标里也能拆出“可局部优化的子链”

很多新手看到 `KVO` 这种指标，会觉得：

- 这么复杂，必须整体重写才能优化

其实不一定。

更稳的做法通常是：

- 先找出热循环里最像已知模式的部分
- 只修那一小段

这次就是典型例子：

- `vf / trend / cm` 先不动
- 只修 `short_ema / long_ema`

### 2. 有效优化不一定一步到位，但只要方向明确就值得分批提交

这次 `KVO 4096` 还没有完全进最严格阈值，但已经从 `1.63x` 压到 `1.18x`。

这类改动仍然值得提交，因为它满足：

- 方向正确
- 收益明确
- 没引入行为回归
- 剩余边界也说得清楚

这比“非要一次全做完才提交”更适合持续迭代。

## 这次留下的边界

- `KVO 4096` 仍有小尾差
- `vf / trend / cm` 的热循环分支和 `fabs/div` 路径还没有继续深挖
- 如果要再追，下一步就不只是 `mul_add` 了，而要继续看：
  - 分支形状
  - 除法和 `fabs` 的指令成本
  - 状态装配是否还能写薄

## 一句话总结

这次说明了一件很实用的事：

> 对复杂指标，不必一开始就重写整条算法链。先从汇编里找出最像已知热点模式的子链，单独修它，往往就能先拿回一大块收益。
