# 引擎优化任务清单

本文档是 `mquickjs-rs` **仅面向引擎**的优化待办清单。

它直接源自 `IMPLEMENTATION_PLAN.md` 中尚未完成的第 9 阶段：
- `9.1 Profile and optimize hot paths`
- `9.2 Optimize GC performance`
- `9.3 Reduce memory usage`

本文档*不包含* `led-runtime` 产品层工作。

相关 benchmark 分析：
- `docs/BENCHMARK_ANALYSIS.md`
- `docs/BENCHMARK_ANALYSIS.zh.md`

## 适用范围

本文档只覆盖：
- `mquickjs-rs` 的 parser / compiler / VM / runtime
- benchmark 的正确性与性能分析
- 引擎自身的 GC 与内存行为

本文档不覆盖：
- `led-runtime` 主机 API 人体工学
- effect 脚本/产品语义
- GUI / demo / 产品层集成

## 当前优化主题

结合当前代码和 benchmark 形状，最值得关注的引擎热点是：
- `src/vm/interpreter.rs` 中的调用与方法分发
- `src/vm/interpreter.rs` 和 `src/vm/natives.rs` 中的 native / builtin 参数整理
- `src/vm/interpreter.rs` 和 `src/vm/property.rs` 中的 dense array 访问
- `src/vm/interpreter.rs` 中的 opcode dispatch 开销
- `src/gc/collector.rs` 中的 GC 实现质量
- `src/vm/types.rs`、`src/context.rs` 和 `src/runtime/*` 中的运行时分配与容器布局

## 优先级总结

### P0

- benchmark 真实基线清理
- 调用路径热点优化
- Native/builtin 调用参数传递优化
- Dense array 快速路径

### P1

- 最热点 opcode 的 dispatch 简化
- GC：从保守的 `mark_all` 行为迁移到真正的 root-based marking
- 运行时分配与内存占用评估

### P2

- Builtin/runtime 边界结构清理
- 新 benchmark 验证后的第二轮微优化

## 详细任务清单

## 9.1 分析并优化热点路径

### 9.1.1 Benchmark 基线清理

**优先级**: P0

**原因**

- 只有 benchmark 数据可信，优化工作才有意义。
- 本地脚本和 CI workflow 之前存在不一致。
- 一些历史 benchmark 结论基于错误的对比目标。

**任务**

- 保留一个可信的本地 benchmark 流程用于验证。
- 保持 CI benchmark 行为与本地 benchmark 行为一致。
- 区分：进程启动开销与脚本净执行时间。
- 维护一个统一的基线表，至少覆盖：
  - `fib`
  - `loop`
  - `array`
  - `sieve`
  - `json`

**验证方式**

- Benchmark 多次运行结果可复现。
- `docs/BENCHMARK_ANALYSIS.md` 内部一致。

**当前已完成**

- 2026-03-16：已定义当前规范的 benchmark 集合。
- 2026-03-16：已分离并记录本地 Criterion、本地 Rust-vs-C 对比、CI Summary 三者的职责。
- 2026-03-16：`.github/workflows/bench.yml` 现在会同时输出 Rust-vs-C 对比表和 Rust-only 的 Criterion 表。
- 2026-03-16：`docs/BENCHMARK_ANALYSIS.md` 已重写为当前基线参考文档。
- 状态：对于当前引擎优化阶段，此任务可视为已完成。

### 9.1.2 调用路径热点优化

**优先级**: P0

**热点文件**

- `src/vm/interpreter.rs`
- `src/vm/stack.rs`

**原因**

- `fib` 和 `loop` 强烈表明调用开销与高频 dispatch 开销仍是主要成本。
- 当前 `Call` 路径已有改进，但 `remove_at_offset()` 仍会调用 `Vec::remove()`，导致元素迁移。

**任务**

- 重新设计调用栈布局，避免在热调用路径上使用 `Vec::remove()`。
- 分别针对 `Call`、`CallMethod`、`CallConstructor` 做专门优化。
- 减少普通 JS 函数调用中的临时参数重排。
- 重新评估调用路径中的字符串提升成本。

**预期收益**

- 主要改善目标：`fib`
- 次要改善目标：`loop`

**当前已完成**

- 2026-03-16：第一轮 `method_chain` 相关优化已完成，在数组高阶方法中去除了每个元素回调时的临时 `Vec<Value>` 参数分配。
- 已添加链式 `map().filter().reduce()` 行为回归测试。
- Benchmark 结果：Criterion 中 `method_chain 5k` 从约 `1.88-1.54 ms` 提升到 `0.80-0.82 ms`。

