# 只收 `cmo` 这一笔有效收益，不把不稳定的 `adosc` 混进来

这次提交有一个很重要的工程习惯：  
不是“既然都改了，就全都一起交”，而是只把**确认有效**的那部分留下。

前一轮我同时试了两类指标：

- `adosc`
- `cmo`

结果并不一样：

- `cmo` 的性能改进很稳定，属于明显收益
- `adosc` 的结果不稳定，尤其在 `65536` 上 sample 连目标时长都没跑到，说明这次 benchmark 结果不够可信

所以这次只提交 `cmo`，把 `adosc` 那部分试验性改动回掉，不让“半确定的东西”混进历史。

## 这次具体改了什么

在 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/oscillators.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/oscillators.rs) 里：

- `Cmo::run()` 不再走 `CmoStream::feed()`
- 新增 `run_in_place(...)`
- 新增 `run_cmo_batch(...)`

也就是说，`cmo` 的 batch 现在改成了真正的 direct batch kernel。

它的结构很适合这么做，因为 C 版本本来就是：

- 维护一个 `up_sum`
- 维护一个 `down_sum`
- 每次滑窗时，把窗口左端贡献减掉，再把新样本贡献加上

Rust 这次做的也是同样的事，只是保留了 Rust 的契约检查接口。

## 为什么 `cmo` 适合收，`adosc` 不适合

### `cmo`

focused benchmark 很清楚：

- `cmo 4096`: `2.14 -> 1.88`，倍率 `0.878x`
- `cmo 65536`: `2.60 -> 1.66`，倍率 `0.638x`

这说明：

- 它确实吃到了 direct batch kernel 的收益
- 结果稳定
- 方向明确

这就是该提交的那种切片。

### `adosc`

`adosc` 的这轮结果就没这么干净。  
尤其是 `65536` 那条，Rust sample 的中位时间只有几毫秒，根本没跑到设定的目标时长附近。

这意味着：

- 当前这次 benchmark 还不够稳
- 即使倍率好看或难看，也都不该急着下结论

所以这里最稳妥的处理不是“先交上去再说”，而是：

- 先回掉试验性改动
- 后面单独查清楚再做

## 这次验证结果

我最终保留并验证的是：

```bash
TI_BENCH_INDICATORS=cmo \
TI_BENCH_SIZES=4096,65536 \
TI_BENCH_TARGET_MS=120 \
TI_BENCH_CALIBRATION_MS=30 \
TI_BENCH_REPEATS=5 \
cargo run --release --bin indicator-bench-compare
```

结果：

- `cmo batch 4096`: `0.878x`
- `cmo batch 65536`: `0.638x`

也就是：

- 小输入已经回到阈值内
- 大输入直接比 C 更快

## 给 Rust 新手的两个知识点

### 1. 不是所有“看起来可能有收益”的改动都应该提交

做性能优化时，最容易犯的错之一是：

- 一次试了两个点
- 其中一个有收益
- 另一个结果模糊
- 最后把两者一起交了

这会污染历史，因为以后你根本不知道：

- 到底哪一部分是真收益
- 哪一部分只是噪声

更好的做法是这次这样：

- 有效的留下
- 不稳定的回掉

这会让 commit 历史更像一组可靠实验，而不是混杂的试错堆。

### 2. benchmark 不稳时，最好的动作常常是“先不要提交”

很多人以为性能优化最重要的是“不断往前改”。  
其实不是。

当 benchmark 告诉你：

- 这个点的 sample 时长不对
- 或者波动明显异常

最专业的动作往往是：

- 暂停
- 不提交
- 先把结果查清楚

这种克制本身就是工程能力。

## 这一步之后怎么继续

这次之后，`cmo` 可以从热点清单里划掉。  
而 `adosc` 则被明确标记成：

- 值得继续查
- 但还不该进入提交历史

这就是一条健康的性能迭代链路：  
不是每次都“大步前进”，而是每次只把真正确定的增量留下。
