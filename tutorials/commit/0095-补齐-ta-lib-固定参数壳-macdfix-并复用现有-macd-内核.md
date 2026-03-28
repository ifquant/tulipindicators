# 补齐 TA-Lib 固定参数壳：`macdfix`

这次补的不是一个“全新算法”，而是 TA-Lib 里一个很典型的固定参数 API 壳：

- `macdfix`

它的语义不是“另一种 MACD 算法”，而是：

- 固定 `short_period = 12`
- 固定 `long_period = 26`
- 只让调用方传 `signal_period`

所以这次最重要的设计判断不是“怎么再写一份 MACD”，而是“怎么复用现有 `macd` 内核，同时把 TA-Lib 风格的接口补齐”。

## 这次改了什么

### 1. C 侧补成一个很薄的 wrapper

新增：

- `/Users/dev/workspace2/hc_apps/tulipindicators/c/indicators/macdfix.c`

它只做两件事：

- `start` 时把 `{12, 26, signal_period}` 组装好，再转发给 `ti_macd_start`
- `run` 时把同样的固定参数组装好，再转发给 `ti_macd`

也就是说，C 侧没有复制一份 MACD 实现，只是补了一个固定参数壳。

### 2. Rust 侧也按同样思路实现

更新了：

- `/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/macd.rs`
- `/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/indicators/indicator/mod.rs`
- `/Users/dev/workspace2/hc_apps/tulipindicators/rust/src/registry.rs`

Rust 这次新增了：

- `MACDFIX_METADATA`
- `MacdFix`
- `parse_signal_only(...)`
- `MacdStream::new_fixed(...)`

但真正的 batch 内核仍然直接复用：

- `run_macd_batch(...)`

这样做的好处是：

- `macd`
- `macdfix`

不会在后续优化和修 bug 时各走各的。

### 3. stream metadata 也一起对齐

这次顺手补了一个很容易漏掉的小点：`macdfix` 的 stream 不能继续报自己是 `macd`。

所以现在 `MacdStream` 会带一份 `metadata` 指针：

- 普通 `macd` stream 返回 `MACD`
- `macdfix` stream 返回 `MACDFIX`

这能避免后面 benchmark、日志或调试界面里把固定参数壳误标成普通 `macd`。

### 4. parity 和 benchmark 覆盖一起补上

更新了：

- `/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/talib_missing_parity.rs`
- `/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/talib_missing_benchmark.rs`
- `/Users/dev/workspace2/hc_apps/tulipindicators/tutorials/ta-lib-semantic-mapping.md`

这里最重要的 parity 思路不是“手算一整组 MACD 值”，而是直接断言：

- `macdfix(signal_period)`
- 必须等价于 `macd(12, 26, signal_period)`

这比单独维护两份期望值更稳，因为它直接锁定了语义本身。

## 为什么这次不复制一份 MACD

因为这里缺的不是算法，而是接口。

如果复制一份 `macdfix` 内核，会立刻出现两个问题：

- `macd` 优化了，`macdfix` 忘了同步
- `macd` 修 bug 了，`macdfix` 也要再修一次

而 TA-Lib 这类“固定参数壳”最适合的实现方式，本来就应该是：

- 把参数固定
- 复用主内核

## Rust 新手知识点 1：metadata 也是行为的一部分

很多 Rust 新手会把 metadata 当成“只是文档字符串”。

但在这种指标库里，metadata 往往参与：

- registry 查找
- benchmark 标记
- stream 输出标识
- 调试信息

所以这次 `MacdStream` 带 `metadata` 指针，不是装饰，而是为了保证 `macd` 和 `macdfix` 的行为边界是清楚的。

## Rust 新手知识点 2：固定参数壳常常比复制算法更高级

看到一个“新 API 名”时，不一定要马上新写一个内核。

更好的顺序往往是：

1. 先判断它是不是已有算法的特化
2. 如果是，就优先做 wrapper
3. 只在 wrapper 不够表达语义时，才复制或拆分内核

这是一种很常见的设计技巧：

- 复用已经验证过的核心逻辑
- 把新功能实现成“更薄的一层”
- 降低未来维护成本

## 结果

这次之后：

- TA-Lib 语义映射里的 `macdfix` 已经不再是缺失项
- C 和 Rust 都有正式实现
- parity 会直接检查它和 `macd(12,26,signal_period)` 的等价性
- benchmark 管线也已经覆盖到它

这次的重点不是“多实现了一个复杂指标”，而是把 TA-Lib 那类“固定参数语义壳”接进了 Tulip 的 C/Rust 双实现体系。
