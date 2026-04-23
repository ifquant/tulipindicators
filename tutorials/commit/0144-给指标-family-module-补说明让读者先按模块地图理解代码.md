# 给指标 family module 补说明，让读者先按模块地图理解代码

这次补的是指标目录的“地图”，不是单个指标公式。

之前用户打开 `rust/src/indicators/` 时，只能看到一堆 `mod` 和 `pub use`。熟悉项目的人知道 `indicator`、`overlay`、`math`、`simple` 的区别，新读者不一定知道。

## 四个 family 的边界

现在模块文档里明确了四类入口：

- `indicator`：震荡、趋势、动量、波动率、成交量等命名技术指标。
- `overlay`：移动平均、价格带、通道、叠加在价格上的指标。
- `math`：滚动数学工具和向量工具，不直接带市场解释。
- `simple`：逐元素 unary/binary transform，没有 lookback。

这个边界对后续维护很重要。新增指标时，先看它属于哪类，再决定放在哪个模块，而不是只按文件名随手塞。

## simple 模块为什么单独解释宏

`simple/mod.rs` 里大量指标是宏生成的，例如 `abs`、`sin`、`add`、`div`。

它们的结构高度一致：

- unary：一个 `real` 输入，输出同长度序列。
- binary：两个 `real` 输入，输出同长度序列。
- 没有 lookback。
- stream 和 batch 都是逐元素推进。

所以文档解释生成模式，比给每个 simple 指标重复写一份更清楚。

## shared.rs 的定位

`shared.rs` 不是用户 API，但它是后续维护者最容易误改的地方。

里面有 EMA、Wilder 平滑、RSI 状态、方向运动、滚动极值、滚动 sum、rolling variance、WMA、true range 等复用状态。文档的目标不是公开这些类型，而是让维护者知道“这个 helper 是给哪些指标形状服务的”。

## 一个小经验

模块级文档不应该写成公式大全。

公式细节适合放在具体指标文件里。family module 更适合回答三个问题：

1. 这个目录是什么边界？
2. 读者应该从哪个子模块继续看？
3. 新指标或 helper 应该怎么归类？
