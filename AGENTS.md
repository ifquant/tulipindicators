# AGENTS.md

## 项目概览
- 本仓库是：Tulip Indicators 技术指标库，当前以 ANSI C 实现技术分析指标、蜡烛图模式、CLI、示例程序和单文件 amalgamation，并正在规划 Rust 风格重构。
- 主要技术栈：C99 / Make / Tcl 代码生成脚本 / Shell 工具
- 包管理与构建工具：无包管理器；使用 `make`、系统 C 编译器（`gcc` 或 `clang`）、`tclsh`
- 本仓库的主要目标：维护指标计算正确性与接口稳定性，保留静态库、示例程序、测试程序和 `tiamalgamation.c` 产物
- 非本仓库职责范围：官方站点、其他语言 bindings、外部消费者项目的接入代码、独立部署或基础设施

## 适用范围
- 本文件适用于 `/Users/dev/workspace2/hc_apps/tulipindicators` 及其子目录。
- 若子目录存在更具体的 `AGENTS.md`，以更靠近目标文件的说明为准。
- 用户明确提出的任务要求，高于本文件中的一般性约束。
- 根目录 [`/Users/dev/workspace2/hc_apps/AGENTS.md`](/Users/dev/workspace2/hc_apps/AGENTS.md) 中的通用提交纪律继续生效；本文件只补充 `tulipindicators` 特有规则。

## 目录导航
- `c/`：ANSI C 实现目录，包含稳定指标、beta 指标、构建脚本、生成模板、示例程序、测试程序和单文件产物
- `c/indicators/`：稳定 C 指标实现
- `c/beta/`：实验性或候选 C 指标实现，不默认视为稳定 API 承诺
- `c/utils/`：C 运行时与测试辅助代码
- `c/tests/`：C golden 数据与测试输入；Rust golden test 也复用这里的数据
- `rust/`：Rust 实现目录，包含 crate 源码、benchmark 入口和 Rust 集成测试
- `rust/src/`：Rust 核心库与指标实现
- `rust/tests/`：Rust 集成测试
- `tutorials/commit/`：按提交顺序维护的协作教程，面向新手解释每次提交的背景、目标、关键设计、验证和未覆盖项
- `c/templates/`：生成 `indicators.h`、`indicators.c`、`candles.h`、`candles.c` 的模板
- 根目录入口：`Makefile`、`Cargo.toml`、`build.tcl`、`doc.tcl`
- `plan.md`：Rust 风格重构计划文档，不是实现代码
- `c/tiamalgamation.c`：单文件分发产物；修改相关逻辑时要关注兼容性

## 常用命令
- 安装依赖：无包管理器；要求系统具备 `make`、`gcc`/`clang`、`ar`、`ranlib`、`tclsh`
- 启动开发环境：无常驻开发服务器；通常直接编辑代码后运行构建或测试命令
- Lint：无独立 lint 配置，不要虚构 lint 命令
- Rust 静态检查：`cargo clippy --all-targets --all-features`
- Rust 默认检查入口：`make rust-check`
- 关闭 clippy 的 Rust 检查入口：`TI_ENABLE_CLIPPY=0 make rust-check`
- Type check：无独立 type check 配置
- 单元测试：`make smoke`
- 集成测试：`make smoke_amal`
- Rust benchmark：`cargo run --release --bin indicator-bench`
- C contract benchmark：`make benchmark_contract`
- C/Rust benchmark 对比：`cargo run --release --bin indicator-bench-compare`
- 如需提高 benchmark 稳定性，优先调大：`TI_BENCH_TARGET_MS`、`TI_BENCH_MIN_ITERATIONS`、`TI_BENCH_REPEATS`
- 构建：`make`
- 仅运行某个模块/包：根目录可继续运行 `make sample`、`make example1`、`make example2`、`make cli`、`make benchmark`、`make fuzzer`；C 侧真实实现位于 `c/`，Rust 侧真实实现位于 `rust/`
- 清理构建产物：`make clean`
- 清理构建产物与生成文件：`make veryclean`

