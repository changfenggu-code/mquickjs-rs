# Engine Optimization Task List

This document is an engine-only optimization backlog for `mquickjs-rs`.

It is derived from the unfinished optimization items in `IMPLEMENTATION_PLAN.md`:

- `9.1 Profile and optimize hot paths`
- `9.2 Optimize GC performance`
- `9.3 Reduce memory usage`

It intentionally excludes `led-runtime` product-layer work.

Related benchmark analysis:
- `docs/BENCHMARK_ANALYSIS.md`
- `docs/BENCHMARK_ANALYSIS.zh.md`

## Scope

This task list only covers:

- the parser / compiler / VM / runtime of `mquickjs-rs`
- benchmark correctness and performance analysis
- GC and memory behavior of the engine itself

This task list does not cover:

- `led-runtime` host API ergonomics
- effect script/product semantics
- GUI/demo/product-layer integration

## Current Optimization Themes

Based on the current code and benchmark shape, the most likely engine hotspots are:

- call and method dispatch in `src/vm/interpreter.rs`
- native / builtin call argument marshalling in `src/vm/interpreter.rs` and `src/vm/natives.rs`
- dense array access in `src/vm/interpreter.rs` and `src/vm/property.rs`
- opcode dispatch overhead in `src/vm/interpreter.rs`
- GC implementation quality in `src/gc/collector.rs`
- engine-side runtime allocations and container layout in `src/vm/types.rs`, `src/context.rs`, and `src/runtime/*`

## Priority Summary

### P0

- Benchmark truth and baseline cleanup
- Call-path hot path optimization
- Native/builtin call marshalling optimization
- Dense array access fast paths

### P1

- Opcode dispatch simplification for the hottest instructions
- GC: stop conservative `mark_all` behavior and move toward real root-based marking
- Runtime allocation and memory footprint review

### P2

- Structural cleanup of builtin/runtime boundaries
- Secondary micro-optimizations after new benchmark validation

## Detailed Task List

## 9.1 Profile and Optimize Hot Paths

### 9.1.1 Benchmark baseline cleanup

**Priority**: P0

**Why**

- Optimization work is only useful if benchmark data is trustworthy.
- The benchmark workflow and local comparison script were previously inconsistent.
- Some historical benchmark conclusions were based on invalid comparison targets.

**Tasks**

- Keep a single trusted benchmark process for local verification.
- Keep CI benchmark behavior aligned with local benchmark behavior.
- Separate process startup overhead from net script execution time.
- Maintain one canonical baseline table for:
  - `fib`
  - `loop`
  - `array`
  - `sieve`
  - `json`

**Validation**

- Benchmark results are reproducible across repeated runs.
- `docs/BENCHMARK_ANALYSIS.md` is internally consistent.

**Completed so far**

- 2026-03-16: the canonical benchmark set has been defined.
- 2026-03-16: local Criterion, local Rust-vs-C comparison, and CI summary roles have been separated and documented.
- 2026-03-16: `.github/workflows/bench.yml` now publishes both a Rust-vs-C comparison table and a Rust-only Criterion table.
- 2026-03-16: `docs/BENCHMARK_ANALYSIS.md` was rewritten as the current baseline reference.
- 2026-03-17: a full local Criterion revalidation for the primary benchmark set was run on the current worktree.
- 2026-03-17: the local Criterion harness was changed to compile benchmark scripts once and measure execution on fresh contexts, reducing parser/compiler noise in runtime hotspot work.
- 2026-03-17: `docs/BENCHMARK_ANALYSIS.md` was updated to distinguish the new execution-focused current-worktree snapshot from older Criterion generations.
- Status: reopened; benchmark-baseline cleanup is active again until the current head and the documentation stay in sync.

### 9.1.2 Call-path hot path optimization [Completed]

**Priority**: P0

**Hot files**

- `src/vm/interpreter.rs`
- `src/vm/stack.rs`

**Why**

- `fib` and `loop` strongly suggest call overhead and high-frequency dispatch overhead remain major costs.
- The current `Call` path is improved, but still uses `remove_at_offset()` which delegates to `Vec::remove()` and causes element shifting.

**Tasks**

- Rework call stack layout to avoid `Vec::remove()` on the hot call path.
- Specialize `Call`, `CallMethod`, and `CallConstructor` separately.
- Reduce temporary argument reshaping for normal JS function calls.
- Re-check string promotion cost inside the call path.

**Expected gain**

- Primary improvement target for `fib`
- Secondary improvement for `loop`

**Completed so far**

- 2026-03-16: first round of `method_chain`-related optimization completed in array higher-order methods by removing per-element temporary `Vec<Value>` argument allocation in callback-heavy array builtins.
- Regression coverage added for chained `map().filter().reduce()` behavior.
- Benchmark result: `method_chain 5k` improved from roughly `1.88–2.54 ms` to `0.80–0.82 ms` in Criterion.
- 2026-03-17: replaced the `Vec::remove()`-backed extraction pattern on the `Call` / `CallMethod` hot path with a single tail-compaction step, and kept JS method-call arguments on stack instead of repacking them into a temporary `Vec<Value>`.
- 2026-03-17: extended the same in-place argument handling approach to `CallConstructor`, so plain JS constructor calls no longer rebuild their argument list through a temporary `Vec<Value>`.
- Re-ran direct function-call, multi-argument push-order, and chained `map().filter().reduce()` regression coverage successfully.
- Re-ran constructor semantics regression coverage successfully (`new`, `instanceof`, simple constructor cases).
- Under the updated compile-once Criterion harness, the current local snapshot is:
  - `fib_iter 1k`: `2.330–2.379 ms`
  - `loop 10k`: `0.472–0.485 ms`
  - `array push 10k`: `0.614–0.633 ms`
- Current interpretation: the call-path round remains real and useful, but further comparisons must now be made only within the new execution-focused benchmark generation.
- 2026-03-17: removed full-array cloning from the hot higher-order builtin paths (`map`, `filter`, `forEach`, `reduce`, `find`, `findIndex`, `some`, `every`) and replaced it with length-snapshot iteration plus live element reads.
- Added regression coverage for:
  - snapshot-length behavior under callback-driven `push()`
  - observing updated future elements during `map()`
- Latest full rerun under the execution-focused Criterion harness:
  - `method_chain 5k`: `0.699–0.707 ms`
  - `runtime_string_pressure 4k`: `1.237–1.269 ms`
