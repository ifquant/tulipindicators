# 把 `mama` 的一堆 `VecDeque` history 收成固定 ring buffer 状态

这次处理的不是一个“小优化”，而是一笔状态表示重构。

之前 `mama` 在 Rust 里虽然算法是对的，但状态结构很重：`MamaStream` 里挂着很多个 `SmallHistory`，而 `SmallHistory` 背后是 `VecDeque`。这会让热循环不断经过 `push_back` / `pop_front` 一类的通用容器逻辑。C 版不是这样，它用的是固定大小的局部 ring buffer 和固定偏移访问，所以 steady-state 很紧。

## 这次改了什么

- 去掉 `SmallHistory(VecDeque)` 这层通用容器
- 给 `MamaStream` 需要的每条历史序列换成固定容量 `RingHistory<const N: usize>`
- 保留原来的算法和索引语义，只改变状态表示
- `hilbert_transform(...)` 也一起切到 `RingHistory`

## 为什么这次方向对

前面已经验证过，`mama` 不是简单的 wrapper 税：

- 给它补 `run_in_place` 不但没救回来，反而更慢
- 汇编里能看到很多 `VecDeque` 相关路径
- 这说明主成本在“状态表示太重”，不是 batch 接口少了一层薄包装

所以这次不是继续抠 API，而是直接把状态模型往 C 的实现形状靠。

## 结果

focused benchmark：

- `mama batch 4096 = 1.067x`
- `mama batch 65536 = 1.005x`
- `mama stream 4096 = 1.184x`
- `mama stream 65536 = 0.741x`

可以看到：

- batch 两档基本都回到了 parity
- 长输入 stream 甚至已经快于 C
- 小输入 stream 还留一点尾差，但整体已经比之前那种 `1.6x+` 的形状好很多

## 这次最值得记住的知识点

### 1. 不要把“容器方便”误当成“热路径合适”

`VecDeque` 很方便，但它是通用容器。对这种固定窗口、固定偏移、频繁更新的状态机，固定 ring buffer 往往更接近你真正想要的机器码。

### 2. 有些指标的主成本根本不在公式里

`mama` 不是先靠 `mul_add` 或 `run_in_place` 收回来的，而是先靠“状态表示降重”收回来的。这类 case 的第一刀应该先问：

- 我到底慢在公式？
- 还是慢在状态结构和数据访问？
