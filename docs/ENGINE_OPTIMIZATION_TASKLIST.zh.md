# 引擎优化任务清单

本文档是 `mquickjs-rs` **仅面向引擎本体**的优化待办清单。

它直接展开自 `IMPLEMENTATION_PLAN.md` 中尚未完成的第 9 阶段：

- `9.1 Profile and optimize hot paths`
- `9.2 Optimize GC performance`
- `9.3 Reduce memory usage`

本文档**不包含** `led-runtime` 产品层工作。

相关 benchmark 分析见：
- `docs/BENCHMARK_ANALYSIS.md`
- `docs/BENCHMARK_ANALYSIS.zh.md`

## 适用范围

本文档只覆盖：

- `mquickjs-rs` 的 parser / compiler / VM / runtime
- benchmark 的正确性与性能分析
- 引擎自身的 GC 与内存行为

本文档不覆盖：

- `led-runtime` 宿主 API 易用性
- effect 脚本/产品语义
- GUI / demo / 产品层集成

## 当前优化主题

结合当前代码和 benchmark 形态，最值得关注的引擎热点是：

- `src/vm/interpreter.rs` 中的调用与方法分发
- `src/vm/interpreter.rs` 与 `src/vm/natives.rs` 中的 native / builtin 参数整理
- `src/vm/interpreter.rs` 与 `src/vm/property.rs` 中的 dense array 访问
- `src/vm/interpreter.rs` 中的 opcode dispatch 开销
- `src/gc/collector.rs` 中的 GC 实现质量
- `src/vm/types.rs`、`src/context.rs` 与 `src/runtime/*` 中的运行时分配与容器布局

## 优先级总览

### P0

- benchmark 真实性与基线收口
- 调用热路径优化
- native/builtin 参数传递优化
- dense array 快速路径

### P1

- 最热 opcode 的 dispatch 精简
- GC：从保守 `mark_all` 迁移到真实 root-based marking
- 运行时分配与内存占用复盘

### P2

- builtin/runtime 边界结构收口
- 在新 benchmark 口径下做第二轮微优化

## 详细任务清单

## 9.1 分析并优化热点路径

### 9.1.1 benchmark 基线收口

**优先级**：P0

**原因**

- 只有 benchmark 数据可信，优化工作才有意义。
- 本地脚本和 CI workflow 之前存在不一致。
- 一些历史 benchmark 结论曾基于错误对比对象。

**任务**

- 保留一套可信的本地 benchmark 流程。
- 保持 CI benchmark 行为与本地行为一致。
- 区分：
  - 进程启动开销
  - 脚本净执行时间
- 维护一张统一的基线表，至少覆盖：
  - `fib`
  - `loop`
  - `array`
  - `sieve`
  - `json`

**验证方式**

- benchmark 多次运行结果可复现。
- `docs/BENCHMARK_ANALYSIS.md` 内部口径一致。

**当前已完成**

- 2026-03-16：已定义当前规范的 benchmark 集合。
- 2026-03-16：已区分并文档化本地 Criterion、本地 Rust-vs-C 对比、CI Summary 三者的职责。
- 2026-03-16：`.github/workflows/bench.yml` 现在会同时输出 Rust-vs-C 对比表和 Rust-only 的 Criterion 表。
- 2026-03-16：`docs/BENCHMARK_ANALYSIS.md` 已重写为当前基线参考文档。
- 状态：对当前这一轮引擎优化而言，本任务可以视为已完成。

### 9.1.2 调用热路径优化

**优先级**：P0

**热点文件**

- `src/vm/interpreter.rs`
- `src/vm/stack.rs`

**原因**

- `fib` 和 `loop` 强烈暗示调用开销与高频 dispatch 开销仍是主要瓶颈。
- 当前 `Call` 路径虽已优化，但 `remove_at_offset()` 仍然调用 `Vec::remove()`，会触发元素搬移。

**任务**

- 重新设计调用栈布局，避免在热调用路径上使用 `Vec::remove()`。
- 分别针对 `Call`、`CallMethod`、`CallConstructor` 做专门优化。
- 减少普通 JS 函数调用中的临时参数重排。
- 重新评估调用路径中的字符串提升成本。

**预期收益**

- 主要改善 `fib`
- 次要改善 `loop`

**当前已完成**

- 2026-03-16：已完成第一轮 `method_chain` 相关优化，在数组高阶方法中去掉了每个元素回调时的临时 `Vec<Value>` 参数分配。
- 已补对应的 `map().filter().reduce()` 链式调用回归测试。
- benchmark 结果：Criterion 下 `method_chain 5k` 大约从 `1.88–2.54 ms` 降到 `0.80–0.82 ms`。

### 9.1.3 native/builtin 参数传递优化

