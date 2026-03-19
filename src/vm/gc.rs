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

use crate::value::Value;
use crate::vm::types::{ForInIterator, ForOfIterator};
use alloc::vec::Vec;

/// Sentinel value meaning "slot is free / never allocated"
pub const SLOT_FREE: u32 = u32::MAX;

/// GC bookkeeping state (phase, trigger, stats).
///
/// The generation arrays themselves live directly in `Interpreter` to avoid
/// borrow checker conflicts with recursive mark traversal.
#[derive(Default)]
pub struct GcState {
    /// Current GC phase. Incremented on every collection.
    /// A slot is live if `gen[slot_idx] == gc_phase`.
    pub phase: u32,

    /// Number of allocations since last GC.
    /// Triggers GC when reaching `gc_trigger`.
    pub alloc_count: u32,

    /// Allocation count threshold that triggers a GC cycle.
    /// Adaptive: grows after high-survival runs, shrinks after many deaths.
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
            gc_trigger: 1024,
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
    /// Also resets alloc_count.
    #[inline]
    pub fn start_cycle(&mut self) {
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

        if survival_rate > 0.8 {
            self.gc_trigger = (self.gc_trigger * 3 / 2).clamp(1024, 65536);
        } else if survival_rate < 0.5 {
            self.gc_trigger = (self.gc_trigger * 2 / 3).max(128);
        }
    }

