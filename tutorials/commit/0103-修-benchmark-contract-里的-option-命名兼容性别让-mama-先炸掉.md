## 背景

这次不是在修 `mama` 指标本身，而是在修 benchmark 基础设施。

问题现象是：

- 跑全量 `indicator-bench-compare` 时，C 侧 `benchmark_contract` 会在 `mama` 上直接失败
- 失败信息是 `batch benchmark failed for mama (1)`

先不要急着怀疑算法或者 C/Rust 对齐坏了，因为这类错误很可能发生在：

- 输入数据生成
- 默认 option 生成
- benchmark 调度/过滤逻辑

而不是指标实现本体。

## 根因

这次根因很典型：**生成出来的 C 指标元数据里，`mama` 的 option 名字带空格，但 benchmark 默认值映射只认不带空格的版本。**

在 C 生成链里，`mama` 的定义来自：

- `c/build.tcl`

它保留的是 TA-Lib 风格名字：

- `fast limit`
- `slow limit`

生成后的 C 元数据也是这两个名字。

但 benchmark 默认值映射里只写了：

- `fastlimit`
- `slowlimit`

所以 `benchmark_contract` 在给 `mama` 生成默认 options 时，命中了 fallback 分支，喂了错误值，最后 batch benchmark 直接失败。

Rust benchmark 里也有同样的问题，只是这次先在 C 侧炸出来了。

## 修改

这次只做了一件事：让 C 和 Rust 的 benchmark 默认 option 映射同时兼容两种写法。

### C

文件：

- `c/benchmark_contract.c`

把：

- `fastlimit`
- `slowlimit`

扩成同时接受：

- `fastlimit`
- `fast limit`
- `slowlimit`
- `slow limit`

### Rust

文件：

- `rust/src/benchmark.rs`

同样把 benchmark 默认 option 匹配扩成同时接受带空格和不带空格的两种名字。

这样做的目的不是“迁就脏命名”，而是：

- benchmark 合约要和真实生成出来的 indicator metadata 对齐
- 不能假设所有 option 名字都已经规范化

## 为什么这一步很重要

性能基准最怕两类问题：

1. 指标真的慢
2. benchmark 自己先坏了

第二类问题更危险，因为它会把后续所有性能结论污染掉。

这次修完以后，至少能保证：

- `mama` 不会因为 option 名字格式差异直接把全量 benchmark 跑挂
- C/Rust 两边 benchmark contract 继续保持同一套默认值口径

## 新手知识点

### 1. 元数据字符串也是接口的一部分

像 `option_names` 这种字段，看起来只是文案，但一旦 benchmark、测试、自动生成代码都依赖它，它就已经是接口的一部分了。

如果生成链输出的是：

- `fast limit`

那消费方就必须认这个名字，不能只认自己心里更整洁的：

- `fastlimit`

### 2. 基础设施 bug 会伪装成算法 bug

这次 `mama` 报的是 benchmark failure，很容易第一反应去怀疑：

- MAMA 算法实现错了
- beta 指标不稳定

但真正坏掉的其实只是 benchmark 默认 option 映射。

所以遇到“某个指标一跑就炸”的情况，先查：

- 输入
- option
- 调度
- schema

再去怀疑算法本体，通常更高效。
