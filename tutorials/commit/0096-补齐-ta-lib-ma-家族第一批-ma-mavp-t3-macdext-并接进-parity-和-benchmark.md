# 补齐 TA-Lib MA 家族第一批：`ma`、`mavp`、`t3`、`macdext`

这次补的不是一堆彼此无关的新指标，而是一批很典型的 TA-Lib“均线家族接口”：

- `ma`
- `mavp`
- `t3`
- `macdext`

如果不先做语义拆分，很容易误判成“缺 4 个算法”。  
但真正的情况是：

- `ma` 更像一个“均线分发壳”
- `mavp` 更像“变量 period 的均线选择器”
- `macdext` 更像“可选均线类型的 MACD 组合壳”
- 只有 `t3` 才更接近一个独立算法内核

所以这次最重要的设计点不是“尽快把函数名补齐”，而是：

- 先判断哪些是壳
- 哪些是内核
- 然后尽量复用现有 Tulip 指标，而不是复制算法

## 这次改了什么

### 1. 先把 `mama` 从 beta 提升到 stable 依赖

TA-Lib 的 `MAType=7` 对应的是 `MAMA`。

如果 `ma` 想完整支持 TA-Lib 的均线类型分发表，就不能只在 Rust 里偷偷调用一个 beta 指标，而要让 C 侧稳定构建链也能正式引用它。

所以这次做了两件事：

- `c/build.tcl` 去掉了 `mama` 的 `beta` 标记
- `c/Makefile` 把 `beta/mama.c` 纳入 stable 构建

这一步不是性能优化，而是语义补齐的前提。

### 2. `ma` 用“分发壳”实现，而不是复制九套均线

新增：

- `/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/ma.c`
- `/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/talib_ma.rs`

核心思路一样：

- 解析 `period + ma_type`
- 把它分发到现有的 `sma/ema/wma/dema/tema/trima/kama/t3/mama`

也就是说，这次没有复制一份“万能均线算法”，而是把 TA-Lib 的接口语义翻译成 Tulip 现有内核。

这比复制算法更稳，因为后面：

- `ema` 优化
- `tema` 修 bug
- `t3` 再调优

都能直接让 `ma` 受益。

### 3. `mavp` 不是简单循环 period，而是“变量 period + 缓存”

新增：

- `/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/mavp.c`
- Rust 侧同样在 `talib_ma.rs` 里实现

这里最容易写成一个非常慢的版本：

- 每个输出点都临时跑一遍均线

这次没有这么做，而是：

- 先把 period clamp 到 `min_period..max_period`
- 只对出现过的唯一 period 计算一次完整均线序列
- 再按位置取值

所以 `mavp` 本质上是“按 period 做结果缓存”，不是“逐点重算”。

### 4. `t3` 才是这批里真正的新内核

新增：

- `/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/t3.c`
- `/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/t3.rs`

这次直接实现了 TA-Lib 风格的 `period + vfactor` 版本，并把 lookback 对齐成：

- `6 * (period - 1)`

它不是靠现有 `ema` 简单包一下就能得出的，所以这一项是这批里真正的新算法实现。

### 5. `macdext` 不能只做“任意 MA 组合”，还要对齐 `macd` 的 EMA 语义

新增：

- `/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/macdext.c`
- `/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/macdext.rs`

这次有一个很关键的坑：

- 直接用通用 `ma` 去拼 `macdext`
- 在 `EMA/EMA/EMA` 情况下
- 不一定和现有 `macd` 的输出起点一致

原因是：

- 单独的 `ema` lookback 是 `0`
- 但 `macd` 的语义是从 `long_period - 1` 那一层对齐开始输出

所以这次最后补了一条专门规则：

- 如果 `macdext` 的三段 MA 类型都是 `EMA`
- 就直接转发到现有 `macd`

这一步很重要，因为它保证了：

- `macdext(EMA, EMA, EMA)` 不只是数学上类似
- 而是和已有 `macd` 在 Tulip 里的具体输出语义完全一致

## 这次还补了什么验证

除了功能实现，还一起补了：

- parity 测试
- benchmark 覆盖
- 语义映射文档

更新了：

- `/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/talib_missing_parity.rs`
- `/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/talib_missing_benchmark.rs`
- `/Users/dev/workspace2/hc_apps/tulipindicators/tutorials/ta-lib-semantic-mapping.md`

这一步的意义是：

- 不只证明“函数能跑”
- 还证明它们已经进入了 C/Rust 对账和性能管线

本轮 focused benchmark 结果是：

- `ma`: `4096 = 0.980x`, `65536 = 0.985x`
- `macdext`: `4096 = 0.967x`, `65536 = 0.931x`
- `t3`: `4096 = 0.973x`, `65536 = 1.041x`
- `mavp`: `4096 = 5.962x`, `65536 = 6.594x`

也就是说：

- `ma`
- `macdext`
- `t3`

都已经基本追平 C。

但 `mavp` 明显还慢，这一项这次先保证语义和覆盖完整，性能优化留到后续单独处理。

## Rust 新手知识点 1：`enum` 分发和“复制算法”不是一回事

这次 Rust 侧用了一个 `TalibMaType` 枚举。

它的价值不是“为了抽象而抽象”，而是把：

- TA-Lib 的数字类型码

变成：

- Rust 里可读、可匹配、可复用的分发表

这样 `ma`、`mavp`、`macdext` 就能共享同一套均线类型语义，而不是每个指标都自己再写一遍 `if ma_type == 3 ...`。

对新手来说，一个很有用的判断标准是：

- 如果多个 API 共享一套离散语义
- 就优先考虑 `enum + match`
- 而不是散落的魔法数字判断

## Rust 新手知识点 2：语义复用不等于行为完全一致

这次 `macdext` 最后要 special-case `EMA/EMA/EMA`，就是一个很典型的例子。

表面上看：

- `macdext`
- `macd`

都在做 MACD。

但真正要对齐时，不能只看公式，还要看：

- lookback
- 输出起点
- 现有库里的历史语义

所以做“兼容 TA-Lib”或“兼容旧库”时，一个很重要的工程技巧是：

- 先复用语义
- 再专门补那些已经存在的历史行为边界

不要过早假设“数学等价 = API 行为等价”。

## 结果

这次之后，TA-Lib 映射里又少了一整批真缺失项：

- `ma`
- `mavp`
- `t3`
- `macdext`

其中：

- `ma` / `macdext` 主要是接口语义补齐
- `mavp` 是变量 period 均线壳
- `t3` 是真正的新内核

而且它们已经一起接进：

- C 实现
- Rust 实现
- parity 测试
- benchmark 覆盖
- TA-Lib 语义映射文档

下一步最自然的剩余项就是：

- `sarext`
- `ht_*`

以及后续对 `mavp` 做单独性能优化。