## 开发原则
- 优先做最小可行改动，避免无关重构。
- 不要顺手重命名、搬移、格式化大批文件，除非任务明确要求。
- 优先复用现有指标模式、辅助函数和生成流程，不新造第二套机制。
- 不要引入新依赖，除非现有方案明显不适合；引入时要说明理由、构建影响和分发影响。
- 修改公开 API、生成文件格式、构建入口或测试基线时，要明确说明兼容性影响。
- 规划 Rust 风格重构时，先保持现有数值语义和测试基线，再调整工程结构。

## 代码约定
- 使用本仓库现有的构建、生成和测试约定，不要自创风格。
- 新代码优先遵循现有目录结构、命名方式、错误码和边界处理模式；若偏离，需说明原因。
- 避免“大而全”的抽象；优先选择当前代码库里已经被广泛采用的实现模板。
- 非必要不要引入全局状态、隐式副作用或隐藏 I/O。
- 日志、异常和错误信息不得泄露敏感信息。
- 生成文件、性能敏感路径和核心指标循环中，避免无理由增加堆分配或隐藏拷贝。

## 应用领域硬约束

### 数据与一致性
- 不得无说明改变指标数学语义、`start`/`lookback` 行为或输出长度规则。
- 输入窗口、边界条件、空输出行为必须与现有测试基线保持一致，除非任务明确要求修正行为。
- 浮点计算行为的改动必须用测试或明确说明证明其必要性，不能静默漂移。
- 流式计算与批处理计算的结果必须保持一致；若暂时做不到，必须在变更说明中明确标注。

### 认证与权限
- 本仓库不涉及用户认证、租户权限或鉴权模型；不要编造相关机制。
- 若未来接入外部服务或远程依赖，应先与用户确认，不要在当前库内擅自引入。

### 安全与合规
- 不要把本机路径、环境变量、密钥或外部凭据写入源码、测试数据或日志。
- 不要提交真实市场私有数据、私有账户信息或测试外的敏感样本。
- 示例程序和 CLI 的错误输出应保持简洁，不泄露不必要的环境细节。

### 兼容性
- `indicators.h`、`candles.h`、`libindicators.a`、`tiamalgamation.c` 的对外兼容性默认应保持稳定。
- 修改 `templates/` 或 `build.tcl` 时，不要只手改对应生成文件的一部分；要么更新生成来源并重生成，要么明确说明为何仅改生成产物。
- `beta/` 目录中的实现和稳定 `indicators/` 要分开处理，不要把实验指标提升为稳定承诺而不说明。
- 如果引入 Rust 重构骨架或兼容层，不要把未来目标状态描述成当前已生效接口。

## 测试与验收
- 改动代码后，至少运行：`make smoke`
- 改动单文件构建、生成脚本或公共头文件后，优先再运行：`make smoke_amal`
- 改动 Rust 指标实现、流式路径或性能敏感逻辑时，优先再运行：`cargo run --release --bin indicator-bench`
- 改动 Rust 代码结构、错误处理、公共接口或常量表达式时，优先再运行：`cargo clippy --all-targets --all-features`
- 如果只是想临时跳过 clippy，可使用：`TI_ENABLE_CLIPPY=0 make rust-check`；默认仍应保持 `clippy` 开启
- 改动 benchmark contract、性能对比口径或回归阈值逻辑时，优先再运行：`cargo run --release --bin indicator-bench-compare`
- 修复 bug 时，优先补一个能复现该 bug 的测试，或把现有 `tests/*.txt` 样例扩充到能覆盖该场景。
- 改动公开接口、示例程序或生成逻辑时，更新相关文档与说明。
- 每次提交前，确认已新增一篇对应的 `tutorials/commit/NNNN-*.md` 教程；如果没有这篇教程，提交不算完整。
- 完成任务时，报告：
  - 改了哪些文件
  - 跑了哪些命令
  - 测试是否通过
  - 已知风险与未覆盖项

