# 从 `kvo` 汇编里把 `vf` 符号路径和双 EMA 更新都收成更接近 C 的形状

这次继续用“先看汇编，再改热循环”的方法修 `kvo`。

focused benchmark 之前是：

- `4096 = 1.541x`
- `65536 = 1.238x`

而且这次很有代表性：

- `4096` 偶尔已经接近 parity
- `65536` 长输入 steady-state 才更稳定地慢

这说明：

- 不是 wrapper 税
- 更像热循环本体里还有一点机器码形状差异

## C 和 Rust 的差异在哪

C 版 `kvo` 热循环里有两块比较关键：

1. `vf` 的符号路径更接近 branchless
   - 会看到 `fnmul`
   - 以及 `fcsel`

2. 两条 EMA 递推都比较接近 fused multiply-add 形状

Rust 之前的问题是：

- `vf` 的最后一段还是
  ```rust
  * if trend != 0 { 1.0 } else { -1.0 }
  ```
  这会让 sign 路径更像分支选择，而不是更接近 C 的 select/fused 形状。

- 两条 EMA 虽然已经用了 `mul_add`
  但还是写成：
  ```rust
  (vf - ema).mul_add(per, ema)
  ```
  这次把它们收成更明确的 state-side 形式，和前面 `dema/tema` 的经验一致。

## 这次怎么改

### 1. 把 `vf` 的 sign 收成共享 helper

新增：

```rust
fn signed_volume_force(dm, cm, volume, trend) -> Real
```

里面做两件事：

- `signal = ((trend != 0) as u8 as Real).mul_add(2.0, -1.0)`
- `magnitude = volume * (dm / cm).mul_add(2.0, -1.0).abs() * 100.0`

这样至少让 sign 和 `2*x-1` 这段更像 C 的无分支/融合形状。

### 2. 把两条 EMA 更新改成更明确的 state-side `mul_add`

从：

```rust
ema = (vf - ema).mul_add(per, ema)
```

改成：

```rust
let part = vf * per;
ema = ema.mul_add(1.0 - per, part);
```

这和前面 `dema/tema` 的结论一致：

- 数学等价
- 但 state-side 的 `mul_add` 更容易贴近 C 当前的依赖链形状

## 结果

focused benchmark 结果：

- `kvo 4096 = 0.890x`
- `kvo 65536 = 1.056x`

也就是两档都回到了阈值内。

## 这次得到的经验

### 1. 不是所有热点都先拆 wrapper

`kvo` 已经是 direct batch kernel 了。

这次收益来自：

- `vf` 的符号路径
- EMA 递推链的机器码形状

### 2. state-side `mul_add` 不是只对 `dema/tema` 有用

只要递推链里有：

```text
state = value * per + state * (1-per)
```

都值得检查：

- `mul_add` 放在哪一边
- 哪边才是关键依赖链

### 3. `screen` 榜单抓热点后，focused run 还是必须做

如果没有 focused run，很容易看不出：

- `kvo` 其实不是入口税
- 而是长输入 steady-state 的热循环差异
