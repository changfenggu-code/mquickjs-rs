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

- 精确的纯 Rust 计时分析
- 优化前后的验证
- 热点确认

判断引擎优化是否真的有效时，这是首选来源。

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

以下数据是第一波 benchmark 扩展和初始优化轮次之后的当前本地 Criterion 基线。

| Benchmark | 当前纯 Rust 基线 |
|-----------|----------------|
| `fib_iter 1k` | `2.056-2.102 ms` |
| `loop 10k` | `0.484-0.499 ms` |
| `array push 10k` | `0.672-0.691 ms` |
| `json parse 1k` | `0.856-0.919 ms` |
| `sieve 10k` | `2.014-2.074 ms` |
| `method_chain 5k` | `0.720-0.763 ms` |
| `runtime_string_pressure 4k` | `2.893-3.379 ms` |
| `for_of_array 20k` | `3.471-3.071 ms` |
| `deep_property 200k` | `19.510-22.446 ms` |
| `switch 1k` | `0.132-0.136 ms` |
| `try_catch 5k` | `0.341-0.349 ms` |
| `for_in_object 20x2000` | `3.743-3.804 ms` |

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
- `fib` 和 `loop` 仍然指向调用路径和 dispatch 开销。
- `array` 和 `sieve` 仍然指向 dense array 读写开销。
- `runtime_string_pressure` 仍然是有意义的内存/字符串创建路径。
- `for_of_array` 和 `deep_property` 现已成为规范基线的一部分，应作为一级优化目标。
- `switch_case` 现已作为次要控制流 benchmark 被跟踪，最新的 `StrictEq` 热路径优化已显示出可测量的改善。
- `try_catch` 现已作为次要异常控制 benchmark 被跟踪，已记录清理后的基线。
- `for_in_object` 现已作为次要迭代器/控制流 benchmark 被跟踪，迭代器初始化清理后记录了第一个基线。

### 重要注意事项

- Criterion 数据和含进程启动的 Rust vs C 数据回答的是不同问题。
- 短运行脚本对比对启动开销更敏感。
- 在验证 Rust 引擎内部的优化时，优先使用 Criterion。

## 已完成的优化工作（已反映在本基线中）

当前基线已包含以下完成的第一轮优化：

- `deep_property`
  - 小对象属性查找快速路径
  - 统一 `GetField` / `GetField2` 属性分发
- `method_chain`
  - 去除数组高阶 builtin 中每个元素的临时 `Vec<Value>` 分配
  - 为 `CallMethod` native 添加小参数快速路径
  - 为 `Array.prototype.push` native 添加 `argc == 1` 专用快捷方式
- `for_of_array`
  - 从 `ForOfStart` 中去除完整数组克隆
- `loop` / `sieve`
  - 为常见语句更新模式添加 `Dup + PutLocX + Drop` peephole 快速路径
  - 添加 `Lt/Lte` + `IfFalse/IfTrue` 分支融合

这些改动在以下文档中跟踪：

- `docs/ENGINE_OPTIMIZATION_TASKLIST.md`
- `docs/ENGINE_OPTIMIZATION_TASKLIST.zh.md`

## 「9.1.1 Benchmark 基线清理」的含义

本基线清理任务在满足以下条件时视为完成：

- benchmark 职责已明确分离：
  - 本地 Criterion → 精确的纯 Rust 分析
  - 本地 Rust vs C 对比 → 跨实现对比
  - CI 摘要 → 趋势跟踪 + GitHub 可见分析表
- 规范 benchmark 集合已定义
- 当前基线表已记录在一处
- 后续优化轮次可以将本文档用作基线参考

本文档即为当前基线参考。

## 运行时字符串来源分析

当前 dump 模式探测显示：

- `runtime_string_pressure` 几乎完全由拼接驱动。
- `for_in_object` 主要由重复的 for-in key 创建请求主导。
- `deep_property` 基本不产生运行时字符串。
- `json_parse` 目前大多落入通用的 other 类别。

嵌入式说明：
- 运行时字符串预算限制有意推迟到后续设备集成工作中，不在此阶段硬编码到引擎中。