### 9.1.3 Native/builtin 调用参数传递优化

**优先级**: P0

**热点文件**

- `src/vm/interpreter.rs`
- `src/vm/natives.rs`

**原因**

- Native 和 builtin 调用仍在构建临时 `Vec<Value>` 缓冲区并做 reverse。
- 此路径影响 `Math.*`、`JSON.*`、数组方法和其他内置函数。

**任务**

- 为 0/1/2 参数的 native 调用增加专门的快速路径。
- 避免为短参数列表进行堆分配。
- 减少或消除 native/builtin 调用准备阶段的 `reverse()`。
- 在安全前提下考虑使用基于栈的参数切片传递。

**预期收益**

- 改善内置函数密集型脚本
- 帮助 `array`、`json` 和数学密集型工作负载

**当前已完成**

- 2026-03-16：为 `CallMethod` 的 native 路径添加了小参数数量的快速路径，在 `argc <= 2` 时去除临时参数 `Vec` 分配。
- 已添加多参数 `Array.prototype.push` 参数顺序回归测试。
- Benchmark 结果：Criterion 中 `array push 10k` 从约 `0.897-0.911 ms` 提升到 `0.672-0.691 ms`。
- Benchmark 结果：Criterion 中 `method_chain 5k` 进一步从约 `0.986-1.182 ms` 提升到 `0.720-0.763 ms`。
- 2026-03-16：在 `CallMethod` 中为 `Array.prototype.push` 添加了原生快速路径，并针对 `argc == 1` 场景增加了专用快捷方式，从热数组初始化路径中消除了通用 native-call 开销。
- 重用现有 `Array.prototype.push` 回归测试验证语义。
- Benchmark 结果：Criterion 中 `sieve 10k` 从约 `2.038-2.078 ms` 提升到 `2.014-2.074 ms`。

### 9.1.4 Dense array 快速路径

**优先级**: P0

**热点文件**

- `src/vm/interpreter.rs`
- `src/vm/property.rs`
- `src/runtime/array.rs`

**原因**

- `array` 和 `sieve` 是典型的 dense-array benchmark。
- 当前访问仍经过若干通用层。

**任务**

- 缩短 `GetArrayEl`、`GetArrayEl2` 和 `PutArrayEl` 路径。
- 为 dense integer-index 访问做专门处理。
- 对明显的数组操作避免通用 property lookup。
- 分别审查 `push`、索引读取和索引写入路径。

**预期收益**

- 主要改善目标：`array`
- 对 `sieve` 也有明显改善

**当前已完成**

- 2026-03-16：完成第一个深度属性优化，为常规对象属性查找添加了小对象快速路径，并统一了 `GetField` / `GetField2` 的属性分发路径。
- 已添加深度属性链访问回归测试。
- Benchmark 结果：Criterion 中 `deep_property 200k` 从约 `28-29 ms` 提升到 `15.7-17.0 ms`。

### 9.1.5 Opcode dispatch 精简

**优先级**: P1

**热点文件**

- `src/vm/interpreter.rs`

**原因**

- `loop` 仍表明有意义的指令 dispatch 开销。
- 大型基于 match 的 dispatch 正确且可维护，但在最热路径上仍有效益成本。

**任务**

- 通过 benchmark 驱动分析找出最热的 10-20 个 opcode。
- 缩短 dispatch 循环中每次迭代的工作。
- 减少热指令中的重复 decode / branch / error-path 开销。
- 对算术、局部变量、跳转和调用指令优先做本地快速路径。

**预期收益**

- `loop` 的最佳次要目标
- 广泛惠及多个 benchmark

**当前已完成**

- 2026-03-16：添加了 `try_catch` benchmark，覆盖重复 throw/catch 控制流。
- 2026-03-16：通过统一异常分发和用基于 `truncate` / `drop_n` 的 unwind 替代重复的 pop unwind，降低了异常路由开销。
- 已添加"循环内重复 throw/catch"回归测试。
- Benchmark 结果：Criterion 中 `try_catch 5k` 基线记录为 `340-349 μs`。
- 2026-03-16：在 `dump` feature 下添加了运行时 opcode 计数器，并通过 `Context` 暴露给 profiling 工作使用。
- 已添加 `dump` 模式回归测试，确保 opcode 计数记录真实执行。
- 运行时热点发现：
  - `loop` 主要由 `GetLoc1`、`Goto`、`Add`、`Dup`、`Drop`、`GetLoc0`、`PutLoc0`、`PutLoc1`、`Lt`、`IfFalse` 主导。
  - `sieve` 主要由 `Goto`、`Drop`、`IfFalse`、`GetLoc3`、`Add`、`Dup`、`GetLoc0`、`Lte`、`GetLoc2`、`PutArrayEl`、`PutLoc3`、`GetArrayEl`、`CallMethod` 主导。
