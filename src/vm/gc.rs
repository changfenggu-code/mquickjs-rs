//! Mark-Sweep Garbage Collector
//!
//! This module implements a generation-based mark-sweep GC on top of the existing
//! Vec-based object storage. It does NOT require pointer-based allocation or object
//! movement — objects are still stored as indices into Vecs.
//!
//! ## Design
//!
//! Each GC-managed container has a parallel `Vec<u32>` (the "generation array").
//! A global `gc_phase` counter increments on each collection.
//! - `gen[i] == gc_phase` → slot `i` is live
//! - `gen[i] == u32::MAX` → slot `i` is free / never allocated
//!
//! This approach:
//! - Requires no changes to the Value encoding (still uses indices)
//! - Requires no pointer updates (indices are stable across Vec reallocations)
//! - Handles cycles naturally (mark-sweep traversal)
//! - No stop-the-world cost beyond the mark+sweep passes
//! - Incremental: each GC cycle only scans live + newly dead slots
//! - Iterative marking via heap-allocated worklist (no call-stack overflow)

use crate::value::Value;
use crate::vm::types::{ForInIterator, ForOfIterator};
use alloc::vec::Vec;

/// Sentinel value meaning "slot is free / never allocated"
pub const SLOT_FREE: u32 = u32::MAX;

/// GC trigger thresholds
const GC_TRIGGER_DEFAULT: u32 = 1024;
const GC_TRIGGER_MIN: u32 = 128;
const GC_TRIGGER_MAX: u32 = 65536;
/// Growth factor when survival rate > 80%
const GC_TRIGGER_GROWTH_NUM: u32 = 3;
const GC_TRIGGER_GROWTH_DEN: u32 = 2;
/// Shrink factor when survival rate < 50%
const GC_TRIGGER_SHRINK_NUM: u32 = 2;
const GC_TRIGGER_SHRINK_DEN: u32 = 3;
/// Survival rate thresholds
const GC_SURVIVAL_HIGH: f64 = 0.8;
const GC_SURVIVAL_LOW: f64 = 0.5;

/// GC bookkeeping state (phase, trigger, stats).
///
/// The generation arrays themselves live directly in `Interpreter`.
#[derive(Default)]
pub struct GcState {
    /// Current GC phase. Incremented on every collection.
    /// A slot is live if `gen[slot_idx] == gc_phase`.
    pub phase: u32,

    /// Number of allocations since last GC.
    pub alloc_count: u32,

    /// Allocation count threshold that triggers a GC cycle.
    pub gc_trigger: u32,

    /// Number of live slots found in last sweep (per container).
    pub live_closures: usize,
    pub live_var_cells: usize,
    pub live_arrays: usize,
    pub live_objects: usize,
    pub live_for_in_iterators: usize,
    pub live_for_of_iterators: usize,
    pub live_error_objects: usize,
    pub live_regex_objects: usize,
    pub live_typed_arrays: usize,
    pub live_array_buffers: usize,
    pub live_timers: usize,

    /// Total slots swept in last run (per container).
    pub sweep_closures: usize,
    pub sweep_var_cells: usize,
    pub sweep_arrays: usize,
    pub sweep_objects: usize,
    pub sweep_for_in_iterators: usize,
    pub sweep_for_of_iterators: usize,
    pub sweep_error_objects: usize,
    pub sweep_regex_objects: usize,
    pub sweep_typed_arrays: usize,
    pub sweep_array_buffers: usize,
    pub sweep_timers: usize,
}

impl GcState {
    /// Create a new GC state with default trigger threshold
    pub fn new() -> Self {
        Self {
            gc_trigger: GC_TRIGGER_DEFAULT,
            ..Default::default()
        }
    }

    /// Record one allocation and check if GC should trigger.
    #[inline]
    pub fn record_alloc(&mut self) -> bool {
        self.alloc_count += 1;
        self.alloc_count >= self.gc_trigger
    }

    /// Increment phase and reset for a new GC cycle.
    #[inline]
    pub fn start_cycle(&mut self) {
        // Reserve u32::MAX as SLOT_FREE sentinel; reset to 0 when approaching overflow
        if self.phase == u32::MAX - 1 {
            self.phase = 0;
        } else {
            self.phase += 1;
        }
        self.alloc_count = 0;
    }

    /// Adjust trigger threshold based on survival rate from last GC.
    pub fn adjust_trigger(&mut self) {
        let total_slots = self.sweep_closures
            + self.sweep_var_cells
            + self.sweep_arrays
            + self.sweep_objects
            + self.sweep_for_in_iterators
            + self.sweep_for_of_iterators
            + self.sweep_error_objects
            + self.sweep_regex_objects
            + self.sweep_typed_arrays
            + self.sweep_array_buffers
            + self.sweep_timers;

        if total_slots == 0 {
            return;
        }

        let total_live = self.live_closures
            + self.live_var_cells
            + self.live_arrays
            + self.live_objects
            + self.live_for_in_iterators
            + self.live_for_of_iterators
            + self.live_error_objects
            + self.live_regex_objects
            + self.live_typed_arrays
            + self.live_array_buffers
            + self.live_timers;

        let survival_rate = total_live as f64 / total_slots as f64;

        if survival_rate > GC_SURVIVAL_HIGH {
            self.gc_trigger = (self.gc_trigger * GC_TRIGGER_GROWTH_NUM / GC_TRIGGER_GROWTH_DEN)
                .clamp(GC_TRIGGER_MIN, GC_TRIGGER_MAX);
        } else if survival_rate < GC_SURVIVAL_LOW {
            self.gc_trigger = (self.gc_trigger * GC_TRIGGER_SHRINK_NUM / GC_TRIGGER_SHRINK_DEN)
                .max(GC_TRIGGER_MIN);
        }
    }

