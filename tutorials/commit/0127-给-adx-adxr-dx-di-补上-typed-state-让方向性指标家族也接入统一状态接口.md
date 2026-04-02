这次把方向性指标家族也接进了 typed state，但没有动现有高性能层。

做法很直接：

- `dx`
  - 复用现有 `DxStream`
  - 补 `update_one(high, low)`
  - 再包一层 `DxState`

- `di`
  - 复用现有 `DiStream`
  - 补 `update_one(high, low, close)`
  - 再包一层 `DiState`

- `adx`
  - 复用现有 `AdxStream`
  - 补 `update_one(high, low)`
  - 再包一层 `AdxState`

- `adxr`
  - 复用现有 `AdxrStream`
  - 补 `update_one(high, low)`
  - 再包一层 `AdxrState`

这样做的好处是：

1. 不碰 batch / `run_in_place`
2. 不碰 benchmark 已经打磨过的内核
3. 只是在状态层外面补更顺手的 typed API

这批也继续验证了统一状态接口的一个原则：

- 计算状态和历史缓存分开
- 计算状态继续复用各指标自己的 stream/state 内核
- 历史输出统一用固定容量 ring buffer

这样 `seed/update/latest/get/reset` 的外形就统一了，但底下的高性能层完全不用回头重构。

这次顺手补的测试覆盖了：

- 单输出双输入：`dx`
- 双输出三输入：`di`
- 单输出双输入且二次平滑：`adx`
- 单输出双输入且带内部历史：`adxr`

一个给新人的点：

`adxr` 有两层“历史”：

- 一层是指标算法本身为了算 `ADXR` 保存的内部 ADX 历史
- 一层是统一状态接口为了 `get(index_from_latest)` 保存的输出历史

这两层不要混在一起。前者服务算法，后者服务 API。
