# 指标实现模板与性能收口规则

这份文档不是某一次提交的教程，而是把仓库里已经验证过有效的实现模式收成长期规则，减少后续继续补指标、抠性能时的风格漂移。

## 为什么需要这份文档

`tulipindicators` 现在已经同时维护：

- 原始 C 实现
- Rust 实现
- Rust stream 路径
- C/Rust parity 测试
- benchmark 对比与 self-best 基线

如果没有统一模板，代码很容易慢慢变成“每个指标都有一点自己的写法”。这会带来两个问题：

1. 新人很难判断某个实现是历史遗留，还是当前推荐形态。
2. 性能优化经验容易只散落在提交教程里，不能稳定复用。

## 当前推荐的五类实现模板

### 1. Vector / Elementwise

适用例子：

- `abs`
- `sub`
- `avgprice`
- `medprice`
- `typprice`
- `wcprice`

特点：

- 没有窗口
- 没有递推状态
- 每个样本只做固定组合或简单运算

推荐形态：

- `run()` 里直接分配输出缓冲区
- `run_in_place()` 直接写调用方提供的切片
- 不要默认用 `map().collect()` 作为最终热路径

### 2. Rolling Window

适用例子：

- `midpoint`
- `midprice`
- `stoch`
- `cci`
- `minmax`
- `correl`

特点：

- 有固定窗口
- 常见模式是滑动最值、滑动和、滑动统计量

推荐形态：

- 优先有专门的 `run_<indicator>_batch(...)`
- 滑窗状态尽量在 batch kernel 内部单次推进
- 如果 stream 也存在，stream 共享相同数学语义，但不要反过来让 batch 依赖 `stream.feed()`

### 3. Smoothing / Recurrence

适用例子：

- `ema`
- `wilders`
- `rsi`
- `macd`
- `trix`
- `kama`
- `ppo`
- `natr`

特点：

- 每一步依赖上一步状态
- 常见瓶颈不在 wrapper，而在热循环 codegen

推荐形态：

- 把 warmup 和 steady-state 尽量分开
- 热循环里避免额外 helper、分支和不必要的边界检查
- 如果 C 汇编已经出现 `fmadd`，Rust 默认检查 `mul_add` 是否能对齐出同样的指令形状

### 4. Branch-heavy State Machine

适用例子：

- `psar`
- `sarext`
- candle engine 的部分 pattern 状态推进

特点：

- 分支多
- 反转、夹逼、状态切换很频繁
- 不一定能像递推平滑那样靠 `mul_add` 大幅收回

推荐形态：

- 优先保持状态结构清楚
- 先做 correctness / parity
- 再用 focused benchmark 和汇编分析判断是否值得继续优化

### 5. Multi-output Recurrence

适用例子：

- `macd`
- `stoch`
- `fisher`
- `aroon`

特点：

- 同一条热循环里同时写多路输出

推荐形态：

- 一个 batch kernel 同时生成多路输出
- `run_in_place()` 里优先一次性拆开输出切片，不要重复绕 owned `Vec`

## 什么时候算“还没收口”

看到下面这些信号，通常说明实现还没进入当前推荐形态：

- `run()` 直接 `stream.feed(inputs)`
- batch benchmark 仍然走 owned `Vec<Vec<Real>>` 回退路径
- 同一家族里只有个别指标有 `run_in_place()`
- 热循环里还有明显可以抽到 warmup 外的分支
- C 汇编已经是 `fmadd`，Rust 还停留在 `fmul + fadd`

## benchmark 的默认解释顺序

1. 先看 focused benchmark，而不是先看混跑全量榜单。
2. 如果仍然慢，判断是：
   - wrapper 税
   - kernel 本体
   - benchmark 噪声
3. 如果是递推型指标，再去看汇编里有没有 `fmadd` / `mul_add` 差异。
4. 如果是 branch-heavy 指标，不要默认假设还能靠一条算术指令优化解决。

## 这份文档的用途

以后遇到下面几类工作，都应该先回来看这份文档：

- 新增 TA-Lib 对齐指标
- 把旧指标从历史路径收口到统一模板
- 分析 C/Rust benchmark 差异
- 决定某个热点该优先看 wrapper、kernel 还是汇编
