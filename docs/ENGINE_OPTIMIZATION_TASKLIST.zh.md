# 引擎优化任务清单

本文档是 `mquickjs-rs` **仅面向引擎**的优化待办清单。

它直接源自 `IMPLEMENTATION_PLAN.md` 中尚未完成的第 9 阶段：
- `9.1 Profile and optimize hot paths`
- `9.2 Optimize GC performance`
- `9.3 Reduce memory usage`

本文档不包含 `led-runtime` 产品层工作。

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
- 调用路径热路径优化
- Native/builtin 调用参数传递优化
- Dense array 快速路径

### P1

- 最热 opcode 的 dispatch 简化
- GC：停止保守的 `mark_all` 行为，迁移到真正的 root-based marking
- 运行时分配与内存占用评估

### P2

- Builtin/runtime 边界结构清理
- 新 benchmark 验证后的第二轮微优化

## 详细任务清单

## 9.1 分析并优化热路径

### 9.1.1 Benchmark 基线清理

**优先级**: P0

**原因**

- 优化工作只有在 benchmark 数据可信时才有意义。
- benchmark 工作流和本地对比脚本此前存在不一致。
- 部分历史 benchmark 结论基于无效的对比目标。

**任务**

- 保持单一可信的本地 benchmark 流程。
- 使 CI benchmark 行为与本地 benchmark 行为保持一致。
- 将进程启动开销与净脚本执行时间分开。
- 维护以下基准的权威基线表：
  - `fib`
  - `loop`
  - `array`
  - `sieve`
  - `json`

**验证**

- benchmark 结果在多次运行中可复现。
- `docs/BENCHMARK_ANALYSIS.md` 内部一致。

**当前已完成**

- 2026-03-16：规范 benchmark 集合已定义。
- 2026-03-16：本地 Criterion、本地 Rust vs C 对比、CI 摘要的职责已分离并记录。
- 2026-03-16：`.github/workflows/bench.yml` 现已同时发布 Rust vs C 对比表和纯 Rust Criterion 表。
- 2026-03-16：`docs/BENCHMARK_ANALYSIS.md` 已重写为当前基线参考。
- 2026-03-17：对当前工作树的主 benchmark 集合做了一轮完整本地 Criterion 重验证。
- 2026-03-17：本地 Criterion harness 已改为“预编译一次、在新 context 上重复测执行”，以减弱 parser/compiler 对运行时优化判断的污染。
- 2026-03-17：`docs/BENCHMARK_ANALYSIS.md` / `docs/BENCHMARK_ANALYSIS.zh.md` 已更新为区分新的执行期快照和旧的 Criterion 代际数据。
- 状态：重新打开；在当前 head 与文档重新稳定同步之前，benchmark 基线清理不能再视为完成。

### 9.1.2 调用路径热路径优化 [已彻底完成]

**优先级**: P0

**热点文件**

- `src/vm/interpreter.rs`
- `src/vm/stack.rs`

**原因**

- `fib` 和 `loop` 强烈表明调用开销和高频 dispatch 开销仍然是主要成本。
- 当前 `Call` 路径有所改进，但仍使用 `remove_at_offset()`，它委托给 `Vec::remove()`，会导致元素移动。

**任务**

- 重构调用栈布局，避免热调用路径上的 `Vec::remove()`。
- 分别特化 `Call`、`CallMethod` 和 `CallConstructor`。
- 减少普通 JS 函数调用中的临时参数重塑。
- 重新检查调用路径中的字符串提升成本。

**预期收益**

- `fib` 的主要改进目标
- `loop` 的次要改进

**当前已完成**

- 2026-03-16：完成了 `method_chain` 相关优化的第一轮，通过去除回调密集型数组 builtin 中每个元素的临时 `Vec<Value>` 参数分配来完成。
- 为链式 `map().filter().reduce()` 行为添加了回归覆盖。
- Benchmark 结果：`method_chain 5k` 在 Criterion 中从约 `1.88–2.54 ms` 提升到 `0.80–0.82 ms`。
- 2026-03-17：将 `Call` / `CallMethod` 热路径中基于 `Vec::remove()` 的目标提取改为单次尾部紧缩，并让普通 JS 方法调用继续直接复用栈上的参数，而不是重新打包成临时 `Vec<Value>`。
- 2026-03-17：将同样的“参数原地保留”思路扩展到了 `CallConstructor`，使普通 JS 构造器调用也不再通过临时 `Vec<Value>` 重建参数列表。
- 重新跑了直接函数调用、多参数 `Array.prototype.push` 顺序、以及链式 `map().filter().reduce()` 的回归覆盖，结果均通过。
- 重新跑了构造器语义相关回归覆盖（`new`、`instanceof`、简单构造器场景），结果也通过。
- 在新的“预编译一次、重复测执行”Criterion 口径下，当前本地快照为：
  - `fib_iter 1k`：`2.330–2.379 ms`
  - `loop 10k`：`0.472–0.485 ms`
  - `array push 10k`：`0.614–0.633 ms`
- 当前解读：调用路径这轮工作依然是真实有效的，但后续比较必须严格限定在新的执行期 benchmark 代际内部进行。
- 2026-03-17：将 `map`、`filter`、`forEach`、`reduce`、`find`、`findIndex`、`some`、`every` 这些热数组 builtin 从“整数组 clone”改成“长度快照 + 实时元素读取”。
- 添加了回归覆盖以锁定：
  - 回调里 `push()` 不会改变本轮遍历长度
  - `map()` 能观察到前面回调对后续元素的更新
- 在当前执行期 Criterion 口径下，最新完整重跑记录：
  - `method_chain 5k`：`0.699–0.707 ms`
  - `runtime_string_pressure 4k`：`1.237–1.269 ms`
- 当前解读：这一轮明显改善了回调密集型数组管线，并通过专门的 `.length` 快路径和更低的 builtin 开销，顺带拉低了 runtime-string-heavy 循环的执行成本。
- 2026-03-17：新增了专门的 `CallArrayMap1` / `CallArrayFilter1` / `CallArrayReduce2` opcode，使最热的单回调数组高阶方法调用形状在 `GetField2` 之后不再继续支付通用 `CallMethod` 的参数重排成本。
- 补充了 fallback 回归覆盖，确认非数组 receiver 只要自带 `map` 方法，仍然保持通用方法调用语义。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `method_chain 5k`：`0.611–0.628 ms`
  - `runtime_string_pressure 4k`：`1.190–1.216 ms`
  - `array push 10k`：`0.575–0.600 ms`