**优先级**：P0

**热点文件**

- `src/vm/interpreter.rs`
- `src/vm/natives.rs`

**原因**

- native 与 builtin 调用路径仍在构造临时 `Vec<Value>` 并做 `reverse`。
- 这条路径会影响 `Math.*`、`JSON.*`、数组方法和其他内建函数。

**任务**

- 为 0/1/2 参数 native 调用增加专门 fast path。
- 对短参数列表避免堆分配。
- 减少或消除 native/builtin 参数准备阶段的 `reverse()`。
- 在安全前提下，考虑直接使用基于栈的参数切片。

**预期收益**

- 改善内建函数密集型脚本
- 对 `array`、`json`、数值函数调用密集场景更有帮助

**当前已完成**

- 2026-03-16：已为 `CallMethod` 的 native 路径增加小参数 fast path，在 `argc <= 2` 时去掉临时参数 `Vec` 分配。
- 已补 `Array.prototype.push` 多参数顺序回归测试。
- benchmark 结果：Criterion 下 `array push 10k` 大约从 `0.897–0.911 ms` 降到 `0.672–0.691 ms`。
- benchmark 结果：Criterion 下 `method_chain 5k` 进一步从 `0.986–1.182 ms` 降到 `0.720–0.763 ms`。

### 9.1.4 dense array 快速路径

**优先级**：P0

**热点文件**

- `src/vm/interpreter.rs`
- `src/vm/property.rs`
- `src/runtime/array.rs`

**原因**

- `array` 和 `sieve` 都是典型的 dense-array benchmark。
- 当前访问路径仍经过若干通用层。

**任务**

- 缩短 `GetArrayEl`、`GetArrayEl2`、`PutArrayEl` 的执行路径。
- 对 dense integer-index 访问做专门分支。
- 对明显是数组的操作避免进入通用 property lookup。
- 分开审查 `push`、索引读取、索引写入三条路径。

**预期收益**

- 主要改善 `array`
- 对 `sieve` 也应有明显帮助

**当前已完成**

- 2026-03-16：已完成第一轮深度属性访问优化，为普通对象属性读取增加了小对象 fast path，并统一了 `GetField` / `GetField2` 的属性分发路径。
- 已补深度属性链访问回归测试。
- benchmark 结果：Criterion 下 `deep_property 200k` 大约从 `28–29 ms` 降到 `15.7–17.0 ms`。

### 9.1.5 opcode dispatch 精简

**优先级**：P1

**热点文件**

- `src/vm/interpreter.rs`

**原因**

- `loop` 仍说明指令分发开销有意义。
- 大型 `match` 分发虽然正确且易维护，但在热路径上仍有成本。

**任务**

- 用 benchmark 驱动找出最热的 10–20 个 opcode。
- 缩短 dispatch loop 每次迭代里的固定开销。
- 减少热指令里的重复 decode / branch / error-path 开销。
- 对算术、局部变量、跳转、调用等最热指令优先做本地 fast path。

**预期收益**

- 是 `loop` 的第二主要优化方向
- 对多数 benchmark 都会有普遍收益

**当前已完成**

- 2026-03-16：已补 `try_catch` benchmark，用于覆盖高频 throw/catch 控制流。
- 2026-03-16：已收口异常路由路径，把重复 `pop` 展开改成基于 `truncate` / `drop_n` 的回退方式，降低异常展开开销。
- 已补“循环中重复 throw/catch”回归测试。
- benchmark 结果：Criterion 下 `try_catch 5k` 当前基线为 `340–349 µs`。
- 2026-03-16：已在 `dump` 特性下加入运行期 opcode 计数，并通过 `Context` 暴露给 profiling 分析使用。
- 已补 `dump` 模式回归测试，确认 opcode 计数会记录真实执行。
- 运行期热点结论：
  - `loop` 主要被 `GetLoc1`、`Goto`、`Add`、`Dup`、`Drop`、`GetLoc0`、`PutLoc0`、`PutLoc1`、`Lt`、`IfFalse` 主导。
  - `sieve` 主要被 `Goto`、`Drop`、`IfFalse`、`GetLoc3`、`Add`、`Dup`、`GetLoc0`、`Lte`、`GetLoc2`、`PutArrayEl`、`PutLoc3`、`GetArrayEl`、`CallMethod` 主导。
- 当前判断：下一轮最值得打的应是 `Dup/Drop` 与局部变量存储配合模式，或分支/控制流成本，而不是继续拍脑袋优化单个算术 helper。

### 9.1.6 算术/比较微优化复盘

**优先级**：P1

**热点文件**

- `src/vm/ops.rs`

**原因**

- 核心算术和比较 helper 已有一轮 `#[inline]` 处理。
- 这部分仍然重要，但收益大概率低于调用/数组/native 热路径。

