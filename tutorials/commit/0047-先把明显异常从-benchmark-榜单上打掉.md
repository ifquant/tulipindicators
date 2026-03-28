# 先把明显异常从 benchmark 榜单上打掉

## 这次改动的背景

前一轮完整 benchmark 里，`sma`、`bbands`、`md`、`adxr` 这一组出现了很刺眼的异常值。  
这类指标本身并不复杂，如果它们在 Rust 里明显慢于 C，通常先不要怀疑公式本身，而要先怀疑一件事：是不是还在走“高层接口 + 分配”的慢路径。

这次改动的目标很直接：

- 先把这组明显异常从榜单上打掉
- 让 batch benchmark 尽量测到真正的算法内核，而不是测到 `Vec` 包装成本
- 顺便验证 `adxr` 那个夸张倍率到底是真热点，还是 benchmark 噪声

## 这次主要做了什么

### 1. 给 `sma` 补了真正的 `run_in_place`

之前 `sma` 的 batch 路径虽然公式已经很薄，但 benchmark 还是会走高层 `run()`，返回 `Vec<Vec<Real>>`。  
这次直接补了 `run_in_place`，让 benchmark 可以直接写入调用方提供的输出缓冲区。

这样 `sma` 的 batch 路径就更接近 C 的模式了：

- 调用方先准备输出空间
- 指标内部只做滑窗求和
- 不额外创建结果 `Vec`

### 2. 给 `bbands` 补了 in-place 三输出路径

`bbands` 有三个输出：`lower / middle / upper`。  
以前 Rust 这边虽然公式是直接滑窗，但最后仍然是构造三个 `Vec` 再返回。  
现在改成直接写到三块输出切片里，这样 benchmark 更公平，也更接近 C 的指针写法。

### 3. 把 `md` 的 batch 路径从 stream 风格剥离出来

`md` 的数学复杂度本来就不低，因为每个窗口都要重新算一遍 mean deviation。  
但以前 Rust 版本还是先 new 一个 stream 状态对象，再喂数据。  
这次给它加了直接 batch kernel：

- 维护当前窗口和
- 每次移动窗口后，只对当前切片重新计算 mean deviation

这样不能改变它的理论复杂度，但能把不必要的状态机和分配成本去掉。

### 4. 把 `adxr` 改成更接近 C 的 direct batch kernel

`adxr` 原来那条 batch 路径内部还在用 `VecDeque` 存历史 ADX 值。  
这次改成了更接近 C 版的方式：

- 直接维护 `dmup / dmdown`
- 直接滚动更新 `adx`
- 用一个固定长度的环形数组保存 `period - 1` 个历史值

这样做的意义不是“更 Rust”，而是“更接近这个算法在 C 里的真实高性能形态”。

## 结果怎么样

这轮 focused benchmark 的结果很清楚：

- `sma batch 4096`: `0.940x`
- `sma batch 65536`: `0.198x`
- `bbands batch 4096`: `0.927x`
- `bbands batch 65536`: `0.468x`
- `md batch 4096`: `0.287x`
- `md batch 65536`: `0.329x`

也就是说：

- `sma`
- `bbands`
- `md`

这三个之前的明显异常，现在都已经掉出回归区了。

`adxr` 这里要多解释一句。  
如果把它和别的方向性指标一起跑，结果仍然会有抖动；但单独用更重的 benchmark 参数跑时，已经能回到：

- `adxr batch 4096`: `1.106x`
- `adxr batch 65536`: `0.841x`

这说明之前那个特别离谱的倍率，大概率不是纯算法问题，而是基准噪声和混跑干扰叠加出来的异常。

## 给 Rust 新手的两个知识点

### 知识点 1：`&mut [&mut [T]]` 是“多输出缓冲区”很实用的写法

这次 `bbands` 的 in-place 接口里，输出参数是：

```rust
outputs: &mut [&mut [Real]]
```

它的意思不是“一个二维数组”，而是“一个装着多个可变切片的列表”。  
这很适合指标库这种场景，因为很多指标有多个输出：

- `bbands` 有 3 个输出
- `macd` 有 3 个输出
- `stoch` 有 2 个输出

这种写法的好处是：

- 调用方自己控制内存
- 指标实现不用额外分配
- 不同输出长度规则也更容易统一校验

### 知识点 2：性能优化时，先分清“算法复杂度”和“接口层成本”

`md` 是个很好的例子。  
它本来就需要对窗口里的每个值重新累加偏差，所以复杂度不低。  
但是这不代表它就该慢很多，因为还要区分两件事：

- 算法本身就需要做多少工作
- 代码外围有没有额外状态机、分配、包装

优化时，先把第二类成本切掉，才能更准确地看见第一类成本。  
这也是为什么这次我们先给它补 direct batch kernel，而不是一上来就改公式。

## 这次验证怎么做的

- `cargo fmt --all`
- `cargo test`
- `cargo clippy --all-targets --all-features`
- `TI_BENCH_INDICATORS=adxr,sma,bbands,md TI_BENCH_SIZES=4096,65536 TI_BENCH_TARGET_MS=300 TI_BENCH_REPEATS=3 cargo run --release --bin indicator-bench-compare`
- `TI_BENCH_INDICATORS=adxr TI_BENCH_SIZES=4096,65536 TI_BENCH_TARGET_MS=1000 TI_BENCH_REPEATS=7 cargo run --release --bin indicator-bench-compare`

## 下一步准备做什么

这轮已经证明一件事：异常榜单里有不少点，本质上只是没有吃到高性能接口。  
下一步就继续沿着这个思路，把剩下还在回归区的真热点往下压，尤其是：

- `dx`
- `dm`
- `wilders`
- `kama`
- `kvo`

先把“还能明显看见的热点”一批批清掉，再看最后剩下的是不是只能靠更深的算法级优化。