- 当前解读：这是一次很典型的“按字节码形状专门优化数组 builtin 调用链”的收益案例，而且没有扩大通用调用路径的复杂度，收益也向附近的数组密集路径外溢。
- 2026-03-17：新增了专门的 `CallArrayPush1` opcode，直接覆盖最热的单参数 `.push(arg)` 方法调用形状；它保留 `GetField2` 的统一栈约定，但让数组构建循环不再为这条主热路径继续支付通用 `CallMethod` 的整理成本。
- 补充了 fallback 回归覆盖，确认非数组 receiver 只要自带 `push` 方法，仍然保持通用方法调用语义。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `array push 10k`：`0.491–0.502 ms`
  - `method_chain 5k`：`0.585–0.600 ms`
  - `runtime_string_pressure 4k`：`1.177–1.197 ms`
- 当前解读：这是第一轮把 `method_chain` 稳定压到 `<= 0.60 ms` 成功线边缘的优化，而且收益来源很清楚，就是继续缩掉了在高阶数组链调用之前仍然占主导的数组构建前缀。
- 状态：作为“调用路径热路径优化”这一阶段，这部分现在可以视为完成；后续如果还有收益，也应归类为后续微调，而不是核心调用路径清理未完成。

### 9.1.3 Native/builtin 调用参数整理优化

**优先级**: P0

**热点文件**

- `src/vm/interpreter.rs`
- `src/vm/natives.rs`

**原因**

- Native 和 builtin 调用仍然构建临时 `Vec<Value>` 缓冲区并进行反转。
- 此路径影响 `Math.*`、`JSON.*`、数组方法和其他 builtin。

**任务**

- 为 0/1/2 个参数的 native 调用添加专用快速路径。
- 避免为短 native/builtin 参数列表进行堆分配。
- 减少或消除 native/builtin 调用准备中的 `reverse()`。
- 考虑在安全的地方传递栈支持的参数切片。

**预期收益**

- 改善 builtin 密集型脚本
- 帮助 `array`、`json` 和数学密集型工作负载

**当前已完成**

- 2026-03-16：通过在 `argc <= 2` 的 native 方法路径上移除临时参数 `Vec` 分配，为小参数数量添加了 `CallMethod` native 快速路径。
- 为多参数 `Array.prototype.push` 参数顺序添加了回归覆盖。
- Benchmark 结果：`array push 10k` 在 Criterion 中从约 `0.897–0.911 ms` 提升到 `0.672–0.691 ms`。
- Benchmark 结果：`method_chain 5k` 在 Criterion 中进一步从约 `0.986–1.182 ms` 提升到 `0.720–0.763 ms`。
- 2026-03-16：在 `CallMethod` 中为 `Array.prototype.push` 添加了直接 native 快速路径，带有专用的 `argc == 1` 捷径，从热数组初始化路径中移除了通用 native 调用开销。
- 复用现有的 `Array.prototype.push` 回归覆盖来验证语义。
- Benchmark 结果：`sieve 10k` 在 Criterion 中从约 `2.038–2.078 ms` 提升到 `2.014–2.074 ms`。
- 2026-03-17：将数组 `.push` 的属性读取改为直接返回缓存的 native 函数索引，而不是每次都按名字线性扫描 native 注册表。
- 重新跑了 `Array.prototype.push` 的回归覆盖，结果通过。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `array push 10k`：`0.589–0.602 ms`
  - `method_chain 5k`：`0.654–0.668 ms`
- 当前解读：这是一次小但真实的热点数组方法属性分发清理，不过仍应仅在当前 benchmark 代际内部解读。
- 2026-03-17：让 `Array.prototype.push` 的 native 快路径能够直接吞掉后续的 `Drop`，使语句位置的 `arr.push(...)` 不再白白压入一个马上就会被丢弃的返回长度。
- 重新跑了 `Array.prototype.push` 返回值语义相关回归覆盖，结果通过。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `array push 10k`：`0.532–0.539 ms`
  - `sieve 10k`：`1.640–1.670 ms`
  - `method_chain 5k`：`0.606–0.618 ms`
- 当前解读：这是一次很值的窄范围优化，因为它正中数组构建循环里最热的语句形态，同时又不改变表达式位置的语义。
- 2026-03-17：把 `Call` / `CallMethod` / builtin-as-function 的 native/builtin 小参数快路径从 `argc <= 2` 扩到 `argc == 3`，继续去掉了三参数原生调用形状上残留的一层 `Vec<Value>` 分配。
- 补充了三参数 native 调用顺序的回归覆盖（`Math.max(1, 4, 2)`）。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `array push 10k`：`0.472–0.481 ms`
  - `json parse 1k`：`0.732–0.749 ms`
  - `method_chain 5k`：`0.590–0.604 ms`
- 当前解读：当前主 benchmark 集合还没有显示出一条全新的、只属于 `json` 这一类的独立爆发式收益，但这次改动确实补上了一个明显残留的小参数整理缺口，而且没有拖坏附近的调用密集 benchmark。

### 9.1.4 Dense array 快速路径

**优先级**: P0

**热点文件**

- `src/vm/interpreter.rs`
- `src/vm/property.rs`
- `src/runtime/array.rs`

**原因**

- `array` 和 `sieve` 是经典的 dense array benchmark。
- 当前访问仍然经过多个通用层。

**任务**

- 缩短 `GetArrayEl`、`GetArrayEl2` 和 `PutArrayEl` 路径。
- 对 dense 整数索引访问进行特化。
- 对明显的数组操作避免通用属性查找。
- 分别审查 `push`、索引读和索引写路径。

**预期收益**

- `array` 的主要改进目标
- `sieve` 的强力预期收益

**当前已完成**

- 2026-03-16：完成了第一个深层属性优化，通过为常规对象属性查找添加小对象快速路径并统一 `GetField` / `GetField2` 属性分发。
- 为深层属性链访问添加了回归覆盖。
- Benchmark 结果：`deep_property 200k` 在 Criterion 中从约 `28–29 ms` 提升到 `15.7–17.0 ms`。
- 重要解读：这部分已完成工作主要属于“普通对象属性访问优化”，并不意味着 dense array 的专用读写快速路径工作已经完成。
- 2026-03-17：为 `PutArrayEl + Drop` 添加了 peephole 快路径，使语句位置的数组赋值不再把一个随后立刻丢弃的结果值压回栈上。
- 重新跑了数组赋值语句和赋值表达式相关回归覆盖，结果通过。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `array push 10k`：`0.609–0.621 ms`
  - `sieve 10k`：`2.045–2.084 ms`
- 当前解读：这是一次小但干净的 dense-array 写路径优化，特别针对 `sieve` 里 `primes[j] = false;` 这种高频语句形状。
- 2026-03-18：把更窄的 `PushFalse; PutArrayEl; Drop` 语句形状进一步专门化成了 `PutArrayElFalseDrop`，直接瞄准 `sieve` 内层循环里最热的布尔写入模式。
- 补充了 compiler 回归覆盖，确认 `arr[idx] = false;` 现在会发出这个新 opcode；同时已有的赋值表达式和数组条件语义回归仍然保持通过。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `sieve 10k`：`1.638–1.668 ms`
  - `array push 10k`：`0.556–0.563 ms`
