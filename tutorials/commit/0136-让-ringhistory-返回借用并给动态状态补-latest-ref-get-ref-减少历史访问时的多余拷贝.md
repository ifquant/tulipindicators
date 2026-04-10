# 让 `RingHistory` 返回借用，并给动态状态补 `latest_ref/get_ref`，减少历史访问时的多余拷贝

这笔改动解决的是 state API 里一个很隐蔽但持续存在的接口税：历史数据明明已经在 ring buffer 里了，但每次 `latest()` / `get()` 都先 clone 一份再交出去。

对 typed state 来说，这个问题不大，因为很多输出是 `f64` 或小 tuple，clone 代价几乎可以忽略。但对 `DynamicIndicatorState` 来说，历史项是 `Vec<Real>`。如果上层只是想“看一眼最新值”或者“按索引读一下历史”，之前每次访问都会重新分配并复制一份向量。

## 这次怎么收

1. `RingHistory<T>` 的内部访问改成返回 `Option<&T>`。
2. typed `IndicatorState` 实现继续对外返回原来的拥有值接口，只是在边界上做 `.cloned()`，所以公开 trait 不变。
3. `DynamicIndicatorState` 新增：
   - `latest_ref(&self) -> Option<&[Real]>`
   - `get_ref(&self, index_from_latest: usize) -> Option<&[Real]>`
4. `DynamicIndicatorState::latest/get` 仍然保留原来的拥有值接口，兼容现有调用方。
5. 动态 batch update 内部在“已有历史里取最新输出”这个路径上，先走借用，再按需要 `to_vec()`，避免底层 ring buffer 自己先 clone 一次。

## 为什么不直接改 public trait

`IndicatorState` 现在的 typed API 已经被测试、文档和调用方使用。如果直接把 trait 改成借用返回，会把一大批 typed state 的签名一起改掉，这不是一笔“收噪声”的小改动，而是公开 API 破坏。

这次的做法是先把内部历史容器改成更合理的零额外拷贝语义，再只给真正受益最大的动态接口补借用视图。这样收益先落地，兼容性也不受伤。

## 结果

- `RingHistory<Vec<Real>>` 不再强制每次读取都 clone。
- `DynamicIndicatorState` 用户如果只需要读，不需要拥有，就可以直接走 `latest_ref/get_ref`。
- typed state API 保持原样，不需要调用方迁移。
