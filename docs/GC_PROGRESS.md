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
- [x] `gc_mark_roots_iterative()` iterative traversal using heap-allocated worklist (no call-stack overflow)
- [x] `mark_slot_*()` per-type helpers with deduplication check (`gen[idx] != phase` before write)
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
- [x] Measure runtime overhead / trigger behavior
- [x] Decide whether mark-compact is still needed

Current probe:

- `src/bin/gc_memory_probe.rs`
- `src/bin/gc_overhead_probe.rs`

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

Current `cargo run --bin gc_overhead_probe` snapshot:

- `manual_gc_object_cycles`
  - `eval_ms`: `0.979`
  - `gc_ms`: `0.017`
  - `gc_count`: `0 -> 0 -> 1`
- `auto_gc_object_cycles`
  - `eval_ms`: `6.934`
  - `gc_ms`: `0.006`
  - `gc_count`: `0 -> 1 -> 2`
- `auto_gc_transient_arrays`
  - `eval_ms`: `12.324`
  - `gc_ms`: `0.006`
  - `gc_count`: `0 -> 4 -> 5`

Interpretation update:

- GC trigger behavior is now observable and repeatable from local probes.
- Automatic GC is active during internal JS call workloads, not only during top-level entrypoints.
- Manual `gc()` itself is currently very cheap on these small synthetic workloads.
- The remaining open question is no longer “is trigger behavior measurable?”, but whether current trigger heuristics are already good enough for real workloads or need tuning.

### Phase 6: Future Full GC (Plan C) Reference

The `src/gc/` directory is preserved as the starting point for a future full
mark-compact GC implementation (Plan C). The current active GC is Plan B
(generation-based mark-sweep) in `src/vm/gc.rs`.

#### What `src/gc/` provides today

| File | Status | Purpose |
|------|--------|---------|
| `allocator.rs` | **In use** | Raw arena memory layout (`Heap`, `BlockHeader`, `MemoryTag`). Used by `Context` for memory statistics (`heap_used`, `total_size`, `free_space`). Plan C arena reuses this. |
| `collector.rs` | **Stub only** | `collect()` always calls `mark_all()` (conservative). `mark_object()` and pointer-update logic are TODO. |
| `GcRef` | **Never instantiated** | Designed for forwarding-pointer updates after compaction, but never used. |
| `mod.rs` | **Partially correct** | Module-level `Heap::collect()` wraps `collector::collect()` but nothing calls it. |

#### What Plan C would need to implement

Plan C's goal is to replace Vec-index storage with pointer-based allocation,
enabling true compaction and eliminating fragmentation. The `src/gc/` files
provide a foundation, but the majority of work is outside `src/gc/`:

1. **`Value` re-encoding** (~20 files, ~1850 lines affected)
   - Change from `u32` index to `*mut T` pointer across `Value` type
   - All `.to_object_idx()`, `.to_array_idx()`, `.to_closure_idx()` calls become pointer dereferences
   - Files: `value.rs`, `interpreter.rs`, `natives.rs`, `property.rs`, `types.rs`, `stack.rs`, `builtins/`, etc.

2. **Pointer update logic**
   - `GcRef` / forwarding table maps old address → new address after compaction
   - Every `Value` pointing into the heap must be updated post-compaction
   - This is the hardest part of any compacting GC

3. **Complete `mark_object()`** (`collector.rs:94-134`)
   - Currently all branches are TODO except `MemoryTag::ValueArray`
   - Needs: object properties, closures, prototypes, constant pools, function bytecode

4. **Arena allocator integration**
   - `allocator.rs` layout is ready; needs `alloc()` / `free()` methods that integrate with Plan B's existing `alloc_slot()` pattern
   - `Heap` used by `Context` for stats; plan to reuse for Plan C's object allocation

#### Phase 6 tasks (deferred — Plan B is the active GC)

- [ ] Update `src/gc/mod.rs` module doc to reflect Plan B active + Plan C reference status
- [ ] Mark `Heap::collect()` and `collector::collect()` with `#[deprecated]` or `#[cfg(plan_c)]` so they are clearly inactive
- [ ] Document the `Value` re-encoding scope before attempting Plan C (`docs/PLAN_C_VALUE_REENCODING.md` to be created)
- [ ] Consider moving `src/gc/allocator.rs` to `src/memory/` if it becomes purely a stats tool, or keep under `gc/` if it is the Plan C arena foundation

  The `Heap` is still used by `Context` for memory statistics (`heap.heap_used()`, `heap.total_size()`, `heap.free_space()`, `heap.stack_used()`), but it is separate from the Plan B Vec-index GC. If the `Heap` allocator is still useful for future Plan C experiments, keep it. If not, move it under a different module.

## Notes

### Phase Counter Overflow

`u32::MAX` is reserved as `SLOT_FREE`. If `gc_phase` approaches that value, generation arrays must be reset before reuse.

### Thread Safety

The GC is intentionally single-threaded. This matches the current JS engine execution model.

### no_std Compatibility

The GC implementation uses `alloc` only and remains compatible with the engine's `no_std` default.

## 2026-03-21 Trigger Follow-up

- Automatic GC trigger accounting no longer runs on every generic JS `Call` / `CallMethod` / `CallConstructor`.
- Instead, `maybe_gc()` is now charged at real GC-managed allocation sites:
  - closures
  - var cells
  - arrays
  - objects
  - iterators
  - error objects
  - regex objects
  - typed arrays
  - array buffers
- Why:
  - the old call-site model was materially distorting high-call / low-allocation workloads such as `fib_iter`, where GC bookkeeping was being paid even without corresponding GC-managed allocation pressure.
- Validation:
  - `test_gc_auto_triggers_during_js_function_workload` still passes
  - `fib_iter 1k` improved from the regressed `5.3292–6.2708 ms` range to `3.5469–4.1842 ms`, with a follow-up rerun at `3.5909–4.2369 ms`
- Current interpretation:
  - trigger behavior is still active and testable;
  - but it is now tied more honestly to actual GC-managed allocation pressure instead of generic JS call frequency.

## 2026-03-21 For-In Key Follow-up

- The `for-in` key path is now safer than before:
  - repeated object keys / array index keys are reused through a small cache;
  - debug-only overflow panic behavior was replaced by the controlled engine error
    `runtime string table exhausted`.
- But this is still not a complete fix:
  - `runtime_strings` are not yet GC-managed;
  - `for_in_key_cache` is append-only;
  - workloads with many unique keys can still exhaust the runtime string table.
- Current status:
  - **mitigated and made safe**
  - **not yet fully eliminated**