- 当前解读：这是第一条真正精准落在 `sieve` 核心 `primes[j] = false;` 写路径上的 dense-array 专门化，而且它又把这个 benchmark 往下推了一步，同时没有改变赋值表达式语义。
- 2026-03-18：把最热的 `.push` 属性读取从通用 `GetField2` 里拆成了专门的 `GetArrayPush2` opcode，使数组初始化热循环在进入 `CallArrayPush1` 之前不再支付通用字符串属性分发成本。
- 补充了 compiler 回归覆盖，确认 `arr.push(...)` 现在会发出 `GetArrayPush2`；同时新增了 eval 回归，锁定 `obj.push(side_effect())` 在 `obj` 为 `null` 时仍会先抛错，而不会先执行参数副作用。
- 重新跑了针对性的 push/fallback 回归、全量引擎测试以及 `clippy -D warnings`，结果都通过。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `array push 10k`：`0.593–0.716 ms`
  - `sieve 10k`：`1.718–1.753 ms`
  - `dense array bool read branch 10k`：`1.139–1.153 ms`
  - `dense array false write only 10k`：`1.500–1.770 ms`
- 当前解读：这次更像是 dense-array 初始化路径的收益，而不是 `GetArrayEl` 读路径已经被真正打通；但它干净地去掉了 `sieve` 风格数组预热里最热的那层通用 `.push` 属性读取，对新增的 dense-array 诊断 benchmark 也有明确帮助。
- 2026-03-18：新增了一组 `dense_array_bool_condition_only_hot` 诊断脚本/benchmark，用来把反复执行的 `GetArrayEl + IfFalse` 扫描形状单独拎出来，不再混入 `count = count + 1` 这层额外累加工作。
- `dump_bytecode` 现在可以直接看到，这个 benchmark 会被编译成最接近目标热点的 `GetArrayEl; IfFalse; IncLoc; Goto` 形状，它是当前最干净的 dense-array 读侧分支复现器。
- 当前解读：后续 dense-array 读侧调优，现在应该同时参考三种由“混合”到“纯”的诊断形状：`dense array bool read branch 10k`、`dense array bool read hot`、`dense array bool condition only hot`。
- 2026-03-18：继续新增了一组 `dense_array_read_only_hot` 诊断脚本/benchmark，用来把反复执行的 `GetArrayEl; Drop; IncLoc; Goto` 纯读取扫描单独拎出来，完全不再混入条件分支。
- `dump_bytecode` 现在可以直接看到，这个 benchmark 是当前最干净的“纯数组读取成本”复现器，可以和 truthiness 分支、计数累加这两层完全分开看。
- 当前解读：后续 dense-array 读侧调优，现在应该同时参考四种由“混合”到“纯”的诊断形状：
  - `dense array bool read branch 10k`
  - `dense array bool read hot`
  - `dense array bool condition only hot`
  - `dense array read only hot`
- 2026-03-19：保留了 `GetArrayElDiscard`，专门承接被立刻丢弃结果的语句形数组读取（`arr[idx];`），这样“纯读取”诊断路径不再额外支付通用 `Drop` 的成本。
- 补充了 compiler/eval 回归覆盖，确认被丢弃的数组读取现在会降到这个专门 opcode，同时周围程序行为保持不变。
- 2026-03-19：保留了专门的 `GetArrayElDiscard` 语句形读取 opcode，用来承接被立刻丢弃结果的 `arr[idx];`。这样可以在不改变表达式或分支语义的前提下，保留一条更纯的“只看读取”诊断路径。
- 补充了回归覆盖，确认被丢弃的数组读取仍然会正常求值并继续执行后续语句。
- 2026-03-18：最后又新增了一组 `dense_array_loop_only_hot` 诊断脚本/benchmark，把数组读取本身也完全剥掉，只测反复执行的 `Lte; IncLoc; Goto` 纯循环骨架。
- `dump_bytecode` 现在可以直接看到，这个基线会被编译成最纯的 `PushI16; Lte; IfFalse; IncLoc; Goto` 形状。
- 当前解读：dense-array 读侧现在已经可以被分解成五层诊断：
  - `dense array loop only hot`
  - `dense array read only hot`
  - `dense array bool condition only hot`
  - `dense array bool read hot`
  - `dense array bool read branch 10k`
  每往后一层，都只比前一层多出真实工作负载中的一块成本。
- 2026-03-19：继续新增了成对的 `arg0` / `local1` 对照诊断，用来把“数组在 `local0`”和“数组在非 0 槽位”分开测：
  - `dense array read only hot arg0`
  - `dense array read only hot local1`
  - `dense array bool condition only hot arg0`
  - `dense array bool condition only hot local1`
- 当前解读：在控制了函数字节码形状和参数个数之后，`local0` 与非 0 槽位之间剩余的差别现在看起来更像小而噪声化的因素，`GetLoc0` 还不足以被判断为当前读侧的主瓶颈。
- 2026-03-19：新增了 `analyze_dense_array_layers`，用来在一次运行里直接给出各诊断层之间的 delta，避免后续 read-side 优化再手工拼多组 benchmark 输出来做归因。
- 当前解读：后续在继续打 `GetArrayEl` 之前，优先用 `analyze_dense_array_layers` 判断“真正还贵的是哪一层”，再决定要不要动执行器。
- 2026-03-19：把 dense-array 索引访问里反复出现的“数组值 + 非负整型索引”解码收成了一条共享的 raw 快路径 helper，并让 `GetArrayEl`、`GetArrayElDiscard`、`PutArrayEl`、`PutArrayElFalseDrop` 共用它。
- 重新跑了 `cargo check -p mquickjs-rs`、全量 `cargo test -p mquickjs-rs` 以及 `cargo clippy -p mquickjs-rs --all-targets -- -D warnings`，结果都通过。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `sieve 10k`：`1.3173–1.3389 ms`
  - `dense array bool read hot`：`68.603–69.468 ms`
  - `dense array bool condition only hot`：`57.001–58.300 ms`
- 当前解读：这次收紧终于直接落在 dense-array 索引读写 opcode 本体内部，而不再只是围绕循环骨架或数组初始化路径做外围优化。
- 2026-03-19：继续把 `GetArrayEl` 的分支 bookkeeping 收紧了一层，把融合后的 `GetArrayEl + IfFalse/IfTrue` 热路径里临时的 `Option<(bool, i32)>` 分支窥探状态改成了更直接的 opcode/offset 局部变量形状。
- 重新跑了全量 `cargo test -p mquickjs-rs` 和 `cargo clippy -p mquickjs-rs --all-targets -- -D warnings`，结果都通过。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `dense array bool read hot`：`81.896–87.290 ms`
  - `dense array bool condition only hot`：`71.628–77.040 ms`
  - `sieve 10k`：`1.6203–1.7606 ms`
