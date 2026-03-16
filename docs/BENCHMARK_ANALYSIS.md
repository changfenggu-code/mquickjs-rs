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

- precise Rust-only timing analysis
- before/after optimization validation
- hotspot confirmation

This is the preferred source when deciding whether an engine optimization actually helped.

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

The following values are the current local Criterion baseline after the first benchmark
expansion pass and initial optimization rounds.

| Benchmark | Current Rust-only baseline |
|-----------|----------------------------|
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
| `loop` | `94.261 ms` | `62.165 ms` | `1.516x` | C faster |
| `array` | `17.673 ms` | `16.467 ms` | `1.073x` | C slightly faster |
| `json` | `44.048 ms` | `64.280 ms` | `0.685x` | Rust faster |
| `sieve` | `52.476 ms` | `35.676 ms` | `1.471x` | C faster |
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
- `fib` and `loop` still point to call-path and dispatch cost.
- `array` and `sieve` still point to dense array access/write cost.
- `runtime_string_pressure` remains a meaningful memory/string creation path.
- `for_of_array` and `deep_property` are now part of the canonical baseline and should be
  treated as first-class optimization targets.
- `switch_case` is now tracked as a secondary control-flow benchmark and already shows measurable improvement from the latest `StrictEq` hot-path tuning.
- `try_catch` is now tracked as a secondary exception-control benchmark and has a recorded post-cleanup baseline.
- `for_in_object` is now tracked as a secondary iterator/control-flow benchmark with a first recorded baseline after iterator setup cleanup.

### Important caution

- Criterion numbers and process-inclusive Rust-vs-C numbers answer different questions.
- Short-running script comparisons can be more sensitive to startup overhead.
- Use Criterion first when validating an optimization inside the Rust engine.

## Already Completed Optimization Work Reflected In This Baseline

The current baseline already includes the first completed optimization rounds for:

- `deep_property`
  - small-object property lookup fast path
  - unified `GetField` / `GetField2` property dispatch
- `method_chain`
  - removed per-element temporary `Vec<Value>` allocation in array higher-order builtins
  - added `CallMethod` native small-argument fast path
- `for_of_array`
  - removed full array cloning from `ForOfStart`

These changes are tracked in:

- `docs/ENGINE_OPTIMIZATION_TASKLIST.md`
- `docs/ENGINE_OPTIMIZATION_TASKLIST.zh.md`

## What “9.1.1 Benchmark Baseline Cleanup” Means Now

This baseline cleanup task is considered complete when:

- benchmark roles are clearly separated:
  - local Criterion -> precise Rust-only analysis
  - local Rust-vs-C comparison -> cross-implementation comparison
  - CI summary -> trend tracking + GitHub-visible analysis table
- the canonical benchmark set is defined
- the current baseline tables are recorded in one place
- future optimization rounds can use this document as the baseline reference

This document is the current baseline reference.
