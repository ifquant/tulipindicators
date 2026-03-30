## 背景

上一笔我们把 benchmark 改成了：

- 先由 C contract 生成固定 iterations 基线
- 后续 compare 默认复用这份基线

这个方向是对的，但默认把固定目标时长设成了 `4s`。  
用户马上给了一个很实际的反馈：

- `4s` 还是太重
- 默认应该是 `400ms`
- 研究型复核再手动拉高

这其实很合理，因为“固定基线”解决的是口径稳定问题，不一定非要默认就用特别重的样本。

## 这次改了什么

### 1. 默认固定目标时长改成 `400ms`

在 `rust/src/benchmark.rs` 里，把：

- `DEFAULT_FIXED_TARGET_MS`

从：

- `4_000`

改成了：

- `400`

也就是默认固定基线现在瞄准的是：

- 每个 `(indicator, mode, input_len)` 样本大约 `400ms`

而不是之前的 `4s`。

### 2. 同步刷新文档说明

更新了：

- `AGENTS.md`
- 上一笔 benchmark 教程里的说明文字

重点强调：

- 默认固定基线目标时长现在是 `400ms`
- 如果要做更重的研究型复核，仍然可以用 `TI_BENCH_FIXED_TARGET_MS` 显式拉高

### 3. 重新生成全量固定基线文件

基线文件：

- `benchmarks/fixed-iterations.tsv`

也重新按新的默认目标时长刷新了一遍。  
所以这次不是“只改了默认值”，而是把仓库里真正提交的固定 iterations 基线也同步到了 `400ms` 口径。

## 为什么这次值得单独成一笔

这不是一个小数字改动而已，而是 benchmark 工作流的默认策略变化：

- `4s` 更偏研究型
- `400ms` 更偏日常型

默认值一旦定下来，会直接影响：

- 全量 compare 的运行时间
- 日常刷新热点榜单的频率
- 是否有人愿意真的去跑 benchmark

所以它值得单独作为一笔提交讲清楚。

## 一个很实际的工程点

这次还顺手验证了一个重要事实：

- 现在 fixed-iterations 生成已经是 calibration-only
- 不会再像最开始那样“顺手把正式样本也全跑一遍”

所以把默认值从 `4s` 改成 `400ms` 之后：

- 基线生成速度进一步可接受
- compare 本身也更适合当日常回归工具

## 现在怎么理解这套默认口径

默认 benchmark 口径现在是：

- 输入规模：`4096`、`16384`
- 固定基线来源：C contract
- 固定目标时长：`400ms`

如果你想要更稳的研究型测量，就显式调大，不要把更重参数硬塞回默认值。

例如：

```bash
TI_BENCH_REFRESH_FIXED_ITERATIONS=1 \
TI_BENCH_FIXED_TARGET_MS=4000 \
TI_BENCH_GENERATE_FIXED_ITERATIONS_ONLY=1 \
cargo run --release --bin indicator-bench-compare
```

## 这次没有做的事

- 没有改变固定基线只由 C 生成这条规则
- 没有改变默认 benchmark 尺寸，仍然是 `4096/16384`
- 没有把 kernel probe 也切换到固定基线口径