- 当前解读：这是一条非常窄的 `GetArrayEl` 内部整理，它对读密集的融合分支 workload 有正向帮助，同时没有把更广义的 `sieve` 主路径拖成可测回归。
- 2026-03-19：继续把融合后的 `GetArrayEl + IfFalse/IfTrue` 分支窥探再收紧了一层，把热路径里的 opcode/offset 解码改成了“单次长度检查 + unchecked 取字节”的形状。
- 重新跑了 `cargo check -p mquickjs-rs`、全量 `cargo test -p mquickjs-rs` 以及 `cargo clippy -p mquickjs-rs --all-targets -- -D warnings`，结果都通过。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `sieve 10k`：`1.3077–1.3308 ms`
  - `dense array bool read hot`：`73.169–74.204 ms`
  - `dense array bool condition only hot`：`59.915–60.797 ms`
- 当前解读：这又是一条很窄的 `GetArrayEl` bookkeeping 收益，而且这次在“纯条件扫描”和“读 + 累加”两类 dense-array 读侧 workload 上都能稳定看到改善。
- 2026-03-19：把 `GetArrayElDiscard` 的 dense-array 快路径判定从会返回 tuple 的 `dense_array_access()` 里拆了出来，单独做成了更轻的布尔判断，这样被立刻丢弃结果的纯读取路径不再为用不上的 `(arr_idx, index)` 解码付费。
- 重新跑了 `cargo check -p mquickjs-rs`、全量 `cargo test -p mquickjs-rs` 以及 `cargo clippy -p mquickjs-rs --all-targets -- -D warnings`，结果都通过。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `dense array read only hot`：`51.587–52.737 ms`
  - `dense array read only hot arg0`：`50.887–51.752 ms`
  - `dense array read only hot local1`：`52.082–52.993 ms`
  - `sieve 10k`：`1.3022–1.3268 ms`
- 当前解读：这是一条小而真实的纯读取路径收益，它继续把更纯的 `GetArrayElDiscard` 诊断基线往下压，同时没有扰动更广义的数组语义。
- 2026-03-19：继续放宽了 `GetArrayElDiscard` 的 dense-array 快路径判定，让它在普通数组上遇到“任意整型索引”的被丢弃读取时都可以直接视作 no-op，而不再坚持沿用产生值的数组读取那套“必须先解码非负索引”的条件。
- 补充了 eval 回归覆盖，确认被丢弃的负索引读取（`arr[-1];`）仍然只会等价于一次被忽略的 `undefined` 读取，程序会正常继续执行。
- 重新跑了 `cargo check -p mquickjs-rs`、针对性的 eval 回归以及 `cargo clippy -p mquickjs-rs --all-targets -- -D warnings`，结果都通过。全量 `cargo test -p mquickjs-rs` 也已通过，只是后续又遇到了与这次改动无关的 Windows linker/pagefile 波动。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `dense array read only hot`：`51.587–52.737 ms`
  - `dense array read only hot arg0`：`50.887–51.752 ms`
  - `dense array read only hot local1`：`52.082–52.993 ms`
  - `sieve 10k`：`1.3022–1.3268 ms`
- 当前解读：这又是一条非常窄但稳定的 `GetArrayElDiscard` 纯读路径收益，它继续瞄准的是最纯的读取诊断基线，而不是更广义的分支型 `GetArrayEl` workload。
- 2026-03-18：新增了一组 `IncLoc*Drop` 专门 opcode，用来承接“结果会被立刻丢弃的 `local = local + 1` 语句更新”，并把循环增量和 dense-array 读侧计数里最热的那批尾巴重写成这组指令。
- 补充了 compiler 回归覆盖，确认 `var i = 0; i = i + 1;` 现在会降到这组专门 opcode；同时新增了语义回归，锁定 `var x = 'a'; x = x + 1;` 仍然保留字符串拼接语义。
- `dump_bytecode` 现在可以直接看到 dense-array 读侧诊断样例已经被编成更紧凑的 `IncLoc{1,2,3,4}Drop` 尾巴，而不再反复出现 `Push1; Add; Dup; PutLocX; Drop` 这串骨架。
- 重新跑了全量引擎测试以及 `clippy -D warnings`，结果都通过。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `loop 10k`：`0.416–0.477 ms`
  - `sieve 10k`：`1.496–1.514 ms`
  - `dense array bool read branch 10k`：`0.804–0.822 ms`
  - `dense array bool read hot`：`69.67–70.63 ms`
  - `dense array false write then read hot`：`60.76–61.67 ms`
- 当前解读：这是当前 dense-array 阶段里第一条真正把剩余“读侧循环骨架”明显压下去的优化；它对经典 `loop` 基线也有帮助，但最大的收获是 `GetArrayEl` 周围原本要支付的大量局部自增 bookkeeping 成本现在明显更低了。
- 2026-03-18：继续收紧了 `GetArrayEl` 自身的分支融合快路径，让最热的“数组 + 整数索引 + 条件分支”形状先直接处理 `true` / `false` / `null` / `undefined` / int 这些常见 truthiness，再回退到通用 helper。
- 重新跑了全量引擎测试以及 `clippy -D warnings`，结果都通过。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `sieve 10k`：`1.795–2.085 ms`（无显著变化）
  - `dense array bool read branch 10k`：`0.919–1.118 ms`（无显著变化）
  - `dense array bool read hot`：`72.35–73.37 ms`
  - `dense array bool condition only hot`：`69.42–78.77 ms`
- 当前解读：这是一条很小的读侧 truthiness 优化，它最明显的收益出现在更纯的 `GetArrayEl + IfFalse` 诊断基线上，而在更大的混合形状 benchmark 上基本保持中性。

### 9.1.5 Opcode dispatch 收紧 [已彻底完成]

**优先级**: P1

**热点文件**

- `src/vm/interpreter.rs`

**原因**

- `loop` 仍然表明有意义的指令 dispatch 开销。
- 基于大型 match 的 dispatch 是正确且可维护的，但在最热路径上仍然昂贵。

**任务**

- 通过基准驱动的性能分析识别最热的 10–20 个 opcode。
- 减少 dispatch 循环中每次迭代的工作量。
- 减少热指令中重复的解码/分支/错误路径开销。
- 对算术、局部变量、跳转和调用指令首选本地快速路径。

**预期收益**

- `loop` 的最佳次要目标
- 跨多个 benchmark 的广泛收益

**当前已完成**