    /// Find a free slot in the generation array, or push a new one.
    /// Returns the slot index. The caller must set the actual container slot.
    #[allow(clippy::ptr_arg)]
    pub fn alloc_slot(&mut self, gen: &mut Vec<u32>) -> usize {
        for (i, g) in gen.iter().enumerate() {
            if *g == SLOT_FREE {
                gen[i] = self.phase;
                return i;
            }
        }
        let idx = gen.len();
        gen.push(self.phase);
        idx
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

/// Mark a Value and all its transitive references.
///
/// `phase` is the current GC phase — slots are marked live by setting
/// `gen[idx] = phase`.
///
/// Each gen array is passed as a separate `&mut [_]` to avoid borrow
/// checker conflicts.
#[inline(always)]
#[allow(clippy::too_many_arguments)]
pub fn gc_mark_value(
    val: Value,
    phase: u32,
    closures: &[crate::vm::types::ClosureData],
    gen_closures: &mut Vec<u32>,
    var_cells: &[Value],
    gen_var_cells: &mut Vec<u32>,
    arrays: &[Vec<Value>],
    gen_arrays: &mut Vec<u32>,
    objects: &[crate::vm::types::ObjectInstance],
    gen_objects: &mut Vec<u32>,
    for_in_iterators: &[ForInIterator],
    gen_for_in_iterators: &mut Vec<u32>,
    for_of_iterators: &[ForOfIterator],
    gen_for_of_iterators: &mut Vec<u32>,
) {
    // --- Array ---
    if let Some(idx) = val.to_array_idx() {
        let idx = idx as usize;
        if idx < gen_arrays.len() {
            gen_arrays[idx] = phase;
        }
        if idx < arrays.len() {
            for elem in &arrays[idx] {
                gc_mark_value(
                    *elem, phase,
                    closures, gen_closures,
                    var_cells, gen_var_cells,
                    arrays, gen_arrays,
                    objects, gen_objects,
                    for_in_iterators, gen_for_in_iterators,
                    for_of_iterators, gen_for_of_iterators,
                );
            }
        }
        return;
    }

    // --- Object ---
    if let Some(idx) = val.to_object_idx() {
        let idx = idx as usize;
        if idx < gen_objects.len() {
            gen_objects[idx] = phase;
        }
        if idx < objects.len() {
            for (_k, v) in &objects[idx].properties {
                gc_mark_value(
                    *v, phase,
                    closures, gen_closures,
                    var_cells, gen_var_cells,
                    arrays, gen_arrays,
                    objects, gen_objects,
                    for_in_iterators, gen_for_in_iterators,
                    for_of_iterators, gen_for_of_iterators,
                );
            }
            if let Some(constructor) = objects[idx].constructor {
                gc_mark_value(
                    constructor, phase,
                    closures, gen_closures,
                    var_cells, gen_var_cells,
                    arrays, gen_arrays,
                    objects, gen_objects,
                    for_in_iterators, gen_for_in_iterators,
                    for_of_iterators, gen_for_of_iterators,
                );
            }
        }
        return;
    }

    // --- Closure ---
    if let Some(idx) = val.to_closure_idx() {
        let idx = idx as usize;
        if idx < gen_closures.len() {
            gen_closures[idx] = phase;
        }
        if idx < closures.len() {
            for &cell_idx in &closures[idx].cell_indices {
                let cell_idx = cell_idx as usize;
                if cell_idx < gen_var_cells.len() {
                    gen_var_cells[cell_idx] = phase;
                }
                if cell_idx < var_cells.len() {
                    gc_mark_value(
                        var_cells[cell_idx], phase,
                        closures, gen_closures,
                        var_cells, gen_var_cells,
                        arrays, gen_arrays,
                        objects, gen_objects,
                        for_in_iterators, gen_for_in_iterators,
                        for_of_iterators, gen_for_of_iterators,
                    );
                }
            }
        }
        return;
    }

    // --- For-in Iterator ---
    if val.is_iterator() {
        if let Some(idx) = val.to_iterator_idx() {
            let idx = idx as usize;
            if idx < gen_for_in_iterators.len() {
                gen_for_in_iterators[idx] = phase;
            }
            if idx < for_in_iterators.len() {
                match &for_in_iterators[idx] {
                    ForInIterator::Object { obj_idx, .. } => {
                        let obj_idx = *obj_idx as usize;
                        if obj_idx < gen_objects.len() {
                            gen_objects[obj_idx] = phase;
                        }
                        if obj_idx < objects.len() {
                            for (_k, v) in &objects[obj_idx].properties {
                                gc_mark_value(
                                    *v, phase,
                                    closures, gen_closures,
                                    var_cells, gen_var_cells,
                                    arrays, gen_arrays,
                                    objects, gen_objects,
                                    for_in_iterators, gen_for_in_iterators,
                                    for_of_iterators, gen_for_of_iterators,
                                );
                            }
                        }
                    }
                    ForInIterator::Array { .. } => {}
                    ForInIterator::Empty => {}
                }
            }
        }
        return;
    }

    // --- For-of Iterator ---
    if val.is_for_of_iterator() {
        if let Some(idx) = val.to_for_of_iterator_idx() {
            let idx = idx as usize;
            if idx < gen_for_of_iterators.len() {
                gen_for_of_iterators[idx] = phase;
            }
            if idx < for_of_iterators.len() {
                match &for_of_iterators[idx] {
                    ForOfIterator::Array { arr_idx, .. } => {
                        let arr_idx = *arr_idx as usize;
                        if arr_idx < gen_arrays.len() {
                            gen_arrays[arr_idx] = phase;
                        }
                        if arr_idx < arrays.len() {
                            for elem in &arrays[arr_idx] {
                                gc_mark_value(
                                    *elem, phase,
                                    closures, gen_closures,
                                    var_cells, gen_var_cells,
                                    arrays, gen_arrays,
                                    objects, gen_objects,
                                    for_in_iterators, gen_for_in_iterators,
                                    for_of_iterators, gen_for_of_iterators,
                                );
                            }
                        }
                    }
                    ForOfIterator::Values { values, .. } => {
                        for v in values {
                            gc_mark_value(
                                *v, phase,
                                closures, gen_closures,
                                var_cells, gen_var_cells,
                                arrays, gen_arrays,
                                objects, gen_objects,
                                for_in_iterators, gen_for_in_iterators,
                                for_of_iterators, gen_for_of_iterators,
                            );
                        }
                    }
                }
            }
        }
    }
}