- 当前判断：下一个基于证据的优化目标更可能是 `Dup/Drop` + 局部存储使用模式或分支/控制流成本，而不是继续优化单个算术 helper。
- 2026-03-16：完成了 `Dup + PutLocX + Drop` peephole 快速路径，用于 `i = i + 1;` 这类常见语句更新模式。
- 已添加局部赋值语句更新回归测试，同时保留赋值表达式行为。
- Benchmark 结果：Criterion 中 `loop 10k` 从约 `0.513-0.525 ms` 提升到 `0.486-0.492 ms`。
- Benchmark 结果：Criterion 中 `sieve 10k` 从约 `2.257-2.310 ms` 提升到 `2.152-2.191 ms`。
- 2026-03-16：通过用直接快速路径栈操作替换通用 checked helper，优化了热 `Dup` / `Drop` opcode 处理器本身。
- 重用相同的局部赋值和赋值表达式回归测试来验证更改。
- 当前基线以 `docs/BENCHMARK_ANALYSIS.md` 为准。
- 2026-03-16：为紧跟 `IfFalse` / `IfTrue` 的 `Lt/Lte` 添加了分支融合快速路径，使比较结果可以直接跳转，而无需在栈上实例化临时布尔值。
- 重用现有 `while`、`switch` 和 `try_catch` 控制流回归测试来验证语义。
- Benchmark 结果：Criterion 中 `loop 10k` 从约 `0.502-0.514 ms` 提升到 `0.484-0.499 ms`。
- Benchmark 结果：Criterion 中 `sieve 10k` 从约 `2.164-2.207 ms` 提升到 `2.038-2.078 ms`。

### 9.1.6 算术/比较微优化

**优先级**: P1

**热点文件**

- `src/vm/ops.rs`

**原因**

- 核心算术和比较 helper 已经部分内联。
- 这块仍有意义，但其预期收益低于 call/array/native 热路径。

**任务**

- 审计剩余热点 `op_*` helper 是否真正值得内联。
- 减少常见 int/int 和 int/float 路径上的重复数值强制转换。
- 在 benchmark 基线稳定后重新评估相等和比较快速路径。

**预期收益**

- 小但广泛的改善

**当前已完成**

- 2026-03-16：通过在单个输出缓冲区中构建最终运行时字符串，而不是先实例化两个操作数为临时拥有的 `String` 值，改进了字符串拼接热路径。
- 已添加混合字符串/数字链式拼接形状回归测试。
- Benchmark 结果：Criterion 中 `runtime_string_pressure 4k` 从约 `2.89-3.38 ms` 提升到 `1.53-1.55 ms`。
- 2026-03-16：通过在同一值、整数和布尔比较添加直接快速路径，然后回退到较慢的通用处理，改进了 `StrictEq` / `StrictNeq` 热 opcode 处理。
- 重新运行了现有 switch 语义回归测试。
- Benchmark 结果：Criterion 中 `switch 1k` 从约 `145-149 μs` 类性能提升到 `132-136 μs`。

## 9.2 优化 GC 性能

### 9.2.1 替换保守的 `mark_all` 行为

**优先级**: P1

**热点文件**

- `src/gc/collector.rs`
- `src/context.rs`

**原因**

- 当前 collector 仍包含保守的临时方案，标记所有对象。
- 这阻碍了有意义的 GC 性能工作。

**任务**

- 用真正的 root 发现替换 `mark_all()`。
- 明确并遍历真正的 roots：
  - stack
  - globals
  - closures
  - active frames
  - runtime-owned containers
- 验证 compaction 后指针更新仍然正确。

**预期收益**

- 降低 GC pause 成本
- 改善对象密集型脚本的可扩展性

### 9.2.2 测量 GC 触发行为

**优先级**: P1

**原因**

- GC 成本不仅取决于 collector 实现，还取决于触发频率。

**任务**

- 测量 benchmark 工作负载期间的 GC 频率。
- 记录代表性脚本的对象/数组/字符串增长。
- 只在收集到真实数据后调整触发启发式。

### 9.2.3 降低引擎自有容器的扫描成本

