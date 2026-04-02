# Benchmark Analysis

Chinese version: `docs/BENCHMARK_ANALYSIS.zh.md`

Related optimization backlog:
- `docs/ENGINE_OPTIMIZATION_TASKLIST.md`
- `docs/ENGINE_OPTIMIZATION_TASKLIST.zh.md`

## Purpose

This document defines the current benchmark workflow and interpretation rules for the
`mquickjs-rs` engine.

It is engine-only documentation and does not cover `led-runtime` product-layer behavior.

## Canonical Canary Set

Phase 1 establishes one canonical canary set for optimization work:

- `method_chain`
- `runtime_string_pressure`
- `for_of_array`
- `deep_property`

The machine-readable source of truth is:

- `benches/manifests/canary_benchmarks.txt`

That manifest is consumed by the local helper and the CI comparison workflow. Do not
create a second hardcoded canary list in scripts or docs.

## Benchmark Roles

Benchmark analysis uses four complementary flows.

### 1. Fast local canary rerun

Command:

```bash
bash benches/run_canaries.sh
```

Purpose:

- rerun only the four optimization canaries
- validate hotspot changes quickly
- avoid paying for the entire benchmark corpus during normal iteration

Useful helper mode:

```bash
bash benches/run_canaries.sh --list
```

This prints the canonical canary manifest entries without running benchmarks.

### 2. Canonical local Rust-vs-C comparison

Command:

```bash
bash benches/compare.sh
```

Purpose:

- compare the Rust engine against the checked-in C reference implementation
- measure cross-implementation distance on the same canary set used by CI
- keep startup, loading, compile, and execution costs in one end-to-end number

Important:

- the canonical comparison expects the reference tree under `contrib/mquickjs/`
- if that tree is unavailable, the canonical mode should fail loudly rather than silently degrade
- explicit non-canonical local modes such as Rust-only are fine for diagnosis, but they are not the Phase 1 baseline contract

### 3. Local execution-only Rust-vs-C comparison

Command:

```bash
bash benches/compare.sh --execution-only
```

Purpose:

- compare Rust and C under a compile-once / execute-many metric
- align the cross-implementation comparison more closely with the Criterion runtime focus
- make it easier to see whether a Rust-vs-C gap is mostly in end-to-end setup or in steady-state execution

Important:

- this mode is a local diagnostic tool; it is not yet the canonical CI contract
- the helper binaries still create fresh engine contexts per iteration, so the comparison is runtime-focused rather than a single-context microbenchmark

### 4. CI benchmark summary

Workflow:

- `.github/workflows/bench.yml`

Purpose:

- publish a canonical Rust-vs-C canary table in GitHub Actions
- publish a separate Criterion table for Rust-only analysis
- make pushes and PRs visibly comparable without changing the local workflow definition

The CI comparison table should be treated as the same canary contract as the local comparison flow, not a separate benchmark list.

## Reference Tree Assumption

The checked-in C reference tree lives under:

- `contrib/mquickjs/`

Canonical benchmark tooling should look there and nowhere else for the baseline Rust-vs-C flow.

## Interpretation Rules

### The three timing views answer different questions

- Criterion answers: "Did this Rust-side change help the targeted hotspot?"
- local/CI Rust-vs-C comparison answers: "How far is the real end-to-end path from the C reference implementation?"
- execution-only Rust-vs-C comparison answers: "How much of that distance remains after compile/setup noise is reduced?"

Do not collapse those into one blended number.

### How to read end-to-end vs execution-only

- if end-to-end looks bad but execution-only is healthy, the current gap is likely in startup, file loading, parse, compile, or bytecode materialization
- if both end-to-end and execution-only lag, the gap is more likely in steady-state runtime execution
- if Criterion improves but execution-only Rust-vs-C does not, the optimization may be real on the Rust side while still leaving a larger structural gap to the C engine

### Canary wins do not replace broad validation

The canary set is the fastest signal for hotspot work, but it is not the whole performance story.

After a meaningful optimization round:

1. rerun the relevant canaries first
2. check targeted regression tests
3. use broader benchmark coverage only when the local canary results justify it

### The canaries are intentionally mechanism-oriented

- `method_chain` stresses callback-heavy array pipelines and call/builtin overhead
- `runtime_string_pressure` stresses runtime-created string behavior
- `for_of_array` stresses iterator and array iteration behavior
- `deep_property` stresses repeated property access paths

This makes them useful as the default smoke test for optimization work in VM, property, builtin, and runtime-string code.

## Maintainer Workflow

When changing benchmark-sensitive engine code:

1. Run the relevant targeted tests.
2. Rerun the canonical canaries with `bash benches/run_canaries.sh`.
3. If the change is intended to improve Rust-vs-C distance, run `bash benches/compare.sh`.
4. If the change is intended to improve steady-state runtime distance, also run `bash benches/compare.sh --execution-only`.
5. Update this document if the benchmark workflow or interpretation rules change.

## Notes

- This document is intentionally focused on workflow and interpretation discipline.
- Historical benchmark snapshots can still be useful, but they should not override the current canonical process defined above.

