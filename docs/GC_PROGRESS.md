# GC Implementation Progress

This document tracks the implementation of Plan B: mark-sweep GC for `mquickjs-rs`.

Start date: 2026-03-19
Strategy: generation-based mark-sweep on the existing `Vec`-index architecture
Upstream: [docs/ENGINE_OPTIMIZATION_TASKLIST.md](ENGINE_OPTIMIZATION_TASKLIST.md), section `9.2`

## Acceptance Criteria

- [x] `cargo test -p mquickjs-rs`
- [x] `cargo test -p led-runtime`
- [x] `cargo clippy -p mquickjs-rs --all-targets -- -D warnings`
- [x] `cargo build --release --no-default-features`
- [x] Benchmark: memory usage visibly decreases after GC vs before
- [x] `gc_count` increments when GC runs
- [x] Cycle reference `{ let a = {}; a.self = a; }` can be collected
- [x] Freed slots are reused instead of leaking

## Core Design

Generation-based marking with no pointer changes:

- `gc_phase: u32` increments on each collection
- `gen[i] == gc_phase` means slot `i` is live
- `gen[i] == u32::MAX` means slot `i` is free

GC-managed containers:

- `closures`
- `var_cells`
- `arrays`
- `objects`
- `for_in_iterators`
- `for_of_iterators`
- `error_objects`
- `regex_objects`
- `typed_arrays`
- `array_buffers`
- `timers`

Roots:

- value stack / active call frames
- `global_vars`
- closures through captured `var_cells`
- `timers.callback`

## Implementation Phases

### Phase 1: GC Infrastructure

- [x] `src/vm/gc.rs` added
- [x] `GcState` with phase, trigger, allocation count, and sweep stats
- [x] `gc_mark_value()` recursive traversal for engine `Value` graphs
- [x] `alloc_slot()` free-slot reuse
- [x] `sweep_container()` dead-slot reclamation
- [x] adaptive trigger adjustment

### Phase 2: Interpreter Integration

- [x] Added generation arrays to `Interpreter`
- [x] Added `gc: GcState` to `Interpreter`
- [x] Routed all managed container allocations through GC slot reuse
- [x] Implemented `gc_mark_roots()`
- [x] Implemented `gc_sweep()`
- [x] Implemented `gc_collect()`
- [x] Implemented `maybe_gc()`

### Phase 3: Context Integration

- [x] `Interpreter::execute()` calls `maybe_gc()`
- [x] `Interpreter::call_function()` calls `maybe_gc()`
- [x] `Context::gc()` manually triggers collection
- [x] native `gc()` now triggers `gc_collect()` instead of only bumping a counter
- [x] internal JS `Call` / `CallMethod` / `CallConstructor` paths now also call `maybe_gc()`

### Phase 4: Testing

- [x] `cargo test -p mquickjs-rs` passes
- [x] `cargo test -p led-runtime` passes
- [x] `cargo clippy -p mquickjs-rs --all-targets -- -D warnings` passes
- [x] `cargo build --release --no-default-features` passes
- [x] Added interpreter-level regression coverage for unrooted self-referential object collection
- [x] Added interpreter-level regression coverage for freed object slot reuse
- [x] Added interpreter-level regression coverage for native `gc()` triggering a real collection
- [x] Added public `Context::gc()` + `memory_stats()` regression coverage for unrooted cycle collection
- [x] Added public regression coverage that automatic GC triggers during a JS function-call workload

### Phase 4.1: Stats Correctness

- [x] `Interpreter::get_stats()` now reports only live GC-managed slots
- [x] array/object/typed-array byte and element counts ignore `SLOT_FREE` entries
- [x] `memory_stats()` now reflects post-GC reclamation instead of raw backing `Vec` lengths

### Phase 5: Benchmarking

- [x] Compare memory usage before and after GC
- [ ] Measure runtime overhead / trigger behavior
- [ ] Decide whether mark-compact is still needed

Current probe:

- `src/bin/gc_memory_probe.rs`

Current `cargo run --bin gc_memory_probe` snapshot:

- `object_cycles`
  - `gc_count`: `0 -> 0 -> 1`
  - `objects`: `0 -> 200 -> 0`
  - `estimated_object_bytes`: `0 -> 9600 -> 0`
- `transient_arrays`
  - `gc_count`: `0 -> 0 -> 1`
  - `arrays`: `0 -> 400 -> 0`
  - `estimated_object_bytes`: `0 -> 12800 -> 0`
- `auto_gc_cycles`
  - `gc_count`: `0 -> 1 -> 2`
  - `objects`: `0 -> 1500 -> 0`
  - `estimated_object_bytes`: `0 -> 22992 -> 0`

Interpretation:

- Manual `gc()` now visibly reclaims unreachable self-referential objects.
- Freed array/object slots disappear from public `memory_stats()` after collection.
- Automatic GC now also triggers during pure JS function-call workloads instead of only through manual `gc()`.
- The remaining open GC work is no longer “does reclamation happen at all?”, but “what is the runtime overhead / trigger behavior under benchmark workloads?”.

## Notes

### Phase Counter Overflow

`u32::MAX` is reserved as `SLOT_FREE`. If `gc_phase` approaches that value, generation arrays must be reset before reuse.

### Thread Safety

The GC is intentionally single-threaded. This matches the current JS engine execution model.

### no_std Compatibility

The GC implementation uses `alloc` only and remains compatible with the engine's `no_std` default.
