# 继续打 `wilders` 和 `vosc`，并解释 owned 返回值的接口税

这次是一次很适合新手看的性能切片，因为它把两类热点分开了：

1. 真的只是“batch 走错了路径”
2. 已经走对路径了，但还剩下一些 Rust 公共接口本身的固定成本

这两类问题，看起来都叫“性能慢”，但处理办法不一样。

## 这次改了什么

### 1. `vosc` 从 stream-backed batch 拆成 direct batch kernel

在 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/price_volume.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/price_volume.rs) 里，`Vosc` 之前的 batch 路径还是：

- `run()` 里先 `VoscStream::new(...)`
- 再 `stream.feed(inputs)`

这跟前面修掉的 `wad / emv` 是同一类问题：  
数学本身不复杂，但 batch 先绕了一层 stream 状态机。

所以这次改成了：

- `run()` 直接走 `run_vosc_batch(...)`
- 新增 `run_in_place(...)`
- 内核直接做双窗口滑动和输出写入

结果也很直接：`vosc` 这类问题，拆掉 stream-backed batch 后基本就回到阈值附近了。

### 2. `wilders` 没有再走错路径，但继续把热循环写薄

`Wilders` 和 `vosc` 不一样。  
它在这次改动前就已经有：

- `run_in_place(...)`
- direct batch kernel

所以它不是“路径错了”，而是“内核还不够薄”。

这次在 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/wilders.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/wilders.rs) 里做的是更小的一刀：

- 把 `output[out_index] = value; out_index += 1;`
  改成
- `output[1..].iter_mut().zip(&input[period..])`

这能少一点索引写入形式带来的负担，让热循环更接近 C 的“指针往前推”风格。

## 这次 benchmark 说明了什么

我跑的是：

```bash
TI_BENCH_INDICATORS=wilders,vosc \
TI_BENCH_SIZES=4096,65536 \
TI_BENCH_TARGET_MS=120 \
TI_BENCH_CALIBRATION_MS=30 \
TI_BENCH_REPEATS=5 \
cargo run --release --bin indicator-bench-compare
```

结果是：

- `vosc batch 4096`: `0.926x`
- `vosc batch 65536`: `1.272x`
- `wilders batch 4096`: `1.879x`
- `wilders batch 65536`: `1.450x`

这组结果特别有价值，因为它说明：

### `vosc`：主要问题是路径问题

`vosc` 在拆掉 stream-backed batch 后，`4096` 已经回到阈值内，`65536` 也只是轻微超线。  
这说明它之前慢，主要不是数学本身，而是 batch 接口路径不对。

### `wilders`：主要问题已经不是路径，而是“简单指标上的接口税 + 内核尾差”

`wilders` 这次只收回了一部分，没有像 `wad / emv` 那样一刀见效。  
这说明它剩下的问题更像：

- 每次调用的公共契约检查
- Rust 切片 / 索引 / 错误返回这一层固定成本
- 非常轻量的数学循环本身，对这些固定成本特别敏感

也就是说，`wilders` 不再是“实现路线走错”，而是“Rust 公共安全接口比 C 薄循环更重”。

## 这和你问的 `Result<Vec<Vec<Real>>, IndicatorError>` 有什么关系

你前面问得很对：  
`Result<Vec<Vec<Real>>, IndicatorError>` 到底是什么，它会不会天然拖慢性能？

答案是：`会，它是 owned 返回值，不是引用。`

这意味着：

- 要构造 `Result`
- 成功时要构造外层 `Vec`
- 每个输出通道还要构造自己的内层 `Vec`
- 最后把整块拥有所有权的数据交给调用方

这不是“零开销”接口。  
所以我们后来才分层出：

- 高层易用接口：`run() -> Result<Vec<Vec<Real>>, IndicatorError>`
- 低层高性能接口：`run_in_place(..., outputs: &mut [&mut [Real]]) -> Result<usize, IndicatorError>`

`wad / emv / vosc` 这类案例说明：

- 如果 batch 还走高层 owned-result 路径，轻量指标很容易被接口层拖慢
- 一旦改成 in-place，很多点立刻就平了

但 `wilders` 又说明：

- 就算已经走 in-place
- 只要指标本体非常轻
- Rust 公共安全接口和 C 的最薄循环之间，仍然可能有一截固定尾差

## 给 Rust 新手的两个知识点

### 1. owned 返回值和借用引用不是一回事

`Vec<Vec<Real>>` 是“调用方最终拥有这块数据”的意思。  
它不是“把现成数组借出来看一下”。

所以它通常意味着：

- 分配
- 填充
- 返回所有权

如果你的目标是“通用好用”，这没问题。  
如果你的目标是“极限性能”，那就往往要再补一层 `in-place` 接口。

### 2. 性能优化要先区分“路径问题”和“尾差问题”

不是所有慢点都一样。

有些热点像 `wad / emv / vosc`：

- 先拆正确路径
- 效果立刻明显

有些热点像 `wilders`：

- 路径已经对了
- 剩下的是更小、更顽固的固定成本

这时候如果还按“再拆一次路径”去想，就会浪费时间。  
你得承认：问题类型已经变了。

## 这一步之后，怎么判断下一步该不该继续打 `wilders`

现在最合理的工程判断是：

- `vosc` 已经基本可以先放下
- `wilders` 还有优化空间，但它不再是“低垂果实”

如果继续打 `wilders`，下一步就不该只是继续机械改 loop，而应该认真判断：

- 要不要进一步优化公共契约检查
- 要不要接受它在安全接口下就是会比 C 薄循环多一截固定税

这就是这次切片最重要的结论：  
有些热点，一刀下去就平；有些热点，打到后面就变成“API 设计成本”问题了。