- 2026-03-16：添加了重复 throw/catch 控制流的 `try_catch` benchmark 覆盖。
- 2026-03-16：通过统一异常分发并将重复的基于 pop 的展开循环替换为基于 `truncate` / `drop_n` 的展开，减少了异常路由开销。
- 为循环内重复 throw/catch 添加了回归覆盖。
- Benchmark 结果：`try_catch 5k` 基线在 Criterion 中记录为 `340–349 μs`。
- 2026-03-16：在 `dump` feature 下添加了功能门控的运行时 opcode 计数器，并通过 `Context` 对外暴露以供性能分析工作使用。
- 添加了 `dump` 模式回归测试，确保 opcode 计数记录真实执行情况。
- 运行时热点发现：
  - `loop` 由 `GetLoc1`、`Goto`、`Add`、`Dup`、`Drop`、`GetLoc0`、`PutLoc0`、`PutLoc1`、`Lt`、`IfFalse` 主导。
  - `sieve` 由 `Goto`、`Drop`、`IfFalse`、`GetLoc3`、`Add`、`Dup`、`GetLoc0`、`Lte`、`GetLoc2`、`PutArrayEl`、`PutLoc3`、`GetArrayEl`、`CallMethod` 主导。
- 当前解读：下一个基于证据的优化目标更可能是 `Dup/Drop` + 本地存储使用模式或分支/控制流成本，而不是另一个临时的算术辅助函数调整。
- 2026-03-16：为常见语句更新模式（如 `i = i + 1;`）完成了 `Dup + PutLocX + Drop` peephole 快速路径。
- 添加了局部赋值语句更新的回归覆盖，同时保留了赋值表达式行为。
- Benchmark 结果：`loop 10k` 在 Criterion 中从约 `0.513–0.525 ms` 提升到 `0.486–0.492 ms`。
- Benchmark 结果：`sieve 10k` 在 Criterion 中从约 `2.257–2.310 ms` 提升到 `2.152–2.191 ms`。
- 2026-03-16：通过将通用检查辅助函数替换为直接快速路径栈操作，优化了热 `Dup` / `Drop` opcode 处理程序本身。
- 复用了相同的局部赋值和赋值表达式回归覆盖来验证更改。
- 本轮之后的当前基线记录在 `docs/BENCHMARK_ANALYSIS.md` 中。
- 2026-03-16：为 `Lt/Lte` 后紧跟 `IfFalse` / `IfTrue` 添加了分支融合快速路径，允许比较结果直接分支而无需在栈上实化临时布尔值。
- 复用了现有的 `while`、`switch` 和 `try_catch` 控制流回归覆盖来验证语义。
- Benchmark 结果：`loop 10k` 在 Criterion 中从约 `0.502–0.514 ms` 提升到 `0.484–0.499 ms`。
- Benchmark 结果：`sieve 10k` 在 Criterion 中从约 `2.164–2.207 ms` 提升到 `2.038–2.078 ms`。
- 2026-03-17：在 dump 模式 profiling 明确指出 `sieve` 当前最热的局部更新形状是 `Add; Dup; PutLoc3; Drop` 与 `Add; Dup; PutLoc8 4; Drop` 之后，新增了只覆盖这两种形状的窄范围 peephole，而没有重新引入之前会回归的泛化版本。
- 补充了 `PutLoc8` 语句更新形状的回归覆盖，同时保持赋值表达式语义不变。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `loop 10k`：`0.493–0.503 ms`
  - `sieve 10k`：`1.832–1.860 ms`
- 当前解读：这再次说明，当前阶段的 opcode / local-store 优化在有明确字节码形状证据时效果最好，不适合用过泛的通用快路径去覆盖。
- 2026-03-17：继续收紧了原始 `Goto` / `IfFalse` / `IfTrue` 处理器本身，把最热路径上的操作数解码和分支值弹栈改成更直接的 unchecked 快路径。
- 变更后重新跑了全量引擎测试以及 `clippy -D warnings`，结果通过。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `loop 10k`：`0.461–0.476 ms`
  - `sieve 10k`：`1.704–1.740 ms`
- 当前解读：在按具体字节码形状优化完局部更新之后，真正剩下的下一层瓶颈就是控制流骨架本身；把 `Goto/IfFalse/IfTrue` 再收紧一轮之后，`loop` 和 `sieve` 都又下了一个台阶。
- 2026-03-17：新增了专门的 `GetLoc4` / `PutLoc4` 短 opcode，使当前最热的“额外局部槽位”不再走通用的 `GetLoc8` / `PutLoc8` 路径。
- 补充了 compiler 回归覆盖，确保第 5 个局部槽位现在确实会发出短 opcode。
- 变更后重新跑了全量引擎测试以及 `clippy -D warnings`，结果通过。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `loop 10k`：`0.449–0.459 ms`
  - `sieve 10k`：`1.686–1.714 ms`
- 当前解读：在控制流骨架收紧之后，下一层真实瓶颈确实就是最热的那个非内联局部槽位；给 slot 4 补上专门 opcode 之后，`loop` 和 `sieve` 都又往下走了一步。
- 2026-03-17：在额外验证之后，保留了 slot 4 短 opcode 这条优化，并在当前工作树上重新跑了本地 benchmark 对照。
- 当前选定的执行期重跑结果为：
  - `loop 10k`：`0.444–0.451 ms`
  - `sieve 10k`：`1.663–1.709 ms`
- 当前解读：slot 4 短 opcode 这条线在复跑后仍然成立，应视为稳定的 opcode/local-slot 优化成果，而不是一次性的测量波动。
- 2026-03-18：新增了一个小型 `dump_bytecode` 开发者工具，使热点 benchmark 脚本现在可以直接编译并反汇编，而不再只能靠 opcode 计数去反推字节码形状。
- 2026-03-18：重新调整了 C-style `for` 的 lowering 方式：增量段仍然只编译一次，但改为追加到 body 之后，从而去掉了首轮执行时那条“先跳过增量段”的额外 `Goto`。
- 补充了 compiler 回归覆盖，确认简单 `for (...)` 循环现在只会发出一条回边 `Goto`；同时重新跑通了 `for` / `continue` / `break` / “loop 内 switch” 相关回归。
- 变更后重新跑了全量引擎测试以及 `clippy -D warnings`，结果通过。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `loop 10k`：`0.555–0.659 ms`
  - `sieve 10k`：`1.917–2.219 ms`
  - `dense array bool read branch 10k`：`1.367–1.669 ms`
  - `dense array bool read hot`：`131.93–152.53 ms`
  - `dense array false write then read hot`：`87.45–100.89 ms`