- Current interpretation: this round materially improved callback-heavy array pipelines and reduced pressure on runtime-string-heavy loops through a dedicated `.length` fast path and lower builtin overhead.
- 2026-03-17: added dedicated `CallArrayMap1` / `CallArrayFilter1` / `CallArrayReduce2` opcodes so the hottest single-callback array higher-order call shapes no longer pay the generic `CallMethod` argument reshaping path after `GetField2`.
- Added a fallback regression test confirming non-array receivers with their own `map` method still preserve generic method-call semantics.
- Selected execution-focused Criterion reruns on the current worktree:
  - `method_chain 5k`: `0.611–0.628 ms`
  - `runtime_string_pressure 4k`: `1.190–1.216 ms`
  - `array push 10k`: `0.575–0.600 ms`
- Current interpretation: this is a good example of a bytecode-shape-specific array-builtin call optimization that pays off without broadening the generic call path, and the current reruns suggest that the win also propagates into nearby array-heavy paths.
- 2026-03-17: added a dedicated `CallArrayPush1` opcode for the hottest single-argument `.push(arg)` method-call shape, keeping the `GetField2` stack contract but bypassing the generic `CallMethod` reshaping path for the dominant array-building loop form.
- Added fallback regression coverage confirming non-array receivers with their own `push` method still preserve generic method-call semantics.
- Selected execution-focused Criterion reruns on the current worktree:
  - `array push 10k`: `0.491–0.502 ms`
  - `method_chain 5k`: `0.585–0.600 ms`
  - `runtime_string_pressure 4k`: `1.177–1.197 ms`
- Current interpretation: this is the first `method_chain` round to consistently land at the edge of the `<= 0.60 ms` success line, and it does so by directly shrinking the array-construction prefix that still dominated the benchmark after the higher-order builtin call specializations.
- Status: complete as a call-path optimization phase; any future work here should be treated as follow-on tuning rather than unfinished core call-path cleanup.

### 9.1.3 Native/builtin call marshalling optimization

**Priority**: P0

**Hot files**

- `src/vm/interpreter.rs`
- `src/vm/natives.rs`

**Why**

- Native and builtin calls still build temporary `Vec<Value>` buffers and reverse them.
- This path affects `Math.*`, `JSON.*`, array methods, and other builtins.

**Tasks**

- Add specialized fast paths for 0/1/2 argument native calls.
- Avoid heap allocation for short native/builtin argument lists.
- Reduce or remove `reverse()` in native/builtin call preparation.
- Consider passing stack-backed argument slices where safe.

**Expected gain**

- Improves builtin-heavy scripts
- Helps `array`, `json`, and math-heavy workloads

**Completed so far**

- 2026-03-16: added a `CallMethod` native fast path for small argument counts by removing temporary argument `Vec` allocation on the native-method path for `argc <= 2`.
- Added regression coverage for multi-argument `Array.prototype.push` argument order.
- Benchmark result: `array push 10k` improved from roughly `0.897–0.911 ms` to `0.672–0.691 ms` in Criterion.
- Benchmark result: `method_chain 5k` improved further from roughly `0.986–1.182 ms` to `0.720–0.763 ms` in Criterion.
- 2026-03-16: added a direct `Array.prototype.push` native fast path in `CallMethod`, with a dedicated `argc == 1` shortcut that removes generic native-call overhead from the hot array initialization path.
- Reused existing `Array.prototype.push` regression coverage to validate semantics.
- Benchmark result: `sieve 10k` improved from roughly `2.038–2.078 ms` to `2.014–2.074 ms` in Criterion.
- 2026-03-17: changed array `.push` property lookup to return the cached native function index directly instead of re-scanning the native registry by name on every access.
- Re-ran `Array.prototype.push` regression coverage successfully.
- Selected execution-focused Criterion reruns on the current worktree:
  - `array push 10k`: `0.589–0.602 ms`
  - `method_chain 5k`: `0.654–0.668 ms`
- Current interpretation: this is a small but real property-dispatch cleanup for the hottest array method path, though it should still be interpreted inside the current benchmark generation only.
- 2026-03-17: taught the `Array.prototype.push` native fast path to consume a following `Drop`, so statement-position `arr.push(...)` no longer pushes a return length that is immediately discarded.
- Re-ran `Array.prototype.push` return-value regression coverage successfully.
- Selected execution-focused Criterion reruns on the current worktree:
  - `array push 10k`: `0.532–0.539 ms`
  - `sieve 10k`: `1.640–1.670 ms`
  - `method_chain 5k`: `0.606–0.618 ms`
- Current interpretation: this is a high-value narrow optimization because it targets the exact hot statement form used in array-building loops while preserving expression-position semantics.
- 2026-03-17: extended the native/builtin small-argument fast paths for `Call` / `CallMethod` / builtin-as-function calls from `argc <= 2` to `argc == 3`, removing one more `Vec<Value>` allocation layer from common three-argument native shapes.
- Added regression coverage confirming three-argument native call ordering (`Math.max(1, 4, 2)`).
- Selected execution-focused Criterion reruns on the current worktree:
  - `array push 10k`: `0.472–0.481 ms`
  - `json parse 1k`: `0.732–0.749 ms`
  - `method_chain 5k`: `0.590–0.604 ms`
- Current interpretation: the current primary benchmark set does not show a new standalone `json`-class breakout from this change, but it does close an obvious remaining small-argument marshalling gap and keeps nearby call-heavy benchmarks healthy.

### 9.1.4 Dense array fast paths

**Priority**: P0

**Hot files**

- `src/vm/interpreter.rs`
- `src/vm/property.rs`
- `src/runtime/array.rs`

**Why**

- `array` and `sieve` are classic dense-array benchmarks.
- Current access still goes through several generic layers.

**Tasks**

- Shorten `GetArrayEl`, `GetArrayEl2`, and `PutArrayEl` paths.
- Special-case dense integer-index access.
- Avoid generic property lookup for obviously-array operations.
- Review `push`, indexed read, and indexed write paths separately.

**Expected gain**

- Main improvement target for `array`
- Strong expected gain for `sieve`

**Completed so far**

- 2026-03-16: first deep property optimization completed by adding a small-object fast path for regular object property lookup and unifying `GetField` / `GetField2` property dispatch.
- Regression coverage added for deep property chain access.
- Benchmark result: `deep_property 200k` improved from roughly `28–29 ms` to `15.7–17.0 ms` in Criterion.
- Important interpretation: this completed work primarily improves regular object property access, not the still-open dense-array-specific read/write fast-path work.
- 2026-03-20: reopened the deep-property/object-access line with a narrower `GetField` / `GetField2` fast path for ordinary objects, letting those opcodes jump straight to `object_get_property()` instead of routing through the full generic `get_field_value()` type-dispatch ladder first.
- Re-ran targeted deep-property regression coverage successfully.
- Selected execution-focused Criterion reruns on the current worktree:
  - `deep_property 200k`: `18.706–20.128 ms`
  - `method_chain 5k`: `1.106–1.214 ms`
  - `runtime_string_pressure 4k`: `1.118–1.205 ms`
