# 补齐 TA-Lib 窗口索引家族并接进 parity 和 benchmark

这次补的是一整组很适合 Tulip 风格的 TA-Lib 缺失指标：

- `maxindex`
- `minindex`
- `minmax`
- `minmaxindex`

它们看起来是四个名字，但本质上共享同一套窗口极值语义，所以最合理的做法不是拆成四笔零散改动，而是当成一个“窗口索引家族”一次补齐。

## 先确认语义，再写代码

这一组最危险的地方不是滑窗本身，而是“索引到底表示什么”。

我先用 TA-Lib Python 包确认了语义：

- `MAXINDEX` / `MININDEX` / `MINMAXINDEX` 输出的是**原始输入序列里的绝对索引**
- 不是“窗口内第几个元素”
- TA-Lib Python wrapper 对整数输出前导段会给默认整数值，但 Tulip 这边仍保持一贯风格：只输出有效段，不补前导 `NaN`

所以最后的 Tulip 风格是：

- lookback 仍然是 `period - 1`
- 输出长度仍然是 `input_len - period + 1`
- 每个输出值保留 TA-Lib 的“绝对索引”语义

## C 和 Rust 是怎么落地的

### C 侧

新增了四个指标文件：

- `c/indicators/maxindex.c`
- `c/indicators/minindex.c`
- `c/indicators/minmax.c`
- `c/indicators/minmaxindex.c`

这四个实现都沿用 Tulip 现有 `midpoint` / `max` / `min` 那种直接滑窗风格，而不是引入额外抽象。

### Rust 侧

Rust 放在 `math` 模块里：

- 单输出索引：复用 `MonotonicQueue::front_index()`
- 双输出值：`minmax`
- 双输出索引：`minmaxindex`

这样做的好处是：

- 复用了现有窗口极值状态结构
- API 形状和现有 math 指标保持一致
- batch、stream、registry、benchmark 可以一起接上

## 这次顺手能学到的知识

### 1. “语义映射” 比 “名字映射” 更重要

`MIDPOINT` 和 `MEDPRICE`、`MIDPRICE` 很像，但不是一回事；  
`MAXINDEX` 和 `MAX` 也不是简单换个名字。

如果不先确认语义，很容易做出“名字对了，行为偏了”的实现。  
这也是为什么这次先查 TA-Lib 的真实输出形状，再开始写代码。

### 2. 输出类型统一成 `Real`，不代表索引语义被丢掉

Tulip 这套 Rust/C 实现里，输出容器都是 `Real`。  
索引类指标虽然本质上是整数语义，但仍然可以稳定地用 `Real` 来承载，只要：

- 文档明确说明这是绝对索引
- parity 测试把值按整数语义去验证

这是一种常见的工程取舍：保持统一的指标输出接口，避免为了少量索引类指标单独引入另一套输出类型系统。

## 结果

这批指标已经接进：

- C 实现
- Rust registry
- parity 测试
- benchmark 覆盖测试
- C/Rust release compare

focused benchmark 结果是：

- `maxindex`: `4096 = 0.791x`, `65536 = 0.847x`
- `minindex`: `4096 = 0.581x`, `65536 = 0.728x`
- `minmax`: `4096 = 0.588x`, `65536 = 0.703x`
- `minmaxindex`: `4096 = 0.745x`, `65536 = 0.661x`

也就是说，这一组不仅补齐了语义，而且 Rust 在当前 benchmark 下已经整体快于 C。

## 还没做的事

- `beta` / `correl` 这类统计双输入指标还没开始
- `ma` / `mavp` / `macdfix` / `macdext` 这类 TA-Lib 特有接口层仍未补
- `ht_*` Hilbert Transform 家族仍然还没进入 Tulip 线
