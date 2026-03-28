# 0027 Separate public contract checks from internal invariants

## 背景

前面为了给 Rust 指标库补高性能 batch 接口，我们引入了 `run_in_place`。这条路径对性能很重要，但也让一个老问题更明显了：有些检查属于公开 API 的一部分，必须在 release 中保留；有些检查只是开发时帮助我们抓实现 bug，不应该让 release 二进制因为内部断言直接崩掉。

## 主要目标

把 Rust 性能层里的检查拆成两类：

- 调用方契约检查：继续保留，并返回正常错误
- 内部一致性检查：不再靠 `expect` 或 release 断言直接崩溃

## 改动概览

- `run_in_place` 默认实现现在先按 metadata 校验调用方提供的输出缓冲区数量，这属于公共契约
- 如果某个指标实现自己返回了错误数量的输出序列，现在会返回 `InternalInvariant`，明确这是实现侧 bug，而不是用户传参错误
- benchmark 路径里原先的 `expect(...)` 已改成普通错误返回，避免 release benchmark 因内部假设失败直接 panic
- 新增了 `MissingStreamSupport` 和 `InternalInvariant` 两个错误类型，用来把“用户错误”和“实现错误”分开表达

## 关键知识

这次最重要的设计区分是：

- `WrongOutputCount`、`OutputTooSmall` 这类错误，表示调用方没有遵守 API 契约
- `InternalInvariant` 这类错误，表示库内部实现违反了自己的约束

这两个概念不能混在一起。混在一起以后，调用方会拿到误导性的错误信息，也不利于后面做性能优化和缺陷定位。

## 补充知识

1. Rust 新手常见误区：`assert!` 和 `Result` 不是同一种工具。`Result` 适合表达外部输入错误，`assert!` / `debug_assert!` 更适合表达“这段代码按设计不该发生”的内部不变量。

2. 做性能层设计时，不要一看到检查就想删。更好的做法是先判断这是不是公开契约的一部分。契约检查删掉以后，得到的往往不是“更快”，而是“更容易 silent corruption”。

## 验证

- `cargo test` (`PASS`)
- `TI_BENCH_SIZES=256,4096 TI_BENCH_TARGET_MS=20 cargo run --release --bin indicator-bench-compare` (`PASS`)

## 未覆盖项

- 这次只清理了性能层和 benchmark 里的契约/不变量边界，没有全面清扫所有指标实现内部的 `expect(...)`
- 还没有把这条“契约检查保留、内部断言不进 release”原则同步写回 `cagent`
