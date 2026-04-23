# 给核心公共接口补 rustdoc，让发布前用户能从源码直接看懂三层 API

这次不是改指标算法，而是给 release-facing 的公共接口补文档。

目标是让用户打开 `cargo doc` 或源码时，不需要先读测试和 README，也能理解这个库的几层入口：

- batch API：一次性输入列，返回完整输出。
- stream API：增量喂数据，不保留可索引历史。
- state API：增量更新，并保留固定容量历史。
- registry API：按名字或静态句柄找到指标。
- candle API：用稀疏结果表示 K 线形态命中。

## 这次补了哪些接口

核心 trait 现在说明了 `Indicator::run`、`run_single`、`run_in_place`、`IndicatorStream::feed`、`feed_single`、`feed_in_place` 的差异。

最重要的接口语义是分配行为：

- `run` 返回 `Vec<Vec<Real>>`，调用方便，但会拥有一份新输出。
- `run_single` 是单输出指标的便利包装，仍然返回 owned `Vec<Real>`。
- `run_in_place` 使用调用方提供的输出 buffer，适合性能敏感路径。
- `feed_single` 是 stream 版本的单输出便利包装。

## state API 的关键语义

`DynamicIndicatorState` 这次特别补了两个容易踩坑的点：

1. `history_capacity` 必须大于 0。
2. `seed_columns` 和 `seed_rows` 是原子 reseed：新数据校验和 seed 成功后，才替换旧状态。

另外 `latest_ref` 和 `get_ref` 返回的是内部历史的借用。它们适合避免拷贝，但不能跨过下一次 mutable 调用长期持有；如果要保存结果，应使用 `latest` 或 `get` 的 owned 版本。

## registry 和 candle 文档

registry 的 `pub static` 句柄也加了 rustdoc。这样用户可以在文档里看到 `RSI`、`MACD`、`EMA`、`SMA` 等静态入口，而不只是看到 `find()`。

candle API 现在说明了输入必须是 open/high/low/close 四列，并且输出是稀疏命中结果，不是普通指标那种 dense output series。

## 一个小经验

rustdoc 示例最好直接跑 doctest。

这次 dynamic-state 示例最初就暴露过一个问题：一个 state update 了新数据，另一个没有 update，却拿两者 `latest` 比较。`cargo test --doc` 能直接抓出这种文档里的假示例，避免 release 文档看起来对、实际跑不通。