**任务**

- 复查剩余热 `op_*` helper 是否值得 inline。
- 减少常见 int/int 与 int/float 路径上的重复数值转换。
- 在 benchmark 基线稳定后，重新评估比较和相等判断 fast path。

**预期收益**

- 小而广的增益

**当前已完成**

- 2026-03-16：已优化字符串拼接热路径，改为直接写入单个结果缓冲区，而不是先构造两个临时拥有所有权的 `String` 再拼接。
- 已补“字符串 + 数字 + 字符串”链式拼接回归测试。
- benchmark 结果：Criterion 下 `runtime_string_pressure 4k` 大约从 `2.89–3.38 ms` 降到 `1.53–1.55 ms`。
- 2026-03-16：已优化 `StrictEq` / `StrictNeq` 热 opcode 路径，为“值完全相同”“整数比较”“布尔比较”增加直接 fast path，再回退到较慢通用逻辑。
- 已重新验证现有 switch 语义回归测试。
- benchmark 结果：Criterion 下 `switch 1k` 大约改善到 `132–136 µs`。

## 9.2 优化 GC 性能

### 9.2.1 替换保守的 `mark_all` 行为

**优先级**：P1

**热点文件**

- `src/gc/collector.rs`
- `src/context.rs`

**原因**

- 当前 collector 仍带有临时性的保守策略：直接标记全部对象。
- 这会阻碍真正有意义的 GC 性能优化。

**任务**

- 用真实 root 发现替换 `mark_all()`。
- 明确定义并遍历真实 roots：
  - stack
  - globals
  - closures
  - active frames
  - runtime-owned containers
- 确认 compact 后的指针更新仍然正确。

**预期收益**

- 降低 GC pause 成本
- 提高对象密集型脚本的可扩展性

### 9.2.2 测量 GC 触发行为

**优先级**：P1

**原因**

- GC 成本不仅取决于 collector 实现，也取决于触发频率。

**任务**

- 测量 benchmark 场景中的 GC 触发频率。
- 记录典型脚本下对象/数组/字符串增长趋势。
- 只有在拿到真实数据后，再调整 GC 触发启发式。

### 9.2.3 降低引擎内部容器扫描成本

**优先级**：P2

**热点文件**

- `src/vm/types.rs`
- `src/context.rs`

**任务**

- 复查这些 runtime 向量的扫描成本：
  - `objects`
  - `closures`
  - `runtime_strings`
  - `typed_arrays`
  - `array_buffers`
- 在有必要时，把热 live data 和长生命周期 metadata 分离。

## 9.3 减少内存使用

### 9.3.1 先把测量做好

**优先级**：P0

**热点文件**

- `src/context.rs`
- `src/vm/types.rs`

**原因**

- `MemoryStats` 已经可用，但内存优化必须基于真实的大头来源。

**任务**

- 以 `MemoryStats` 作为基线测量来源。
- 记录 benchmark 场景下 object/string/closure/typed-array 的数量变化。
- 在确认最大内存类别前，不做激进布局重构。

### 9.3.2 减少热路径中的临时分配

**优先级**：P0

**原因**

- 临时 `Vec` 和瞬时重排会同时增加 CPU 与内存 churn。

**任务**

- 去掉热调用路径中剩余的临时 `Vec<Value>` 分配。
- 检查数组/内建函数密集执行中的短生命周期分配模式。
- 在安全前提下优先使用保留栈布局和借用数据。

### 9.3.3 复查 runtime string 增长

**优先级**：P1

**热点文件**

- `src/vm/interpreter.rs`
- `src/context.rs`

**原因**

- `runtime_strings` 在 `MemoryStats` 中单独计数，可能悄悄增长。

**任务**

- 测量 benchmark 下 `runtime_strings` 的增长曲线。
- 检查字符串提升是否在热路径上过于激进。
- 找出重复字符串创建的机会点。

### 9.3.4 复查 object 与 array 布局开销

**优先级**：P1

**热点文件**

- `src/runtime/object.rs`
- `src/runtime/array.rs`
- `src/vm/types.rs`

**任务**

- 比较 dense array 与通用 object-backed 访问的内存成本。
- 检查高频创建的 runtime 结构是否可以更小。
- 只在测量支撑下做有针对性的布局改动。

## 补充性引擎任务

### S1. 保持 builtin/runtime 边界真实可见

**优先级**：P2

**原因**

- `src/builtins/` 当前基本还是结构占位。
- 真正的 builtin 行为主要在 `src/vm/natives.rs` 和 `src/vm/property.rs`。

**任务**

- 记录真实实现所在位置。
- 避免优化时误把占位模块当成真实热点。
- 除非会阻碍性能工作，否则结构迁移放在热点优化之后。