## 禁止事项
- 不要修改根目录外的其他仓库或上级工作区文件，除非任务明确要求。
- 不要提交密钥、证书、`.env` 实值或测试外的真实数据。
- 不要跳过失败测试后直接宣称完成。
- 不要为了“顺手优化”改动与当前任务无关的大段代码。
- 不要擅自删除指标、头文件、导出符号、测试基线或消息字段。
- 不要手工改动生成文件后又忽略对应模板或生成脚本的来源差异。

## 需要先确认的情况
- 新增构建依赖或生产依赖
- 删除文件、大规模重构或整体目录迁移
- 修改生成脚本、生成模板或公共头文件对外形态
- 修改 `Makefile` 默认目标、构建产物名称或分发方式
- 修改指标数学定义、输出长度语义或兼容性基线
- 引入外部服务、外部 API、网络访问或新的后台任务
- 进行破坏性数据操作或批量删除测试基线

## 提交 / PR 要求
- Commit 标题与正文格式遵循根目录 [`/Users/dev/workspace2/hc_apps/AGENTS.md`](/Users/dev/workspace2/hc_apps/AGENTS.md) 的多段式提交规范。
- 推荐 scope 使用 `tulipindicators` 子系统，例如：`docs(tulipindicators)`、`refactor(tulipindicators-build)`、`fix(tulipindicators-ema)`
- `tulipindicators` 中的每一次提交，都必须同时新增一篇提交教程到 `tutorials/commit/`；不要把多次提交合并写成一篇，也不要省略。
- AI 产出的 commit message 必须服务于“让人类快速跟上 AI 变更”，不能只写结果，必须把目的、实现、验证和明确未处理项写清楚。
- 非 trivial 提交一律使用多行 commit message；不要提交 `fix`、`update`、`misc`、`wip` 这类信息不足的标题。
- 提交教程文件名必须以四位序号开头，范围 `0001` 到 `1000`，推荐格式：`tutorials/commit/0001-short-topic.md`。
- 新教程必须使用当前目录下按数值递增的下一个可用序号；不要复用旧编号，不要跳号抢未来编号。
- 如果已经存在 `0007-...md`，下一篇就从 `0008-...md` 开始；序号不足四位时必须左侧补零。
- 提交教程默认使用 Markdown，面向新手读者；假设读者知道这次提交发生了什么是错误的。
- 提交教程默认使用中文撰写，目标读者默认是中文新手；不要默认写英文教程。
- 如需保留英文术语、函数名、命令名或行业缩写，优先写“中文解释 + 英文术语”而不是整段改成英文。
- 提交教程不是 changelog 摘抄，必须解释背景知识、主要目标、设计想法、关键取舍、验证方式，以及这次为什么值得单独提交。
- 提交正文优先回答四个问题：
  - 这次改动的目的是什么
  - 实际做了什么关键改动
  - 用什么方式验证了结果
  - 哪些相关问题还没处理或故意没包含
- `Changes:` 应写行为和设计层面的高信息密度内容，不要写成“修改 A 文件、修改 B 文件”的低价值清单。
- `Verification:` 只允许写实际运行过的命令或人工检查；文档改动若未运行命令，明确写 `- not run (documentation-only change)`。
- `Not included:` 不要省略。只要改动不是完整闭环，就明确写出未覆盖的指标、未迁移的路径、未确认的兼容性，避免制造“已经全好了”的错觉。
- 对于“部分实现”“只标记一部分形式”“只迁移一批指标”这类改动，提交信息必须显式列出已覆盖范围和明确未覆盖范围。
- Rust 检查默认通过 `make rust-check` 执行，且默认开启 `clippy`；只有在用户明确要求或当前任务确实需要暂时绕过时，才使用 `TI_ENABLE_CLIPPY=0 make rust-check`。
- 如果涉及兼容性风险，正文首段或 `Not included:` 中必须点出受影响的对外接口、生成产物或测试基线。
- `Co-Authored-By:` 仅在确实需要记录协作来源时再加；不要为了好看而机械追加。

