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

### 历史重验证快照（2026-03-17）

下表来自 compile-once 执行期 Criterion 口径落地后的第一轮重跑。它们仍然有历史参考价值，但已经不应继续被视为当前 head 的“当前可信快照”。

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

### 当前 head 的最新 broad rerun（2026-03-21）

这轮重跑的重要结论，不是“这就是新的黄金基线”，而是当前 head 相对 2026-03-17 那组历史快照已经出现了广泛漂移；因此 benchmark baseline correctness 必须重新视为打开状态。

| Benchmark | 最新 broad rerun |
|-----------|------------------|
| `array push 10k` | `766.00–946.17 µs` |
| `string concat 1k` | `164.20–205.55 µs` |
| `json parse 1k` | `1.8986–2.3272 ms` |
| `sieve 10k` | `2.3860–2.8523 ms` |
| `method_chain 5k` | `1.4008–1.7708 ms` |
| `runtime_string_pressure 4k` | `1.4943–1.8702 ms` |
| `for_of_array 20k` | `2.1365–2.5288 ms` |
| `deep_property 200k` | `19.605–23.419 ms` |

当前解读：

- 这已经不是“继续挑下一个微热点”的阶段；
- 当前 head 的 broad rerun 说明，多条 headline benchmark 相比 3 月 17 日那组历史快照都已经明显漂移；
- 因此接下来的优化轮次，应该先以“重新校准 benchmark 基线”为前提，而不是继续直接拿 3 月 17 日那组快照做判断。

### 次级跟踪 benchmark

这些数值现在应拆开看：

- 一组是 2026-03-17 的历史快照
- 另一组是 2026-03-21 在当前稳定工作树上的重跑结果

这样可以明确区分哪些数字只是历史参考，哪些数字已经真正对当前 head 做过复测。

| Benchmark | 历史快照 | 当前 head 重跑 |
|-----------|----------|----------------|
| `switch 1k` | `0.132–0.136 ms` | `0.282–0.337 ms` |
| `try_catch 5k` | `0.341–0.349 ms` | `0.433–0.544 ms` |
| `for_in_object 20x2000` | `3.743–3.804 ms` | `9.911–11.992 ms` |

## 当前基线（本地 Rust vs C，含进程启动）

以下数据基于本地多次进程执行的平均值，适用于跨实现对比。包含进程启动成本。

### 启动基线（最新本地重跑，2026-03-21）

| 场景 | Rust | C | 比率 |
|------|------|---|------|
| `mqjs -e "0"` | `41.300 ms` | `51.700 ms` | `0.799x` |

### 脚本对比（最新本地重跑，2026-03-21）

| Benchmark | Rust | C | 比率 | 备注 |
|-----------|------|---|------|------|
| `fib` | `63.200 ms` | `49.900 ms` | `1.266x` | C 更快 |
| `loop` | `38.900 ms` | `34.800 ms` | `1.119x` | C 更快 |
| `array` | `35.300 ms` | `31.100 ms` | `1.135x` | C 更快 |
| `json` | `51.600 ms` | `44.500 ms` | `1.159x` | C 更快 |
| `sieve` | `55.500 ms` | `47.600 ms` | `1.166x` | C 更快 |
| `method_chain` | `31.700 ms` | `26.700 ms` | `1.191x` | C 更快 |
| `runtime_string_pressure` | `29.700 ms` | `25.500 ms` | `1.162x` | C 更快 |
| `for_of_array` | `39.900 ms` | `47.600 ms` | `0.840x` | Rust 更快 |
| `deep_property` | `53.500 ms` | `72.400 ms` | `0.739x` | Rust 更快 |
| `switch_case` | `72.300 ms` | `43.400 ms` | `1.666x` | C 更快 |
| `try_catch` | `41.900 ms` | `61.300 ms` | `0.684x` | Rust 更快 |
| `for_in_object` | `58.200 ms` | `64.900 ms` | `0.897x` | Rust 更快 |

## 解读说明

### 当前最强信号

