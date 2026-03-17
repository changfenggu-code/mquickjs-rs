# 基准分析

英文版：`docs/BENCHMARK_ANALYSIS.md`

相关优化清单：
- `docs/ENGINE_OPTIMIZATION_TASKLIST.md`
- `docs/ENGINE_OPTIMIZATION_TASKLIST.zh.md`

## 用途

本文档定义 `mquickjs-rs` 引擎当前的 benchmark 基线和解读规则。

这是纯引擎文档，不覆盖 `led-runtime` 产品层行为。

## 基线策略

Benchmark 分析使用三个互补的来源。

### 1. 本地 Criterion（`cargo bench`）

主要用途：

- 精确的纯 Rust 执行期计时分析
- 优化前后的验证
- 热点确认

判断引擎优化是否真的有效时，这是首选来源。

自 2026 年 3 月 17 日起，Criterion harness 会先编译一次 benchmark 脚本，再在新建 context 上重复测执行阶段，从而尽量减少 parser/compiler 噪声对运行时优化判断的污染。

### 2. 本地 Rust vs C 对比（`benches/compare.sh` 或等价的本地对比）

主要用途：

- 对比当前 Rust 引擎与 C `mqjs` 实现的性能
- 估算某条路径距离 C 实现还差多远

评估跨实现差距时，这是首选来源。

### 3. CI benchmark 摘要（`.github/workflows/bench.yml`）

主要用途：

- 在 GitHub Actions 中跟踪趋势
- 在 push 和 PR 时快速发现回归
- GitHub 可见的摘要表

CI 摘要现在提供：

- Rust vs C 对比表
- 纯 Rust Criterion 表
- 启动基线行

CI 结果在 GitHub 上有用且可见，但本地 Criterion 和本地 Rust vs C 对比仍然是详细调查的主要来源。

## 规范 Benchmark 集合

当前引擎基线集合：

- 核心遗留集：
  - `fib`
  - `loop`
  - `array`
  - `json`
  - `sieve`
- 第一波扩展集：
  - `method_chain`
  - `runtime_string_pressure`
  - `for_of_array`
  - `deep_property`
- 当前次要跟踪 benchmark：
  - `switch_case`
  - `try_catch`
  - `for_in_object`

为当前 `no_std` 优先路径延后：

- `regexp_test`
- `regexp_exec`

## 当前基线（本地 Criterion，纯 Rust）

2026 年 3 月 17 日的当前状态需要特别说明：

- Criterion harness 已改为“编译一次、重复测执行”
- 我们已经在当前工作树上对主 benchmark 集合重新跑了一整轮本地 Criterion
- 2026 年 3 月 16 日记录的多项数值已经不能代表当前 head
- 旧的 2026 年 3 月 16 日 Criterion 数值因此不能直接与当前口径对比
- 因此 benchmark 基线清理应重新视为进行中工作，而不是已关闭工作

### 当前工作树的重验证快照（2026-03-17）

下表来自当前工作树上的重新测量，使用的是更新后的“编译一次、重复测执行”Criterion 口径，应视为主 benchmark 集合的当前可信快照。

| Benchmark | 当前本地快照 |
|-----------|--------------|
| `fib_iter 1k` | `2.330–2.379 ms` |
| `loop 10k` | `0.472–0.485 ms` |
| `array push 10k` | `0.491–0.502 ms` |
| `json parse 1k` | `0.736–0.754 ms` |
| `sieve 10k` | `2.069–2.103 ms` |
| `method_chain 5k` | `0.585–0.600 ms` |
| `runtime_string_pressure 4k` | `0.899–0.915 ms` |
| `for_of_array 20k` | `1.796–1.959 ms` |
| `deep_property 200k` | `14.925–15.235 ms` |

### 次级跟踪 benchmark（2026-03-17 尚未重跑，沿用上次记录）

这些数值仍然有参考意义，但截至 2026 年 3 月 17 日，它们还没有在新的“编译一次、重复测执行”Criterion 口径下重新验证。