### 提交信息写法

- 标题行：用最小有意义 scope + 具体动作，直接说明“改了什么”。
- 首段：说明为什么要做这次改动，以及它带来的实际效果。
- `Changes:`：列 2 到 5 条最高信号改动，优先写行为、约束、兼容性处理，不写琐碎编辑动作。
- `Verification:`：列真实执行的命令和结果；若结果含 warning，可简要注明。
- `Not included:`：列当前故意没做、已知有问题、后续要跟进的部分。

### 面向本仓库的提交提示

- 改指标实现时，标题应带上具体指标名或子系统名，例如 `fix(tulipindicators-rsi)`、`refactor(tulipindicators-stream)`。
- 改 `templates/`、`build.tcl`、`indicators.h`、`candles.h`、`tiamalgamation.c` 时，正文要明确说明生成链和兼容性是否受影响。
- 改测试基线时，正文要说明是修正预期、补覆盖，还是接受行为变化。
- 改 Rust 重构骨架时，正文要明确哪些仍是计划状态，哪些已经成为当前实现。

### AI 提交前检查清单

- 提交前先问自己：如果人类只看 commit message、不打开 diff，能否知道“为什么改、改了什么、怎么验证、还差什么”。
- 提交前先检查：本次提交是否已经新增对应的 `tutorials/commit/NNNN-*.md` 教程文件。
- 教程编号是否是当前目录中按数值递增的下一个编号，且保持四位零填充。
- 教程是否默认使用中文写给中文新手，而不是无必要地写成英文。
- 教程是否真的在教新手理解这次提交，而不是把 commit message 换个格式重复一遍。
- 标题是否具体到子系统或指标，而不是泛泛地写 `fix`、`update`、`cleanup`。
- 首段是否解释了改动目的和实际效果，而不是把标题换种说法重复一遍。
- `Changes:` 是否只保留 2 到 5 条最高价值信息，并且都在描述行为、约束、兼容性或实现边界。
- `Verification:` 是否只列出真实执行过的命令或人工检查，并写明 `PASS`、`FAIL` 或事实结果。
- 如果没有运行测试，是否明确写了 `- not run (reason)`，而不是留空。
- `Not included:` 是否写出了当前故意没做、暂未确认、后续继续处理的内容。
- 如果改动是部分覆盖，是否明确写出了“已覆盖范围”和“未覆盖范围”。
- 如果改动影响生成链、公共头文件、测试基线或分发产物，是否在正文里点明兼容性影响。
- 如果改动只是文档、计划或注释，是否明确写出这是非行为变更，避免让读者误判已经实现。

### AI 提交生成流程

- 先找出 `tutorials/commit/` 中已有的最大序号，分配下一个四位编号。
- 先写教程，再写 commit message；教程用于把背景知识和设计动机讲清楚，commit message 用于高密度总结。
- 先总结一句话：这次改动解决了什么具体问题。
- 再提炼 2 到 5 条最关键实现变化，删掉文件名导向和编辑动作导向的低价值描述。
- 再回填真实验证结果，只写实际跑过的命令和检查。
- 最后强制补 `Not included:`，列出当前没有完成的邻近问题、未验证路径或刻意保留的旧实现。
- 生成后再读一遍，如果 message 给人的感觉像“总结 diff”，而不是“解释决策和边界”，就重写。

### 提交教程要求

- 教程文件路径：`tutorials/commit/NNNN-short-topic.md`
- 教程标题要能让新手理解这篇内容的主题，不要只写提交标题。
- 教程正文默认使用中文；命令、代码符号、API 名、文件名可保留原文，但解释段落应以中文为主。
- 教程至少应包含这些部分：
  - `背景`：这次为什么需要改，读者需要先知道什么上下文
  - `主要目标`：这次提交想解决什么具体问题
  - `改动概览`：这次实际做了哪些关键改动
  - `关键知识`：指标、构建链、生成链、兼容性或测试方面的必要背景
  - `验证`：运行了什么命令，结果如何
  - `未覆盖项`：还有哪些相关问题没解决，避免新手误判进度