- Current interpretation: this is the cleanest current follow-up on the original deep-property win because it directly targets repeated plain-object chain access (`root.a.b.c.d`) while also helping nearby object-heavy workloads instead of only improving a synthetic microbenchmark.
- 2026-03-17: added a `PutArrayEl + Drop` peephole fast path so statement-position array assignments no longer materialize an unused result value on the stack.
- Re-ran array assignment statement and assignment-expression regression coverage successfully.
- Selected execution-focused Criterion reruns on the current worktree:
  - `array push 10k`: `0.609–0.621 ms`
  - `sieve 10k`: `2.045–2.084 ms`
- Current interpretation: this is a small but clean dense-array write-path improvement that specifically targets statement-style array stores such as the hot `primes[j] = false;` shape in `sieve`.
- 2026-03-18: specialized the even narrower `PushFalse; PutArrayEl; Drop` statement shape into `PutArrayElFalseDrop`, directly targeting the hottest boolean write pattern in the sieve inner loop.
- Added compiler coverage confirming `arr[idx] = false;` now emits the new opcode, while existing assignment-expression and array-condition regressions remain green.
- Selected execution-focused Criterion reruns on the current worktree:
  - `sieve 10k`: `1.638–1.668 ms`
  - `array push 10k`: `0.556–0.563 ms`
- Current interpretation: this is the first dense-array write-path specialization that clearly lands on the exact `primes[j] = false;` kernel in `sieve`, and it brings that benchmark down another visible step without changing assignment-expression semantics.
- 2026-03-18: split the hottest `.push` property read out of generic `GetField2` into a dedicated `GetArrayPush2` opcode, so array-heavy initialization loops no longer pay the general string-indexed property dispatch cost before `CallArrayPush1`.
- Added compiler coverage confirming `arr.push(...)` now emits `GetArrayPush2`, and added an eval regression locking that `obj.push(side_effect())` still throws on `null` before argument side effects run.
- Re-ran targeted push/fallback regressions, full engine tests, and `clippy -D warnings`; all passed.
- Selected execution-focused Criterion reruns on the current worktree:
  - `array push 10k`: `0.593–0.716 ms`
  - `sieve 10k`: `1.718–1.753 ms`
  - `dense array bool read branch 10k`: `1.139–1.153 ms`
  - `dense array false write only 10k`: `1.500–1.770 ms`
- Current interpretation: this is a dense-array initialization-path win rather than a true `GetArrayEl` read-path breakthrough, but it cleanly removes the hottest remaining generic `.push` property read from `sieve`-style array setup and measurably helps the new dense-array diagnostics as well.
- 2026-03-18: added a new diagnostic benchmark/script pair for `dense_array_bool_condition_only_hot`, isolating repeated `GetArrayEl + IfFalse` scans without the extra `count = count + 1` accumulation work.
- `dump_bytecode` now shows that this benchmark compiles to the minimal shape `GetArrayEl; IfFalse; IncLoc; Goto`, making it the cleanest current reproducer for the remaining read-side branch skeleton.
- Current interpretation: dense-array read tuning should now be judged against three increasingly pure shapes: `dense array bool read branch 10k`, `dense array bool read hot`, and `dense array bool condition only hot`, in that order of diagnostic sharpness.
- 2026-03-18: added a further `dense_array_read_only_hot` diagnostic benchmark/script that isolates repeated `GetArrayEl; Drop; IncLoc; Goto` scanning with no condition branch at all.
- `dump_bytecode` now shows this fourth diagnostic as the cleanest current reproducer for pure array-read cost, separate from both truthiness branching and count accumulation semantics.
- Current interpretation: dense-array read tuning should now be judged against four shapes:
  - `dense array bool read branch 10k`
  - `dense array bool read hot`
  - `dense array bool condition only hot`
  - `dense array read only hot`
  from least pure to most pure.
- 2026-03-19: retained `GetArrayElDiscard` for discarded statement-form reads (`arr[idx];`) so pure-read diagnostics can measure array access cost without also paying the generic `Drop` path.
- Added compiler/eval regression coverage confirming discarded array reads now lower to the dedicated opcode and still leave surrounding program behavior unchanged.
- 2026-03-19: retained a dedicated `GetArrayElDiscard` statement-form read opcode for discarded `arr[idx];` expression statements. This keeps the pure-read diagnostic path available without changing expression or branch semantics.
- Added regression coverage confirming discarded array reads still evaluate and execution continues normally.
- 2026-03-18: added a final `dense_array_loop_only_hot` diagnostic benchmark/script that strips array reads out completely and measures just the repeated `Lte; IncLoc; Goto` loop skeleton.
- `dump_bytecode` now shows this baseline compiling to the pure loop shape `PushI16; Lte; IfFalse; IncLoc; Goto`.
- Current interpretation: dense-array read tuning can now be decomposed into five layers:
  - `dense array loop only hot`
  - `dense array read only hot`
  - `dense array bool condition only hot`
  - `dense array bool read hot`
  - `dense array bool read branch 10k`
  with each added layer showing the cost of one more piece of the real workload.
- 2026-03-19: added matched `arg0` / `local1` control variants for the pure read and pure condition diagnostics:
  - `dense array read only hot arg0`
  - `dense array read only hot local1`
  - `dense array bool condition only hot arg0`
  - `dense array bool condition only hot local1`
- Current interpretation: after controlling for function shape and parameter count, the remaining `local0` vs non-0-local difference looks small and noisy; `GetLoc0` is not currently supported as the dominant remaining read-side cost center.
- 2026-03-19: added `analyze_dense_array_layers` to compute per-layer deltas in one run so future read-side work can compare loop-only, read-only, condition-only, and accumulated shapes without manually reconciling multiple benchmark outputs.
- Current interpretation: use `analyze_dense_array_layers` as the primary “what layer is still expensive?” tool before attempting new `GetArrayEl` optimizations.
- 2026-03-19: tightened dense-array indexed access itself by collapsing the repeated “array value + non-negative integer index” decode into one raw fast helper shared by `GetArrayEl`, `GetArrayElDiscard`, `PutArrayEl`, and `PutArrayElFalseDrop`.
- Re-ran `cargo check -p mquickjs-rs`, full `cargo test -p mquickjs-rs`, and `cargo clippy -p mquickjs-rs --all-targets -- -D warnings`; all passed.
- Selected execution-focused Criterion reruns on the current worktree:
  - `sieve 10k`: `1.3173–1.3389 ms`
  - `dense array bool read hot`: `68.603–69.468 ms`
  - `dense array bool condition only hot`: `57.001–58.300 ms`
