# 把 DX 从 stream-state 批处理改成更贴近 C 的直接循环

这次收的是 `dx`。

它和前面那批“只差一个 `fmadd`”的指标不一样。`dx` 真正的问题不是某一条乘加没 fused，而是 Rust 的 batch 路径结构本身就比 C 厚一层：

- C：直接批处理循环
- Rust：`run()` 里 new 一个状态机，再逐点 `feed()`，最后 `push` 输出

这种差异在性能上往往比“某条汇编指令是否 fused”更大。

## 这次先从 C 和 Rust 的结构差异看

`C` 的 `dx` 很直接：

1. 先做 warmup，把 `dmup / dmdown` 累起来
2. 输出第一个 `dx`
3. steady-state 循环里每次：
   - 算本轮方向变化
   - 做 Wilder 风格平滑
   - 算 `fabs(up - down) / (up + down) * 100`

也就是一条标准的 batch kernel。

而 Rust 原来是：

```rust
let mut state = DirectionalMovementState::new(period);
for (...) {
    if let Some((up, down)) = state.feed(...) {
        output.push(directional_ratio(up, down));
    }
}
```

这会带来几层额外成本：

- 通过 `Option` 表达 warmup
- 每次走一次状态对象的方法调用
- batch 结果通过 `Vec::push` 累出来

这些在语义上没错，但和 C 的热路径明显不是一个层级。

## 这次怎么改

这次不是继续抠 wrapper，也不是去猜某条 `fmadd`，而是直接把 batch 路径写成和 C 更接近的结构：

```rust
for index in 1..period {
    let (current_up, current_down) = directional_movement(...);
    up += current_up;
    down += current_down;
}

output[0] = directional_ratio(up, down);

for index in period..high.len() {
    let (current_up, current_down) = directional_movement(...);
    up = up.mul_add(per, current_up);
    down = down.mul_add(per, current_down);
    output[out_index] = directional_ratio(up, down);
}
```

这次一并补了：

- `run_in_place()`
- 共享 `run_dx_batch(...)`

这样 benchmark 主路径终于不再被 stream-state 结构拖着走。

## 为什么这里也顺手用了 `mul_add`

`dx` 的 steady-state 平滑是：

```rust
up = up * per + current_up;
down = down * per + current_down;
```

这和前面已经验证过的 EMA / Wilder 家族一样，也适合写成：

```rust
up = up.mul_add(per, current_up);
down = down.mul_add(per, current_down);
```

不过这次的主收益来源不是这条指令本身，而是：

> 先把 batch 路径从 stream-state 结构里拆出来，再谈指令形状。

## Rust 新手可以学到的 1：性能问题有“结构层级”

很多新手看到性能差，第一反应是：

- “是不是要先看汇编？”

这当然重要，但顺序更重要。

如果 C 是：

- 直接循环

而 Rust 是：

- `state.feed()` + `Option` + `Vec::push`

那说明差距首先在**结构层级**，不是某一条指令。

这时候先做的应该是：

1. 让 batch 路径和 C 站到同一层
2. 然后再看汇编细节

## Rust 新手可以学到的 2：`run_in_place` 不只是“少分配”

很多人第一次看到 `run_in_place` 会只想到：

- 它少了一次 `Vec` 分配

但更重要的是它会逼你把代码写成真正的 batch kernel：

- 先知道输出长度
- 直接按索引写输出
- 不再靠 `push`
- 更容易和 C 的热循环对齐

所以 `run_in_place` 在这里不只是优化内存，更是在逼实现结构变薄。

## 这次留下的结论

- `dx` 这次的关键问题不是 wrapper，而是 batch 结构还停在 stream-state 形状
- 先改成 direct batch kernel，才有资格继续谈汇编差异
- 性能优化时，要先判断你面对的是“指令问题”，还是“结构层级问题”
