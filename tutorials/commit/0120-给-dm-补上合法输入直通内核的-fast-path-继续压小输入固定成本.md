## 背景

前一刀已经把 `dm` 的 steady-state 递推改成更接近 C 的形状了：

- 两条递推都改成了 state-side `mul_add`
- steady-state 输出也不再走最厚的索引形状

那之后的结果是：

- `4096 ≈ 1.28x`
- `16384 ≈ 1.10x`

也就是说，长输入已经很接近 parity，剩下更像小输入固定成本问题。

## 为什么这次不再动数学核

`dm` 的 C/Rust 汇编对照已经说明：

- 递推链本身已经基本收对了
- 剩下的主要差距，不像是又少了某条 `fmadd`

更像是 `run_in_place(...)` 入口那层：

- `double_input(...)`
- `validate_output_slices(...)`
- `ensure_output_len(...)`

对小输入更敏感。

## 这次改了什么

这次没有改变错误语义，也没有硬写新的错误构造。

做法是：

- 在 `run_in_place(...)` 前面加一条 fast path
- 只要碰到“合法正常输入”：
  - 2 路输入
  - 2 路输出
  - 输入长度一致
  - `period` 解析成功
  - 输出切片长度足够

就直接调用 `run_dm_batch(...)`。

一旦不满足，就仍然回退到原来的：

- `double_input(...)`
- `validate_output_slices(...)`
- `ensure_output_len(...)`

所以这笔的重点不是“删掉校验”，而是：

- 把热路径和错误路径分开
- 让 benchmark 常见路径不必每次都穿过那套通用 helper

## 结果

focused benchmark：

- `dm 4096 = 1.209x`
- `dm 16384 = 1.103x`

和前一版相比，又往下压了一截，尤其是 `4096`。

## 经验

这种 fast path 很适合：

- 数学核已经被证明确实有效
- 长输入已经快接近 parity
- 只剩小输入 still-slow

如果还在这种阶段继续改数学核，往往回报会越来越小；先把入口正常路径写薄，通常更划算。  
