# 补齐 README 和 state API 文档里的 `run_single` 指引，避免新示例继续默认写 `run(...)` 再取 `batch[0]`

这次没有继续改代码调用点，因为我把 `parity`、benchmark、bench 工具和现有示例重新筛了一轮，没找到新的稳定单输出热点。

真正还缺的是文档层的默认引导。

如果 README 和 `tutorials/state-api.md` 还只写：

- `run(...)`
- `run_in_place(...)`

那后来人很自然就会继续写：

```rust
let batch = indicator.run(...)?;
let values = &batch[0];
```

即使仓库里已经有了 `run_single(...)`，也很容易被忽略。

## 这次补了什么

1. README 的 Rust State API 小节里，把 batch 层更新为：
   - `run(...)`
   - `run_single(...)`
   - `run_in_place(...)`
2. README 增加了一个单输出 batch 示例，明确说明 `rsi`/`ema`/`atr` 这类指标优先用 `run_single(...)`。
3. `tutorials/state-api.md` 的 API Layers 也同步加入 `run_single(...)`。
4. `tutorials/state-api.md` 增加单输出 batch 示例，直接示范如何避免 `batch[0]` 拆包。

## 为什么这一笔也值得单独做

接口税的收口不只是改代码，还包括改默认心智模型。

如果文档没有把“单输出就用 `run_single`”说清楚，那仓库会不断长出新的 `run(...)[0]`，最后还得反复回头清理。把文档默认值改掉，后续新增代码自然就会更整洁。
