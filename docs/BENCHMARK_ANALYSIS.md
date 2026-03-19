# Benchmark Analysis

Chinese version: `docs/BENCHMARK_ANALYSIS.zh.md`

Related optimization backlogs:
- `docs/ENGINE_OPTIMIZATION_TASKLIST.md`
- `docs/ENGINE_OPTIMIZATION_TASKLIST.zh.md`

## Purpose

This document defines the current benchmark baseline and interpretation rules for the
`mquickjs-rs` engine.

It is engine-only documentation and does not cover `led-runtime` product-layer behavior.

## Baseline Policy

Benchmark analysis uses three complementary sources.

### 1. Local Criterion (`cargo bench`)

Primary use:

- precise Rust-only execution-phase timing analysis
- before/after optimization validation
- hotspot confirmation

This is the preferred source when deciding whether an engine optimization actually helped.

As of March 17, 2026, the Criterion harness compiles each benchmark script once and then
measures execution on fresh contexts. This intentionally reduces parser/compiler noise in
runtime optimization work.

### 2. Local Rust-vs-C comparison (`benches/compare.sh` or equivalent local comparison)

Primary use:

- compare current Rust engine performance against the C `mqjs` implementation
- estimate how far a path still is from the C implementation

This is the preferred source when evaluating cross-implementation distance.

### 3. CI benchmark summary (`.github/workflows/bench.yml`)

Primary use:

- trend tracking in GitHub Actions
- quick regression visibility on pushes and PRs
- GitHub-visible summary tables

The CI summary now provides:

- a Rust-vs-C comparison table
- a Rust-only Criterion table
- a startup baseline row

CI results are useful and visible on GitHub, but local Criterion and local Rust-vs-C
comparison remain the main sources for detailed investigation.

## Canonical Benchmark Set

The current engine baseline set is:

- legacy core set:
  - `fib`
  - `loop`
  - `array`
  - `json`
  - `sieve`
- first-wave expansion set:
  - `method_chain`
  - `runtime_string_pressure`
  - `for_of_array`
  - `deep_property`
- current secondary tracked benchmark:
  - `switch_case`

Deferred for the current `no_std`-first path:

- `regexp_test`
- `regexp_exec`

Future secondary additions:

- `switch_case`
- `for_in_object`

Currently tracked secondary benchmarks:

- `switch_case`
- `try_catch`
- `for_in_object`

## Current Baseline (Local Criterion, Rust-only)

Important current status on March 17, 2026:

- the Criterion harness was updated to compile once and benchmark execution on fresh contexts
- a full local Criterion revalidation was run for the primary benchmark set on the current worktree
- several values previously recorded on March 16, 2026 no longer match the current head
- older March 16 Criterion numbers are therefore not directly comparable to the current harness
- benchmark baseline cleanup should therefore be treated as active work again, not as closed work

### Revalidated Current-Worktree Snapshot (2026-03-17)

These values come from a fresh local rerun on the current worktree using the updated
compile-once execution-focused Criterion harness and should be treated as the current trusted
snapshot for the primary benchmark set.

| Benchmark | Current local snapshot |
|-----------|------------------------|
| `fib_iter 1k` | `2.330–2.379 ms` |
| `loop 10k` | `0.472–0.485 ms` |
| `array push 10k` | `0.491–0.502 ms` |
| `json parse 1k` | `0.736–0.754 ms` |
| `sieve 10k` | `2.069–2.103 ms` |
| `method_chain 5k` | `0.585–0.600 ms` |
| `runtime_string_pressure 4k` | `0.899–0.915 ms` |
| `for_of_array 20k` | `1.796–1.959 ms` |
| `deep_property 200k` | `14.925–15.235 ms` |

### Secondary Tracked Benchmarks (last recorded snapshot, not rerun on 2026-03-17)

These values are still useful for context, but they have not yet been revalidated against the
current worktree under the updated compile-once Criterion harness on March 17, 2026.