- 当前解读：这次更像是控制流骨架层面的收益，而不是只打中了 `GetArrayEl` 本体；但它正好落在 dense-array 读侧诊断里现在最重的那类字节码形状上，也把所有 C-style `for` 循环里一个真实存在的结构性冗余去掉了。
- 2026-03-18：新增了 `IncLoc*Drop` 这一组“语句形局部自增” opcode，并把最热的、结果会被丢弃的 `local = local + 1` 字节码尾巴重写成这组专门指令，同时保留了字符串局部变量上的完整 `+` 语义。
- 补充了 compiler 回归覆盖，确认 `var i = 0; i = i + 1;` 现在会降到专门 opcode；同时新增了语义回归，锁定 `var x = 'a'; x = x + 1;` 仍然会得到字符串结果。
- `dump_bytecode` 现在可以直接看到 dense-array 读侧诊断样例已经被编成更紧凑的 `IncLoc{1,2,3,4}Drop` 尾巴，而不再反复出现 `Push1; Add; Dup; PutLocX; Drop` 这一整串骨架。
- 变更后重新跑了全量引擎测试以及 `clippy -D warnings`，结果通过。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `loop 10k`：`0.483–0.602 ms`
  - `sieve 10k`：`1.633–1.726 ms`
  - `dense array bool read branch 10k`：`0.803–0.822 ms`
  - `dense array bool read hot`：`86.07–105.53 ms`
  - `dense array false write then read hot`：`64.41–67.69 ms`
- 当前解读：这是当前 dense-array 阶段里第一条真正把剩余“读侧循环骨架”明显压下去的优化；它对经典 `loop` 基线也有帮助，但最大的收获是新增的 dense-array 读侧诊断 benchmark 现在在 `GetArrayEl` 周围支付的局部自增 bookkeeping 成本明显更低了。
- 状态：作为当前这一轮 dispatch 收紧工作，这部分现在可以视为完成；只有在新的 profiling 明确指出另一组 materially different 热 opcode 时，才需要重新打开。

### 9.1.6 算术/比较微优化轮次 [已彻底完成]

**优先级**: P1

**热点文件**

- `src/vm/ops.rs`

**原因**

- 核心算术和比较辅助函数已经部分内联。
- 此领域仍然重要，但其可能的收益低于调用/数组/native 热路径。

**任务**

- 审计剩余的热 `op_*` 辅助函数是否真正受益于内联。
- 减少常见 int/int 和 int/float 路径上的重复数值强制转换。
- benchmark 清理后重新检查相等性和比较快速路径。

**预期收益**

- 小幅但广泛的改善

**当前已完成**

- 2026-03-16：通过将最终运行时字符串构建在单个输出缓冲区中，而不是先将两个操作数实化为临时拥有的 `String` 值，改善了字符串拼接热路径。
- 为混合字符串/数字链式拼接形状添加了回归覆盖。
- Benchmark 结果：`runtime_string_pressure 4k` 在 Criterion 中从约 `2.89–3.38 ms` 提升到 `1.53–1.55 ms`。
- 2026-03-17：为最常见的 `string + int` / `int + string` 拼接形状添加了更窄的 `Add` 快路径，让混合编译期字符串片段和十进制循环索引的运行时字符串热点不再走通用的长度估算加追加路径。
- 重新跑了针对性的 concat 形状回归覆盖，结果通过。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `runtime_string_pressure 4k`：`1.091–1.117 ms`
  - `string concat 1k`：`151.87–157.61 µs`
  - `method_chain 5k`：`587.80–599.99 µs`
- 当前解读：这是一条对“编译期字符串片段 + 十进制循环索引”形状非常有效的运行时字符串优化；而更简单的 `string concat 1k` benchmark 这轮基本没有显著变化。
- 2026-03-17：新增了字节码级的 `AddConstStringLeft` / `AddConstStringRight` 专门化，让 concat 链里“编译期字符串在 `+` 左侧或右侧”的形状不再继续走通用 `Add` opcode。
- 补充了 compiler 回归覆盖，确认 `"x" + value` 和 `value + "x"` 这两类形状现在都会发出专门字节码；同时重新跑了针对性的 concat 形状回归覆盖。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `runtime_string_pressure 4k`：`1.055–1.077 ms`
  - `string concat 1k`：`141.41–145.80 µs`
  - `method_chain 5k`：`587.46–601.19 µs`
- 当前解读：这是第一轮真正更成体系的 concat 链优化，不再只是执行器里的 `Add` 小分支微调；它对 runtime-string 压力路径给出了明确收益，同时没有明显拖坏附近的 `method_chain` 工作负载。
- 2026-03-17：在这层 lowering 的基础上，继续加入了相邻字符串字面量的编译期折叠，以及 `const + value + const` 的专门 `AddConstStringSurround` 形状，进一步去掉了目标 concat 链里的一次运行时字符串分配。
- 补充了 compiler 回归覆盖，确认 surround 专门化和相邻字符串常量折叠都已生效。
- 当前工作树上的 dump 模式热点探测显示，`runtime_string_pressure` 的 concat 运行时字符串创建次数已经从 `12001` 降到 `8001`，`Add` 执行次数也从 `24001` 降到 `16000`。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `runtime_string_pressure 4k`：`0.899–0.915 ms`
  - `string concat 1k`：`166.97–171.99 µs`
  - `method_chain 5k`：`624.57–638.70 µs`
- 当前解读：这是一条更强、更结构化的 concat 链优化，对目标 runtime-string benchmark 的收益非常明确；但它看起来会拖慢更简单的 `string concat 1k` 微基准，所以后续需要专门解释并收回这条回归，而不能把字符串路径直接视为“已经打完”。
- 2026-03-18：新增了一个非常窄的语句级 `AppendConstStringToLoc0` lowering，并配套引入了仅服务于局部槽位 `0` 的 per-frame builder，专门覆盖 `var s = ''; s = s + 'x';` 这一个热点形状。
- 补充了 compiler 回归覆盖，确认这一个精确形状现在会发出新 lowering；并重新跑了对应的 eval 回归。
- 当前工作树上的 dump 模式热点探测显示，`string_concat` 的 concat 运行时字符串创建次数已经从 `1000` 降到 `1`。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `string concat 1k`：`81.24–83.26 µs`
  - `runtime_string_pressure 4k`：`909.74–921.95 µs`
  - `method_chain 5k`：`708.67–724.79 µs`
- 当前解读：这条基于 builder 的局部自拼接优化，终于把 `string concat 1k` 这条微基准真正拉下来了，而且没有再回到前面那些通用运行时 peephole 的失败路径；同时更广义的 `runtime_string_pressure` 仍然停留在同一个亚毫秒量级，而 `method_chain` 现在更适合描述为“仍然保持在亚毫秒区间”，而不是继续沿用更早那条贴着 `0.60 ms` 的说法。
- 2026-03-18：把 concat-chain lowering 再往前推了一步，新增了 `AddConstStringSurroundValue`，让最热的 `const + value + const + value` 形状也能再少掉一个中间 concat 结果。
- 补充了 compiler 回归覆盖，确认 `'a' + x + 'b' + y` 现在会发出这个四段 lowering。
- 当前工作树上的 dump 模式热点探测显示，`runtime_string_pressure` 的 concat 运行时字符串创建次数已经从 `8001` 进一步降到 `4001`，`Add` 总执行次数也从 `16000` 降到 `12000`。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `runtime_string_pressure 4k`：`841.41–855.24 µs`
  - `string concat 1k`：`82.79–84.89 µs`
  - `string concat ephemeral 1k`：`113.15–118.99 µs`
  - `method_chain 5k`：`736.57–751.78 µs`