- 如果这次提交只是计划、文档或规则整理，教程也要说明“为什么这种文档改动会影响协作效率”，而不是空泛地说“补充文档”。
- 如果这次提交是部分实现，教程要明确写出“本次覆盖范围”和“下一步通常会做什么”。
- 教程可以比 commit message 更长，但仍然要以帮助新人理解为目标，不要写成自我表扬或流水账。

### 推荐提交模板

```text
<type>(<scope>): <specific imperative summary>

<why this change exists, what practical outcome it produces, and what risk or confusion it reduces>

Changes:
- <highest-signal implementation change>
- <highest-signal implementation change>
- <highest-signal implementation change>

Verification:
- <real command or manual check> (<result>)
- <real command or manual check> (<result>)

Not included:
- <known limitation, deferred path, or intentionally untouched area>
- <known limitation, deferred path, or intentionally untouched area>
```

### 不合格信号

- 读完标题和首段，仍然不知道为什么要改。
- 没有新增 `tutorials/commit/NNNN-*.md` 教程，或教程编号错误。
- 教程只是把 diff 或 commit message 重新排版，没有提供新手需要的背景知识和上下文。
- `Changes:` 只是文件列表、重命名列表或“清理代码”这类空话。
- `Verification:` 写了没有运行过的命令，或只写“tested”。
- 完全不写 `Not included:`，让局部实现看起来像完整交付。
- 用“优化”“改进”“整理”这种词，但没有说清楚具体行为变化。
- 文档改动却写得像功能已经落地，或者计划改动却写得像代码已经实现。

### 参考示例

```text
feat(tulipindicators-ema): preserve stream output alignment for partial runs

Fix the EMA stream path so partial chunk execution matches batch output and no
longer drifts when callers feed uneven step sizes.

Changes:
- update the EMA stream state machine to retain the previous smoothed value across chunk boundaries
- keep batch and stream lookback/output length rules aligned with the existing C API
- add regression coverage for uneven chunk sizes against the existing smoke-style expectations

Verification:
- make smoke (PASS)
- manual stream vs batch comparison for EMA uneven chunks (PASS)

Not included:
- other stream-enabled indicators still use their existing implementations
- no public API changes were made to `indicators.h` or `tiamalgamation.c`
```

- 如果提交属于“部分落地”，可以用更直接的风格，明确写出“已验证”和“未覆盖”的边界，风格可以接近下面这样：

```text
feat(tulipindicators-rust): scaffold crate entry points for the first migration slice

Verified via cargo check (PASS) and smoke baseline review (no behavior change in C path):
- add Cargo manifest and crate root for the planned Rust migration
- keep the existing C build and smoke entry points untouched

Not included:
- no indicators are migrated yet
- no FFI compatibility layer exists yet
- `make` remains the only supported build path for production outputs
```

- PR 描述至少包含：
  - 背景
  - 方案
  - 影响面
  - 验证方式
  - 回滚思路（如适用）
- 涉及生成逻辑、公共头文件、测试基线或 breaking change 时，必须写明兼容性影响和升级说明。

## 参考资料
- 项目说明：`README.md`
- Rust 风格重构计划：`plan.md`
- 提交协作教程：`tutorials/commit/`
- 构建与生成入口：`Makefile`、`build.tcl`、`doc.tcl`
- 测试数据：`tests/atoz.txt`、`tests/candles.txt`、`tests/extra.txt`、`tests/untest.txt`
- 公共接口：`indicators.h`、`candles.h`

## 子目录约定
- `indicators/`、`beta/`、`utils/`、`templates/`、`tutorials/` 如后续需要更细规则，建议各自维护更具体的 `AGENTS.md`。
- 子目录文件只写“比根目录和当前文件更具体”的规则，不重复整份上层文件。
