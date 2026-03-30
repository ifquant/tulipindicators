## 为什么要改

`mavp` 是这轮 focused 热点里最顽固的一项。  
前面已经确认过两件事：

- benchmark 默认 `mavp` 走的是 `ma_type = 0`，也就是 `SMA`
- 默认 `periods` 输入只会命中 `2..8` 这 7 个 period

也就是说，问题不在“period 太分散”，而在 Rust 原来的实现模型：

- 先按 period 做 cache
- 每命中一个新 period，就整段生成一条完整 MA 序列
- 再按当前位置回读结果

这个思路对复杂 MA 类型还能接受，但对 `SMA` 来说太浪费了，因为 `SMA` 本来就可以直接用前缀和在 O(1) 时间回答任意窗口和。

## 改了什么

这次只改：

- `rust/src/indicators/overlay/talib_ma.rs`

具体做法是：

1. 在 `run_mavp_batch(...)` 里给 `TalibMaType::Sma` 加专门快路径
2. 新增 `run_mavp_sma_batch(...)`
3. 用一条 prefix sum 数组代替“为每个 period 生成一整条 SMA 序列”

核心公式很简单：

```rust
let sum = prefix[index + 1] - prefix[index + 1 - period];
output[out_index] = sum / period as Real;
```

这样每个输出点只需要：

- clamp 一次当前 period
- 做两次 prefix 读取
- 做一次减法和一次除法

不再需要：

- 为 `2..8` 这几个 period 分别生成完整的 `Vec`
- 再回头按当前位置索引这些临时序列

## 为什么这次比前两次实验更有效

前两次我试过：

1. 把 `BTreeMap` cache 改成更扁平的 period cache
2. 给 `SMA` 主路径加更薄的 batch dispatch

这两次都没站住。  
原因是它们虽然减少了一部分调度成本，但都还保留着“为每个 period 先生成一整条序列”的核心模型。

这次真正拿到收益，是因为直接把 `SMA-MAVP` 的算法模型改成了更适合变量窗口的形式，而不是只改外围容器或 dispatch。

## 结果

我实际跑了：

```bash
cargo fmt --all
cargo test --test talib_missing_parity --test stable_parity
cargo clippy --all-targets --all-features
TI_BENCH_INDICATORS=mavp TI_BENCH_SIZES=4096,65536 TI_BENCH_TARGET_MS=180 TI_BENCH_CALIBRATION_MS=50 TI_BENCH_REPEATS=11 cargo run --release -q --bin indicator-bench-compare
```

focused 结果：

- `mavp 4096 = 0.793x`
- `mavp 65536 = 0.947x`

也就是说：

- `4096` 已经明显快于 C
- `65536` 也回到 parity 区间

这说明这次终于踩中了 `mavp` 的主成本。

## 给新手的两个提醒

### 1. 不要把“缓存命中率”错当成性能问题的全部

前面我一开始以为 `mavp` 慢是因为 `BTreeMap`。  
但真正的大头不是“怎么找 period”，而是“找到 period 后要不要整段重算一条均线序列”。

缓存结构只是外围，算法模型才是内核。

### 2. 对变量窗口问题，前缀和常常比“多条固定窗口序列”更自然

`SMA` 是最典型的例子：

- 固定窗口 `SMA` 可以用 rolling sum
- 变量窗口 `SMA` 更适合 prefix sum

所以遇到“窗口长度每个样本都可能变”的问题时，先问自己：

- 是不是应该换算法模型
- 而不是先去抠容器、接口、调度层
