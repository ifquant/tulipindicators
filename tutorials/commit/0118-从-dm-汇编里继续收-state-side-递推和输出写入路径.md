## 背景

这次新的 focused 榜单里，`avgprice` 和 `dm` 一起冒头：

- `avgprice` 单独复跑后只有 `1.05x` 左右，属于 screen 噪声。
- `dm` 则稳定慢：
  - `4096 ≈ 1.69x`
  - `16384 ≈ 1.56x`

所以这一刀只值得打 `dm`。

## 先看 C 和 Rust 的差异

`dm` 的 C 实现本来就很薄：

- 预热阶段先累加 `plus_dm` / `minus_dm`
- steady-state 里每步只做：
  - `CALC_DIRECTION(dp, dm)`
  - `dmup = dmup * per + dp`
  - `dmdown = dmdown * per + dm`
  - 写两路输出

在 `clang -O2` 的汇编里，C 的热循环已经是两条 `fmadd`：

```asm
fmadd d1, d1, d2, d3
fmadd d0, d0, d2, d4
```

Rust 原来的 `run_dm_batch(...)` 逻辑虽然数学等价，但 steady-state 仍是：

```rust
dmup = dmup * per + dp;
dmdown = dmdown * per + dm;
plus[out_index] = dmup;
minus[out_index] = dmdown;
```

也就是：

- 两条递推都还是普通 `* +`
- 输出写入也还是索引式

这种 case 和前面已经打过的 `ema / pvi / kvo` 很像：不是算法错了，而是热循环还没压成最有利于 codegen 的形状。

## 这次改了什么

只做两件事：

1. 把两条递推改成 state-side `mul_add`

```rust
dmup = dmup.mul_add(per, dp);
dmdown = dmdown.mul_add(per, dm);
```

2. 把 steady-state 输出从 `out_index` 改成 `iter_mut()` 驱动

这样做的目的不是“写法更 Rust”，而是：

- 让两条递推更容易落成 fused multiply-add
- 顺手把输出写入的边界检查路径压薄一点

## 结果

focused benchmark：

- 改前：
  - `dm 4096 = 1.687x`
  - `dm 16384 = 1.564x`
- 改后：
  - `dm 4096 = 1.582x`
  - `dm 16384 = 1.394x`

所以这笔是有效收益，但不是“直接追平 C”的那种大收益。

这很正常，说明：

- `dm` 确实存在能从汇编形状上拿回来的成本
- 但它的差距不只是一条 `fmadd`
- 还夹着方向判断和控制流本身的成本

## 经验

这一类双状态递推指标，可以先用一个简单判断来筛：

- C 汇编已经是 `fmadd`
- Rust 还在 `fmul + fadd`

如果满足，就值得先试 state-side `mul_add`。

但也要保持预期：

- 这类改动经常能拿回一截
- 不保证一定直接回到 parity

所以它更适合作为“先收明显指令差异”的第一刀，而不是默认认为“一改就能赢”。 
