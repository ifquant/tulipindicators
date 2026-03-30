## 背景

`correl` 是这轮半全量筛查里最像真热点的一个：

- `4096` 慢很多
- `65536` 也慢

这和那种“小输入下固定成本放大”的假热点不一样，更像热循环本体真的还不够贴近 C。

所以这次没有先改 API，也没有先改 benchmark，而是直接看：

- C 的 `ti_correl`
- Rust 的 `run_correl_batch`

## 汇编里看到了什么

先看 C。

在滑窗 steady-state 更新里，C 编译后的机器码很像：

- `fnmul`
- `fmadd`
- `fadd`

比如这类逻辑：

- `sxx += add_x * add_x - sub_x * sub_x`
- `syy += add_y * add_y - sub_y * sub_y`
- `sxy += add_x * add_y - sub_x * sub_y`

C 的编译器会把它压成“负乘 + fused multiply-add”的形状。

Rust 原来的 `run_correl_batch()` 则更接近：

- `fmul`
- `fsub`
- `fadd`

也就是数学上等价，但指令链更长。

类似地，在最终输出前这几步：

- `xdiff = n * sxx - sx * sx`
- `ydiff = n * syy - sy * sy`
- `numer = n * sxy - sx * sy`

C 也已经在用 fused 形状，而 Rust 还是普通乘减。

## 这次怎么改

文件：

- `rust/src/indicators/math/mod.rs`

只改 `run_correl_batch()` 的热循环表达式，不改算法结构。

### 1. 滑窗差分项改成 `mul_add`

把：

- `add_x * add_x - sub_x * sub_x`
- `add_y * add_y - sub_y * sub_y`
- `add_x * add_y - sub_x * sub_y`

改成更接近 C 机器码形状的写法，例如：

```rust
(-sub_x).mul_add(sub_x, add_x * add_x)
```

这样更容易生成：

- `fnmul`
- `fmadd`

而不是单独的：

- `fmul`
- `fsub`

### 2. 最终分子分母也改成 fused 形状

把：

- `n * sxx - sx * sx`
- `n * syy - sy * sy`
- `n * sxy - sx * sy`

也改成 `mul_add` 驱动的形状，比如：

```rust
(-sx).mul_add(sx, n * sxx)
```

## 为什么这次收益明显

`correl` 的实现本来就已经很接近 C：

- 已经有 `run_in_place`
- 没有走 `stream-backed batch`
- 没有明显 wrapper 税

这就意味着：

- 慢点主要就在热循环和 codegen

所以这类指标特别适合用“先看汇编，再调表达式”的方法。

## 结果

focused benchmark：

- `correl 4096 = 1.030x`
- `correl 65536 = 1.021x`

也就是说，它已经从之前明显慢于 C，回到了 parity 区间。

## 新手知识点

### 1. 数学等价，不代表机器码等价

这三种写法在数学上都可能一样：

- `a * b - c * d`
- `(-c).mul_add(d, a * b)`
- `a.mul_add(b, -(c * d))`

但编译器最终生成的机器码未必一样。

对性能敏感代码来说，**写法会影响编译器能不能生成 fused 指令**。

### 2. 不是所有优化都该先改架构

前面很多收益来自：

- 拆 `stream-backed batch`
- 增加 `run_in_place`

但 `correl` 这次证明了另一件事：

- 当路径已经对了，下一步就该从汇编反推表达式形状

也就是说，性能优化不是永远先做“大重构”，有时最值钱的是把热循环写得更像编译器想要的 kernel。
