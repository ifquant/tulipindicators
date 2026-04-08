这次做的是状态接口文档收口，不是继续补实现。

目标有两个：

1. crate 根文档里直接能看到可运行示例
2. README 不再承担全部状态接口说明

## 1. 为什么要补 crate-level rustdoc

之前状态接口已经能用，但入口信息分散在：

- 提交教程
- 测试文件
- README

这对仓库内协作还够用，但对 crate 使用者不够直接。  
所以这次把两类最核心示例放进了 `rust/src/lib.rs` 顶部文档：

- typed state
- dynamic state

这样：

- 本地 `cargo doc`
- docs.rs 风格阅读
- IDE 悬停 crate 根模块

都能直接看到最基本的使用方式。

## 2. 为什么还要再单独做一页 `state-api.md`

README 已经补过一轮状态接口说明，但继续往里塞会让入口页越来越重。

所以这次把更完整的状态接口说明抽成：

- `tutorials/state-api.md`

它负责解释：

- batch / in-place / state 三层分工
- ring history 的行为
- typed state 用法
- dynamic state 用法
- `seed_columns` / `seed_rows` 的区别

而 README 只保留高层入口和跳转链接。

## 3. 这次的边界

这次没有改状态接口实现，也没有扩 typed state 覆盖面。  
它是纯文档收口，目的是让前面已经完成的能力更容易被发现和正确使用。

## 4. 一个值得记下来的经验

当一个接口已经形成体系时，最容易被低估的不是“功能还差一点”，而是“使用入口还不够清楚”。

对状态接口这种分层能力来说：

- README 适合高层导览
- rustdoc 适合最短上手路径
- 独立文档适合解释设计边界和选择标准

这三层同时存在，才比较像真正可用的公共接口文档。