- `json parse` 仍然是当前最明确的活动主线，因为它确实拿到过稳定的定向收益；但它也正好说明了为什么现在必须先重建 benchmark 基线，再继续解释后续微优化结果。
- 最新一轮本地 Rust vs C 对比里，`json` 已经不再是 Rust 的相对强项；在当前 head 的进程级重跑里，它又回到了 C 更快的一侧。
- `for_of_array`、`deep_property` 和 `try_catch` 在最新的本地进程级对比里则反过来表现为 Rust 更快，而当前 benchmark 集合里的多数其余项目仍然是 C 更快。
- `array push`、`sieve`、`for_of_array`、`deep_property`、`method_chain`、`runtime_string_pressure` 这些 benchmark 仍然重要，但在最新 broad rerun 已明显漂移的前提下，旧的“当前健康区间”说法现在都应该先降级为历史参考，而不是继续当作当前真相。
- 字符串主线、迭代器主线、对象属性主线都确实有过真实收益，但这些收益在当前 head 上需要重新放回新的 baseline 口径里解释，而不是直接沿用旧结论。
- 次级控制流 / 异常 benchmark（`switch_case`、`try_catch`）在当前 head 的重跑里也明显抬高了，这进一步说明 baseline cleanup 仍然没有关单。
- `for_in_object` 当前应视为混合状态：
  - 结构性 key 复用修复已经让 Rust 侧能稳定跑完，不再耗尽 runtime string table；
  - 本地进程级 Rust vs C 对比现在也已经变成 Rust 略快；
  - 但它的当前 Criterion 时间仍远高于更早的历史快照，所以目前更适合解读为“正确性修复已落地，性能基线仍待清理”。
- `switch_case` 现已作为次要控制流 benchmark 被跟踪，并且在当前 head 上又拿到了一条新的结构收益：
  - `SwitchCaseI8` 把最热的整数 case-chain 比较路径收短后，`switch 1k` 已经进到 `223–277 µs` 区间；
  - 但本地 Rust vs C 对比里当前仍然是 C 更快（`1.666x`），所以这条线更适合被视为“局部字节码形状收益”，而不是整个控制流基线已经恢复健康的证明。
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
  - 为语句级 `local0 = local0 + "const"` 引入 `AppendConstStringToLoc0` 和只服务于局部槽位 `0` 的 per-frame builder
  - 为 `const + value + const + value` 引入 `AddConstStringSurroundValue`
  - 引入一个最小可用的延迟 `RuntimeString` 包装层，并让 `.length` 直接读取 cached runtime-string length
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
- `for_in_object` 现在主要由重复的 for-in key 处理，以及 key 值上的 `GetLength` 路径主导。
- `deep_property` 基本不产生运行时字符串。
- `json_parse` 目前大多落入通用的 other 类别。

嵌入式说明：

- 运行时字符串预算限制有意推迟到后续设备集成工作中，不在此阶段硬编码到引擎中。

更新后的 9.3.3 结论：

- `runtime_string_pressure` 仍然完全由 concat 驱动。
- 当前稳定工作树上的 `for_in_object` 已经不会再耗尽 runtime string 表，因为 `for-in` key 路径现在会复用重复 key 的 runtime string。
- object_keys 已经确认是一个独立的 runtime string 来源桶。
- json_parse 目前仍然落在通用 other 桶里，如果继续推进这条主线，下一步应继续把它拆细。

## 2026-03-21 UTF-8 补充：下一条大主线建议

- 当前这轮结构 cleanup 之后，`json parse`、`switch_case`、`for_in_object` 都应先视为阶段性收住：
  - `json parse`：稳定收益已拿到，后续实验开始进入高噪声区；
  - `switch_case`：`SwitchCaseI8` 已经是可保留的结构收益；
  - `for_in_object`：结构/正确性修复已完成，但性能解释继续留在 baseline cleanup 上下文里。
- 当前 broad rerun 和最新 Rust vs C 更像在提醒我们：
  - 下一条值得深做的，不再是 parser 内部或次级控制流小形状；
  - 而是 `loop` / `sieve` 共享的比较与循环骨架。
- 当前最像值得继续打的共享路径是：
  - `GetLoc*`
  - `Lt` / `Lte`
  - `IfFalse`
  - `Goto`
- 这样选的原因是：
  - `loop` 和 `sieve` 在最新本地 Rust vs C 里都还是 Rust 落后；
  - 它们都是 headline benchmark；
  - 同时这条线还能继续做，而不用重新打开已经冻结的 dense-array 读侧微优化。
- 所以，当前 baseline cleanup 收到足够可信的阶段后，推荐的下一条正式主线是：
  - `loop/sieve` 的 comparison-and-branch skeleton tightening

## 2026-03-21 UTF-8 补充：fib / switch_case 复查结论

- 后续又补跑了一轮更窄的定向 benchmark：
  - `fib_iter 1k`：`5.3292–6.2708 ms`
  - `switch 1k`：`281.10–345.33 µs`
