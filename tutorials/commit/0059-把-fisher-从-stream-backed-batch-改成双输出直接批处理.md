# 把 Fisher 从 stream-backed batch 改成双输出直接批处理

这次提交处理的是 `fisher`。

它之前在更稳的 focused benchmark 里已经不是轻微超线，而是比较明确的热点：

- `4096 = 1.30x`
- `65536 = 1.495x`

这说明它不是偶发噪声，而是 batch 路径本身还有明显的抽象税。

## 为什么 `fisher` 值得做

`fisher` 的结构虽然比单输出指标复杂一点，但本质上仍然很适合 direct batch kernel：

- 一个窗口最高值
- 一个窗口最低值
- 一个递推变量 `val1`
- 两个输出通道：`fisher` 和 `signal`

也就是说，难点不在算法，而在 Rust 里把“双输出 + 滑动极值窗口 + 低开销 batch 接口”写清楚。

## 这次改了什么

改动在 [`rust/src/indicators/indicator/oscillators.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/oscillators.rs)：

1. `Fisher::run()` 不再走 `FisherStream::feed()`
2. 新增 `run_in_place()`
3. 新增 `run_fisher_batch(...)`

新的 batch 内核仍然复用了现有的 `MonotonicQueue`：

- `max_queue`
- `min_queue`

但执行模型已经从“先生成 owned 输出 Vec，再包装返回”改成了直接写输出切片。

## 这次的一个典型 Rust 细节

第一次实现 `run_in_place()` 时，Rust 编译器直接报错：

```text
cannot borrow `*outputs[_]` as mutable more than once at a time
```

原因是我想同时拿：

- `&mut outputs[0][..]`
- `&mut outputs[1][..]`

Rust 不允许你这样直接从同一个切片里借出两份可变引用，因为它没法静态证明两者一定不重叠。

最后的正确写法是：

```rust
let (first, second) = outputs.split_at_mut(1);
let fisher_out = &mut first[0][..output_len];
let signal_out = &mut second[0][..output_len];
```

这等于显式告诉编译器：

- 左边是一段
- 右边是另一段
- 它们互不重叠

然后双输出借用就成立了。

## 给 Rust 新手的两个知识点

### 1. 多输出高性能接口里，`split_at_mut` 很常见

这不是为了写得花哨，而是为了让借用边界对编译器也足够明确。

在 C 里，你可能直接拿两个指针就写了；  
在 Rust 里，如果你想保持安全，又想保留低开销，就经常要把“不重叠”这个事实写出来。

### 2. batch 优化的收益不一定来自“换算法”，也可能只是“去掉输出包装层”

这次 `fisher` 的核心公式没有变，极值窗口的维护逻辑也没有变。

真正回收性能的地方是：

- 不再走 `stream.feed()`
- 不再先组装 `Vec<Vec<Real>>`
- 双输出直接写进调用方提供的 buffer

结果 focused benchmark 就直接变成：

- `4096 = 0.669x`
- `65536 = 0.660x`

## 这次的结论

`fisher` 这笔说明了一个很实用的规律：

> 对带多个输出通道的指标，direct batch kernel 往往不只是“少一次分配”，而是能同时切掉包装、状态对象和多输出复制成本。

所以以后看到“窗口逻辑不算特别重，但有多个输出，而且 batch 还在走 stream”的指标时，优先级应该拉高。
