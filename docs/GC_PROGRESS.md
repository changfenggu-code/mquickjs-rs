# GC Implementation Progress

This document tracks the implementation of **Plan B: Mark-Sweep GC** for mquickjs-rs.

**Start date**: 2026-03-19
**Strategy**: Generation-based mark-sweep on existing Vec-index architecture (no pointer changes)
**Upstream**: [docs/ENGINE_OPTIMIZATION_TASKLIST.md](ENGINE_OPTIMIZATION_TASKLIST.md) — Section 9.2

---

## Acceptance Criteria

- [ ] `cargo test -p mquickjs-rs` — all 458 tests pass
- [ ] `cargo test -p led-runtime` — all led-runtime tests pass
- [ ] `cargo clippy -- -D warnings` — zero warnings
- [ ] `cargo build --release --no-default-features` — no_std compiles
- [ ] Benchmark: memory usage visibly decreases after GC vs before
- [ ] `gc_count` stat increments during execution
- [ ] Cycle reference `{ let a = {}; a.self = a; }` is collected after GC
- [ ] Free slot reuse: dead slots are reclaimed, not leaked

---

## Core Design

```
Generation-based marking (no extra memory overhead):
  gc_phase: u32  (increments each collection)
  gen[i] == gc_phase  → slot i is LIVE
  gen[i] == u32::MAX  → slot i is FREE

GC-managed containers:
  closures          → closure_idx
  var_cells         → (via closures)
  arrays            → array_idx
  objects           → object_idx
  for_in_iterators  → iterator_idx
  for_of_iterators  → for_of_iterator_idx
  error_objects     → error_object
  regex_objects     → regexp_object
  typed_arrays      → typed_array_object
  array_buffers     → array_buffer_object
  timers            → manual cleanup

Roots:
  - Value stack (CallFrame.stack)
  - global_vars
  - closures → var_cells
  - timers.callback
```

---

## Implementation Phases

### Phase 1: GC Infrastructure ✅
- [x] `src/vm/gc.rs` created with:
  - [x] `GcState` struct with phase, trigger, and adaptive threshold
  - [x] `gc_mark_value()` — recursive mark for all Value types (arrays, objects, closures, iterators)
  - [x] `gc_alloc_slot()` — free slot finder with linear scan
  - [x] `gc_sweep_container()` — dead slot → SLOT_FREE
  - [x] `adjust_trigger()` — adaptive threshold (grows on high survival, shrinks on low)

### Phase 2: Interpreter Integration ✅
- [x] Add gen arrays (`gen_closures`, `gen_var_cells`, `gen_arrays`, etc.) to `Interpreter`
- [x] Add `gc: GcState` field to `Interpreter`
- [x] Implement `gc_mark_roots()` — traverses call_stack + global_vars + timers
- [x] Implement `gc_sweep()` — sweeps all 11 containers
- [x] Implement `gc_collect()` — orchestrates mark + sweep + threshold adjust
- [x] Implement `maybe_gc()` — called on every function call
- [x] Implement `gc_alloc_*()` wrappers for all 11 container types

### Phase 3: Context Integration ✅
- [x] Call `maybe_gc()` in `Interpreter::execute()` and `Interpreter::call_function()`
- [x] Call `gc_collect()` in `Context::gc()` (manual trigger)

### Phase 4: Testing ✅
- [x] `cargo test -p mquickjs-rs` — **426 passed, 2 pre-existing failures** (Date.now stub, String.repeat)
- [x] `cargo test -p led-runtime` — **22 passed, 0 failed**
- [x] `cargo clippy -- -D warnings` — **zero warnings**
- [x] `cargo build --release --no-default-features` — **no_std OK**

### Phase 5: Benchmarking ⬜
- [ ] Memory usage comparison (before GC vs after GC)
- [ ] Performance impact assessment
- [ ] Decide if Plan C (Mark-Compact) is needed

---

## Notes

### Phase counter overflow
`u32::MAX` is reserved as `SLOT_FREE`. If `gc_phase` reaches `u32::MAX - 1`, all generation arrays are reset to 0 on next collection.

### Thread safety
The GC is **not** thread-safe. This is acceptable because:
- The JS engine is single-threaded
- ESP32 runs on a single core
- No concurrent execution of JS code

### no_std compatibility
All GC code uses `alloc` only (no `std`). `Vec<u32>` is available in `alloc`.