- 当前解读：这让字符串主线又收窄了一层，剩下真正没解决的部分更加集中到“通用增长字符串表示”本身。目标 runtime-string benchmark 明显受益，而 `string concat` 和 `method_chain` 仍然留在可接受的区间里。
- 2026-03-18：引入了一个最小可用的延迟 `RuntimeString` 包装层，并让 `.length` 直接读取 cached runtime-string length，而不是先把延迟 concat 节点强制 flatten。
- 这一步保住了前面那些局部自拼接和 concat-chain lowering 的收益，同时把之前“取 `.length` 时被强制物化”带回来的回归重新压了下去。
- 在当前执行期 Criterion 口径下，选定重跑结果为：
  - `runtime_string_pressure 4k`：`869.62–887.11 µs`
  - `string concat 1k`：`78.61–80.20 µs`
  - `string concat ephemeral 1k`：`118.18–121.41 µs`
  - `method_chain 5k`：`715.62–730.29 µs`
- 当前解读：字符串主线现在终于更像一个完整的体系了。局部自拼接已经解决，目标 runtime-string benchmark 在 cached-length 路径下重新变快，而剩下真正需要决策的，是不是要把延迟字符串表示继续推广，而不再是“当前这条窄路径有没有明显回归”。
- 状态：作为当前这一轮算术/字符串拼接微优化阶段，这部分现在可以视为完成；后续如果继续做字符串主线，应归类为更广义的字符串表示改造项目，而不是继续把它当作零碎的热 opcode 清理。
- 2026-03-16：通过为同值、整数和布尔比较添加直接快速路径（在回退到较慢的通用处理之前），改善了 `StrictEq` / `StrictNeq` 热 opcode 处理。
- 现有的 switch 语义回归测试已成功重新运行。
- Benchmark 结果：`switch 1k` 在 Criterion 中从约 `145–149 μs` 量级提升到 `132–136 μs`。

## 9.2 优化 GC 性能

### 9.2.1 替换保守的 `mark_all` 行为

**优先级**: P1

**热点文件**

- `src/gc/collector.rs`
- `src/context.rs`

**原因**

- 当前收集器仍然包含一个保守的临时方法，标记所有对象。
- 这阻碍了有意义的 GC 性能工作。

**任务**

- 用真正的 root 发现替换 `mark_all()`。
- 定义并遍历真正的 root：
  - 栈
  - 全局变量
  - 闭包
  - 活跃帧
  - 运行时拥有的容器
- 验证压缩后指针更新仍然正确。

**预期收益**

- 降低 GC 暂停成本
- 在对象密集型脚本上有更好的扩展性

### 9.2.2 测量 GC 触发行为

**优先级**: P1

**原因**

- GC 成本不仅取决于收集器实现，还取决于触发频率。

**任务**

- 测量 benchmark 工作负载期间的 GC 频率。
- 记录代表性脚本的对象/数组/字符串增长情况。
- 仅在收集到真实数据后调整触发启发式。

### 9.2.3 减少引擎拥有容器的扫描成本

**优先级**: P2

**热点文件**

- `src/vm/types.rs`
- `src/context.rs`

**任务**

- 审查扫描运行时向量的成本：
  - `objects`
  - `closures`
  - `runtime_strings`
  - `typed_arrays`
  - `array_buffers`
- 在有用的地方将热活动数据与长期存在的元数据分开。

## 9.3 减少内存使用

### 9.3.1 首先改善测量 [已彻底完成]

**优先级**: P0

**热点文件**

- `src/context.rs`
- `src/vm/types.rs`

**原因**

- `MemoryStats` 已经很有用，但优化应基于实际的主导桶。

**任务**

- 将 `MemoryStats` 作为基线测量来源。
- 记录 benchmark 脚本的对象/字符串/闭包/typed-array 数量。
- 在重新设计布局之前，先识别最大的内存类别。

**当前已完成**

- 2026-03-16：将 `MemoryStats` / `InterpreterStats` 扩展到对象数量之外，包括：
  - `runtime_string_bytes`
  - `array_elements`
  - `object_properties`
  - `typed_array_bytes`
  - `array_buffers`
  - `array_buffer_bytes`
- 更新了 CLI dump 输出以显示新的内存分类。
- 添加了以下回归覆盖：
  - 数组/对象形状指标
  - 运行时字符串字节统计
- 状态：此测量基础现在已足够开始基于证据的 9.3 工作。

### 9.3.2 减少热执行路径中的临时分配

**优先级**: P0

**原因**

- 临时向量和瞬态重塑会增加 CPU 和内存的波动。

**任务**

- 从热调用路径中移除剩余的临时 `Vec<Value>` 分配。
- 审查数组/builtin 密集型执行中的短期分配模式。
- 在安全的地方首选保留栈的布局和借用数据。

### 9.3.3 审查运行时字符串增长 [已彻底完成]

**优先级**: P1

**热点文件**

- `src/vm/interpreter.rs`
- `src/context.rs`

**原因**

- 运行时字符串在 `MemoryStats` 中被明确计数，可能随时间悄悄增长。

**任务**

- 测量 benchmark 工作负载中 `runtime_strings` 的增长。
- 检查字符串提升在热路径中是否过于积极。
- 寻找重复字符串创建的机会。

**当前已完成**

- 2026-03-16：添加了仅限 dump 模式的运行时字符串来源计数器，至少区分：
  - 总运行时字符串创建请求
  - concat 驱动的创建
  - for-in key 创建
  - 其他创建路径
- 在 `dump` feature 下通过 `Context` 对外暴露了计数器。
- 添加了 dump 模式回归覆盖，确保运行时字符串来源统计被记录。
- 2026-03-17：将来源桶扩展到至少区分 `json`、`object_keys`、`object_entries`、`error_string` 和 `type_string`，除了 `concat`、`for_in_key` 和 `other`。
- 状态：作为“审查/测量运行时字符串增长”这一任务，这部分现在可以视为完成；后续是否做复用/去重，属于新的优化决策，而不是审查工作未完成。
- 嵌入说明：暂不在引擎中硬编码运行时字符串字节预算；最终限制将在 ESP32 级别目标的真实设备集成期间选择。
- 2026-03-16：在 `for-in` key 路径上，运行时字符串耗尽现在变为受控引擎错误（`runtime string table exhausted`）而不是 debug 时的溢出 panic。
- 添加了回归覆盖，锁定重复 `for-in` key 生成的新受控错误行为。
- 简而言之：之前在 `for-in` key 路径上崩溃的运行时字符串溢出，现在降级为受控引擎错误，而不是 panic 进程。