    /// Find a free slot in the generation array, or push a new one.
    /// Returns `(slot_index, is_new)`.
    #[allow(clippy::ptr_arg)]
    pub fn alloc_slot(&mut self, gen: &mut Vec<u32>) -> (usize, bool) {
        for (i, g) in gen.iter().enumerate() {
            if *g == SLOT_FREE {
                gen[i] = self.phase;
                return (i, false);
            }
        }
        let idx = gen.len();
        gen.push(self.phase);
        (idx, true)
    }

    /// Sweep a container: mark all dead slots as SLOT_FREE.
    /// Returns the number of live slots found.
    pub fn sweep_container(&mut self, gen: &mut [u32], len: usize) -> usize {
        let mut live = 0;
        for g in &mut gen[..len] {
            if *g == self.phase {
                live += 1;
            } else {
                *g = SLOT_FREE;
            }
        }
        live
    }
}

// ── Iterative Mark (no recursion, no stack overflow) ───────────────

/// Check if a gen slot is already marked live in the current phase.
#[inline]
fn is_marked(gen: &[u32], idx: usize, phase: u32) -> bool {
    idx < gen.len() && gen[idx] == phase
}

/// Mark a gen slot live and return true if it was newly marked.
#[inline]
fn mark_slot(gen: &mut [u32], idx: usize, phase: u32) -> bool {
    if idx < gen.len() && gen[idx] != phase {
        gen[idx] = phase;
        true
    } else {
        false
    }
}

/// Bundles all mutable generation arrays needed for iterative mark.
/// This avoids passing 6 separate &mut [u32] parameters.
pub struct GcMarkRoots<'a> {
    pub gen_closures: &'a mut [u32],
    pub gen_var_cells: &'a mut [u32],
    pub gen_arrays: &'a mut [u32],
    pub gen_objects: &'a mut [u32],
    pub gen_for_in_iterators: &'a mut [u32],
    pub gen_for_of_iterators: &'a mut [u32],
}

/// Iteratively mark all values reachable from the given root set.
///
/// Uses a `Vec<Value>` worklist allocated on the heap — no call-stack overflow,
/// safe for embedded targets with small stacks (8KB or less).
///
/// Slots are marked via `mark_slot()` only when unvisited (read phase, write new phase).
#[allow(clippy::too_many_arguments)]
pub fn gc_mark_roots_iterative(
    roots: &[Value],
    phase: u32,
    closures: &[crate::vm::types::ClosureData],
    var_cells: &[Value],
    arrays: &[Vec<Value>],
    objects: &[crate::vm::types::ObjectInstance],
    for_in_iterators: &[ForInIterator],
    for_of_iterators: &[ForOfIterator],
    mark: &mut GcMarkRoots<'_>,
) {
    // Worklist: heap-allocated Vec, grows as needed, never overflows the call stack
    let mut worklist: Vec<Value> = Vec::with_capacity(64);

    // Seed: push all roots onto worklist
    for &root in roots {
        worklist.push(root);
    }

    // Iterative DFS: pop value → mark it → push children
    while let Some(val) = worklist.pop() {
        // --- Array ---
        if let Some(idx) = val.to_array_idx() {
            let idx = idx as usize;
            if mark_slot(mark.gen_arrays, idx, phase) && idx < arrays.len() {
                for elem in &arrays[idx] {
                    worklist.push(*elem);
                }
            }
            continue;
        }

        // --- Object ---
        if let Some(idx) = val.to_object_idx() {
            let idx = idx as usize;
            if mark_slot(mark.gen_objects, idx, phase) && idx < objects.len() {
                for (_k, v) in &objects[idx].properties {
                    worklist.push(*v);
                }
                if let Some(constructor) = objects[idx].constructor {
                    worklist.push(constructor);
                }
            }
            continue;
        }

        // --- Closure ---
        if let Some(idx) = val.to_closure_idx() {
            let idx = idx as usize;
            if mark_slot(mark.gen_closures, idx, phase) && idx < closures.len() {
                for &cell_idx in &closures[idx].cell_indices {
                    let cell_idx = cell_idx as usize;
                    if mark_slot(mark.gen_var_cells, cell_idx, phase) && cell_idx < var_cells.len()
                    {
                        worklist.push(var_cells[cell_idx]);
                    }
                }
            }
            continue;
        }

        // --- For-in Iterator ---
        if val.is_iterator() {
            if let Some(idx) = val.to_iterator_idx() {
                let idx = idx as usize;
                if mark_slot(mark.gen_for_in_iterators, idx, phase) && idx < for_in_iterators.len()
                {
                    match &for_in_iterators[idx] {
                        ForInIterator::ObjectKeys { keys, .. } => {
                            for key in keys {
                                worklist.push(*key);
                            }
                        }
                        ForInIterator::Array { .. } | ForInIterator::Empty => {}
                    }
                }
            }
            continue;
        }

        // --- For-of Iterator ---
        if val.is_for_of_iterator() {
            if let Some(idx) = val.to_for_of_iterator_idx() {
                let idx = idx as usize;
                if mark_slot(mark.gen_for_of_iterators, idx, phase) && idx < for_of_iterators.len()
                {
                    match &for_of_iterators[idx] {
                        ForOfIterator::Array { arr_idx, .. } => {
                            let arr_idx = *arr_idx as usize;
                            if mark_slot(mark.gen_arrays, arr_idx, phase) && arr_idx < arrays.len()
                            {
                                for elem in &arrays[arr_idx] {
                                    worklist.push(*elem);
                                }
                            }
                        }
                        ForOfIterator::Values { values, .. } => {
                            for v in values {
                                worklist.push(*v);
                            }
                        }
                    }
                }
            }
        }
    }
}
