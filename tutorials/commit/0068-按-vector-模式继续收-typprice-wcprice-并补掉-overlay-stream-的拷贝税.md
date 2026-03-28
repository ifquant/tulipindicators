# 按 vector 模式继续收 `typprice` / `wcprice`，并补掉 overlay stream 的拷贝税

## 背景

这次不是继续按单个指标瞎打，而是沿着一个更明确的模式往下收：

- 这类指标没有窗口
- 没有复杂状态
- 本质就是按元素做固定组合

典型例子就是：

- `avgprice`
- `medprice`
- `typprice`
- `wcprice`

这类指标真正慢的时候，往往不是数学本身慢，而是：

- 走了过于通用的 helper
- 默认分配输出 `Vec`
- stream 默认走 `feed -> 分配 -> 再 copy`

所以这次的目标很直接：

- 把 `typprice / wcprice` 改得更像 C
- 顺手把 overlay stream 的 `feed_in_place()` 补齐

## 这次主要做了什么

### 1. 给 `typprice / wcprice` 单独写 batch kernel

之前它们还在走通用的 `run_triple_overlay(...)` 和 `run_triple_overlay_in_place(...)`。

这次改成了专门的：

- `run_typprice_batch(...)`
- `run_wcprice_batch(...)`

写法也更贴近 C：

- 固定长度
- 直接按下标写输出
- 不再绕通用三输入 helper 的那层热路径

### 2. 给 overlay stream 补 `feed_in_place()`

`DoubleOverlayStream`
`TripleOverlayStream`
`QuadOverlayStream`

现在都补了 `feed_in_place()`。

这意味着如果调用方愿意自己提供输出 buffer，就不必先：

1. `feed()` 分配一个 `Vec`
2. 默认实现再把这个 `Vec` copy 到调用方的输出切片

这正是很典型的“内存申请 + 内存拷贝税”。

## 结果怎么理解

focused benchmark：

- `typprice 4096 = 1.546x`
- `typprice 65536 = 0.836x`
- `wcprice 4096 = 1.254x`
- `wcprice 65536 = 0.993x`

这组结果说明：

- 长输入下，这条模式收得是有效的
- 小输入 `4096` 还留着固定成本尾差

所以这次的结论要写得诚实：

- 它不是“全线追平”
- 但它已经把 `typprice / wcprice` 从更差的状态拉回了合理区间

## 给 Rust 新手的两个知识点

### 1. `feed_in_place()` 的意义，不只是“少一个函数”，而是少一轮分配和拷贝

如果 trait 只有：

```rust
fn feed(&mut self, inputs: &[&[Real]]) -> Result<Vec<Vec<Real>>, IndicatorError>
```

那调用方就算已经提前准备好了输出 buffer，也没法直接复用。  
它只能先拿到一个 owned `Vec`，然后再拷贝一次。

所以 `feed_in_place()` 真正解决的是：

- 内存申请
- 结果拷贝

而不只是“接口看起来更低级”。

### 2. 对超轻指标，通用 helper 很容易比公式本身更重

像 `typprice` 这种公式：

```rust
(high + low + close) / 3.0
```

本身非常薄。  
这时如果外面包太多通用层，真正被 benchmark 测出来的就不再是公式，而是：

- helper
- 参数校验
- 输出包装

这就是为什么性能敏感库里经常要接受一个现实：

- 不是所有地方都该统一到最漂亮的抽象

## 这次没有包含什么

- `avgprice` 和 `medprice` 这次没有继续改 batch 内核
- `typprice / wcprice` 的 `4096` 档还没有进入新的 `0.8x` 目标
- 这次只是把 overlay 这条 vector 模式继续收窄，没有去碰 simple vector 指标族
