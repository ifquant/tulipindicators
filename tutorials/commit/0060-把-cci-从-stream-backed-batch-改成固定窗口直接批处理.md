# 把 CCI 从 stream-backed batch 改成固定窗口直接批处理

这次提交处理的是 `cci`。

它不是那种“一改就从 2x 掉到 0.6x”的超级热点，而是比较典型的尾差型指标：

- 原来 `4096` 和 `65536` 都还偏慢
- 尤其长输入下，说明 batch 里持续有多余成本

## 为什么 `cci` 适合 direct batch kernel

看 C 版实现就能发现，它的核心非常直接：

1. 维护一个固定长度窗口和
2. 当前窗口平均值 `avg = sum / period`
3. 再扫一遍窗口，算平均绝对偏差
4. 输出 `(today - avg) / (0.015 * mean_deviation)`

所以它虽然不是“完全 O(1) 更新”的指标，但也没必要走 `VecDeque + stream.feed()` 那一套。

对 batch 来说，更自然的写法是：

- 用固定长度 ring buffer 存 `typprice`
- 用 running sum 维护窗口和
- 每步在当前窗口上做一段短扫描

## 这次改了什么

改动在 [`rust/src/indicators/indicator/oscillators.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/oscillators.rs)：

1. `Cci::run()` 不再走 `CciStream::feed()`
2. 新增 `run_in_place()`
3. 新增 `run_cci_batch(...)`

新的 batch 内核里：

- `values` 是固定长度 ring buffer
- `sum` 维护当前窗口和
- 每次命中输出区间时，再遍历 `values` 计算 mean deviation

这个思路和 C 版非常接近，只是保留了 Rust 安全接口的边界检查。

## 这次的 benchmark 结果

focused benchmark：

- `cci 4096 = 1.155x`
- `cci 65536 = 0.974x`

这里有一个很值得注意的点：

- 按 benchmark 报告里的默认回归线 `1.15x`，`4096` 还高了很小一点点
- 但按这轮性能收尾的实际目标 `1.2x`，它已经进线

也就是说，这笔不是“完全极致”，但已经把主要尾差收回来了。

## 给 Rust 新手的两个知识点

### 1. 不是所有滑窗指标都适合 `VecDeque`

`VecDeque` 很方便，但它更适合“你非常在意通用队列语义”的情况。

对性能敏感的批处理指标，如果窗口长度固定，常见替代方案是：

- `Vec<Real>` 作为固定长度环形数组
- 手动维护 `ring_index`

这么做通常会更贴近 C，也更容易减少抽象成本。

### 2. “收尾优化”经常不是追求绝对最快，而是把尾差压进目标线

像 `cci` 这类指标，真正花时间的部分本来就包含“扫窗口算偏差”。  
所以它不一定会像 `ao` 或 `fisher` 那样出现巨大收益。

这时更现实的目标是：

- 先把明显多余的 stream-backed batch 成本切掉
- 再看是否已经进入目标线

如果已经达标，就不一定要继续死抠。

## 这次的结论

`cci` 这笔是一个很典型的“把结构对齐 C，然后把尾差压进目标区间”的案例：

- 不是爆炸式提速
- 但把长输入拖慢的问题基本收回来了
- 小输入也已经接近目标线

这类提交对整体性能收尾很重要，因为它们能把最后那批“不是巨坑、但总在榜上”的指标一个个清掉。