| Benchmark | 上次记录的快照 |
|-----------|----------------|
| `switch 1k` | `0.132–0.136 ms` |
| `try_catch 5k` | `0.341–0.349 ms` |
| `for_in_object 20x2000` | `3.743–3.804 ms` |

## 当前基线（本地 Rust vs C，含进程启动）

以下数据基于本地多次进程执行的平均值，适用于跨实现对比。包含进程启动成本。

### 启动基线

| 场景 | Rust | C | 比率 |
|------|------|---|------|
| `mqjs -e "0"` | `18.130 ms` | `17.407 ms` | `1.042x` |

### 脚本对比

| Benchmark | Rust | C | 比率 | 备注 |
|-----------|------|---|------|------|
| `fib` | `183.099 ms` | `118.815 ms` | `1.541x` | C 更快 |
| `loop` | `127.598 ms` | `86.453 ms` | `1.476x` | C 更快 |
| `array` | `17.673 ms` | `16.467 ms` | `1.073x` | C 略快 |
| `json` | `44.048 ms` | `64.280 ms` | `0.685x` | Rust 更快 |
| `sieve` | `37.343 ms` | `27.791 ms` | `1.344x` | C 更快 |
| `method_chain` | `15.795 ms` | `14.109 ms` | `1.119x` | C 略快 |
| `runtime_string_pressure` | `22.470 ms` | `19.185 ms` | `1.171x` | C 更快 |
| `for_of_array` | `19.509 ms` | `17.358 ms` | `1.124x` | C 更快 |
| `deep_property` | `32.465 ms` | `24.001 ms` | `1.353x` | C 更快 |
| `switch_case` | `18.495 ms` | `16.856 ms` | `1.097x` | C 略快 |
| `try_catch` | `16.097 ms` | `14.508 ms` | `1.110x` | C 略快 |
| `for_in_object` | `22.356 ms` | `17.633 ms` | `1.268x` | C 更快 |

## 解读说明

### 当前最强信号

- `json` 仍然是 Rust 引擎的相对优势项。
- 更新后的执行期口径重跑说明，`fib` 和 `loop` 仍然是调用路径与 dispatch 的有效观察窗。
- `array` 和 `sieve` 仍然是 dense array 与 builtin 调用成本的重要观察窗。
- 在 parser/compiler 噪声被削弱之后，`runtime_string_pressure` 和 `method_chain` 在最新完整重跑里又有一轮明显改善；其中 `method_chain` 仍然基本贴着 `0.60 ms` 目标线，而 `runtime_string_pressure` 现在又吃到了更深一层 concat-chain lowering 的收益，运行时字符串创建次数明显下降。不过，更简单的 `string concat 1k` 微基准在最新定向重跑里出现了回归，所以这条字符串路线还不能算彻底收尾。
- 更新后的语句级局部自拼接 lowering（`AppendConstStringToLoc`）已经把专门的 `string concat 1k` 微基准明显拉下来了，并把它的运行时字符串创建次数降到了 `1`；与此同时，更广义的 `runtime_string_pressure` 仍然保持在同一个亚毫秒量级。
- `for_of_array` 在 `ForOfNext` 分支融合之后又有一轮明显改善，现在健康得多，但仍然是有价值的迭代器/控制流观察窗。
- `deep_property` 仍然是高价值的对象属性访问 benchmark，而且当前看起来比迭代器/字符串压力这组路径更健康。
- `switch_case` 现已作为次要控制流 benchmark 被跟踪，最新的 `StrictEq` 热路径优化已显示出可测量的改善。
- `try_catch` 现已作为次要异常控制 benchmark 被跟踪，已记录清理后的基线。
- `for_in_object` 现已作为次要迭代器/控制流 benchmark 被跟踪，迭代器初始化清理后记录了第一个基线。

### 重要注意事项

- Criterion 数据和含进程启动的 Rust vs C 数据回答的是不同问题。
- 短运行脚本对比对启动开销更敏感。
- 在验证 Rust 引擎内部的优化时，优先使用 Criterion。

## 代码中已经实现的优化工作

