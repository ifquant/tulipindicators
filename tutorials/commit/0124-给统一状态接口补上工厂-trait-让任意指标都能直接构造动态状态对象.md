# 给统一状态接口补上工厂 trait，让任意指标都能直接构造动态状态对象

## 背景

到上一笔为止，库里已经有了完整的一层统一状态接口：

- 有 stream 的指标走 stream backend
- 没有 stream 的指标走 batch fallback
- 调用方已经可以用 `DynamicIndicatorState` 做 `seed / update / latest / get`

但还有一个很现实的小问题：入口还不够顺手。

调用方如果想构造统一状态对象，得自己记住：

- `DynamicIndicatorState::new(indicator, options, history_capacity)`
- 或者 `DynamicIndicatorState::from_name(name, options, history_capacity)`

它们都能用，但还不够像“库自己的自然 API”。

## 主要目标

这次只做一件小事：把统一状态接口的入口再收口一下，让任意指标对象都能直接构造动态状态对象。

目标不是加新能力，而是把已经完成的能力变得更顺手。

## 改动概览

### 1. 新增 `IndicatorStateFactory`

修改文件：

- `rust/src/state.rs`

这次新增了一个非常薄的扩展 trait：

- `IndicatorStateFactory`

它提供：

- `dynamic_state(&self, options, history_capacity)`

这个方法本质上只是把：

- `DynamicIndicatorState::new(self, ...)`

封装成了指标对象本身的方法。

### 2. 默认实现覆盖所有 `Indicator`

`IndicatorStateFactory` 不是给少数指标单独写实现，而是对所有实现了 `Indicator` 的类型统一提供默认实现。

这意味着：

- `Rsi.dynamic_state(...)`
- `Dm.dynamic_state(...)`
- 通过 registry 找到的指标对象也可以直接走这条入口

不需要再一个个补 builder。

### 3. 从 crate 根导出工厂 trait

修改文件：

- `rust/src/lib.rs`

这样外部使用者可以直接 `use tulipindicators::IndicatorStateFactory;`，不用自己翻内部模块。

### 4. 新增工厂入口测试

修改文件：

- `rust/tests/state_api.rs`

这次新增了一个很薄的测试，专门确认：

- `Rsi.dynamic_state(...)`
- 构造出的对象
- 可以正常 `seed_columns(...)`
- 并且 `latest()` 和 batch 结果一致

这个测试的目的不是重复验证状态接口本身，而是确认“新入口确实可用”。

## 关键知识

## 为什么这次要用 trait，而不是继续加更多自由函数

因为这层 API 的核心概念是：

- 指标对象
- 状态对象

最自然的写法应该是“从指标对象直接构造状态对象”，而不是继续增加很多独立入口函数。

trait 的好处是：

- 入口更自然
- 不用为每个指标单独加方法
- 和已有 `Indicator` 概念保持一致

## 为什么这次没有删掉 `DynamicIndicatorState::new/from_name`

因为这两个入口仍然有价值：

- `new(...)` 适合已经拿到指标对象的底层调用
- `from_name(...)` 适合动态配置、命令行和脚本型调用
- `dynamic_state(...)` 适合最自然的静态使用方式

这三者是互补关系，不是互斥关系。

## 验证

- `cargo fmt --all` (`PASS`)
- `cargo test --test state_api --test stable_parity --test golden_indicators` (`PASS`)
- `cargo clippy --all-targets --all-features` (`PASS`)

## 未覆盖项

- 这次没有新增新的状态能力，只是把入口变得更顺手
- 这次没有给 typed `RsiState / DmState` 再补更统一的 builder
- 这次也没有处理状态接口的文档页面或 README 示例，仍以后续整理为主
