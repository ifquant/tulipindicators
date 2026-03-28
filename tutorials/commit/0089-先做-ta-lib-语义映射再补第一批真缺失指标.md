# 先做 TA-Lib 语义映射，再补第一批真缺失指标

这次不是直接“看 TA-Lib 有什么就抄什么”，而是先把 TA-Lib 和 Tulip 的语义关系理清，再只实现真正缺失的那一批。

如果不先做映射，很容易犯两个错误：
- 名字不同，但其实算法已经有了，又重复实现一份
- 名字很像，但语义其实不同，结果误把错误算法塞进同名接口

## 这次改动的目的

把 TA-Lib 对照工作从“名字匹配”提升到“语义匹配”，并落下第一批真正缺失、且最适合 Tulip 现有风格的指标：

- `linearregangle`
- `midpoint`
- `midprice`
- `rocr100`

## 做了什么

- 新增 [`/Users/dev/workspace2/hc_apps/tulipindicators/tutorials/ta-lib-semantic-mapping.md`](/Users/dev/workspace2/hc_apps/tulipindicators/tutorials/ta-lib-semantic-mapping.md)，把 TA-Lib 和 Tulip 现有指标的语义对应、易混淆项、真缺失项整理清楚
- 在 C 侧新增：
  - [`/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/linearregangle.c`](/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/linearregangle.c)
  - [`/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/midpoint.c`](/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/midpoint.c)
  - [`/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/midprice.c`](/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/midprice.c)
  - [`/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/rocr100.c`](/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/rocr100.c)
- 在 Rust 侧同步新增：
  - `LinearRegAngle`
  - `MidPoint`
  - `MidPrice`
  - `Rocr100`
- 给 Rust 补了一份新测试 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/talib_missing_parity.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/talib_missing_parity.rs)，用自定义样例同时校验 Rust 输出、C oracle 输出和手算期望值

## 为什么第一批是这四个

它们都能自然落进 Tulip 现有模式里：

- `rocr100` 本质上是 `rocr` 的一个简单比例变体
- `linearregangle` 本质上是 `linregslope` 的后处理
- `midpoint` 可以沿 `max/min` 的窗口极值模型实现
- `midprice` 可以沿价格类 overlay 的高低价窗口模型实现

相比之下，像 `MACDEXT`、`MAVP`、`HT_*`、`SAREXT` 这类指标，要么需要更复杂的 MA 抽象，要么带有 TA-Lib 自己特有的参数体系，不适合拿来做第一批。

## 新手知识点 1：名字像，不代表语义相同

最容易踩坑的是：

- `medprice` 是单根 bar 的 `(high + low) / 2`
- `midprice` 是窗口里的 `(highest high + lowest low) / 2`

这两个名字只差一个字母，但语义完全不是一回事。  
所以做跨库对照时，应该先找“数学定义”，再找“接口名字”。

## 新手知识点 2：先做语义映射，再做实现，是一种设计技巧

当你对接另一个库、标准或 API 时，先问：

- 这里到底是“缺功能”
- 还是“功能已有，只是命名不同”
- 还是“名字一样，但定义不同”

这样做的好处是：
- 避免重复代码
- 避免错误兼容
- 后面的测试和文档也会更干净