当前代码中仍然保留着以下第一轮优化实现：

- `deep_property`
  - 小对象属性查找快速路径
  - 统一 `GetField` / `GetField2` 属性分发
- `method_chain`
  - 去除数组高阶 builtin 中每个元素的临时 `Vec<Value>` 分配
  - 为 `CallMethod` native 添加小参数快速路径
  - 为 `Array.prototype.push` native 添加 `argc == 1` 专用快捷方式
  - 将数组 `.push` 的属性读取改成直接走缓存的 native 索引
  - 将高阶数组 builtin 从整数组 clone 改成“长度快照 + 实时元素读取”
  - 新增专门的 `CallArrayMap1` / `CallArrayFilter1` / `CallArrayReduce2` opcode
  - 为最热的单参数数组构建形状新增 `CallArrayPush1`
- `runtime_string_pressure`
  - 将 concat 结果直接构建在单个输出缓冲区中，而不是先分别实化两个操作数
  - 为十进制循环索引主导的 `string + int` / `int + string` 拼接形状新增更窄的快路径
  - 为 concat 链里的编译期字符串片段新增字节码级 `AddConstStringLeft` / `AddConstStringRight` 专门化
  - 对相邻字符串字面量做编译期折叠，并把 `const + value + const` lowering 成专门的 `AddConstStringSurround`
  - 为语句级 `local = local + "const"` 引入 `AppendConstStringToLoc` 和按 frame 存活的局部字符串 builder
- `for_of_array`
  - 从 `ForOfStart` 中去除完整数组克隆
  - 为 `ForOfNext` 常见 `IfTrue` 退出形状增加分支融合
- `loop` / `sieve`
  - 为常见语句更新模式添加 `Dup + PutLocX + Drop` peephole 快速路径
  - 添加 `Lt/Lte` + `IfFalse/IfTrue` 分支融合

这些改动在以下文档中跟踪：

- `docs/ENGINE_OPTIMIZATION_TASKLIST.md`
- `docs/ENGINE_OPTIMIZATION_TASKLIST.zh.md`

当前需要特别强调的是：

- 这些实现本身是真实存在的，代码里也还保留着
- 2026 年 3 月 17 日不仅有数值变化，Criterion 的测量口径本身也发生了变化
- 因此旧的 Criterion 数字应视为历史代际数据，而上面的表格应视为当前可信的执行期快照

## 「9.1.1 Benchmark 基线清理」的含义

本基线清理任务在满足以下条件时视为完成：

- benchmark 职责已明确分离：
  - 本地 Criterion → 精确的纯 Rust 分析
  - 本地 Rust vs C 对比 → 跨实现对比
  - CI 摘要 → 趋势跟踪 + GitHub 可见分析表
- 规范 benchmark 集合已定义
- 当前基线表已记录在一处
- 后续优化轮次可以将本文档用作基线参考

到了 2026 年 3 月 17 日，这个任务必须重新视为打开状态，因为不仅记录基线和当前工作树出现了漂移，Criterion 的测量口径本身也做了调整。本文档现在既是当前基线参考，也是这次方法变化被明确记录下来的地方。

## 运行时字符串来源分析

当前 dump 模式探测显示：

- `runtime_string_pressure` 几乎完全由拼接驱动。
- `for_in_object` 主要由重复的 for-in key 创建请求主导。
- `deep_property` 基本不产生运行时字符串。
- `json_parse` 目前大多落入通用的 other 类别。

嵌入式说明：

- 运行时字符串预算限制有意推迟到后续设备集成工作中，不在此阶段硬编码到引擎中。

更新后的 9.3.3 结论：

- `runtime_string_pressure` 仍然完全由 concat 驱动。
- `for_in_object` 在当前“无复用”路径下仍会耗尽 runtime string 表，但现在会以可控引擎错误失败，而不是 panic。
- object_keys 已经确认是一个独立的 runtime string 来源桶。
- json_parse 目前仍然落在通用 other 桶里，如果继续推进这条主线，下一步应继续把它拆细。