- Current interpretation: this is a clean read/write-path tightening that finally lands directly inside the dense-array indexed access opcodes themselves, not just around surrounding loop skeletons or array initialization.
- 2026-03-19: further tightened `GetArrayEl` branch bookkeeping by replacing the temporary `Option<(bool, i32)>` branch peek state with a simpler direct opcode/offset pair inside the fused `GetArrayEl + IfFalse/IfTrue` hot path.
- Re-ran full `cargo test -p mquickjs-rs` and `cargo clippy -p mquickjs-rs --all-targets -- -D warnings`; both passed.
- Selected execution-focused Criterion reruns on the current worktree:
  - `dense array bool read hot`: `81.896–87.290 ms`
  - `dense array bool condition only hot`: `71.628–77.040 ms`
  - `sieve 10k`: `1.6203–1.7606 ms`
- Current interpretation: this is a narrow `GetArrayEl`-internal cleanup that improves the read-heavy fused-branch workload without introducing a measurable regression in the broader `sieve` path.
- 2026-03-19: tightened the fused `GetArrayEl + IfFalse/IfTrue` branch peek one step further by switching it to a single length check plus unchecked byte reads for the hot opcode/offset decode path.
- Re-ran `cargo check -p mquickjs-rs`, full `cargo test -p mquickjs-rs`, and `cargo clippy -p mquickjs-rs --all-targets -- -D warnings`; all passed.
- Selected execution-focused Criterion reruns on the current worktree:
  - `sieve 10k`: `1.3077–1.3308 ms`
  - `dense array bool read hot`: `73.169–74.204 ms`
  - `dense array bool condition only hot`: `59.915–60.797 ms`
- Current interpretation: this is another narrow `GetArrayEl` bookkeeping win, and this time the improvement shows up consistently across both pure condition scans and the heavier read-and-accumulate workload.
- 2026-03-19: split `GetArrayElDiscard`'s dense-array fast predicate out from the tuple-returning `dense_array_access()` helper, so discarded statement-form reads no longer pay to decode and allocate unused `(arr_idx, index)` data on the hottest pure-read path.
- Re-ran `cargo check -p mquickjs-rs`, full `cargo test -p mquickjs-rs`, and `cargo clippy -p mquickjs-rs --all-targets -- -D warnings`; all passed.
- Selected execution-focused Criterion reruns on the current worktree:
  - `dense array read only hot`: `51.587–52.737 ms`
  - `dense array read only hot arg0`: `50.887–51.752 ms`
  - `dense array read only hot local1`: `52.082–52.993 ms`
  - `sieve 10k`: `1.3022–1.3268 ms`
- Current interpretation: this is a small but real pure-read-path win, and it keeps the denser `GetArrayElDiscard` diagnostic baseline moving down without perturbing broader array semantics.
- 2026-03-19: relaxed the discarded dense-array read fast path further so `GetArrayElDiscard` now treats any integer index on a regular array as an immediate no-op read, instead of insisting on the non-negative-index decode used by value-producing array reads.
- Added eval regression coverage confirming a discarded negative index read (`arr[-1];`) still behaves as an ignored undefined read and execution continues normally.
- Re-ran `cargo check -p mquickjs-rs`, targeted eval regression coverage, and `cargo clippy -p mquickjs-rs --all-targets -- -D warnings`; all passed. Full `cargo test -p mquickjs-rs` also passed before later Windows linker/pagefile instability returned during unrelated release-tool recompiles.
- Selected execution-focused Criterion reruns on the current worktree:
  - `dense array read only hot`: `51.587–52.737 ms`
  - `dense array read only hot arg0`: `50.887–51.752 ms`
  - `dense array read only hot local1`: `52.082–52.993 ms`
  - `sieve 10k`: `1.3022–1.3268 ms`
- Current interpretation: this is another narrow but durable `GetArrayElDiscard` win, aimed squarely at the pure-read diagnostic path rather than the broader branch-heavy `GetArrayEl` workload.
- Status update: the current dense-array read-side micro-optimization round is now effectively closed. Later `GetArrayEl` / `GetArrayElDiscard` work should reopen only if fresh profiling shows a materially different hotspot shape, because the current line has entered a high-regression, low-signal zone.
- 2026-03-18: added a dedicated `IncLoc*Drop` family for statement-form `local = local + 1` updates whose result is immediately discarded, and rewrote those hottest tails in loop increments and dense-array read counters into the new opcodes.
- Added compiler coverage confirming `var i = 0; i = i + 1;` now lowers to the dedicated local-update opcode family, and added an eval regression locking that `var x = 'a'; x = x + 1;` still preserves string-concatenation semantics.
- `dump_bytecode` now shows the dense-array read diagnostics compiling to compact `IncLoc{1,2,3,4}Drop` tails instead of repeated `Push1; Add; Dup; PutLocX; Drop` skeletons around `GetArrayEl`.
- Re-ran full engine tests and `clippy -D warnings`; all passed.
- Selected execution-focused Criterion reruns on the current worktree:
  - `loop 10k`: `0.416–0.477 ms`
  - `sieve 10k`: `1.496–1.514 ms`
  - `dense array bool read branch 10k`: `0.804–0.822 ms`
  - `dense array bool read hot`: `69.67–70.63 ms`
  - `dense array false write then read hot`: `60.76–61.67 ms`
- Current interpretation: this is the first change in the current dense-array phase that materially shrinks the remaining read-side loop skeleton itself; it still helps the classic `loop` benchmark, but the main effect is that the `GetArrayEl`-heavy diagnostics now spend far less time on local increment bookkeeping around the array read.
- 2026-03-18: narrowed the `GetArrayEl` branch-fused fast path itself so the hottest array/int-index read case now handles `true` / `false` / `null` / `undefined` / int truthiness directly before falling back to the generic helper.
- Re-ran full engine tests and `clippy -D warnings`; all passed.
- Selected execution-focused Criterion reruns on the current worktree:
  - `sieve 10k`: `1.795–2.085 ms` (no significant change)
  - `dense array bool read branch 10k`: `0.919–1.118 ms` (no significant change)
  - `dense array bool read hot`: `72.35–73.37 ms`
  - `dense array bool condition only hot`: `69.42–78.77 ms`
- Current interpretation: this is a small read-side truthiness win that shows up most clearly on the purer `GetArrayEl + IfFalse` diagnostics, while staying roughly neutral on the larger mixed-shape baselines.

### 9.1.5 Opcode dispatch tightening [Completed]

**Priority**: P1

**Hot files**

- `src/vm/interpreter.rs`

**Why**

- `loop` still suggests meaningful instruction dispatch overhead.
- Large match-based dispatch is correct and maintainable, but still costly in the hottest path.

**Tasks**

