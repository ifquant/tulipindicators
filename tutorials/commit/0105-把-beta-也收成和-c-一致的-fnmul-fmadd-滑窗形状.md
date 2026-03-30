## 背景

在把 `correl` 收回到 parity 之后，下一步最自然的对象就是 `beta`。

原因很简单：

- 它和 `correl` 是同一家族
- 都是双输入滑窗统计
- 都维护一组滚动和：
  - `sx`
  - `sy`
  - `sxx`
  - `sxy`

所以如果 `correl` 的慢点来自“Rust 没生成 C 那样的 fused 浮点热循环”，那 `beta` 很可能也有同样问题。

## 汇编差异

这次先看 C 的 `ti_beta`。

在 steady-state 热循环里，C 编译出来的是这种形状：

- `fnmul`
- `fmadd`
- `fadd`

比如这几步：

- `sxx += add_x * add_x - sub_x * sub_x`
- `sxy += add_x * add_y - sub_x * sub_y`
- `denom = n * sxx - sx * sx`
- `numer = n * sxy - sx * sy`

都已经是 fused 形状。

Rust 原来的 `run_beta_batch()` 则更像：

- `fmul`
- `fsub`
- `fadd`

数学等价，但指令链更长。

## 修改

文件：

- `rust/src/indicators/math/mod.rs`

这次只改 `run_beta_batch()`，不动 API，不动测试，不动 benchmark 基础设施。

### 1. 滑窗更新改成 fused 形状

把：

- `add_x * add_x - sub_x * sub_x`
- `add_x * add_y - sub_x * sub_y`

改成更容易生成 `fnmul + fmadd` 的写法，例如：

```rust
(-sub_x).mul_add(sub_x, add_x * add_x)
```

### 2. 分母和分子也改成 fused 形状

把：

- `n * sxx - sx * sx`
- `n * sxy - sx * sy`

改成：

```rust
(-sx).mul_add(sx, n * sxx)
(-sx).mul_add(sy, n * sxy)
```

## 结果

focused benchmark：

- `beta 4096 = 0.878x`
- `beta 65536 = 0.954x`

也就是说，这一笔不仅追平了 C，而且小输入已经明显快于 C。

## 为什么这次能直接复用 `correl` 的思路

因为这两个指标虽然数学定义不同，但热循环的骨架非常像：

- 都维护滚动统计量
- 都有“新增项 - 移除项”的滑窗差分
- 都有最后一步把滚动和组合成输出

所以一旦确认 `correl` 的慢点来自指令形状，`beta` 通常也值得按同样方式查。

## 新手知识点

### 1. 同一家族指标，优化经验经常可以复用

一旦某个优化规律在一个指标上被证明成立，比如：

- `correl` 的 `fmul/fsub/fadd` 能收成 `fnmul/fmadd`

那同家族的 `beta` 就不该从零猜。

更合理的做法是：

- 先看它是不是同样的滑窗结构
- 再直接验证汇编是否也有同样差异

### 2. 性能优化不只是“改算法”，也可以是“改表达式形状”

这次 `beta` 的数学逻辑没有变化，变化的是写法。

这种优化最容易被忽略，因为代码层面看起来“只是换了一种等价表达式”，但对编译器来说，这可能正好决定：

- 是生成三条浮点指令
- 还是生成一条 fused 指令