| Benchmark | Last recorded snapshot |
|-----------|------------------------|
| `switch 1k` | `0.132–0.136 ms` |
| `try_catch 5k` | `0.341–0.349 ms` |
| `for_in_object 20x2000` | `3.743–3.804 ms` |

## Current Baseline (Local Rust vs C, process-inclusive)

These values are based on local repeated process execution averages and are useful for
cross-implementation comparison. They include process startup cost.

### Startup baseline

| Case | Rust | C | Ratio |
|------|------|---|-------|
| `mqjs -e "0"` | `18.130 ms` | `17.407 ms` | `1.042x` |

### Script comparisons

| Benchmark | Rust | C | Ratio | Notes |
|-----------|------|---|-------|-------|
| `fib` | `183.099 ms` | `118.815 ms` | `1.541x` | C faster |
| `loop` | `127.598 ms` | `86.453 ms` | `1.476x` | C faster |
| `array` | `17.673 ms` | `16.467 ms` | `1.073x` | C slightly faster |
| `json` | `44.048 ms` | `64.280 ms` | `0.685x` | Rust faster |
| `sieve` | `37.343 ms` | `27.791 ms` | `1.344x` | C faster |
| `method_chain` | `15.795 ms` | `14.109 ms` | `1.119x` | C slightly faster |
| `runtime_string_pressure` | `22.470 ms` | `19.185 ms` | `1.171x` | C faster |
| `for_of_array` | `19.509 ms` | `17.358 ms` | `1.124x` | C faster |
| `deep_property` | `32.465 ms` | `24.001 ms` | `1.353x` | C faster |
| `switch_case` | `18.495 ms` | `16.856 ms` | `1.097x` | C slightly faster |
| `try_catch` | `16.097 ms` | `14.508 ms` | `1.110x` | C slightly faster |
| `for_in_object` | `22.356 ms` | `17.633 ms` | `1.268x` | C faster |

## Interpretation Notes

### Strongest current signals

- `json` remains a relative strength for the Rust engine.
- the updated execution-focused rerun still shows `fib` and `loop` as useful call-path and dispatch signals.
- `array` and `sieve` remain meaningful dense-array and builtin-call signals.
- `runtime_string_pressure` and `method_chain` both remain useful runtime-focused targets. The latest string-specific reruns put `runtime_string_pressure` in the `0.87–0.89 ms` class and `method_chain` in the `0.72–0.73 ms` class.
- A newer narrow statement-form local-self-concat lowering (`AppendConstStringToLoc0`) substantially improves the dedicated `string concat 1k` microbenchmark and reduces its runtime-string creation count to `1`, while leaving the broader `runtime_string_pressure` benchmark in the same sub-millisecond class.
- The new string-specific diagnostics indicate that repeated copying of the growing string content is still the dominant remaining cost in generic concat loops; the local-update skeleton itself is no longer the main bottleneck for the `string concat 1k` shape.
- For the current optimization phase, the practical string-path goal has been met: the dedicated `string concat 1k` path is fixed, `runtime_string_pressure` stays in the sub-millisecond class, and `method_chain` is not being re-broken by string work. Any further effort here should be treated as a broader string-representation project.
- `for_of_array` improved materially again after a `ForOfNext` branch-fusion pass and now looks much healthier, though it remains a useful iterator/control-flow target.
- `deep_property` remains a strong object-property benchmark and currently looks healthier than the iterator/string-pressure group.
- `switch_case` is now tracked as a secondary control-flow benchmark and already shows measurable improvement from the latest `StrictEq` hot-path tuning.
- `try_catch` is now tracked as a secondary exception-control benchmark and has a recorded post-cleanup baseline.
- `for_in_object` is now tracked as a secondary iterator/control-flow benchmark with a first recorded baseline after iterator setup cleanup.

### Important caution

