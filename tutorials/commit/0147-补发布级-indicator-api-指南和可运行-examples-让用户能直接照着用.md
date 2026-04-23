# 补发布级 indicator API 指南和可运行 examples，让用户能直接照着用

这次补的是面向用户的 release 文档，不是源码注释。

前面几批已经把源码里的 trait、state、indicator family 都解释清楚了，但用户第一次用库时，不应该必须打开源码。需要有几页能直接回答“我该怎么调用”的文档。

## 新增的用户文档

新增：

- `tutorials/indicator-api.md`
- `tutorials/indicator-reference.md`

`indicator-api.md` 解释几层入口：

- `run(...)`
- `run_single(...)`
- `run_in_place(...)`
- `create_stream(...)`
- `feed(...)`
- `feed_single(...)`
- `feed_in_place(...)`
- `registry::find(...)`

`indicator-reference.md` 是完整 registry 表，一共 148 行。表格来自实际 `registry::all()` metadata 的快照，再额外标注 typed state 是否存在。

## examples

新增三个可运行示例：

- `examples/batch_rsi.rs`
- `examples/in_place_macd.rs`
- `examples/dynamic_state.rs`

这三个分别覆盖：

1. 单输出 batch：`Rsi.run_single(...)`
2. 多输出 in-place：`Macd.run_in_place(...)`
3. 运行时选择指标：`DynamicIndicatorState::from_name(...)`

验证时不只跑了 `cargo test --examples`，还实际跑了：

```bash
cargo run --quiet --example batch_rsi
cargo run --quiet --example in_place_macd
cargo run --quiet --example dynamic_state
```

## 为什么 reference 表要全量

最初可以写 curated reference，但这类表最容易造成错觉：用户以为是完整清单，结果漏了很多指标。

所以最终改成全量 registry metadata snapshot。这样它至少和当前代码的 registry 形状一致：

- 指标名
- category
- inputs
- options
- outputs
- typed state availability

后面如果继续收口，最好再加一个同步测试，防止 registry metadata 改了但文档没更新。

## 一个小经验

Markdown 教程里的 Rust 代码块不要混用 rustdoc 隐藏行风格。

例如这种写法适合 doctest：

```rust
# Ok::<(), Error>(())
```

但不适合普通教程。用户复制时容易拿到一段不能直接编译的代码。所以这次教程里的代码块都改成完整：

```rust
fn main() -> Result<(), tulipindicators::IndicatorError> {
    Ok(())
}
```
