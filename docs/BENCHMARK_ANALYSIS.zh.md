# 基准分析

英文版：`docs/BENCHMARK_ANALYSIS.md`

相关优化清单：
- `docs/ENGINE_OPTIMIZATION_TASKLIST.md`
- `docs/ENGINE_OPTIMIZATION_TASKLIST.zh.md`

## 文档目的

本文档用于定义 `mquickjs-rs` 引擎当前的 benchmark 基线与解释规则。

它只面向引擎本体，不覆盖 `led-runtime` 产品层行为。

## 基线策略

当前 benchmark 分析使用三类互补来源。

### 1. 本地 Criterion（`cargo bench`）

主要用途：

- 精确的 Rust-only 时间分析
- 优化前后对比验证
- 热点确认

当你需要判断“一次引擎优化是否真的生效”时，它是首要依据。

### 2. 本地 Rust-vs-C 对比（`benches/compare.sh` 或等价本地对比方式）

主要用途：

- 比较 Rust 引擎和 C 版 `mqjs` 的当前差距
- 判断某条路径离 C 实现还有多远

当你需要判断“跨实现差距”时，它是首要依据。

### 3. CI benchmark 汇总（`.github/workflows/bench.yml`）

主要用途：

- 在 GitHub Actions 上观察趋势
- 发现 push / PR 之后的明显回退
- 在 GitHub 上直接查看分析表格

当前 CI summary 会同时提供：

- Rust-vs-C 对比表
- Rust-only 的 Criterion 表
- 启动基线行

CI 结果现在已经足够有用，也方便在 GitHub 上查看；但在需要深入分析某次优化时，
本地 Criterion 与本地 Rust-vs-C 对比仍然是主要依据。

## 当前规范 benchmark 集合

当前规范集合包括：

- 传统核心集合：
  - `fib`
  - `loop`
  - `array`
  - `json`
  - `sieve`
- 第一波扩展集合：
  - `method_chain`
  - `runtime_string_pressure`
  - `for_of_array`
  - `deep_property`
- 当前已跟踪的第二批 benchmark：
  - `switch_case`

对当前 `no_std` 主线后置：

- `regexp_test`
- `regexp_exec`

未来第二批候选：

- `switch_case`
- `for_in_object`

当前已跟踪的第二批 benchmark：

- `switch_case`
- `try_catch`
- `for_in_object`

## 当前基线（本地 Criterion，Rust-only）

下面这组数据是第一波 benchmark 扩展和初步优化之后的当前本地 Criterion 基线。

| Benchmark | 当前 Rust-only 基线 |
|-----------|---------------------|
| `fib_iter 1k` | `2.056–2.102 ms` |
| `loop 10k` | `0.512–0.528 ms` |
| `array push 10k` | `0.672–0.691 ms` |
| `json parse 1k` | `0.856–0.919 ms` |
| `sieve 10k` | `2.556–2.687 ms` |
| `method_chain 5k` | `0.720–0.763 ms` |
| `runtime_string_pressure 4k` | `2.893–3.379 ms` |
| `for_of_array 20k` | `3.471–4.071 ms` |
| `deep_property 200k` | `19.510–22.446 ms` |
| `switch 1k` | `0.132–0.136 ms` |
| `try_catch 5k` | `0.341–0.349 ms` |
| `for_in_object 20x2000` | `3.743–3.804 ms` |

## 当前基线（本地 Rust vs C，含进程启动）

下面这组数据基于本地重复进程执行平均值，适合做 Rust-vs-C 横向比较。它包含进程启动开销。

### 启动基线

| 场景 | Rust | C | 比值 |
|------|------|---|------|
| `mqjs -e "0"` | `18.130 ms` | `17.407 ms` | `1.042x` |

### 脚本对比

| Benchmark | Rust | C | 比值 | 说明 |
|-----------|------|---|------|------|
| `fib` | `183.099 ms` | `118.815 ms` | `1.541x` | C 更快 |
| `loop` | `94.261 ms` | `62.165 ms` | `1.516x` | C 更快 |
| `array` | `17.673 ms` | `16.467 ms` | `1.073x` | C 略快 |
| `json` | `44.048 ms` | `64.280 ms` | `0.685x` | Rust 更快 |
| `sieve` | `52.476 ms` | `35.676 ms` | `1.471x` | C 更快 |
| `method_chain` | `15.795 ms` | `14.109 ms` | `1.119x` | C 略快 |
| `runtime_string_pressure` | `22.470 ms` | `19.185 ms` | `1.171x` | C 更快 |
| `for_of_array` | `19.509 ms` | `17.358 ms` | `1.124x` | C 更快 |
| `deep_property` | `32.465 ms` | `24.001 ms` | `1.353x` | C 更快 |
| `switch_case` | `18.495 ms` | `16.856 ms` | `1.097x` | C 略快 |
| `try_catch` | `16.097 ms` | `14.508 ms` | `1.110x` | C 略快 |
| `for_in_object` | `22.356 ms` | `17.633 ms` | `1.268x` | C 更快 |

## 当前结论

### 当前最明显的信号

- `json` 仍然是 Rust 引擎的相对优势项。
- `fib` 与 `loop` 仍然说明调用路径与 dispatch 成本较高。
- `array` 与 `sieve` 仍然说明 dense array 读写路径有优化空间。
- `runtime_string_pressure` 说明字符串创建与运行时字符串增长仍然值得继续优化。
- `for_of_array` 与 `deep_property` 现在已经纳入规范基线，应作为正式热点长期跟踪。
- `switch_case` 现在作为第二批控制流 benchmark 进行跟踪，并且已经从最近一次 `StrictEq` 热路径优化中获得了可测提升。
- `try_catch` 现在作为第二批异常控制流 benchmark 进行跟踪，并且已经有第一版稳定基线。
- `for_in_object` 现在作为第二批迭代器/控制流 benchmark 进行跟踪，并且已经在迭代器初始化收口后记录了第一版基线。

### 需要注意的地方

- Criterion 数据和“含进程启动的 Rust-vs-C 数据”回答的问题不同。
- 执行时间较短的脚本更容易受到启动开销影响。
- 在验证一次引擎优化是否有效时，应优先看 Criterion。

## 当前基线中已包含的优化

当前基线已经包含以下第一轮已完成优化：

- `deep_property`
  - 普通小对象属性读取 fast path
  - `GetField` / `GetField2` 分发收口
- `method_chain`
  - 数组高阶方法中去掉每元素回调的临时 `Vec<Value>` 分配
  - 为 `CallMethod` 的 native 小参数调用增加 fast path
- `for_of_array`
  - `ForOfStart` 不再整数组复制

这些变更同步记录在：

- `docs/ENGINE_OPTIMIZATION_TASKLIST.md`
- `docs/ENGINE_OPTIMIZATION_TASKLIST.zh.md`

## 现在“9.1.1 benchmark 基线收口”意味着什么

当满足以下条件时，可以认为 9.1.1 已完成：

- benchmark 角色已经明确区分：
  - 本地 Criterion -> 精确 Rust-only 分析
  - 本地 Rust-vs-C -> 跨实现差距分析
  - CI summary -> 趋势跟踪 + GitHub 可视化分析表
- 当前规范 benchmark 集合已经定义
- 当前基线表已经集中记录在一处
- 后续优化都可以以本文档作为统一基线参考

现在这份文档就是当前统一基线参考。
