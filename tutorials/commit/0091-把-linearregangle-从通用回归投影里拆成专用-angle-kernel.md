# 把 linearregangle 从通用回归投影里拆成专用 angle kernel

这次改动的目标很具体：`linearregangle` 在 Rust 里功能是对的，但小输入 benchmark 仍然比 C 慢。对照汇编后，问题不是公式错了，而是 Rust 还在走一个“通用回归投影 kernel”，循环里要带着 `match projection` 一起跑；C 版本则是共享模板展开后，直接把 `atan(b) * 180 / PI` 写进最终输出。

## 这次改了什么

我们保留了现有的通用 `run_regression_batch(...)`，因为 `linreg`、`linregslope`、`tsf` 这些仍然适合共用它。  
但给 `linearregangle` 单独补了一条 `run_linearregangle_batch(...)`：

- 直接复用相同的线性回归统计量计算
- 在热循环里不再做 `match projection`
- 直接把 `slope.atan() * DEG_PER_RAD` 写到输出

这样做的目的不是“多写一份代码”，而是让编译器看到一个更单态、更直的热循环，更接近 C 的展开结果。

## 为什么这次值得单独拆

这类优化的关键不是“代码行数更少”，而是“热循环里少一个抽象层”。  
`linearregangle` 每个输出点都要做一次 `atan`，它本来就比纯加减乘除重；如果再把一个通用枚举分支塞进循环里，小输入时那点额外成本就更容易放大。

这次 focused benchmark 复跑后的结果是：

- `linearregangle 4096 = 0.987x`
- `linearregangle 65536 = 1.085x`

也就是两档都回到了 parity 区间。

## 这次顺手能学到的 Rust 知识

### 1. “泛型/共享 helper 很优雅” 不等于 “机器码一定最省”

Rust 写共享逻辑很方便，但编译器能不能把你的抽象完全压平，取决于具体代码形状。  
这次的问题不是 trait，也不是虚调用，而是一个看起来很普通的 `match projection` 仍然留在了热循环里。对性能敏感代码来说，这种“逻辑上小、循环里重复很多次”的分支就值得怀疑。

### 2. 抽出专用 kernel，不等于放弃复用

这次不是把整套回归指标都复制了一遍，而是只对 `linearregangle` 抽了一个专用 batch kernel。  
这是一种很常见的性能工程做法：

- 默认走共享实现，保证维护成本低
- 只有被 benchmark 和汇编一起证明是热点的 case，才单独拆出来

这样既能保住代码结构，也能在热点处接近 zero-cost。

## 还没做的事

- `linearregangle` 的 stream 路径这次没有单独特化
- 这次只处理了 `Angle` 这个投影，其他回归投影是否还值得进一步专门化，要看后续 benchmark 和汇编结果
