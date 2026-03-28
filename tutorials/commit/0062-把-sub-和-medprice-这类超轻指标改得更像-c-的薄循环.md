# 把 `sub` 和 `medprice` 这类超轻指标改得更像 C 的薄循环

## 背景

这一轮继续按全量 benchmark 的 `>0.8x` 热点往下打。

和上一轮的 `decay` 不一样，这次挑的是另一类问题：

- 指标公式非常轻
- C 版本基本就是一个最薄的 `for` 循环
- Rust 版本虽然已经有 `run_in_place(...)`，但仍然带着通用 helper 的额外成本

这类点的典型代表就是：

- `sub`
- `medprice`

它们本身每个样本只做一次简单算术，所以如果 Rust 代码里还有多余抽象，这些开销就会被放大得很明显。

## 这次主要做了什么

### 1. 把 simple binary helper 写得更薄

`sub` 属于 simple binary family。

这次没有只给 `sub` 单独打补丁，而是把共享 helper 本身写得更接近 C：

- `run_binary(...)` 不再走 iterator + collect 风格
- `run_binary_in_place(...)` 也改成简单的索引循环

这样做的好处是：

- `sub` 直接受益
- 同类二元简单指标以后也能一起受益

### 2. 给 `medprice` 补专门的 batch kernel

`medprice` 之前还是走通用 `run_double_overlay(...)`。

现在改成了自己的 `run_medprice_batch(...)`：

- `run(...)` 直接预分配输出
- `run_in_place(...)` 直接顺序写切片

这比“复用通用 overlay helper”更贴近 C 版那条极薄路径。

## 为什么这次没有带 `crossover`

我同一轮也试了 `crossover` 的 batch 热循环，但 focused benchmark 没有形成稳定收益，反而还是慢。

所以这次故意没有把 `crossover` 混进提交。

这是这套协作流程里一个很重要的原则：

- 可以同时试多个方向
- 但提交时只保留“验证站稳”的那部分

不要把“试过”误写成“已经优化成功”。

## 验证结果

focused benchmark：

- `sub 4096`: `0.881x`
- `sub 65536`: `0.997x`
- `medprice 4096`: `1.102x`
- `medprice 65536`: `0.862x`

所以这笔的结论是：

- `sub` 两档都已经明显收回
- `medprice` 两档也都进入了当前阶段更现实的合格区间

## 给 Rust 新手的两个知识点

### 1. “已经有 `run_in_place`” 不等于“已经够薄了”

很多人会以为：

- 只要代码不是返回 `Vec<Vec<_>>`
- 只要已经改成 `in-place`

那性能问题就应该解决了。

其实不一定。

因为性能还取决于：

- 你是不是还在走通用 helper
- helper 里面是不是还用了额外抽象
- 对这种超轻指标来说，哪怕只多一层统一包装，也可能被 benchmark 放大

所以性能优化经常是分层推进的：

1. 先从“返回 owned Vec”改成 `in-place`
2. 再从“通用 helper”改成“更薄的专门内核”

### 2. 通用性和极致性能，经常要分开建模

像 `medprice` 这种指标，从代码复用角度看，走通用 `double overlay` 很自然。

但从性能角度看，它和 `C` 版本最接近的形态其实就是：

```rust
for i in 0..len {
    output[i] = (high[i] + low[i]) * 0.5;
}
```

这说明一个很典型的工程现实：

- 通用 helper 更容易维护
- 专门内核更容易逼近 C 性能

性能敏感库里，经常要允许这两种形态并存，而不是强迫所有实现都走同一层抽象。

## 这次没有包含什么

- `crossover` 的实验没有形成稳定收益，所以已经撤回
- 还没有处理 `dema`、`cvi` 这些当前榜单里的下一批热点
- 也没有改 stream 路径，这次只收 batch 轻量热点