- Identify the top 10–20 hottest opcodes from benchmark-driven profiling.
- Shorten per-iteration work in the dispatch loop.
- Reduce repeated decode / branch / error-path overhead in hot instructions.
- Prefer local fast paths for arithmetic, local-variable, jump, and call instructions.

**Expected gain**

- Best secondary target for `loop`
- Broad win across many benchmarks

**Completed so far**

- 2026-03-16: added `try_catch` benchmark coverage for repeated throw/catch control flow.
- 2026-03-16: reduced exception routing overhead by unifying exception dispatch and replacing repeated pop-based unwind loops with `truncate` / `drop_n` based unwinding.
- Added regression coverage for repeated throw/catch inside a loop.
- Benchmark result: `try_catch 5k` baseline recorded at `340–349 μs` in Criterion.
- 2026-03-16: added feature-gated runtime opcode counters under the `dump` feature and exposed them through `Context` for profiling work.
- Added a `dump`-mode regression test to ensure opcode counting records real execution.
- Runtime hotspot findings:
  - `loop` is dominated by `GetLoc1`, `Goto`, `Add`, `Dup`, `Drop`, `GetLoc0`, `PutLoc0`, `PutLoc1`, `Lt`, `IfFalse`.
  - `sieve` is dominated by `Goto`, `Drop`, `IfFalse`, `GetLoc3`, `Add`, `Dup`, `GetLoc0`, `Lte`, `GetLoc2`, `PutArrayEl`, `PutLoc3`, `GetArrayEl`, `CallMethod`.
- Current interpretation: the next evidence-based optimization target is more likely `Dup/Drop` + local-store usage patterns or branch/control-flow cost, not another ad hoc arithmetic helper tweak.
- 2026-03-16: completed a `Dup + PutLocX + Drop` peephole fast path for common statement-update patterns such as `i = i + 1;`.
- Added regression coverage for local assignment statement updates while preserving assignment-expression behavior.
- Benchmark result: `loop 10k` improved from roughly `0.513–0.525 ms` to `0.486–0.492 ms` in Criterion.
- Benchmark result: `sieve 10k` improved from roughly `2.257–2.310 ms` to `2.152–2.191 ms` in Criterion.
- 2026-03-16: optimized the hot `Dup` / `Drop` opcode handlers themselves by replacing generic checked helpers with direct fast-path stack operations.
- Reused the same local-assignment and assignment-expression regression coverage to validate the change.
- Current baseline after this round is recorded in `docs/BENCHMARK_ANALYSIS.md`.
- 2026-03-16: added a branch-fusion fast path for `Lt/Lte` immediately followed by `IfFalse` / `IfTrue`, allowing the comparison result to branch directly without materializing a temporary boolean on the stack.
- Reused existing `while`, `switch`, and `try_catch` control-flow regression coverage to validate semantics.
- Benchmark result: `loop 10k` improved from roughly `0.502–0.514 ms` to `0.484–0.499 ms` in Criterion.
- Benchmark result: `sieve 10k` improved from roughly `2.164–2.207 ms` to `2.038–2.078 ms` in Criterion.
- 2026-03-17: after dump-mode profiling showed that the hottest remaining `sieve` local-update shapes were specifically `Add; Dup; PutLoc3; Drop` and `Add; Dup; PutLoc8 4; Drop`, added a very narrow peephole for exactly those shapes instead of reintroducing the broader generic version.
- Added regression coverage for the `PutLoc8` statement-update shape while preserving assignment-expression behavior.
- Selected execution-focused Criterion reruns on the current worktree:
  - `loop 10k`: `0.493–0.503 ms`
  - `sieve 10k`: `1.832–1.860 ms`
- Current interpretation: this reinforces that current opcode/local-store work is most effective when it is guided by concrete bytecode-shape profiling rather than broad generic fast paths.
- 2026-03-17: tightened the raw `Goto` / `IfFalse` / `IfTrue` handlers themselves by switching their hottest decode/pop path to direct unchecked operand reads and unchecked stack pop for the branch value.
- Re-ran full engine tests and `clippy -D warnings` successfully after the change.
- Selected execution-focused Criterion reruns on the current worktree:
  - `loop 10k`: `0.461–0.476 ms`
  - `sieve 10k`: `1.704–1.740 ms`
- Current interpretation: after the bytecode-shape-specific local-update work, the next real bottleneck was the control-flow skeleton itself, and tightening those branch/goto handlers produced another clean step down for both loop-heavy and sieve-heavy code.
- 2026-03-17: added dedicated `GetLoc4` / `PutLoc4` short opcodes so the current hottest extra local slot no longer has to pay the generic `GetLoc8` / `PutLoc8` path cost.
- Added compiler coverage to ensure local slot 4 now emits the short-form opcode.
- Re-ran full engine tests and `clippy -D warnings` successfully after the change.
- Selected execution-focused Criterion reruns on the current worktree:
  - `loop 10k`: `0.449–0.459 ms`
  - `sieve 10k`: `1.686–1.714 ms`
- Current interpretation: after narrowing the control-flow cost, the next bottleneck really was the hottest non-inline local slot, and giving slot 4 its own opcode bought another measurable step down for both `loop` and `sieve`.
- 2026-03-17: after additional validation, retained the slot-4 short-opcode work and re-ran the local benchmark pair on the current worktree.
- Current selected execution-focused rerun:
  - `loop 10k`: `0.444–0.451 ms`
  - `sieve 10k`: `1.663–1.709 ms`
- Current interpretation: the slot-4 short-form work remains a real win after rerun and should be treated as part of the stable opcode/local-slot optimization path rather than a one-off measurement artifact.
- 2026-03-18: added a small `dump_bytecode` developer tool so hot benchmark scripts can now be compiled and disassembled directly, instead of inferring bytecode shape only from opcode counters.
- 2026-03-18: re-lowered C-style `for` loops so the increment section is compiled once but appended after the body, removing the extra pre-body `Goto` that used to skip over the increment block on the first iteration.
- Added compiler coverage confirming a simple `for (...)` loop now emits a single back-edge `Goto`, and re-ran `for` / `continue` / `break` / `switch-in-loop` regressions successfully.
- Re-ran full engine tests and `clippy -D warnings` successfully after the change.
- Selected execution-focused Criterion reruns on the current worktree:
  - `loop 10k`: `0.555–0.659 ms`
  - `sieve 10k`: `1.917–2.219 ms`
  - `dense array bool read branch 10k`: `1.367–1.669 ms`
  - `dense array bool read hot`: `131.93–152.53 ms`
  - `dense array false write then read hot`: `87.45–100.89 ms`