- Criterion numbers and process-inclusive Rust-vs-C numbers answer different questions.
- Short-running script comparisons can be more sensitive to startup overhead.
- Use Criterion first when validating an optimization inside the Rust engine.

## Already Completed Optimization Work Reflected In The Codebase

The current codebase still contains the first completed optimization rounds for:

- `deep_property`
  - small-object property lookup fast path
  - unified `GetField` / `GetField2` property dispatch
- `method_chain`
  - removed per-element temporary `Vec<Value>` allocation in array higher-order builtins
  - added `CallMethod` native small-argument fast path
  - added direct `Array.prototype.push` native fast path with an `argc == 1` shortcut
  - changed array `.push` property lookup to use the cached native index directly
  - replaced full-array cloning in higher-order array builtins with length-snapshot iteration plus live element reads
  - added dedicated `CallArrayMap1` / `CallArrayFilter1` / `CallArrayReduce2` opcodes
  - added dedicated `CallArrayPush1` for the dominant single-argument array-construction shape
- `runtime_string_pressure`
  - builds concat results in a single output buffer instead of materializing both operands separately
  - adds a narrower `string + int` / `int + string` fast path for decimal loop-index concatenation
  - adds bytecode-level `AddConstStringLeft` / `AddConstStringRight` specializations for compile-time string fragments in concat chains
  - folds adjacent compile-time string literals and lowers `const + value + const` into a dedicated `AddConstStringSurround` shape
  - adds `AppendConstStringToLoc0` plus a per-frame builder for the hottest statement-form local self-concat loop
  - adds `AddConstStringSurroundValue` for the common `const + value + const + value` concat-chain shape
  - adds a minimal deferred `RuntimeString` wrapper and serves `.length` from cached runtime-string lengths without forced flattening
- `for_of_array`
  - removed full array cloning from `ForOfStart`
  - added `ForOfNext` branch fusion for the common `IfTrue` loop-exit shape
- `loop` / `sieve`
  - added a `Dup + PutLocX + Drop` peephole fast path for common statement-update patterns
  - added `Lt/Lte` + `IfFalse/IfTrue` branch fusion

These changes are tracked in:

- `docs/ENGINE_OPTIMIZATION_TASKLIST.md`
- `docs/ENGINE_OPTIMIZATION_TASKLIST.zh.md`

Important current interpretation:

- these implementation changes are real and still present in the code
- the benchmark harness itself changed on March 17, 2026, so older Criterion numbers and current Criterion numbers must be treated as different measurement generations
- treat the table above as the current trusted execution-focused snapshot

## What "9.1.1 Benchmark Baseline Cleanup" Means Now

This baseline cleanup task is considered complete when:

- benchmark roles are clearly separated:
  - local Criterion -> precise Rust-only analysis
  - local Rust-vs-C comparison -> cross-implementation comparison
  - CI summary -> trend tracking + GitHub-visible analysis table
- the canonical benchmark set is defined
- the current baseline tables are recorded in one place
- future optimization rounds can use this document as the baseline reference

On March 17, 2026, this task had to be treated as reopened because both the recorded baseline
and the benchmark method itself changed. This document is therefore both the current baseline
reference and the place where that methodology change is now recorded explicitly.

## Runtime String Source Findings

Current dump-mode probing shows:

- `runtime_string_pressure` is almost entirely concat-driven.
- `for_in_object` is dominated by repeated `for-in` key creation requests.
- deep_property produces essentially no runtime strings.
- json_parse currently lands mostly in the generic other bucket.


Embedded note:
- Runtime string budget limits are intentionally deferred to later device integration work instead of being hard-coded into the engine at this stage.


Updated 9.3.3 findings:
- `runtime_string_pressure` is still entirely concat-driven.
- `for_in_object` still exhausts the runtime string table on the current no-reuse path, but now fails as a controlled engine error instead of panicking.
- object_keys is now confirmed as a distinct runtime string source bucket.
- json_parse still lands in the generic other bucket and should be split further if we continue this line.