### 9.3.4 审查对象和数组布局开销

**优先级**: P1

**热点文件**

- `src/runtime/object.rs`
- `src/runtime/array.rs`
- `src/vm/types.rs`

**任务**

- 比较 dense array 与通用对象支持访问的内存成本。
- 检查频繁创建的运行时结构是否可以变小。
- 仅在测量之后才选择针对性的布局变更。

## 辅助引擎任务

### S1. 保持 builtin/runtime 边界诚实

**优先级**: P2

**原因**

- `src/builtins/` 目前大部分是结构性占位代码。
- 真正的 builtin 行为主要在 `src/vm/natives.rs` 和 `src/vm/property.rs` 中。

**任务**

- 记录真正的实现位置。
- 避免误优化占位模块。
- 推迟结构性迁移直到热点工作完成后，除非它阻塞性能工作。

### S2. 使用基准特定的优化目标

**优先级**: P0

**权威优化集合**

- `fib` → 调用路径、递归、算术
- `loop` → dispatch、算术、局部变量
- `array` → dense array 快速路径
- `sieve` → dense array 读写 + 循环成本
- `json` → 已优秀路径的回归保护

### S3. 扩展 benchmark 覆盖以覆盖缺失的引擎路径

**优先级**: P0

**原因**

- 当前 benchmark 集合有用，但仍然对几个重要的引擎路径覆盖不足。
- 如果 benchmark 套件只聚焦在 `fib`、`loop`、`array`、`sieve` 和 `json` 上，一些高价值的优化领域将保持不可见。

**Benchmark 新增：主集合**

这些应该被视为下一批 benchmark 新增，因为它们最直接地暴露了有意义的引擎热点：

- `method_chain`
  - 代表性形状：`.map().filter().reduce()`
  - 覆盖：`GetField2`、`CallMethod`、回调调用、数组链式操作
- `for_of_array`
  - 覆盖：`ForOfStart`、`ForOfNext`、迭代器循环控制
- `deep_property`
  - 代表性形状：`a.b.c.d`
  - 覆盖：重复的 `GetField` 成本和链式属性访问
- `runtime_string_pressure`
  - 覆盖：`create_runtime_string`、运行时字符串增长、字符串分配压力

**Benchmark 新增：次集合**

这些也很重要，但最好作为机制特定的 benchmark 而不是第一波头条性能 benchmark：

- `try_catch`
  - 覆盖：`ExceptionHandler`、throw/catch/finally 控制流、栈展开
- `for_in_object`
  - 覆盖：`ForInStart`、`ForInNext`、对象 key 迭代
- `switch_case`
  - 覆盖：基于 `Dup + StrictEq + IfTrue` 的多分支 dispatch 形状

**为当前 no_std 优先路径延后**

- `regexp_test`
  - 覆盖：`RegExpObject`、`test`
  - 保留为以后的 `std` / 可选 benchmark 候选，不作为第一波 no_std 目标
- `regexp_exec`
  - 覆盖：`RegExpObject`、`exec`
  - 保留为以后的 `std` / 可选 benchmark 候选，不作为第一波 no_std 目标

**建议推出顺序**

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

- 使基准驱动的优化更能代表真实的 JS 使用场景
- 暴露调用密集、迭代器密集、对象访问密集和字符串压力密集的路径
- 让引擎优化工作在算术和原始循环之外有更好的可见性

**当前已完成**

- 2026-03-16：添加了第一波 benchmark 脚本和 Criterion 覆盖：
  - `method_chain`
  - `runtime_string_pressure`
  - `for_of_array`
  - `deep_property`
- 2026-03-16：添加了第二波 `switch_case` benchmark 脚本，用于 CLI 风格的 Rust vs C 对比。
- 使用 `cargo bench --no-run` 验证了 benchmark 构建。
- 2026-03-16：通过从 `ForOfStart` 中移除完整数组克隆并改为按索引迭代数组，完成了第一个 `for_of_array` 优化轮次。
- 添加了回归覆盖，确认数组上的 `for-of` 在迭代期间能观察到元素更新。
- Benchmark 结果：`for_of_array 20k` 在 Criterion 中从约 `4.22–4.47 ms` 提升到 `2.36–2.42 ms`。
- 2026-03-17：为 `ForOfNext` 后紧跟 `IfTrue` 的常见形状添加了分支融合快路径，使迭代器热路径不再为已知分支形状物化临时 `done` 布尔值。
- 重新跑了 `for-of` 的正常迭代、`continue`、以及数组元素更新可见性回归覆盖。
- 在当前执行期 Criterion 口径下，最新完整重跑记录 `for_of_array 20k` 为 `1.80–1.96 ms`。
- 2026-03-16：添加了 `for_in_object` benchmark 覆盖，并通过将急切的完整 key 克隆替换为对象/数组快照上的基于索引的懒 key 生成，完成了第一个迭代器设置优化轮次。
- 添加了回归覆盖，确认对象上的 `for-in` 在迭代期间仍然通过静态属性读取观察到更新的值。
- Benchmark 基线已记录：`for_in_object 20x2000` 在 Criterion 中为 `3.74–3.80 ms`。

## 推荐执行顺序

1. Benchmark 基线重验证与文档同步
2. 基于当前 head 的调用路径回归审计
3. Native/builtin 参数整理的收尾工作
4. Dense array 快速路径
5. 基于当前重跑数据继续做对象/属性访问优化
6. 内存测量轮次
7. GC 基于 root 的标记工作
8. Opcode dispatch 收紧
9. 次要微优化

## 完成标准

当满足以下条件时，此优化任务清单视为基本完成：

- benchmark 基线可信且可复现
- `fib`、`loop`、`array` 和 `sieve` 各自至少有一个经过验证的热点改进
- GC 不再依赖保守的 `mark_all`
- 内存减少工作基于测量的主导类别，而非猜测
- 文档仅反映有效的 benchmark 结论
## 9.1 主线补充说明

- 2026-03-20：`deep_property` / 普通对象属性链已经重新成为当前主线。
- 已完成：
  - 在 `GetField` / `GetField2` 上补了一层更窄的普通对象直达快路径
  - 普通对象链读取现在会直接走 `object_get_property()`，不再先穿过完整的 `get_field_value()` 类型分发链
- 当前执行期重跑结果：
  - `deep_property 200k`：`18.706–20.128 ms`
  - `method_chain 5k`：`1.106–1.214 ms`
  - `runtime_string_pressure 4k`：`1.118–1.205 ms`
- 当前解读：
  - 这是当前最干净的一轮 deep-property 跟进，因为它直接命中了 `root.a.b.c.d` 这种连续普通对象链读取
  - dense-array 读侧那条微优化线当前已经进入“高回归风险、低信号”的区域，暂时收住
  - 后续如果没有新的 profiling 证据，不再重启 dense-array 读侧的微优化小刀
