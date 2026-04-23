# 收口 README 和 crate docs，让入口文档只负责导航

这次是文档收口，不是继续加内容。

前面已经有了：

- `tutorials/indicator-api.md`
- `tutorials/state-api.md`
- `tutorials/indicator-reference.md`
- `examples/*.rs`
- 源码 rustdoc

所以 README 不应该再承载一整套长教程和旧指标清单。它应该做入口导航。

## README 的变化

README 现在只保留高层信息：

- 这个仓库同时有 C 实现和 Rust 实现。
- `c/` 放 ANSI C 实现。
- `rust/` 放 Rust crate、测试和工具。
- `examples/` 放可运行 Rust 示例。
- Rust API 有 batch、stream、state 三层。
- 详细使用方式跳转到 guide 和 examples。

原来 README 里有一个 “104 total indicators” 的旧清单，这已经不适合继续保留。

原因是 Rust registry 当前是 148 个指标，继续保留旧清单会和 `indicator-reference.md` 冲突。现在 README 只写“Rust registry currently exposes 148 indicators”，并链接到全量 reference。

## crate-level rustdoc 的变化

`rust/src/lib.rs` 里的 crate docs 也做了同样收口：

- 先说明三层 API。
- 再给最小 batch/stream/state 示例。
- 最后指向 tutorials 和 examples。

它不再试图替代完整教程。

## 门禁变化

`documentation_style` 里启用了 `public_api_modules_have_crate_or_item_docs` 默认检查。

这个检查虽然还是粗粒度，但现在 public API 文件已经都有 crate/item rustdoc，默认启用是安全的。

暂时没有启用 `indicator_source_files_have_explanatory_comments`，因为还有 `ao.rs` 等文件没完成全量说明，提前启用会让 CI 变红。

## 一个小经验

README 最容易变成“什么都塞一点”的文件。

发布前更好的结构是：

- README：入口和路线图。
- tutorial：完整用法。
- examples：可运行代码。
- rustdoc：API 细节。
- reference：全量表格。

这样后续维护时，每类文档职责更清楚，也不容易互相打架。
