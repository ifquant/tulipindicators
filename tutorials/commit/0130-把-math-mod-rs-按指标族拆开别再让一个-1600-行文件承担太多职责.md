这次做的是纯结构收口，没有改任何指标行为。

目标很简单：

- 把 `rust/src/indicators/math/mod.rs`
- 从一个接近 1600 行的大文件
- 拆成按指标族组织的多个子模块

现在的结构是：

- `math/cross.rs`
  - `CrossAny`
  - `Crossover`
- `math/decay.rs`
  - `Decay`
  - `EDecay`
  - `Lag`
- `math/correlation.rs`
  - `Beta`
  - `Correl`
- `math/extrema.rs`
  - `MidPoint`
  - `Max`
  - `Min`
  - `MaxIndex`
  - `MinIndex`
  - `MinMax`
  - `MinMaxIndex`
- `math/mod.rs`
  - 公共 helper
  - `Sum`
  - `pub use`

这样拆的好处不是“文件变多”，而是后面维护会更定点：

- 做交叉类改动时只看 `cross.rs`
- 做衰减类改动时只看 `decay.rs`
- 做极值窗口类改动时只看 `extrema.rs`
- 做相关系数类改动时只看 `correlation.rs`

这次刻意保留了两个边界：

1. 不改对外导出名

调用方还是从原来的 `math` 模块拿：

- `CrossAny`
- `Crossover`
- `Decay`
- `Lag`
- `Max`
- `Min`
- `Sum`

不会因为内部拆文件而影响外部接口。

2. 不顺手做逻辑去重

虽然这轮重构已经把模块边界理顺了，但没有顺手去碰：

- `shared.rs` 和指标文件里的重复 helper
- `IndicatorState::seed` 的默认实现
- `Floor` 的宏路径统一

原因是这几件事都值得单独成笔，避免把“文件拆分”和“行为/实现收口”搅在一起。

一个给新人的经验：

当一个文件已经大到“读者必须先在脑子里建目录树”时，就该拆了。  
拆分的目的不是追求形式上的小文件，而是让后续每一笔修改都更容易做到：

- 只看一个小主题
- 只改一个小区域
- 更容易验证“这次没误伤别的指标族”
