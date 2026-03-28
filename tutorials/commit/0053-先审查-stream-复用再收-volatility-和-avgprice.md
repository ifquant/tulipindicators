# 先审查 stream 复用，再收 `volatility` 和 `avgprice`

这次不是先盯着一个指标死抠，而是先换了一个更有效率的策略：

1. 先审查整个仓库里还有哪些 `run() -> stream.feed()` 复用
2. 再从里面挑最像“低垂果实”的点下手

这一步很重要，因为前面几轮已经反复证明了一件事：

> 对很多批处理指标来说，最大的性能问题不是数学公式，而是 batch 路径误用了 stream 接口。

所以继续做之前，先审查一遍剩余的 stream 复用，比继续猜热点更靠谱。

## 这次先查到了什么

我用 `rg` 扫了一遍所有 `run()` 里直接：

- `let mut stream = ...`
- `stream.feed(inputs)`

的指标。

结果表明，仓库里还剩一批这种模式，比如：

- `volatility`
- `ao`
- `vhf`
- `ultosc`
- `mfi`
- `ad`
- `adosc`
- `aroon`
- `cci`
- `cmo`
- `md`
- `qstick`

这不代表它们全都一定该改，但至少说明：

- 这些指标的 batch 路径值得重点怀疑
- 继续查热点时，优先去看这批，比在已经是 direct batch 的指标上瞎试更划算

## 这次落地了哪两个点

### 1. `volatility`：典型的 stream-backed batch

在 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/volatility.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/volatility.rs) 里，原来的 batch 路径还是：

- `run()` 创建 `VolatilityStream`
- 再调用 `stream.feed(inputs)`

但 C 版本 [`/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/volatility.c`](/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/volatility.c) 是非常直接的滑窗批处理。

所以这次把 Rust 版改成了：

- `run_volatility_batch(...)`
- `run_in_place(...)`

而不再复用 stream。

这个点的收益很明确，因为它完全符合我们前面总结出来的热点模式。

### 2. `avgprice`：不是 stream 问题，而是“超轻指标 + 通用 helper”问题

`avgprice` 和 `volatility` 不一样。  
它本来就有 `run_in_place`，但它用的是通用的 `quad overlay helper`，里面有：

- 多层 `zip`
- 函数指针 `op`

对复杂指标来说，这种抽象很合理。  
但对 `avgprice` 这种几乎只有一次加法和一次乘法的超轻指标，这种通用抽象的固定成本就会被放大。

所以这次在 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/prices.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/prices.rs) 里：

- `AvgPrice` 改成了专门的 direct batch loop
- 不再走通用 `run_quad_overlay(...)`
- 顺手把没再使用的 quad helper 删掉，保持 `clippy` 干净

## 这次 benchmark 的结果

我跑的是：

```bash
TI_BENCH_INDICATORS=volatility,avgprice \
TI_BENCH_SIZES=4096,65536 \
TI_BENCH_TARGET_MS=120 \
TI_BENCH_CALIBRATION_MS=30 \
TI_BENCH_REPEATS=5 \
cargo run --release --bin indicator-bench-compare
```

结果是：

- `volatility 4096`: `1.228x -> 0.909x`
- `volatility 65536`: `1.224x -> 1.062x`
- `avgprice 4096`: `3.419x -> 1.150x`
- `avgprice 65536`: `1.628x -> 1.083x`

这里面最值得注意的点有两个：

### `volatility`：完全证明了 stream 复用审查是值得的

它就是典型的“看起来没那么显眼，但一拆 batch 就明显回收”的点。  
这说明前面那张 stream 复用列表不是装饰，而是真能指导优化顺序。

### `avgprice`：证明“超轻指标”的热点不只一种

`avgprice` 慢，不是因为 stream，而是因为：

- 指标本体太轻
- 通用 helper 的固定开销显得太重

这类问题和 `wad / emv / volatility` 不一样。  
它提醒我们：后面遇到轻量 overlay 类指标时，不能只问“是不是 batch 复用了 stream”，还要问：

- 是不是通用抽象把超轻公式拖慢了

## 给 Rust 新手的两个知识点

### 1. 性能审查不是先改代码，而是先找模式

新手很容易一看到慢点，就立刻去改那个函数。  
更高效的做法其实是：

1. 先看同类实现有没有共同模式
2. 先找“复用 stream”的那批
3. 先收最像低垂果实的一组

这样你改的不是一个点，而是一整类问题。

### 2. 通用抽象有价值，但超轻热路径不一定适合继续共用

`avgprice` 这个例子很典型。  
通用 overlay helper 很整洁，也很好复用，但当一个公式轻到几乎只有几步算术时：

- 函数指针
- 多层 iterator / zip
- 统一包装层

都可能比公式本身还贵。

这不意味着“抽象是错的”，而是意味着：

- 通用抽象适合大多数路径
- 极轻的热点路径有时要专门化

这是工程上的平衡，不是风格对错。

## 这一步之后怎么看 stream 复用审查结果

现在可以更明确地说：

- `stream 复用审查` 是有效方法
- 但它不是唯一方法

它能帮你抓出像 `volatility` 这样的低垂果实。  
但对 `avgprice` 这种点，真正的问题已经转成了“超轻指标上的通用抽象税”。

所以后面继续做时，判断顺序应该是：

1. 先看 batch 是否误复用了 stream
2. 如果没有，再看是不是超轻指标被通用 helper 拖慢
3. 最后才去怀疑公式本身
