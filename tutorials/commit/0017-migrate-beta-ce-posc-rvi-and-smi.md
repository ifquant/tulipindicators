# 迁移 Beta 指标：CE、POSC、RVI、SMI

## 背景

稳定指标已经全部迁到 Rust，beta 指标也只剩最后一小批。

这次先处理四个“复杂但 still 可控”的 beta：

- `ce`
- `posc`
- `rvi`
- `smi`

它们的共同点是都带滚动窗口，但还没有进入 `mama` 那种 Hilbert / MESA 级别的复杂状态机。  
所以这一步的意义不是单纯“再多迁 4 个名字”，而是把 beta 里常见的窗口型状态模式基本补齐。

## 主要目标

这次提交主要想完成三件事：

- 把 `ce`、`posc`、`rvi`、`smi` 接进 Rust registry
- 给这四个指标补 batch / stream 双路径
- 让测试框架能正确处理旧 beta golden 数据里的遗留脏尾值，而不是把基线文件问题误判成实现问题

做完之后，beta 就只剩最后一个最难的 `mama` 没迁。

## 改动概览

- 新增 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/beta_oscillators.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/beta_oscillators.rs)，实现 `posc`、`rvi`、`smi`
- 更新 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/beta_channels.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/beta_channels.rs)，加入 `ce`
- 更新 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/mod.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/mod.rs)、[`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/mod.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/overlay/mod.rs)、[`/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/registry.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/registry.rs)，把四个新指标注册进去
- 更新 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/golden_indicators.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/golden_indicators.rs)，支持解析 beta 测试里的 `};` 行尾，并对“比理论长度多 1 个尾值”的历史脏数据做裁剪

## 设计思路

### `ce`

`ce` 的核心不是公式本身，而是三层状态一起走：

- Wilder 风格 ATR
- 滚动最高价
- 滚动最低价

Rust 版本没有照搬 C 的“索引 + 重扫”写法，而是用单调队列维护窗口内最高/最低，再单独维护 ATR 初始化和递推。

### `posc`

`posc` 的关键是“滚动线性回归斜率 + 投影区间 + EMA”。

这一类指标最容易写错的地方，不是 EMA，而是线性回归窗口里 `x=1..period` 的坐标定义。  
如果这里偏一位，后面整个指标都会漂。

### `rvi`

`rvi` 看起来像“波动率指标”，但实现上更接近：

- 先做滚动线性回归
- 看当前点相对拟合线的偏差
- 把正偏差和负偏差分别平滑

这个结构很像 `rsi`，只是把“涨跌幅”换成了“偏离拟合线的平方”。

### `smi`

`smi` 是典型的“双层 EMA + 滚动 HH/LL”指标：

- 先取 `q_period` 窗口里的最高高和最低低
- 算出当前位置相对中轴的偏移
- 对分子分母都做两层 EMA

它特别适合用来练习“一个指标里并行维护多条状态线”的写法。

## 对新手有用的 Rust 知识

### 1. `Option<T>` 很适合表达“状态还没初始化”

这次 `ce` 里有一个很典型的例子：

- 第一根 K 线时，`previous_close` 还不存在
- ATR 在窗口攒满之前，也还不存在

Rust 里最自然的表达方式就是：

```rust
previous_close: Option<Real>
atr: Option<Real>
```

这比用特殊值比如 `-1`、`NaN`、或者额外整数标志位更清楚。  
你在读代码时，只要看到 `Option`，就应该立刻想到：

- 这个状态可能还没准备好
- 调用方必须处理 `Some` 和 `None` 两种情况

这是 Rust 很重要的一种“把状态显式化”的风格。

### 2. `VecDeque` 很适合滑动窗口

这次 `posc` 里需要保留最近 `period` 个高点和低点。  
如果你用普通 `Vec`，每次窗口前移时常常要做头删，代价高，而且代码容易脏。

`VecDeque` 更适合这种场景：

- `push_back()` 往尾部塞新值
- `pop_front()` 把过期值从头部丢掉

它本质上就是“适合队列语义的数组”。  
对于指标实现来说，只要你看到“最近 N 个样本”，就可以优先想到它。

## 这次一个很重要的调试结论

这次真正第一个失败点不是数学实现，而是测试基线文件。

`ce` 在 [`/Users/dev/workspace2/hc_apps/tulipindicators/c/tests/extra.txt`](/Users/dev/workspace2/hc_apps/tulipindicators/c/tests/extra.txt) 里的输出尾部多了 1 个脏值。  
按 `period=22` 和输入长度 61 计算，理论输出应该是 40 个，但文件里给了 41 个。

这里的处理原则很重要：

- 先用公式和 C 逻辑确认实现是否正确
- 再判断是不是 fixture 本身坏了
- 不要为了迎合脏数据去改坏实现

最后采取的是“在测试解析层做窄兼容裁剪”，只处理“比理论长度多 1”的历史尾值问题，不动指标实现。

## 验证

实际跑过：

- `cargo fmt --all`
- `cargo test`
- `cargo run --release --bin indicator-bench`
- `make -B smoke`

结果：

- Rust golden tests 通过
- Rust stream / batch 一致性测试通过
- benchmark 已包含 `ce`、`posc`、`rvi`、`smi`
- C 侧 smoke 继续通过，beta 仍按原有行为显示 warning

## 未覆盖项

- `mama` 还没迁，它仍然是最后一个 beta 缺口
- 这次没有修改 C 侧 beta 导出逻辑，所以 `make smoke` 里的 beta warning 还保持原状
- 对 `ce` 的历史 golden 脏尾值只做了测试层兼容，没有回写修正原始文本数据