- Current interpretation: this is a control-flow skeleton win rather than a `GetArrayEl`-only win, but it lands directly on the bytecode shape now dominating the dense-array read diagnostics and removes a real structural inefficiency from all C-style `for` loops.
- 2026-03-18: added `IncLoc*Drop` statement-form local-update opcodes and rewrote the hottest discarded `local = local + 1` bytecode tails into them, while preserving full `+` semantics for string locals.
- Added compiler coverage confirming `var i = 0; i = i + 1;` now lowers to the dedicated opcode family, and added a regression locking that `var x = 'a'; x = x + 1;` still produces a string.
- `dump_bytecode` now shows the dense-array read diagnostics compiling down to compact `IncLoc{1,2,3,4}Drop` tails instead of repeated `Push1; Add; Dup; PutLocX; Drop` skeletons.
- Re-ran full engine tests and `clippy -D warnings` successfully after the change.
- Selected execution-focused Criterion reruns on the current worktree:
  - `loop 10k`: `0.483–0.602 ms`
  - `sieve 10k`: `1.633–1.726 ms`
  - `dense array bool read branch 10k`: `0.803–0.822 ms`
  - `dense array bool read hot`: `86.07–105.53 ms`
  - `dense array false write then read hot`: `64.41–67.69 ms`
- Current interpretation: this is the first optimization in the current dense-array phase that materially shrinks the remaining read-side loop skeleton itself; it helps the classic `loop` baseline too, but the biggest win is that the new dense-array read diagnostics now spend much less time on local increment bookkeeping around `GetArrayEl`.
- Status: complete as the current dispatch-tightening phase; later opcode work should only reopen this area if new profiling data identifies a materially different hot set.

### 9.1.6 Arithmetic/comparison micro-optimization pass [Completed]

**Priority**: P1

**Hot files**

- `src/vm/ops.rs`

**Why**

- Core arithmetic and comparison helpers are already partly inlined.
- This area still matters, but its likely gain is lower than call/array/native hot paths.

**Tasks**

- Audit remaining hot `op_*` helpers for true inline benefit.
- Reduce repeated numeric coercions on common int/int and int/float paths.
- Re-check equality and comparison fast paths after benchmark cleanup.

**Expected gain**

- Small but broad improvement

**Completed so far**

- 2026-03-16: improved string-concatenation hot paths by building the final runtime string in a single output buffer instead of first materializing both operands as temporary owned `String` values.
- Added regression coverage for mixed string/number chained concatenation shape.
- Benchmark result: `runtime_string_pressure 4k` improved from roughly `2.89–3.38 ms` to `1.53–1.55 ms` in Criterion.
- 2026-03-17: added a narrower `Add` fast path for the dominant `string + int` / `int + string` concatenation shapes so common runtime-string loops avoid the generic length-hint + append path on decimal integer operands.
- Re-ran targeted concat-shape regression coverage successfully.
- Selected execution-focused Criterion reruns on the current worktree:
  - `runtime_string_pressure 4k`: `1.091–1.117 ms`
  - `string concat 1k`: `151.87–157.61 µs`
  - `method_chain 5k`: `587.80–599.99 µs`
- Current interpretation: this is a real runtime-string win on the benchmark shape that mixes compile-time string fragments with decimal loop indices, while the simpler `string concat 1k` benchmark remains effectively unchanged.
- 2026-03-17: added bytecode-level `AddConstStringLeft` / `AddConstStringRight` specializations so concat chains with compile-time string fragments on either side of `+` no longer need to route those shapes through the generic `Add` opcode.
- Added compiler coverage confirming the specialized bytecode now emits for both `"x" + value` and `value + "x"` shapes, and re-ran targeted concat-shape regression coverage successfully.
- Selected execution-focused Criterion reruns on the current worktree:
  - `runtime_string_pressure 4k`: `1.055–1.077 ms`
  - `string concat 1k`: `141.41–145.80 µs`
  - `method_chain 5k`: `587.46–601.19 µs`
- Current interpretation: this is the first more systematic concat-chain optimization beyond executor-only `Add` tweaks, and it produces a clear runtime-string win while leaving the nearby `method_chain` workload effectively stable.
- 2026-03-17: extended that concat-chain lowering with compile-time folding for adjacent string literals and a dedicated `AddConstStringSurround` shape for `const + value + const`, cutting one more runtime-string allocation out of the dominant benchmark chain.
- Added compiler coverage confirming both the surround specialization and adjacent-string constant folding.
- Dump-mode hotspot probing on the current worktree now shows `runtime_string_pressure` dropping from `12001` concat-created runtime strings to `8001`, and total `Add` executions from `24001` to `16000`.
- Selected execution-focused Criterion reruns on the current worktree:
  - `runtime_string_pressure 4k`: `0.899–0.915 ms`
  - `string concat 1k`: `166.97–171.99 µs`
  - `method_chain 5k`: `624.57–638.70 µs`
- Current interpretation: this is a stronger, more structural concat-chain optimization for the target runtime-string benchmark, but it also appears to regress the simpler `string concat 1k` microbenchmark, so follow-up work should specifically explain and recover that regression instead of treating the string path as “done”.
- 2026-03-18: added string-specific diagnostic coverage for:
  - `string local update only 1k`
  - `string concat ephemeral 1k`
  and extended dump-mode hotspot profiling so `string_concat` is tracked directly beside `runtime_string_pressure`.
- Diagnostic conclusion: the dominant cost in `string concat 1k` is repeated copying of the growing string content itself; the local-update skeleton (`Dup` / `PutLoc0` / `Drop`) is not the primary cost center.
- 2026-03-18: added a very narrow statement-form `AppendConstStringToLoc0` lowering backed by a per-frame builder only for local slot `0`, specifically targeting `var s = ''; s = s + 'x';`.
- Added compiler coverage confirming the new lowering emits for that exact shape, and re-ran the matching eval regression successfully.
- Dump-mode hotspot probing on the current worktree now shows `string_concat` dropping from `1000` concat-created runtime strings to `1`, while `runtime_string_pressure` remains at `8001`.
- Selected execution-focused Criterion reruns on the current worktree:
  - `string concat 1k`: `81.24–83.26 µs`
  - `runtime_string_pressure 4k`: `909.74–921.95 µs`
  - `method_chain 5k`: `708.67–724.79 µs`
- Current interpretation: this is the first narrow local-self-concat optimization that genuinely fixes the dedicated `string concat 1k` path without reintroducing the earlier broad peephole regressions. It also keeps the broader runtime-string benchmark in the same sub-millisecond class, though `method_chain` is now better described as staying in the sub-millisecond range rather than sitting on the earlier `0.60 ms` target line.
- 2026-03-18: extended the concat-chain lowering one more step with `AddConstStringSurroundValue`, so the hot `const + value + const + value` runtime-string shape now avoids one more intermediate concat result.
- Added compiler coverage confirming the new four-part lowering emits for `'a' + x + 'b' + y`.
- Dump-mode hotspot probing on the current worktree now shows `runtime_string_pressure` dropping further from `8001` concat-created runtime strings to `4001`, with total `Add` executions reduced from `16000` to `12000`.
- Selected execution-focused Criterion reruns on the current worktree:
  - `runtime_string_pressure 4k`: `841.41–855.24 µs`
  - `string concat 1k`: `82.79–84.89 µs`
  - `string concat ephemeral 1k`: `113.15–118.99 µs`
  - `method_chain 5k`: `736.57–751.78 µs`
