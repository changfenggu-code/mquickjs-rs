# Benchmark Scripts

This directory is the shared source for benchmark JS workloads.

## Current conventions

- Scripts referenced by `benches/drivers/criterion_benchmarks.rs` are the Criterion-side source of truth.
- Canonical optimization canaries are additionally listed in `../manifests/canary_benchmarks.txt`.
- Tier membership is additionally tracked in `../manifests/benchmark_tiers.txt` so local tooling can consume the same grouping without scraping this README.
- Scripts used only for broader comparison or legacy coverage may still exist alongside Criterion workloads when they represent a meaningfully different benchmark shape.
- Benchmark scripts should avoid `print(...)` and other reporting-oriented output so end-to-end, Criterion, and execution-only flows all measure the workload itself rather than CLI formatting noise.

## Naming guidance

- Use descriptive names that match the benchmark intent, such as `method_chain.js` or `deep_property.js`.
- Use size/count suffixes like `_10k` or `_1k` when that size is part of the workload identity.
- Avoid adding a second script file when an existing script already expresses the exact same workload.

## Cleanup rule

- If two files are semantically identical, keep the more established/shared name and remove the duplicate.
- If two files sound similar but benchmark different shapes or scales, keep both and let the filename communicate that difference.

## Current tiers

The human-readable summary below mirrors `../manifests/benchmark_tiers.txt`. If the tier set changes, update both in the same turn.

### Tier 0: Canonical canaries

These are the default optimization sentinels and are also listed in `../manifests/canary_benchmarks.txt`.

- `method_chain.js`
- `runtime_string_pressure.js`
- `for_of_array.js`
- `deep_property.js`

### Tier 1: Primary headline benchmarks

These are the next set to look at after the canaries when evaluating broader engine performance.

- `fib_iter.js`
- `loop_10k.js`
- `array_push_10k.js`
- `object_create_1k.js`
- `closure_1k.js`
- `string_concat_1k.js`
- `json_parse_property_read.js`
- `json_parse_only.js`
- `sieve_10k.js`

### Tier 2: Secondary behavioral benchmarks

Useful for control-flow, builtin-call, allocation-pressure, and narrower string/runtime checks after Tier 0/1.

- `string_concat_local_update_only.js`
- `string_concat_ephemeral.js`
- `try_catch.js`
- `for_in_object.js`
- `switch_1k.js`
- `do_while_10k.js`
- `recursion_100x100.js`
- `math_max_3arg_10k.js`
- `gc_transient_arrays_2500.js`

### Tier 3: Dense-array diagnostic probes

These are mechanism probes for array read/write/branch behavior. They are primarily for targeted hotspot diagnosis rather than broad release-level comparison.

- `dense_array_bool_read_branch_10k.js`
- `dense_array_false_write_only_10k.js`
- `dense_array_bool_read_hot.js`
- `dense_array_bool_condition_only_hot.js`
- `dense_array_bool_condition_only_hot_arg0.js`
- `dense_array_bool_condition_only_hot_local1.js`
- `dense_array_read_only_hot.js`
- `dense_array_read_only_hot_arg0.js`
- `dense_array_read_only_hot_local1.js`
- `dense_array_loop_only_hot.js`
- `dense_array_false_write_then_read_hot.js`

### Tier 4: Legacy broad-compare workloads

These are older broader scripts that still have value for manual comparison, but they are not the preferred source for Criterion-side hotspot validation anymore.

`json.js` has been retired because `json_parse_only.js` and `json_parse_property_read.js` now cover that line in a clearer, non-overlapping way. The remaining legacy scripts stay only where they still represent a broader combined workload shape than the decomposed Criterion set.

- `fib.js`
- `loop.js`
- `array.js`
- `object.js`
- `closure.js`
- `string.js`
- `sieve.js`
- `switch_case.js`

## Tooling note

- `./benches/compare.sh` still defaults to the canonical canary flow.
- `./benches/compare.sh --tier N` runs one tier from `../manifests/benchmark_tiers.txt`, which is useful for focused cleanup passes such as Tier 3 diagnostics or Tier 4 retirement review.
- `./benches/compare.sh --execution-only` runs a compile-once / execute-many Rust-vs-C comparison, which is the closest cross-implementation match to the Criterion-side runtime focus.