**优先级**: P2

**热点文件**

- `src/vm/types.rs`
- `src/context.rs`

**任务**

- 审查这些运行时向量的扫描成本：
  - `objects`
  - `closures`
  - `runtime_strings`
  - `typed_arrays`
  - `array_buffers`
- 在有用时将热 live data 与长生命周期 metadata 分离。

## 9.3 减少内存使用

### 9.3.1 先做好测量

**优先级**: P0

**热点文件**

- `src/context.rs`
- `src/vm/types.rs`

**原因**

- `MemoryStats` 已经可用，但内存优化必须基于真实的主要来源。

**任务**

- 以 `MemoryStats` 作为基线测量来源。
- 记录 benchmark 脚本中的 object/string/closure/typed-array 数量变化。
- 在确认最大内存类别之前，不做激进的布局重构。

**当前已完成**

- 2026-03-16：将 `MemoryStats` / `InterpreterStats` 从"只管对象数量"扩展为更细粒度的统计，包括：
  - `runtime_string_bytes`
  - `array_elements`
  - `object_properties`
  - `typed_array_bytes`
  - `array_buffers`
  - `array_buffer_bytes`
- 已同步更新 CLI 的内存转储输出。
- 已添加回归测试，覆盖：
  - 数组/对象形状统计
  - 运行时字符串字节统计
- 状态：现已具备继续推进 `9.3` 的测量基础。

### 9.3.2 减少热执行路径中的临时分配

**优先级**: P0

**原因**

- 临时 Vec 和瞬态重排会增加 CPU 和内存 churn。

**任务**

- 去除热调用路径中剩余的临时 `Vec<Value>` 分配。
- 审查数组/builtin 密集型执行中的短生命周期分配模式。
- 在安全前提下优先使用保留栈的布局和借用的数据。

### 9.3.3 审查运行时字符串增长

**优先级**: P1

**热点文件**

- `src/vm/interpreter.rs`
- `src/context.rs`

**原因**

- `runtime_strings` 在 `MemoryStats` 中被单独计数，可能悄悄增长。

**任务**

- 测量 benchmark 中 `runtime_strings` 的增长曲线。
- 检查字符串提升在热路径中是否过于激进。
- 找出重复字符串创建的机会。

**当前已完成**

- 2026-03-16：在 `dump` feature 下添加了运行时字符串来源计数器，至少区分以下类别：
  - 运行时字符串创建请求总数
  - 拼接驱动的创建
  - for-in key 创建
  - 其他创建路径
- 通过 `Context` 在 `dump` feature 下暴露了这些计数器。
- 已添加 dump 模式回归测试，确保运行时字符串来源统计被正确记录。
- 状态：9.3.3 已具备安全的测量/profiling 基础；字符串复用/去重的优化策略仍有意推迟决定。
- 嵌入式说明：暂不在引擎中硬编码运行时字符串字节预算；最终限制将在 ESP32 级目标的真实设备集成阶段确定。
- 2026-03-16：在 for-in key 路径上，运行时字符串表耗尽现在变为受控引擎错误（`runtime string table exhausted`），而不再是调试时的 overflow panic。
- 已添加回归测试，锁定针对重复 `for-in` key 生成的新受控错误行为。
- 简言之：for-in key 路径上之前会 panic 的运行时字符串溢出，现在改为受控引擎错误，不再崩溃进程。

### 9.3.4 审查 object 和 array 布局开销

**优先级**: P1

**热点文件**

- `src/runtime/object.rs`
- `src/runtime/array.rs`
- `src/vm/types.rs`

**任务**

- 比较 dense array 与通用 object-backed 访问的内存成本。
- 检查频繁创建的运行时结构是否可以更小。
- 只在测量支撑下做有针对性的布局改动。

## 支持性引擎任务

### S1. 保持 builtin/runtime 边界真实

**优先级**: P2

**原因**

- `src/builtins/` 当前基本上还是结构占位符代码。
- 真正的 builtin 行为主要在 `src/vm/natives.rs` 和 `src/vm/property.rs` 中。

**任务**

- 记录真实实现所在位置。
- 避免优化时误把占位符模块当作热点。
- 除非阻碍性能工作，否则推迟结构迁移到热点工作之后。

### S2. 用 benchmark 驱动优化目标

**优先级**: P0

**标准优化集合**

- `fib` -> 调用路径、递归、算术
- `loop` -> dispatch、算术、局部变量
- `array` -> dense array 快速路径
- `sieve` -> dense array 读写 + 循环成本
- `json` -> 作为已有好路径的回归保护