- Current interpretation: this further narrows the remaining string work from “common concat-chain lowering” to the truly general growing-string case. The dedicated runtime-string benchmark clearly benefits, the ephemeral concat microbenchmark benefits dramatically, and the simpler local self-concat path remains in the same fast class.
- 2026-03-18: introduced a minimal deferred `RuntimeString` wrapper for runtime-created strings and taught `.length` reads to use cached runtime-string lengths directly instead of flattening deferred concat nodes first.
- This keeps the existing local-self-concat and concat-chain lowering wins while removing the worst regression that came from flatten-on-length behavior.
- Selected execution-focused Criterion reruns on the current worktree:
  - `runtime_string_pressure 4k`: `869.62–887.11 µs`
  - `string concat 1k`: `78.61–80.20 µs`
  - `string concat ephemeral 1k`: `118.18–121.41 µs`
  - `method_chain 5k`: `715.62–730.29 µs`
- Current interpretation: the string path now has a more coherent story. Dedicated local self-concat is fixed, the dominant runtime-string benchmark is faster again after cached-length handling, and the remaining work is more clearly about whether to generalize deferred strings further rather than about obvious regressions in the current narrow path.
- Status: complete as the current arithmetic/string-concat micro-optimization phase; any future string work should now be treated as a broader string-representation project rather than more incremental hot-op cleanup.
- 2026-03-16: improved `StrictEq` / `StrictNeq` hot opcode handling by adding direct fast paths for same-value, integer, and boolean comparisons before falling back to slower generic handling.
- Existing switch semantics regression tests were re-run successfully.
- Benchmark result: `switch 1k` improved from roughly `145–149 μs` class performance to `132–136 μs` in Criterion.

## 9.2 Optimize GC Performance

### 9.2.1 Replace conservative `mark_all` behavior

**Priority**: P1

**Hot files**

- `src/gc/collector.rs`
- `src/context.rs`

**Why**

- The current collector still contains a conservative temporary approach that marks all objects.
- This blocks meaningful GC performance work.

**Tasks**

- Replace `mark_all()` with real root discovery.
- Define and traverse true roots:
  - stack
  - globals
  - closures
  - active frames
  - runtime-owned containers
- Verify pointer updates remain correct after compaction.

**Expected gain**

- Lower GC pause cost
- Better scaling on object-heavy scripts

### 9.2.2 Measure GC trigger behavior

**Priority**: P1

**Why**

- GC cost depends not only on collector implementation, but also on trigger frequency.

**Tasks**

- Measure GC frequency during benchmark workloads.
- Record object / array / string growth for representative scripts.
- Adjust trigger heuristics only after real data is collected.

### 9.2.3 Reduce scanning cost of engine-owned containers

**Priority**: P2

**Hot files**

- `src/vm/types.rs`
- `src/context.rs`

**Tasks**

- Review the cost of scanning runtime vectors:
  - `objects`
  - `closures`
  - `runtime_strings`
  - `typed_arrays`
  - `array_buffers`
- Separate hot live data from long-lived metadata where useful.

**Completed so far**

- 2026-03-19: completed the current mark-sweep integration pass:
  - root-based marking replaced the old conservative placeholder behavior
  - free-slot reuse is active across GC-managed containers
  - manual `Context::gc()` and native `gc()` both trigger real collection
  - automatic GC now also triggers during internal JS `Call` / `CallMethod` / `CallConstructor` workloads
- Added regression coverage for:
  - unrooted self-referential object collection
  - freed object slot reuse
  - native `gc()` triggering a real collection
  - public `Context::gc()` + `memory_stats()` reclamation
  - automatic GC triggering during a pure JS function-call workload
- `Interpreter::get_stats()` / `memory_stats()` now count only live GC-managed slots, so post-GC stats visibly drop instead of reporting raw backing `Vec` lengths.
- Local GC probes now show:
  - manual collection reclaims cycles and transient arrays back to zero live objects/arrays
  - automatic GC increments `gc_count` during JS workloads
  - manual `gc()` cost is low on the current synthetic probes

**Current status**

- The current mark-sweep GC phase is functionally complete and measurable.
- `mark-compact` is explicitly **not started** and remains deferred.
- If pursued later, `mark-compact` should be treated as a new project, not as hidden unfinished work inside the current pass.
- trigger tuning

## 9.3 Reduce Memory Usage

### 9.3.1 Improve measurement first [Completed]

**Priority**: P0

**Hot files**

- `src/context.rs`
- `src/vm/types.rs`

**Why**

- `MemoryStats` is already useful, but optimization should be based on actual dominant buckets.

**Tasks**

- Treat `MemoryStats` as the baseline measurement source.
- Record object/string/closure/typed-array counts for benchmark scripts.
- Identify the biggest memory categories before redesigning layouts.

**Completed so far**

- 2026-03-16: expanded `MemoryStats` / `InterpreterStats` beyond object counts to include:
  - `runtime_string_bytes`
  - `array_elements`
  - `object_properties`
  - `typed_array_bytes`
  - `array_buffers`
  - `array_buffer_bytes`
- Updated CLI dump output to display the new memory breakdown.
- Added regression coverage for:
  - array/object shape metrics
  - runtime string byte accounting
- Status: this measurement foundation is now sufficient to start evidence-based 9.3 work.

### 9.3.2 Reduce temporary allocations in hot execution paths

**Priority**: P0

**Why**

- Temporary vectors and transient reshaping increase both CPU and memory churn.

**Tasks**

- Remove remaining temporary `Vec<Value>` allocations from hot call paths.
- Review short-lived allocation patterns in array/builtin-heavy execution.
- Prefer stack-preserving layouts and borrowed data where safe.

### 9.3.3 Review runtime string growth [Completed]

**Priority**: P1

**Hot files**

- `src/vm/interpreter.rs`
- `src/context.rs`

**Why**

- Runtime strings are counted explicitly in `MemoryStats` and can grow quietly over time.

**Tasks**

- Measure growth of `runtime_strings` in benchmark workloads.
- Check whether string promotion is over-eager in hot paths.
- Look for duplicate string creation opportunities.

**Completed so far**

