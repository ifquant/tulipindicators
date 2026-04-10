这次继续做低风险去重，只收最明显的 helper 重复。

收掉的是三处：

- `atr.rs` 里的 `true_range`
- `di.rs` 里的 `true_range`
- `rsi.rs` 里的 `rsi_value`

它们现在都统一回到了：

- `rust/src/indicators/shared.rs`

这样做的价值不在“少了几行代码”，而在于后面如果要修：

- `true_range` 的边界行为
- `rsi_value` 的零分母处理

就不会再出现“修了一处，忘了另一处”的情况。

这次刻意没有顺手去碰更大的重复：

- `shared::RsiState`
- `rsi.rs::RsiStream`

因为那不是简单 helper 去重，而是状态结构是否要合并的问题。  
这个要单独分析，不能和这次混在一笔里。

一个工程上的经验：

去重要优先挑“同签名、同语义、同边界处理”的东西先收。  
这种去重几乎不改变阅读模型，也最不容易引入行为偏差。