### S3. 扩展缺失的 benchmark 覆盖

**优先级**: P0

**原因**

- 当前 benchmark 集合已有价值，但对更多重要引擎路径覆盖仍不够。
- 如果 benchmark 仍只聚焦 `fib`、`loop`、`array`、`sieve`、`json`，一些高价值优化方向将长期不可见。

**建议新增：主集合**

这些 benchmark 最值得优先加入，因为它们最直接暴露有意义的引擎热点：

- `method_chain`
  - 代表形状：`.map().filter().reduce()`
  - 覆盖：`GetField2`、`CallMethod`、回调调用、数组链式处理
- `for_of_array`
  - 覆盖：`ForOfStart`、`ForOfNext`、迭代器循环控制
- `deep_property`
  - 代表形状：`a.b.c.d`
  - 覆盖：重复 `GetField` 成本与链式属性访问
- `runtime_string_pressure`
  - 覆盖：`create_runtime_string`、运行时字符串增长、字符串分配压力

**建议新增：次集合**

这些同样重要，但更适合作为机制特定的 benchmark，而不是第一波主要性能 benchmark：

- `try_catch`
  - 覆盖：`ExceptionHandler`、throw/catch/finally 控制流、栈展开
- `for_in_object`
  - 覆盖：`ForInStart`、`ForInNext`、对象键迭代
- `switch_case`
  - 覆盖：基于 `Dup + StrictEq + IfTrue` 的多分支分发结构

**为当前 no_std 路径延后**

- `regexp_test`
  - 覆盖：`RegExpObject`、`test`
  - 保留为后置 `std` / 可选 benchmark 候选，不作为第一波 no_std 目标
- `regexp_exec`
  - 覆盖：`RegExpObject`、`exec`
  - 保留为后置 `std` / 可选 benchmark 候选，不作为第一波 no_std 目标

**建议落地顺序**

1. `method_chain`
2. `runtime_string_pressure`
3. `for_of_array`
4. `deep_property`
5. `try_catch`
6. `switch_case`
7. `for_in_object`

延后：
- `regexp_test`
- `regexp_exec`

**预期价值**

- 让 benchmark 驱动的优化更代表真实 JS 使用方式
- 暴露调用密集型、迭代器密集型、对象访问密集型和字符串压力密集型路径
- 让引擎优化工作不再只看算术和原始循环

**当前已完成**

- 2026-03-16：添加了第一波 benchmark 脚本和 Criterion 覆盖：
  - `method_chain`
  - `runtime_string_pressure`
  - `for_of_array`
  - `deep_property`
- 2026-03-16：添加了第二波 `switch_case` benchmark 脚本，用于 CLI 风格的 Rust-vs-C 对比。
- 已通过 `cargo bench --no-run` 验证 benchmark 可编译。
- 2026-03-16：完成了第一轮 `for_of_array` 优化，`ForOfStart` 不再整数组复制，而是改为基于数组索引直接迭代。
- 已添加回归测试，确认 `for-of` 在数组迭代过程中能够观察到后续元素更改。
- Benchmark 结果：Criterion 中 `for_of_array 20k` 从约 `4.22-4.47 ms` 提升到 `2.36-2.42 ms`。
- 2026-03-16：添加了 `for_in_object` benchmark 覆盖，并完成了第一轮迭代器初始化优化，将"预先生成全部 key"改为"基于快照按需生成 key"。
- 已添加回归测试，确认 `for-in` 在对象迭代过程中仍能通过静态属性读取观察到更新后的值。
- Benchmark 基线记录：Criterion 中 `for_in_object 20x2000` 为 `3.74-3.80 ms`。

## 推荐执行顺序

1. Benchmark 基线清理
2. Benchmark 覆盖扩展（优先加入 `method_chain`、`runtime_string_pressure`、`for_of_array`、`deep_property`）
3. 调用路径优化
4. Native/builtin 参数传递优化
5. Dense array 快速路径
6. 内存测量评估
7. GC root-based marking 工作
8. Opcode dispatch 精简
9. 第二轮微优化

## 完成标准

当满足以下条件时，这份优化任务清单可视为"基本完成"：

- benchmark 基线可信、可复现
- `fib`、`loop`、`array`、`sieve` 各自至少有一个经过验证的热点改善
- GC 不再依赖保守的 `mark_all`
- 内存减少工作基于测量的主要类别，而非猜测
- 文档只记录有效的 benchmark 结论