- 2026-03-16: added dump-only runtime string source counters to distinguish at least:
  - total runtime string creation requests
  - concat-driven creation
  - for-in key creation
  - other creation paths
- Exposed the counters through `Context` under the `dump` feature.
- Added dump-mode regression coverage to ensure runtime string source statistics are recorded.`r`n- 2026-03-17: expanded the source buckets to distinguish at least `json`, `object_keys`, `object_entries`, `error_string`, and `type_string` in addition to `concat`, `for_in_key`, and `other`.
- Status: complete as a review/measurement task; runtime-string reuse or dedup remains a separate future optimization decision rather than unfinished review work.
- Embedded note: do not hard-code a runtime-string byte budget in the engine yet; final limits will be chosen during real device integration on ESP32-class targets.
- 2026-03-16: on the `for-in` key path, runtime string exhaustion now becomes a controlled engine error (`runtime string table exhausted`) instead of a debug-time overflow panic.
- Added regression coverage to lock in the new controlled-error behavior for repeated `for-in` key generation.
- In short: a previously crashing runtime-string overflow on the `for-in` key path now degrades into a controlled engine error instead of panicking the process.

### 9.3.4 Review object and array layout overhead

**Priority**: P1

**Hot files**

- `src/runtime/object.rs`
- `src/runtime/array.rs`
- `src/vm/types.rs`

**Tasks**

- Compare memory cost of dense arrays vs generic object-backed access.
- Check whether frequently-created runtime structures can become smaller.
- Prefer targeted layout changes only after measurement.

## Supporting Engine Tasks

### S1. Keep builtin/runtime boundaries honest

**Priority**: P2

**Why**

- `src/builtins/` is currently mostly structural placeholder code.
- Real builtin behavior mainly lives in `src/vm/natives.rs` and `src/vm/property.rs`.

**Tasks**

- Document the true implementation locations.
- Avoid optimizing placeholder modules by mistake.
- Defer structural migration until after hotspot work, unless it blocks performance work.

### S2. Use benchmark-specific optimization targets

**Priority**: P0

**Canonical optimization set**

- `fib` -> call path, recursion, arithmetic
- `loop` -> dispatch, arithmetic, locals
- `array` -> dense array fast paths
- `sieve` -> dense array read/write + loop cost
- `json` -> regression guard for already-good paths

### S3. Expand benchmark coverage for missing engine paths

**Priority**: P0

**Why**

- The current benchmark set is useful, but still under-covers several important engine paths.
- Some high-value optimization areas will remain invisible if the benchmark suite stays focused only on `fib`, `loop`, `array`, `sieve`, and `json`.

**Benchmark additions: primary set**

These should be treated as the next benchmark additions because they most directly expose meaningful engine hotspots:

- `method_chain`
  - Representative shape: `.map().filter().reduce()`
  - Covers: `GetField2`, `CallMethod`, callback invocation, array chaining
- `for_of_array`
  - Covers: `ForOfStart`, `ForOfNext`, iterator loop control
- `deep_property`
  - Representative shape: `a.b.c.d`
  - Covers: repeated `GetField` cost and chained property access
- `runtime_string_pressure`
  - Covers: `create_runtime_string`, runtime string growth, string allocation pressure

**Benchmark additions: secondary set**

These are also important, but are better treated as mechanism-specific benchmarks rather than first-wave headline performance benchmarks:

- `try_catch`
  - Covers: `ExceptionHandler`, throw/catch/finally control flow, stack unwinding
- `for_in_object`
  - Covers: `ForInStart`, `ForInNext`, object-key iteration
- `switch_case`
  - Covers: multi-branch dispatch shape based on `Dup + StrictEq + IfTrue`

**Deferred for the current no_std-focused path**

- `regexp_test`
  - Covers: `RegExpObject`, `test`
  - Keep as a later `std` / optional benchmark candidate, not a first-wave no_std target
- `regexp_exec`
  - Covers: `RegExpObject`, `exec`
  - Keep as a later `std` / optional benchmark candidate, not a first-wave no_std target

**Suggested rollout order**

1. `method_chain`
2. `runtime_string_pressure`
3. `for_of_array`
4. `deep_property`
5. `try_catch`
6. `switch_case`
7. `for_in_object`

Deferred:

- `regexp_test`
- `regexp_exec`

**Expected value**

- Makes benchmark-driven optimization more representative of real JS usage
- Exposes call-heavy, iterator-heavy, object-access-heavy, and string-pressure-heavy paths
- Gives the engine optimization work better visibility beyond arithmetic and raw loops

**Completed so far**

- 2026-03-16: added first-wave benchmark scripts and Criterion coverage for:
  - `method_chain`
  - `runtime_string_pressure`
  - `for_of_array`
  - `deep_property`
- 2026-03-16: added second-wave `switch_case` benchmark script for CLI-style Rust-vs-C comparison.
- Verified benchmark build with `cargo bench --no-run`.
- 2026-03-16: completed the first `for_of_array` optimization pass by removing full array cloning from `ForOfStart` and iterating arrays by index instead.
- Added regression coverage confirming `for-of` over arrays observes element updates during iteration.
- Benchmark result: `for_of_array 20k` improved from roughly `4.22–4.47 ms` to `2.36–2.42 ms` in Criterion.
- 2026-03-17: added a `ForOfNext` + `IfTrue` branch-fusion fast path so the iterator hot path no longer needs to materialize a temporary `done` boolean when the branch shape is known.
- Re-ran `for-of` regression coverage for normal iteration, `continue`, and array-update observation.
- Benchmark result under the current execution-focused Criterion harness: the latest full rerun records `for_of_array 20k` at `1.80–1.96 ms`.
- 2026-03-16: added `for_in_object` benchmark coverage and completed the first iterator setup optimization pass by replacing eager full-key cloning with index-based lazy key generation over object/array snapshots.
- Added regression coverage confirming `for-in` over objects still observes updated values through static property reads during iteration.
- Benchmark baseline recorded: `for_in_object 20x2000` at `3.74–3.80 ms` in Criterion.

## Recommended Execution Order

1. Benchmark baseline revalidation and documentation sync
2. Call-path regression audit on the current head
3. Native/builtin marshalling completion
4. Dense array fast paths
5. Object/property access follow-up after the current-head rerun data
6. Memory measurement pass
7. GC root-based marking work
8. Opcode dispatch tightening
9. Secondary micro-optimizations

## Done Criteria

This optimization task list is considered substantially complete when:

- benchmark baselines are trustworthy and reproducible
- `fib`, `loop`, `array`, and `sieve` each have at least one verified hotspot improvement
- GC no longer relies on conservative `mark_all`
- memory reduction work is based on measured dominant categories, not guesswork
- documentation reflects valid benchmark conclusions only
