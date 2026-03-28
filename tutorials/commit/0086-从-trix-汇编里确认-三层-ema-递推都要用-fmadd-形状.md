# 从 TRIX 汇编里确认：三层 EMA 递推都要用 `fmadd` 形状

这次收的是 `trix`。

它和前面已经优化过的 `ema`、`macd`、`ppo` 有一个共同点：核心热循环都是 EMA 递推。但 `trix` 不是一层，而是三层：

1. 第一层 `ema1`
2. 第二层 `ema2`
3. 第三层 `ema3`

如果 Rust 只把一层写成更接近 C 的 `fmadd` 形状，另外两层还停在普通的 `fmul + fadd`，那最终还是会慢。

## 这次从汇编里看到什么

`C` 版 `trix` 的 steady-state 循环里，三层递推都是同一种形状：

- 先算 `delta = sample - state`
- 再做 `fmadd`

也就是逻辑上等价于：

```c
state = (sample - state) * per + state;
```

编译器在目标平台上把它压成了：

- `fsub`
- `fmadd`

而 Rust 这边在 batch kernel 里原来还是：

```rust
ema1 = (*sample - ema1) * per + ema1;
ema2 = (ema1 - ema2) * per + ema2;
ema3 = (ema2 - ema3) * per + ema3;
```

这类写法在前面的多个 case 里已经证明，不一定会自动得到和 C 一样的 fused multiply-add。

## 这次怎么改

这次只改 batch kernel，不动算法，不动公开接口：

```rust
ema1 = (*sample - ema1).mul_add(per, ema1);
ema2 = (ema1 - ema2).mul_add(per, ema2);
ema3 = (ema2 - ema3).mul_add(per, ema3);
```

要点是：

- `mul_add(a, b)` 表示 `self * a + b`
- 这里把它写成 `delta.mul_add(per, state)`
- 这样更接近 C 的 `fsub + fmadd` 热循环形状

## 为什么这次 stream 不需要改

`trix` 的 stream 路径走的是 `EmaState::feed()`。

而前面已经把共享的 `EmaState` 更新式改成了 `mul_add`。所以这次真正落后的只剩 batch kernel，直接收它就够了。

这也是性能优化里一个很实用的思路：

- 先分清楚 batch 和 stream 谁还慢
- 不要因为同一个指标有两条路径，就默认两边都要重写

## Rust 新手可以学到的 1：`mul_add` 不是语法糖，它会改变 codegen

很多 Rust 新手第一次看到 `mul_add` 会觉得：

- 这不就是 `a * b + c` 吗？

从数学上说是，但从机器码上不一定一样。

在支持 fused multiply-add 的平台上：

- `a * b + c`
  可能生成 `fmul` 再 `fadd`
- `a.mul_add(b, c)`
  更容易直接生成一条 `fmadd`

对递推类指标，这种差异会直接影响 steady-state 吞吐。

## Rust 新手可以学到的 2：性能优化时，要先找“共享模式”

`trix` 看起来像一个“复杂指标”，但这次真正起作用的不是指标知识本身，而是识别出它和 `ema/macd/ppo` 属于同一种模式：

- 递推型 EMA 链
- C 汇编里已经是 `fmadd`
- Rust 还没 fused

一旦看出这个模式，优化就不是瞎猜，而是复用前面已经证实有效的方法。

## 这次留下的结论

- `trix` 的 batch 热点主要来自三层 EMA 递推还没 fully fused
- 只要把三层都改成 `delta.mul_add(per, state)`，就能更接近 C 的热循环
- 对这种指标，优化关键不是“写得更花”，而是“写得更像编译器愿意生成 `fmadd` 的 kernel”
