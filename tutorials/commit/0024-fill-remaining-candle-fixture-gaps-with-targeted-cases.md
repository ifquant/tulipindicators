# 用定向 case 补齐 candle 历史 fixture 的剩余缺口

## 这次为什么还要继续补

上一笔我们已经把 Rust candle 测试补到了：

- `period` parity
- metadata parity
- `shooting_star` 的一个定向正例

但那时还有一个问题没有完全制度化：

- 这只是补了一个 case
- 并没有把“历史 fixture 还缺哪些 pattern”变成测试里会自动检查的事实

如果以后 `candles.txt` 继续遗漏某个 pattern，或者有人新加了 candle pattern 但忘了加样例，测试不会直接把这个结构性缺口指出来。

所以这次做的不是再随手加一条样例，而是把“缺口补丁”变成一个明确机制。

## 这次做了什么

### 1. 显式计算 `candles.txt` 的正例覆盖缺口

现在 [`/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/candles_parity.rs`](/Users/dev/workspace2/hc_apps/tulipindicators/rust/tests/candles_parity.rs) 会先遍历：

- Rust 侧全部 candle metadata
- `c/tests/candles.txt` 里所有正例 expectation

然后找出：

- 哪些 pattern 在历史 fixture 里没有任何正例命中

当前计算出来的唯一缺口就是：

- `shooting_star`

这一步的价值在于，缺口不再靠人工记忆，而是测试自己算出来。

### 2. 把定向 case 变成一个小型用例表，而不是单个散落测试

现在不是只有一个特判函数了，而是有一组显式的 `targeted_candle_cases()`。

每个定向 case 都包含：

- pattern 名字
- pattern bit
- 运行配置
- 输入 OHLC
- 预期命中位置

这样做以后，如果历史 fixture 再缺别的 candle pattern，只要往这个表里补一项就行，不需要再复制粘贴一整段新的测试结构。

### 3. 新增“缺口清单必须等于定向 case 清单”的断言

这次最关键的一条测试不是行为对账，而是结构约束：

- 历史 fixture 缺的正例 pattern
- 必须和定向 case 覆盖的 pattern
- 完全相等

这条断言的好处是：

- 如果未来 `candles.txt` 补上了 `shooting_star`，但我们忘了删定向 case，测试会提示两边不一致
- 如果未来又出现新的缺口，但没人补定向 case，测试也会立刻报错

也就是说，它强迫“历史 fixture”和“补丁测试”保持同步。

### 4. 保留 C/Rust 对账，而不是只做 Rust 自测

定向 case 仍然不是只断言 Rust 自己命中了。  
它继续同时验证：

- Rust 全量 engine 的结果
- Rust 单 pattern helper 的结果
- C 全量 engine 投影到同一 bit 的结果

这样定向 case 不是一个脱离基准的“自嗨样例”，而是继续站在 C/Rust parity 上。

## 为什么这比单条补丁测试更好

如果只补一个 `shooting_star` 测试，短期当然也能工作。  
但它的问题是：测试只知道“这里有个 case”，不知道“它为什么存在”。

现在的写法把这个因果关系写成了代码：

- 因为历史 fixture 缺一个正例
- 所以需要一个 targeted case

这比单纯多一条样例更重要，因为它让后面维护的人能看懂：

- 这不是随手加的 case
- 它是在填一个已知覆盖空洞

## 新手知识点 1：把“临时补丁”升级成“受约束的机制”

新手很容易在发现缺口后写一个修补测试，然后就结束了。  
这种写法短期有用，但长期容易变成：

- 为什么这里有这个测试？
- 它和别的 fixture 是什么关系？
- 现在还需要它吗？

更好的方式是把补丁背后的规则也写进代码，比如这次的：

- 缺口列表必须等于 targeted case 列表

这样测试就不只是“多测一点”，而是在维护一条明确规则。

## 新手知识点 2：测试数据也有“主干”和“补丁层”

你可以把这次 candle 测试看成两层：

- 主干层：历史 fixture，比如 `c/tests/candles.txt`
- 补丁层：专门填主干空洞的 targeted cases

这是一种很实用的测试设计方法。  
原因是历史 fixture 往往：

- 覆盖面广
- 可读性一般
- 不方便频繁改

而补丁层可以：

- 很小
- 很针对
- 明确解释为什么存在

以后你做测试时，如果遇到“老测试集很大但总缺几个边角”，这就是一个很好用的组织方式。

## 这次之后的状态

现在 candle 这条线不只是“补了一个 `shooting_star` case”，而是有了一个可持续的补缺机制：

- 测试会自动计算历史 fixture 的正例缺口
- 定向 case 必须和这些缺口严格对应
- 每个定向 case 仍然继续做 C/Rust parity

这意味着剩余的 candle fixture 缺口已经不是靠记忆维护，而是被测试本身接管了。
