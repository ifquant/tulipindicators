# 从 `ht_dcperiod` 汇编里把 `smooth_period` 递推改成更接近 C 的 `fmadd` 形状

这次继续优化 `ht_*`，但范围只缩到一个很具体的点：

- 不是继续大拆共享 Hilbert 内核
- 不是继续碰 `ht_phasor`
- 只修 `ht_dcperiod` 里最后那条 `smooth_period` 递推

## 为什么盯上这条递推

在前面的汇编对照里，`ht_dcperiod` 和 `ht_phasor` 共用同一个短周期 Hilbert 内核。  
我先试过：

- 改 `% 3` 回绕写法
- 拆短周期专用 kernel

结果都不稳，已经回退。

但重新对照 `run_ht_short` 的 C 汇编和 Rust 汇编之后，发现有一个局部差异很明确：

### C

`smooth_period` 更新已经更接近 fused 乘加：

```c
smooth_period = 0.33 * period + 0.67 * smooth_period;
```

在机器码里更像一条 `fmadd` 链。

### Rust

Rust 这边原来还是普通表达式：

```rust
smooth_period = 0.33 * period + 0.67 * smooth_period;
```

从 release 汇编看，这一段没有像 C 那样稳定地收成同样紧的 fused 形状。

## 这次怎么改

只把这一条递推改成：

```rust
smooth_period = smooth_period.mul_add(0.67 as Real, (0.33 as Real) * period);
```

也就是显式告诉编译器：

- 把“上一状态 `smooth_period * 0.67`”
- 和“新样本项 `0.33 * period`”
- 收成一条更接近 `fmadd` 的形状

## 为什么要显式写 `as Real`

这里顺手踩到了一个 Rust 细节。

`mul_add` 对字面量类型更敏感。  
如果继续让 `0.67` 和 `0.33` 走默认浮点推断，编译器会报：

- `can't call method mul_add on ambiguous numeric type`

所以这里要显式写成：

```rust
0.67 as Real
0.33 as Real
```

这不是多余，是为了让这一段稳定地落在库里统一的 `Real` 类型上。

## 结果怎么样

focused benchmark：

- `ht_dcperiod 4096 = 1.009x`
- `ht_dcperiod 16384 = 1.345x`

这比前一版大约：

- `4096 ≈ 1.53x`
- `16384 ≈ 1.43x`

更好一些。

要实话实说：

- `4096` 这档的 C 侧波动偏大，所以不能把它吹成“已经稳定反超”
- 但 `16384` 这档改善是更可信的
- 这说明这条局部 `fmadd` 收口是有效的，值得保留

## 这次没有碰什么

这次**没有**继续动：

- `ht_phasor`
- 短周期共享 Hilbert 主链
- `% 3` 的索引回绕逻辑

因为这些方向前面都已经试过，但没有形成稳定收益。

## 顺手记两个知识点

### 1. `mul_add` 不只适合 EMA

前面很多收益来自：

- `ema`
- `rsi`
- `macd`

这次说明另一件事：

- 只要汇编里还能看到“C fused 了，Rust 还没 fused”
- 哪怕它不是标准平滑指标
- 也仍然值得局部试 `mul_add`

### 2. 大热点可以拆成小热点

`ht_*` 这一家族整体很复杂，容易让人觉得只能大重构。

但这次证明：

- 先抓住一个局部热表达式
- 只动那一条
- 也可能拿到稳定收益

这比继续大拆共享内核的风险更低，也更容易验证。