### S2. 用 benchmark 项驱动优化目标

**优先级**：P0

**标准优化集合**

- `fib` -> 调用路径、递归、算术
- `loop` -> dispatch、算术、局部变量
- `array` -> dense array 快速路径
- `sieve` -> dense array 读写 + 循环成本
- `json` -> 作为已有较好路径的回归保护项

### S3. 扩展缺失的 benchmark 覆盖

**优先级**：P0

**原因**

- 当前 benchmark 集合虽然有价值，但对若干重要引擎路径覆盖仍不足。
- 如果 benchmark 仍只聚焦 `fib`、`loop`、`array`、`sieve`、`json`，一些高价值优化方向会长期不可见。

**建议新增：主集合**

这批 benchmark 最值得优先加入，因为它们最直接暴露真正重要的引擎热点：

- `method_chain`
  - 代表形态：`.map().filter().reduce()`
  - 覆盖：`GetField2`、`CallMethod`、callback 调用、数组链式处理
- `for_of_array`
  - 覆盖：`ForOfStart`、`ForOfNext`、迭代器循环控制
- `deep_property`
  - 代表形态：`a.b.c.d`
  - 覆盖：重复 `GetField` 成本与链式属性访问
- `runtime_string_pressure`
  - 覆盖：`create_runtime_string`、runtime string 增长、字符串分配压力

**建议新增：次集合**

这批同样重要，但更适合作为机制专项 benchmark，而不是第一波主性能榜 benchmark：

- `try_catch`
  - 覆盖：`ExceptionHandler`、throw/catch/finally 控制流、栈展开
- `for_in_object`
  - 覆盖：`ForInStart`、`ForInNext`、对象键迭代
- `switch_case`
  - 覆盖：基于 `Dup + StrictEq + IfTrue` 的多分支分发结构

**对当前 no_std 主线后置**

- `regexp_test`
  - 覆盖：`RegExpObject`、`test`
  - 保留为后续 `std` / 可选 benchmark 候选，不作为第一波 no_std 目标
- `regexp_exec`
  - 覆盖：`RegExpObject`、`exec`
  - 保留为后续 `std` / 可选 benchmark 候选，不作为第一波 no_std 目标

**建议落地顺序**

1. `method_chain`
2. `runtime_string_pressure`
3. `for_of_array`
4. `deep_property`
5. `try_catch`
6. `switch_case`
7. `for_in_object`

后置：

- `regexp_test`
- `regexp_exec`

**预期价值**

- 让 benchmark 驱动的优化更接近真实 JS 使用方式
- 暴露调用密集、迭代器密集、对象访问密集、字符串压力密集的路径
- 让引擎优化不再只盯算术和纯循环

**当前已完成**

- 2026-03-16：已补第一波 benchmark 脚本与 Criterion 覆盖：
  - `method_chain`
  - `runtime_string_pressure`
  - `for_of_array`
  - `deep_property`
- 2026-03-16：已补第二波 `switch_case` benchmark 脚本，用于 CLI 形式的 Rust-vs-C 对比。
- 已通过 `cargo bench --no-run` 验证 benchmark 可编译。
- 2026-03-16：已完成第一轮 `for_of_array` 优化，`ForOfStart` 不再整数组复制，而是改为基于数组索引直接迭代。
- 已补回归测试，确认 `for-of` 在数组迭代过程中能够观察到后续元素更新。
- benchmark 结果：Criterion 下 `for_of_array 20k` 大约从 `4.22–4.47 ms` 降到 `2.36–2.42 ms`。
- 2026-03-16：已补 `for_in_object` benchmark，并完成第一轮迭代器初始化优化，把“预先整批克隆全部 key”改成“基于索引按需生成 key”。
- 已补回归测试，确认 `for-in` 在对象迭代过程中仍能通过静态属性访问观察到更新后的值。
- benchmark 基线已记录：Criterion 下 `for_in_object 20x2000` 为 `3.74–3.80 ms`。

## 推荐执行顺序

1. benchmark 基线收口
2. benchmark 覆盖扩展（优先加入 `method_chain`、`runtime_string_pressure`、`for_of_array`、`deep_property`）
3. 调用路径优化
4. native/builtin 参数传递优化
5. dense array 快速路径
6. 内存测量复盘
7. GC root-based marking
8. opcode dispatch 精简
9. 第二轮微优化

## 完成标准

当满足以下条件时，这份优化清单可以认为“基本完成”：

- benchmark 基线可信、可复现
- `fib`、`loop`、`array`、`sieve` 各自至少有一项经过验证的热点优化
- GC 不再依赖保守的 `mark_all`
- 内存优化基于真实主导类别，而不是猜测
- 文档中只保留有效 benchmark 结论
