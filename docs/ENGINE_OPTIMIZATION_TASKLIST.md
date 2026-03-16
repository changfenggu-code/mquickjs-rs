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
- Status: this task is considered complete for the current engine optimization phase.

### 9.1.2 Call-path hot path optimization

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

### 9.1.5 Opcode dispatch tightening

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
- Benchmark result: `try_catch 5k` baseline recorded at `340–349 µs` in Criterion.
- 2026-03-16: added feature-gated runtime opcode counters under the `dump` feature and exposed them through `Context` for profiling work.
- Added a `dump`-mode regression test to ensure opcode counting records real execution.
- Runtime hotspot findings:
  - `loop` is dominated by `GetLoc1`, `Goto`, `Add`, `Dup`, `Drop`, `GetLoc0`, `PutLoc0`, `PutLoc1`, `Lt`, `IfFalse`.
  - `sieve` is dominated by `Goto`, `Drop`, `IfFalse`, `GetLoc3`, `Add`, `Dup`, `GetLoc0`, `Lte`, `GetLoc2`, `PutArrayEl`, `PutLoc3`, `GetArrayEl`, `CallMethod`.
- Current interpretation: the next evidence-based optimization target is more likely `Dup/Drop` + local-store usage patterns or branch/control-flow cost, not another ad hoc arithmetic helper tweak.

### 9.1.6 Arithmetic/comparison micro-optimization pass

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
- 2026-03-16: improved `StrictEq` / `StrictNeq` hot opcode handling by adding direct fast paths for same-value, integer, and boolean comparisons before falling back to slower generic handling.
- Existing switch semantics regression tests were re-run successfully.
- Benchmark result: `switch 1k` improved from roughly `145–149 µs` class performance to `132–136 µs` in Criterion.

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

## 9.3 Reduce Memory Usage

### 9.3.1 Improve measurement first

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

### 9.3.2 Reduce temporary allocations in hot execution paths

**Priority**: P0

**Why**

- Temporary vectors and transient reshaping increase both CPU and memory churn.

**Tasks**

- Remove remaining temporary `Vec<Value>` allocations from hot call paths.
- Review short-lived allocation patterns in array/builtin-heavy execution.
- Prefer stack-preserving layouts and borrowed data where safe.

### 9.3.3 Review runtime string growth

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
- 2026-03-16: added `for_in_object` benchmark coverage and completed the first iterator setup optimization pass by replacing eager full-key cloning with index-based lazy key generation over object/array snapshots.
- Added regression coverage confirming `for-in` over objects still observes updated values through static property reads during iteration.
- Benchmark baseline recorded: `for_in_object 20x2000` at `3.74–3.80 ms` in Criterion.

## Recommended Execution Order

1. Benchmark baseline cleanup
2. Benchmark coverage expansion (`method_chain`, `runtime_string_pressure`, `for_of_array`, `deep_property` first)
3. Call-path optimization
4. Native/builtin marshalling optimization
5. Dense array fast paths
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