- 当前结论应更新为：
  - `switch_case` 仍然算“结构收益已落地”：
    - `SwitchCaseI8` 仍在发码；
    - 它不是当前最值得重开的回归线。
  - 真正更重的当前回归信号是 `fib_iter`：
    - 它相比更早的 `2.330–2.379 ms` 历史区间漂移得更厉害；
    - 这说明下一条大主线里，`fib` / call-recursion overhead 的优先级应高于继续重开 `switch_case`。
- 所以当前更实用的优先级应读成：
  - 第一层：`fib` / call-recursion overhead
  - 第二层：`loop/sieve` 的 comparison-and-branch skeleton
  - `switch_case` 继续冻结，除非后面 profiling 出现新的 switch 专属热点形状
- dump-mode 热点现在也更直接支持这个判断：
  - `fib_iter`
    - 当前主要仍然花在“调用邻近的小循环骨架 + 局部槽位流量”上；
    - 内层迭代 fib 本体最显眼的是：
      - `GetLoc3`
      - `Drop`
      - `Dup`
      - `GetLoc0`
      - `Lte`
      - `GetLoc2`
      - `PutLoc2`
      - `Goto`
      - `Add`
      - `GetLoc4`
      - `PutLoc3`
      - `GetLoc8`
      - `PutLoc8`
      - `IncLoc4Drop`
    - 这更像“调用/递归开销叠加小循环与局部更新骨架”的问题，而不是单纯算术本身慢。
  - `switch_case`
    - `SwitchCaseI8` 在最新 dump 快照里仍然执行了 `108000` 次；
    - 所以它当前剩余成本更像是 switch 外围的 loop/add/update 骨架，而不是旧的整数 case 比较链。
- 实际结论：
  - `fib_iter` 现在是更清楚的下一条目标；
  - `switch_case` 继续保持冻结更合理，除非后续出现新的 switch 专属热点形状。
- `fib_iter` 这条线现在也已经拿到了第一条稳定收益：
  - 自动 GC trigger 记账不再挂在每一次通用 JS 调用路径上；
  - 而是改成挂在真实的 GC-managed allocation path 上。
- 结果：
  - 定向重跑：`fib_iter 1k` 下降到 `3.5469–4.1842 ms`
  - 后续复跑仍在同一档：`3.5909–4.2369 ms`
- 当前解读：
  - 这更像是在说明：前面那轮 `fib_iter` 大回退，不是“算术本身突然变慢”，而是 GC trigger bookkeeping 被放进了热调用路径；
  - 现在这条成本被挪回真正的 allocation path 之后，`fib_iter` 已经明显回来了；
  - 对应的 GC 自动触发回归仍然通过，所以这不是拿 GC 功能换来的 benchmark 数字。
- 当前稳定树上的后续重跑又给出了一组更适合收口的数字：
  - `fib_iter 1k`：`3.0507–3.6993 ms`
  - `loop 10k`：`690.21–846.31 µs`
  - `sieve 10k`：`2.9538–3.5064 ms`
- 当前解读应更新为：
  - `fib_iter` 这条线的第一条稳定收益仍然成立；
  - 后续那条会拖慢 `loop` 的跟进实验已经撤回，说明这条线已经进入“再往下做就开始高噪声 / 高误伤”的区域；
-  - 所以按当前 stop rule，更合理的做法不是继续硬抠 `fib_iter`，而是把它视为当前阶段基本收口，然后把主线切回 `loop/sieve`。
- 回到 `loop/sieve` 主线之后，又拿到了一条共享结构收益：
  - 非字符串 `Add / Mul` 在后面紧跟语句级 `PutLoc0..4` / `PutLoc8 <idx>` 时，现在会直接完成本地存储，不再先把结果压栈再立刻被本地存储弹掉。
- 结果：
  - `fib_iter 1k`：`2.2286–2.6849 ms`
  - `loop 10k`：`455.86–559.99 µs`
  - `sieve 10k`：`1.8323–2.1708 ms`
- 当前解读：
  - 这说明当前真正值钱的共享热点，确实是语句级本地算术结果物化；
  - 它不只帮了 `sieve`，也把 `loop` 和 `fib_iter` 一起往下拉了一截；
  - 因此这条收益应记为“回到 loop/sieve 主线后的第一条共享稳定收益”。
  - 到现在为止，`fib_iter` 已经拿到两条稳定收益，按当前 stop rule，可以把它视为本阶段基本完成，然后把主线切回 `loop/sieve`。
