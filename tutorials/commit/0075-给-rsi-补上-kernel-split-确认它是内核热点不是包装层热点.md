# 给 RSI 补上 kernel split，确认它是内核热点，不是包装层热点

这次改动的目标不是直接优化 `rsi`，而是先把问题边界切清楚。之前我们已经给 `ema`、`wilders`、`zlema`、`macd` 做了 `run_in_place` 和 `kernel-only` 的拆分测试。现在 `rsi` 也进入了同一套机制，这样我们终于可以回答一个很关键的问题：

- `rsi` 是慢在 API 包装层吗？
- 还是慢在递推内核本体？

如果这个问题没先回答清楚，后面就很容易在错误的层上反复优化。

## 这次做了什么

我在 benchmark 基础设施里给 `rsi` 增加了两块内容：

- `run_rsi_kernel_probe(...)`
- `run_rsi_kernel(...)`

这样 `indicator-bench-compare` 现在会像对 `ema/macd/zlema/wilders` 一样，对 `rsi` 额外输出：

- `run_in_place ns/input`
- `kernel ns/input`
- `ratio to kernel`

也就是：

- 整个批处理入口花了多少
- 纯递推核心大约花了多少
- 两者差多少

## 结果说明了什么

focused benchmark 的结果很清楚：

- `rsi 4096 = 1.415x`
- `rsi 65536 = 1.402x`

同时 kernel split 显示：

- `4096`: `run_in_place / kernel = 0.968`
- `65536`: `0.946`

这代表什么？

- `run_in_place` 没有比 kernel 更重
- 甚至在这轮测量里，`run_in_place` 还略快于 probe kernel
- 所以 `rsi` 当前的慢点，不在 wrapper

这次最重要的产出不是“`rsi` 变快了”，而是：

- 以后不要再把精力花在 `rsi` 的包装层优化上
- 应该直接去看递推内核、codegen、或者更贴近 C 的热循环写法

## 为什么这类提交值得保留

性能工程里，能把“不要再优化哪一层”讲清楚，本身就是高价值结果。

如果没有这次 benchmark 拆分，后面很可能会继续在：

- `single_input`
- `validate_output_slices`
- `ensure_output_len`

这些公共 helper 上反复试验。但现在已经知道，这些不是 `rsi` 的主要瓶颈。

## 给 Rust 新手的 2 个知识点

### 1. benchmark 也要分层，不然会一直优化错对象

很多人一看到一个函数比 C 慢，就马上去改循环、改变量、改 `unsafe`。这往往太早了。

更合理的顺序是：

1. 先测完整入口
2. 再测纯 kernel
3. 看差距到底落在哪一层

如果不这么做，你优化掉的很可能只是“看起来最显眼”的那层，而不是真热点。

### 2. “wrapper 税” 和 “内核税” 是两回事

对新手来说，这两个词很容易混在一起。

- `wrapper 税`
  - 参数检查
  - 输出长度检查
  - 组织输入输出切片
  - 错误返回装配

- `内核税`
  - 真正逐样本执行的那段热循环
  - 递推状态更新
  - 数学表达式
  - 编译器生成代码质量

`rsi` 这次就是一个很好的例子：它慢，但慢点主要不在 wrapper。

## 这次没有做什么

这次没有直接优化 `rsi` 指标实现，也没有去改它的公式或接口。只做了：

- benchmark 能力增强
- 问题边界确认

下一步如果继续推进 `rsi`，就应该直接瞄准它的递推内核，而不是继续抠 `run_in_place` 外壳。
