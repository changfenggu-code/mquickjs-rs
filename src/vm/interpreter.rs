//! Bytecode interpreter
//!
//! Executes JavaScript bytecode using a stack-based virtual machine.

use crate::runtime::FunctionBytecode;
use crate::util::dtoa::i32_to_str;
use crate::util::unicode::utf16_len;
use crate::value::{float_to_value, Float, Value};
use crate::vm::opcode::OpCode;
use crate::vm::stack::Stack;
use alloc::{collections::BTreeMap, format, string::String, string::ToString, vec, vec::Vec};

// Native function implementations and format helpers (defined in natives.rs)
use super::natives::*;

// Types and constants defined in src/vm/types.rs
pub use super::types::*;

impl Interpreter {
    /// Default stack capacity
    const DEFAULT_STACK_SIZE: usize = 1024;
    /// Default max recursion
    const DEFAULT_MAX_RECURSION: usize = 512;

    /// Create a new interpreter
    pub fn new() -> Self {
        let mut interp = Interpreter {
            stack: Stack::new(Self::DEFAULT_STACK_SIZE),
            call_stack: Vec::with_capacity(64),
            max_recursion: Self::DEFAULT_MAX_RECURSION,
            runtime_strings: Vec::new(),
            closures: Vec::new(),
            var_cells: Vec::new(),
            exception_handlers: Vec::new(),
            arrays: Vec::new(),
            objects: Vec::new(),
            for_in_iterators: Vec::new(),
            for_of_iterators: Vec::new(),
            native_functions: Vec::new(),
            native_func_index: BTreeMap::new(),
            for_in_key_cache: Vec::new(),
            global_vars: BTreeMap::new(),
            error_objects: Vec::new(),
            regex_objects: Vec::new(),
            typed_arrays: Vec::new(),
            array_buffers: Vec::new(),
            current_string_constants: None,
            nested_call_target_depth: None,
            timers: Vec::new(),
            next_timer_id: 1,
            time_provider: None,
            time_origin_millis: 0,
            gc_count: 0,
            gc: super::gc::GcState::new(),
            gen_closures: Vec::new(),
            gen_var_cells: Vec::new(),
            gen_arrays: Vec::new(),
            gen_objects: Vec::new(),
            gen_for_in_iterators: Vec::new(),
            gen_for_of_iterators: Vec::new(),
            gen_error_objects: Vec::new(),
            gen_regex_objects: Vec::new(),
            gen_typed_arrays: Vec::new(),
            gen_array_buffers: Vec::new(),
            gen_timers: Vec::new(),
            random_seed: 12345,
            #[cfg(feature = "dump")]
            runtime_string_source_stats: RuntimeStringSourceStats::default(),
            #[cfg(feature = "dump")]
            opcode_counts: [0; 256],
        };
        interp.time_origin_millis = interp.current_time_millis().unwrap_or(0);
        interp.register_builtins();
        interp
    }

    /// Create an interpreter with custom settings
    pub fn with_config(stack_size: usize, max_recursion: usize) -> Self {
        let mut interp = Interpreter {
            stack: Stack::new(stack_size),
            call_stack: Vec::with_capacity(64),
            max_recursion,
            runtime_strings: Vec::new(),
            closures: Vec::new(),
            var_cells: Vec::new(),
            exception_handlers: Vec::new(),
            arrays: Vec::new(),
            objects: Vec::new(),
            for_in_iterators: Vec::new(),
            for_of_iterators: Vec::new(),
            native_functions: Vec::new(),
            native_func_index: BTreeMap::new(),
            for_in_key_cache: Vec::new(),
            global_vars: BTreeMap::new(),
            error_objects: Vec::new(),
            regex_objects: Vec::new(),
            typed_arrays: Vec::new(),
            array_buffers: Vec::new(),
            current_string_constants: None,
            nested_call_target_depth: None,
            timers: Vec::new(),
            next_timer_id: 1,
            time_provider: None,
            time_origin_millis: 0,
            gc_count: 0,
            gc: super::gc::GcState::new(),
            gen_closures: Vec::new(),
            gen_var_cells: Vec::new(),
            gen_arrays: Vec::new(),
            gen_objects: Vec::new(),
            gen_for_in_iterators: Vec::new(),
            gen_for_of_iterators: Vec::new(),
            gen_error_objects: Vec::new(),
            gen_regex_objects: Vec::new(),
            gen_typed_arrays: Vec::new(),
            gen_array_buffers: Vec::new(),
            gen_timers: Vec::new(),
            random_seed: 12345,
            #[cfg(feature = "dump")]
            runtime_string_source_stats: RuntimeStringSourceStats::default(),
            #[cfg(feature = "dump")]
            opcode_counts: [0; 256],
        };
        interp.time_origin_millis = interp.current_time_millis().unwrap_or(0);
        interp.register_builtins();
        interp
    }

    #[inline]
    pub(crate) fn current_time_millis(&self) -> Option<u64> {
        if let Some(provider) = self.time_provider {
            Some(provider())
        } else {
            #[cfg(feature = "std")]
            {
                use std::time::{SystemTime, UNIX_EPOCH};
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .ok()
                    .map(|d| d.as_millis() as u64)
            }
            #[cfg(not(feature = "std"))]
            {
                None
            }
        }
    }

    #[inline]
    pub fn set_time_provider(&mut self, provider: fn() -> u64) {
        self.time_provider = Some(provider);
        self.time_origin_millis = self.current_time_millis().unwrap_or(0);
    }

    // ---------------------------------------------------------------------------
    // Garbage Collection
    // ---------------------------------------------------------------------------

    /// Check if GC should run and collect if threshold is reached.
    /// Call this on every N allocations to keep memory bounded.
    pub fn maybe_gc(&mut self) {
        if self.gc.record_alloc() {
            self.gc_collect();
        }
    }

    /// Run a full mark-sweep garbage collection.
    /// This marks all reachable objects from roots, then sweeps dead slots.
    pub fn gc_collect(&mut self) {
        self.gc.start_cycle();

        // Phase 1: Mark — traverse from all roots
        self.gc_mark_roots();

        // Phase 2: Sweep — mark dead slots as free
        self.gc_sweep();

        // Phase 3: Adjust trigger threshold based on survival rate
        self.gc.adjust_trigger();

        self.gc_count += 1;
    }

    /// Mark all objects reachable from GC roots using iterative traversal.
    ///
    /// Collects all root values into a Vec, then calls the heap-allocated
    /// worklist-based marker — no recursion, no call-stack overflow.
    fn gc_mark_roots(&mut self) {
        use crate::vm::gc::{gc_mark_roots_iterative, GcMarkRoots};

        let phase = self.gc.phase;

        // Collect all roots into a Vec (heap allocation, not call stack)
        let cap = self.call_stack.len() * 2
            + self.global_vars.len()
            + self.timers.iter().filter(|t| !t.cancelled).count();
        let mut roots: Vec<Value> = Vec::with_capacity(cap.max(16));

        for frame in &self.call_stack {
            roots.push(frame.this_val);
            roots.push(frame.this_func);
        }
        for val in self.global_vars.values() {
            roots.push(*val);
        }
        for timer in &self.timers {
            if !timer.cancelled {
                roots.push(timer.callback);
            }
        }

        // Mark timer slots inline (need index-based access)
        for (i, timer) in self.timers.iter().enumerate() {
            if !timer.cancelled && i < self.gen_timers.len() {
                self.gen_timers[i] = phase;
            }
        }

        // Mutable refs to gen arrays (different fields → simultaneous &mut allowed)
        let gen_closures = &mut self.gen_closures;
        let gen_var_cells = &mut self.gen_var_cells;
        let gen_arrays = &mut self.gen_arrays;
        let gen_objects = &mut self.gen_objects;
        let gen_for_in_iterators = &mut self.gen_for_in_iterators;
        let gen_for_of_iterators = &mut self.gen_for_of_iterators;

        // Immutable refs to data containers
        let closures = &self.closures;
        let var_cells = &self.var_cells;
        let arrays = &self.arrays;
        let objects = &self.objects;
        let for_in_iterators = &self.for_in_iterators;
        let for_of_iterators = &self.for_of_iterators;

        gc_mark_roots_iterative(
            &roots,
            phase,
            closures,
            var_cells,
            arrays,
            objects,
            for_in_iterators,
            for_of_iterators,
            &mut GcMarkRoots {
                gen_closures,
                gen_var_cells,
                gen_arrays,
                gen_objects,
                gen_for_in_iterators,
                gen_for_of_iterators,
            },
        );
    }

    /// Sweep all containers, marking dead slots as free.
    fn gc_sweep(&mut self) {
        // Sweep closures
        self.gc.sweep_closures = self.closures.len();
        self.gc.live_closures = self
            .gc
            .sweep_container(&mut self.gen_closures, self.closures.len());

        // Sweep var_cells
        self.gc.sweep_var_cells = self.var_cells.len();
        self.gc.live_var_cells = self
            .gc
            .sweep_container(&mut self.gen_var_cells, self.var_cells.len());

        // Sweep arrays
        self.gc.sweep_arrays = self.arrays.len();
        self.gc.live_arrays = self
            .gc
            .sweep_container(&mut self.gen_arrays, self.arrays.len());

        // Sweep objects
        self.gc.sweep_objects = self.objects.len();
        self.gc.live_objects = self
            .gc
            .sweep_container(&mut self.gen_objects, self.objects.len());

        // Sweep for-in iterators
        self.gc.sweep_for_in_iterators = self.for_in_iterators.len();
        self.gc.live_for_in_iterators = self
            .gc
            .sweep_container(&mut self.gen_for_in_iterators, self.for_in_iterators.len());

        // Sweep for-of iterators
        self.gc.sweep_for_of_iterators = self.for_of_iterators.len();
        self.gc.live_for_of_iterators = self
            .gc
            .sweep_container(&mut self.gen_for_of_iterators, self.for_of_iterators.len());

        // Sweep error objects
        self.gc.sweep_error_objects = self.error_objects.len();
        self.gc.live_error_objects = self
            .gc
            .sweep_container(&mut self.gen_error_objects, self.error_objects.len());

        // Sweep regex objects
        self.gc.sweep_regex_objects = self.regex_objects.len();
        self.gc.live_regex_objects = self
            .gc
            .sweep_container(&mut self.gen_regex_objects, self.regex_objects.len());

        // Sweep typed arrays
        self.gc.sweep_typed_arrays = self.typed_arrays.len();
        self.gc.live_typed_arrays = self
            .gc
            .sweep_container(&mut self.gen_typed_arrays, self.typed_arrays.len());

        // Sweep array buffers
        self.gc.sweep_array_buffers = self.array_buffers.len();
        self.gc.live_array_buffers = self
            .gc
            .sweep_container(&mut self.gen_array_buffers, self.array_buffers.len());

        // Sweep timers: remove cancelled or dead timers
        self.gc.sweep_timers = self.timers.len();
        self.timers.retain(|t| !t.cancelled);
        self.gen_timers.truncate(self.timers.len());
        self.gc.live_timers = self.timers.len();
    }

    /// Allocate a closure slot, reusing a free slot if available.
    /// Returns the slot index. Caller must push the ClosureData.
    pub fn gc_alloc_closure(&mut self) -> (usize, bool) {
        self.gc.alloc_slot(&mut self.gen_closures)
    }

    /// Allocate a var_cells slot, reusing a free slot if available.
    /// Returns (slot_index, is_new). Caller should push if is_new, else overwrite.
    pub fn gc_alloc_var_cell(&mut self) -> (usize, bool) {
        self.gc.alloc_slot(&mut self.gen_var_cells)
    }

    /// Allocate an array slot, reusing a free slot if available.
    pub fn gc_alloc_array(&mut self) -> (usize, bool) {
        self.gc.alloc_slot(&mut self.gen_arrays)
    }

    /// Allocate an object slot, reusing a free slot if available.
    pub fn gc_alloc_object(&mut self) -> (usize, bool) {
        self.gc.alloc_slot(&mut self.gen_objects)
    }

    /// Allocate a for-in iterator slot, reusing a free slot if available.
    pub fn gc_alloc_for_in_iterator(&mut self) -> (usize, bool) {
        self.gc.alloc_slot(&mut self.gen_for_in_iterators)
    }

    /// Allocate a for-of iterator slot, reusing a free slot if available.
    pub fn gc_alloc_for_of_iterator(&mut self) -> (usize, bool) {
        self.gc.alloc_slot(&mut self.gen_for_of_iterators)
    }

    /// Allocate an error object slot, reusing a free slot if available.
    pub fn gc_alloc_error_object(&mut self) -> (usize, bool) {
        self.gc.alloc_slot(&mut self.gen_error_objects)
    }

    /// Allocate a regex object slot, reusing a free slot if available.
    pub fn gc_alloc_regex_object(&mut self) -> (usize, bool) {
        self.gc.alloc_slot(&mut self.gen_regex_objects)
    }

    /// Allocate a typed array slot, reusing a free slot if available.
    pub fn gc_alloc_typed_array(&mut self) -> (usize, bool) {
        self.gc.alloc_slot(&mut self.gen_typed_arrays)
    }

    /// Allocate an array buffer slot, reusing a free slot if available.
    pub fn gc_alloc_array_buffer(&mut self) -> (usize, bool) {
        self.gc.alloc_slot(&mut self.gen_array_buffers)
    }

    /// Allocate a timer slot, reusing a free slot if available.
    pub fn gc_alloc_timer(&mut self) -> (usize, bool) {
        self.gc.alloc_slot(&mut self.gen_timers)
    }

    /// Get memory statistics from the interpreter
    pub fn get_stats(&self) -> InterpreterStats {
        use crate::vm::gc::SLOT_FREE;

        let closures = self
            .gen_closures
            .iter()
            .filter(|&&g| g != SLOT_FREE)
            .count();
        let arrays = self.gen_arrays.iter().filter(|&&g| g != SLOT_FREE).count();
        let array_elements = self
            .arrays
            .iter()
            .zip(self.gen_arrays.iter())
            .filter(|(_, &g)| g != SLOT_FREE)
            .map(|(a, _)| a.len())
            .sum();
        let objects = self.gen_objects.iter().filter(|&&g| g != SLOT_FREE).count();
        let object_properties = self
            .objects
            .iter()
            .zip(self.gen_objects.iter())
            .filter(|(_, &g)| g != SLOT_FREE)
            .map(|(o, _)| o.properties.len())
            .sum();
        let error_objects = self
            .gen_error_objects
            .iter()
            .filter(|&&g| g != SLOT_FREE)
            .count();
        let regex_objects = self
            .gen_regex_objects
            .iter()
            .filter(|&&g| g != SLOT_FREE)
            .count();
        let typed_arrays = self
            .gen_typed_arrays
            .iter()
            .filter(|&&g| g != SLOT_FREE)
            .count();
        let typed_array_bytes = self
            .typed_arrays
            .iter()
            .zip(self.gen_typed_arrays.iter())
            .filter(|(_, &g)| g != SLOT_FREE)
            .map(|(ta, _)| ta.data.len())
            .sum();
        let array_buffers = self
            .gen_array_buffers
            .iter()
            .filter(|&&g| g != SLOT_FREE)
            .count();
        let array_buffer_bytes = self
            .array_buffers
            .iter()
            .zip(self.gen_array_buffers.iter())
            .filter(|(_, &g)| g != SLOT_FREE)
            .map(|(ab, _)| ab.data.len())
            .sum();

        InterpreterStats {
            gc_count: self.gc_count,
            runtime_strings: self.runtime_strings.len(),
            runtime_string_bytes: self.runtime_strings.iter().map(|s| s.len()).sum(),
            arrays,
            array_elements,
            objects,
            object_properties,
            closures,
            error_objects,
            regex_objects,
            typed_arrays,
            typed_array_bytes,
            array_buffers,
            array_buffer_bytes,
        }
    }

    /// Closure index marker (indices into closures vec are stored as negative values)
    const CLOSURE_INDEX_MARKER: u32 = 0x8000_0000;

    /// Runtime string index offset (indices >= this are runtime strings)
    pub(crate) const RUNTIME_STRING_OFFSET: u16 = 0x8000;

    /// Get string content from a string value
    fn get_string_content<'a>(
        &'a self,
        val: Value,
        bytecode: &'a FunctionBytecode,
    ) -> Option<&'a str> {
        if !val.is_string() {
            return None;
        }
        let idx = val.to_string_idx()?;

        // Check if it's a built-in string
        if let Some(s) = crate::value::get_builtin_string(idx) {
            return Some(s);
        }

        // Check if it's a runtime string
        if idx >= Self::RUNTIME_STRING_OFFSET {
            let runtime_idx = (idx - Self::RUNTIME_STRING_OFFSET) as usize;
            return self.runtime_string_as_str(runtime_idx);
        }

        // Otherwise it's a compile-time string
        bytecode
            .string_constants
            .get(idx as usize)
            .map(|s| s.as_str())
    }

    #[inline]
    fn get_const_string_content<'a>(
        &'a self,
        bytecode: &'a FunctionBytecode,
        str_idx: u16,
    ) -> Option<&'a str> {
        if let Some(s) = crate::value::get_builtin_string(str_idx) {
            return Some(s);
        }
        bytecode
            .string_constants
            .get(str_idx as usize)
            .map(|s| s.as_str())
    }

    /// Create a runtime string without updating profiling counters.
    #[inline]
    fn create_runtime_string_raw(&mut self, s: String) -> Value {
        let idx = self.runtime_strings.len();
        self.runtime_strings.push(s.into());
        Value::string(Self::RUNTIME_STRING_OFFSET + idx as u16)
    }

    /// Create a runtime string and return its Value.
    pub(crate) fn create_runtime_string(&mut self, s: String) -> Value {
        self.bump_runtime_string_other();
        self.create_runtime_string_raw(s)
    }

    #[cfg(feature = "dump")]
    #[inline]
    fn bump_runtime_string_other(&mut self) {
        self.runtime_string_source_stats.total += 1;
        self.runtime_string_source_stats.other += 1;
    }

    #[cfg(feature = "dump")]
    #[inline]
    fn bump_runtime_string_concat(&mut self) {
        self.runtime_string_source_stats.total += 1;
        self.runtime_string_source_stats.concat += 1;
    }

    #[cfg(feature = "dump")]
    #[inline]
    fn bump_runtime_string_for_in_key(&mut self) {
        self.runtime_string_source_stats.total += 1;
        self.runtime_string_source_stats.for_in_key += 1;
    }

    #[cfg(feature = "dump")]
    #[inline]
    fn bump_runtime_string_json(&mut self) {
        self.runtime_string_source_stats.total += 1;
        self.runtime_string_source_stats.json += 1;
    }

    #[cfg(feature = "dump")]
    #[inline]
    fn bump_runtime_string_object_keys(&mut self) {
        self.runtime_string_source_stats.total += 1;
        self.runtime_string_source_stats.object_keys += 1;
    }

    #[cfg(feature = "dump")]
    #[inline]
    fn bump_runtime_string_object_entries(&mut self) {
        self.runtime_string_source_stats.total += 1;
        self.runtime_string_source_stats.object_entries += 1;
    }

    #[cfg(feature = "dump")]
    #[inline]
    fn bump_runtime_string_error(&mut self) {
        self.runtime_string_source_stats.total += 1;
        self.runtime_string_source_stats.error_string += 1;
    }

    #[cfg(feature = "dump")]
    #[inline]
    fn bump_runtime_string_type(&mut self) {
        self.runtime_string_source_stats.total += 1;
        self.runtime_string_source_stats.type_string += 1;
    }

    #[cfg(not(feature = "dump"))]
    #[inline]
    fn bump_runtime_string_other(&mut self) {}

    #[cfg(not(feature = "dump"))]
    #[inline]
    fn bump_runtime_string_concat(&mut self) {}

    #[cfg(not(feature = "dump"))]
    #[inline]
    fn bump_runtime_string_for_in_key(&mut self) {}

    #[cfg(not(feature = "dump"))]
    #[inline]
    fn bump_runtime_string_json(&mut self) {}

    #[cfg(not(feature = "dump"))]
    #[inline]
    fn bump_runtime_string_object_keys(&mut self) {}

    #[cfg(not(feature = "dump"))]
    #[inline]
    fn bump_runtime_string_object_entries(&mut self) {}

    #[cfg(not(feature = "dump"))]
    #[inline]
    fn bump_runtime_string_error(&mut self) {}

    #[cfg(not(feature = "dump"))]
    #[inline]
    fn bump_runtime_string_type(&mut self) {}

    /// Create a runtime string on the `for-in` key path while recording source stats.
    #[inline]
    fn create_runtime_string_for_in_key(&mut self, s: &str) -> InterpreterResult<Value> {
        if let Some((_, value)) = self.for_in_key_cache.iter().find(|(key, _)| key == s) {
            return Ok(*value);
        }

        self.bump_runtime_string_for_in_key();
        let idx = self.runtime_strings.len();
        let max_runtime_strings = u16::MAX as usize - Self::RUNTIME_STRING_OFFSET as usize;
        if idx > max_runtime_strings {
            return Err(InterpreterError::InternalError(
                "runtime string table exhausted".to_string(),
            ));
        }
        let value = self.create_runtime_string_raw(s.to_string());
        self.for_in_key_cache.push((s.to_string(), value));
        Ok(value)
    }

    #[inline]
    pub(crate) fn create_runtime_string_json(&mut self, s: String) -> Value {
        self.bump_runtime_string_json();
        self.create_runtime_string_raw(s)
    }

    #[inline]
    pub(crate) fn create_runtime_string_object_key(&mut self, s: String) -> Value {
        self.bump_runtime_string_object_keys();
        self.create_runtime_string_raw(s)
    }

    #[inline]
    pub(crate) fn create_runtime_string_object_entry_key(&mut self, s: String) -> Value {
        self.bump_runtime_string_object_entries();
        self.create_runtime_string_raw(s)
    }

    #[inline]
    pub(crate) fn create_runtime_string_error(&mut self, s: String) -> Value {
        self.bump_runtime_string_error();
        self.create_runtime_string_raw(s)
    }

    #[inline]
    pub(crate) fn create_runtime_string_type(&mut self, s: String) -> Value {
        self.bump_runtime_string_type();
        self.create_runtime_string_raw(s)
    }
    #[inline]
    fn value_to_string_len_hint(&self, val: Value, bytecode: &FunctionBytecode) -> usize {
        if val.is_string() {
            self.get_string_content(val, bytecode)
                .unwrap_or_default()
                .len()
        } else if let Some(n) = val.to_i32() {
            let mut buf = [0u8; 16];
            i32_to_str(&mut buf, n)
        } else if let Some(f) = val.to_f32() {
            crate::value::format_float(f).len()
        } else if let Some(b) = val.to_bool() {
            if b {
                4
            } else {
                5
            }
        } else if val.is_null() {
            4
        } else if val.is_undefined() {
            9
        } else {
            8
        }
    }

    #[inline]
    fn append_value_to_string(&self, out: &mut String, val: Value, bytecode: &FunctionBytecode) {
        if val.is_string() {
            out.push_str(self.get_string_content(val, bytecode).unwrap_or_default());
        } else if let Some(n) = val.to_i32() {
            let mut buf = [0u8; 16];
            let len = i32_to_str(&mut buf, n);
            // SAFETY: i32_to_str only writes ASCII decimal digits and optional '-'.
            let s = unsafe { core::str::from_utf8_unchecked(&buf[..len]) };
            out.push_str(s);
        } else if let Some(f) = val.to_f32() {
            out.push_str(&crate::value::format_float(f));
        } else if let Some(b) = val.to_bool() {
            out.push_str(if b { "true" } else { "false" });
        } else if val.is_null() {
            out.push_str("null");
        } else if val.is_undefined() {
            out.push_str("undefined");
        } else {
            out.push_str("[object]");
        }
    }

    /// Promote a compile-time string value to a runtime string so it can be
    /// safely stored in objects/arrays that outlive the current bytecode scope.
    #[inline]
    fn promote_string(&mut self, val: Value, bytecode: &FunctionBytecode) -> Value {
        if !val.is_string() {
            return val;
        }
        if let Some(idx) = val.to_string_idx() {
            // Built-in strings and runtime strings are already global
            if crate::value::get_builtin_string(idx).is_some() || idx >= Self::RUNTIME_STRING_OFFSET
            {
                return val;
            }
            // Compile-time string 閳?promote to runtime
            if let Some(s) = bytecode.string_constants.get(idx as usize) {
                return self.create_runtime_string(s.clone());
            }
        }
        val
    }

    /// Get a string by its index (works for built-in, compile-time, and runtime strings)
    /// For compile-time strings, uses current_string_constants if set.
    pub fn get_string_by_idx(&self, str_idx: u16) -> Option<&str> {
        // Check built-in strings first (STR_EMPTY, STR_UNDEFINED, etc.)
        if let Some(s) = crate::value::get_builtin_string(str_idx) {
            return Some(s);
        }
        if str_idx >= Self::RUNTIME_STRING_OFFSET {
            let runtime_idx = (str_idx - Self::RUNTIME_STRING_OFFSET) as usize;
            self.runtime_string_as_str(runtime_idx)
        } else {
            // Compile-time string - use current_string_constants if available
            if let Some(constants_ptr) = self.current_string_constants {
                // SAFETY: The pointer is valid during bytecode execution
                let constants = unsafe { &*constants_ptr };
                constants.get(str_idx as usize).map(|s| s.as_str())
            } else {
                None
            }
        }
    }

    fn runtime_string_part_len(&self, part: &RuntimeStringPart) -> usize {
        match part {
            RuntimeStringPart::Runtime(idx) => {
                let runtime_idx = (*idx - Self::RUNTIME_STRING_OFFSET) as usize;
                self.runtime_strings
                    .get(runtime_idx)
                    .map(|s| s.len())
                    .unwrap_or_default()
            }
            RuntimeStringPart::Owned(s) => s.len(),
        }
    }

    fn runtime_string_part_from_value(
        &self,
        val: Value,
        bytecode: &FunctionBytecode,
    ) -> RuntimeStringPart {
        if let Some(str_idx) = val.to_string_idx() {
            if str_idx >= Self::RUNTIME_STRING_OFFSET {
                return RuntimeStringPart::Runtime(str_idx);
            }
            if let Some(s) = self.get_string_content(val, bytecode) {
                return RuntimeStringPart::Owned(s.to_string());
            }
        }

        let mut out = String::new();
        self.append_value_to_string(&mut out, val, bytecode);
        RuntimeStringPart::Owned(out)
    }

    fn append_runtime_string_part(&self, out: &mut String, part: &RuntimeStringPart) {
        match part {
            RuntimeStringPart::Runtime(idx) => {
                if let Some(s) = self.get_string_by_idx(*idx) {
                    out.push_str(s);
                }
            }
            RuntimeStringPart::Owned(s) => out.push_str(s),
        }
    }

    fn runtime_string_as_str(&self, runtime_idx: usize) -> Option<&str> {
        let runtime = self.runtime_strings.get(runtime_idx)?;
        unsafe {
            let flat = &mut *runtime.flat.get();
            if flat.is_none() {
                let mut out = String::with_capacity(runtime.len);
                if let Some(left) = &runtime.left {
                    self.append_runtime_string_part(&mut out, left);
                }
                if let Some(right) = &runtime.right {
                    self.append_runtime_string_part(&mut out, right);
                }
                *flat = Some(out);
            }
            flat.as_deref()
        }
    }

    fn create_runtime_string_concat(
        &mut self,
        left: RuntimeStringPart,
        right: RuntimeStringPart,
    ) -> Value {
        let len = self.runtime_string_part_len(&left) + self.runtime_string_part_len(&right);
        let idx = self.runtime_strings.len();
        self.runtime_strings
            .push(RuntimeString::concat(left, right, len));
        Value::string(Self::RUNTIME_STRING_OFFSET + idx as u16)
    }

    fn lookup_global_value(&self, name: &str) -> Option<Value> {
        let val = match name {
            "undefined" => Some(Value::undefined()),
            "NaN" => Some(Value::nan()),
            "Infinity" => Some(Value::infinity()),
            "Math" => Some(Value::builtin_object(BUILTIN_MATH)),
            "JSON" => Some(Value::builtin_object(BUILTIN_JSON)),
            "Number" => Some(Value::builtin_object(BUILTIN_NUMBER)),
            "Boolean" => Some(Value::builtin_object(BUILTIN_BOOLEAN)),
            "String" => Some(Value::builtin_object(BUILTIN_STRING)),
            "Object" => Some(Value::builtin_object(BUILTIN_OBJECT)),
            "Array" => Some(Value::builtin_object(BUILTIN_ARRAY)),
            "console" => Some(Value::builtin_object(BUILTIN_CONSOLE)),
            "performance" => Some(Value::builtin_object(BUILTIN_PERFORMANCE)),
            "Date" => Some(Value::builtin_object(BUILTIN_DATE)),
            "Error" => Some(Value::builtin_object(BUILTIN_ERROR)),
            "TypeError" => Some(Value::builtin_object(BUILTIN_TYPE_ERROR)),
            "ReferenceError" => Some(Value::builtin_object(BUILTIN_REFERENCE_ERROR)),
            "SyntaxError" => Some(Value::builtin_object(BUILTIN_SYNTAX_ERROR)),
            "RangeError" => Some(Value::builtin_object(BUILTIN_RANGE_ERROR)),
            "EvalError" => Some(Value::builtin_object(BUILTIN_EVAL_ERROR)),
            "URIError" => Some(Value::builtin_object(BUILTIN_URI_ERROR)),
            "InternalError" => Some(Value::builtin_object(BUILTIN_INTERNAL_ERROR)),
            "RegExp" => Some(Value::builtin_object(BUILTIN_REGEXP)),
            "globalThis" => Some(Value::builtin_object(BUILTIN_GLOBAL_THIS)),
            "ArrayBuffer" => Some(Value::builtin_object(BUILTIN_ARRAY_BUFFER)),
            "Int8Array" => Some(Value::builtin_object(BUILTIN_INT8_ARRAY)),
            "Uint8Array" => Some(Value::builtin_object(BUILTIN_UINT8_ARRAY)),
            "Uint8ClampedArray" => Some(Value::builtin_object(BUILTIN_UINT8_CLAMPED_ARRAY)),
            "Int16Array" => Some(Value::builtin_object(BUILTIN_INT16_ARRAY)),
            "Uint16Array" => Some(Value::builtin_object(BUILTIN_UINT16_ARRAY)),
            "Int32Array" => Some(Value::builtin_object(BUILTIN_INT32_ARRAY)),
            "Uint32Array" => Some(Value::builtin_object(BUILTIN_UINT32_ARRAY)),
            "Float32Array" => Some(Value::builtin_object(BUILTIN_FLOAT32_ARRAY)),
            "Float64Array" => Some(Value::builtin_object(BUILTIN_FLOAT64_ARRAY)),
            _ => self.get_native_func(name),
        };

        val.or_else(|| self.global_vars.get(name).copied())
    }

    /// Create a closure and return a Value that references it
    fn create_closure(
        &mut self,
        bytecode: *const FunctionBytecode,
        cell_indices: Vec<u32>,
    ) -> Value {
        self.maybe_gc();
        let (idx, is_new) = self.gc.alloc_slot(&mut self.gen_closures);
        if is_new {
            self.closures.push(ClosureData::new(bytecode, cell_indices));
        } else {
            self.closures[idx] = ClosureData::new(bytecode, cell_indices);
        }
        Value::closure_idx(idx as u32)
    }

    /// Allocate a new variable cell with the given initial value, return its index.
    fn alloc_var_cell(&mut self, value: Value) -> u32 {
        self.maybe_gc();
        let (idx, is_new) = self.gc.alloc_slot(&mut self.gen_var_cells);
        if is_new {
            self.var_cells.push(value);
        } else {
            self.var_cells[idx] = value;
        }
        idx as u32
    }

    /// Read raw bytes of a TypedArray value.
    ///
    /// Useful in native functions to efficiently extract buffer data.
    /// Returns `None` if the value is not a TypedArray or the index is invalid.
    pub fn read_typed_array(&self, value: Value) -> Option<&[u8]> {
        let idx = value.to_typed_array_idx()?;
        self.typed_arrays
            .get(idx as usize)
            .map(|ta| ta.data.as_slice())
    }

    /// Get a closure by index
    fn get_closure(&self, idx: u32) -> Option<&ClosureData> {
        self.closures.get(idx as usize)
    }

    /// Create an array and return a Value that references it
    fn create_array(&mut self, elements: Vec<Value>) -> Value {
        self.maybe_gc();
        let (idx, is_new) = self.gc.alloc_slot(&mut self.gen_arrays);
        if is_new {
            self.arrays.push(elements);
        } else {
            self.arrays[idx] = elements; // old Vec dropped automatically
        }
        Value::array_idx(idx as u32)
    }

    /// Get an array by index
    pub(crate) fn get_array(&self, idx: u32) -> Option<&Vec<Value>> {
        self.arrays.get(idx as usize)
    }

    /// Get an array by index without bounds checking
    ///
    /// # Safety
    /// Caller must ensure idx < self.arrays.len()
    #[inline]
    unsafe fn get_array_unchecked(&self, idx: u32) -> &Vec<Value> {
        debug_assert!((idx as usize) < self.arrays.len());
        unsafe { self.arrays.get_unchecked(idx as usize) }
    }

    #[inline]
    fn dense_array_access(arr: Value, idx: Value) -> Option<(u32, usize)> {
        let idx_raw = idx.raw().0;
        if (idx_raw & 1) != 0 {
            return None;
        }

        let index = (idx_raw as i64 >> 1) as i32;
        if index < 0 {
            return None;
        }

        let arr_raw = arr.raw().0;
        if (arr_raw & 0x1f) != crate::value::SpecialTag::CatchOffset as u64 {
            return None;
        }

        let tagged = (arr_raw >> 5) as i32;
        if (tagged & crate::value::ARRAY_INDEX_MARKER) == 0 {
            return None;
        }

        Some((
            (tagged & !crate::value::ARRAY_INDEX_MARKER) as u32,
            index as usize,
        ))
    }

    #[inline]
    fn is_dense_array_access(arr: Value, idx: Value) -> bool {
        let idx_raw = idx.raw().0;
        if (idx_raw & 1) != 0 {
            return false;
        }
        if (idx_raw as i64 >> 1) < 0 {
            return false;
        }

        let arr_raw = arr.raw().0;
        if (arr_raw & 0x1f) != crate::value::SpecialTag::CatchOffset as u64 {
            return false;
        }

        (((arr_raw >> 5) as i32) & crate::value::ARRAY_INDEX_MARKER) != 0
    }

    #[inline]
    pub(crate) fn array_element_or_undefined(&self, idx: u32, element: usize) -> Value {
        let arr = unsafe { self.get_array_unchecked(idx) };
        arr.get(element).copied().unwrap_or_default()
    }

    /// Get a mutable array by index
    fn get_array_mut(&mut self, idx: u32) -> Option<&mut Vec<Value>> {
        self.arrays.get_mut(idx as usize)
    }

    /// Get a mutable array by index without bounds checking
    ///
    /// # Safety
    /// Caller must ensure idx < self.arrays.len()
    #[inline]
    unsafe fn get_array_mut_unchecked(&mut self, idx: u32) -> &mut Vec<Value> {
        debug_assert!((idx as usize) < self.arrays.len());
        unsafe { self.arrays.get_unchecked_mut(idx as usize) }
    }

    /// Create a new object and return its value
    fn create_object(&mut self) -> Value {
        self.maybe_gc();
        let (idx, is_new) = self.gc.alloc_slot(&mut self.gen_objects);
        if is_new {
            self.objects.push(ObjectInstance::new());
        } else {
            self.objects[idx] = ObjectInstance::new();
        }
        Value::object_idx(idx as u32)
    }

    /// Create a new object with a constructor reference and return its value
    fn create_object_with_constructor(&mut self, constructor: Value) -> Value {
        self.maybe_gc();
        let (idx, is_new) = self.gc.alloc_slot(&mut self.gen_objects);
        if is_new {
            self.objects
                .push(ObjectInstance::with_constructor(constructor));
        } else {
            self.objects[idx] = ObjectInstance::with_constructor(constructor);
        }
        Value::object_idx(idx as u32)
    }

    /// Get an object by index
    pub(crate) fn get_object(&self, idx: u32) -> Option<&ObjectInstance> {
        self.objects.get(idx as usize)
    }

    /// Get a mutable object by index
    fn get_object_mut(&mut self, idx: u32) -> Option<&mut ObjectInstance> {
        self.objects.get_mut(idx as usize)
    }

    /// Get a property from an object
    #[inline]
    fn object_get_property(&mut self, obj_idx: u32, key: &str) -> InterpreterResult<Value> {
        if let Some(obj) = self.get_object(obj_idx) {
            match obj.properties.as_slice() {
                [(k0, v0)] => {
                    if k0 == key {
                        return Ok(*v0);
                    }
                }
                [(k0, v0), (k1, v1)] => {
                    if k0 == key {
                        return Ok(*v0);
                    }
                    if k1 == key {
                        return Ok(*v1);
                    }
                }
                _ => {
                    for (k, v) in obj.properties.iter() {
                        if k == key {
                            return Ok(*v);
                        }
                    }
                }
            }

            if let Some(accessor) = obj.accessors.iter().find(|a| a.key == key).cloned() {
                if accessor.getter.is_undefined() {
                    return Ok(Value::undefined());
                }
                let this_val = Value::object_idx(obj_idx);
                return self.call_value(accessor.getter, this_val, &[]);
            }
        }
        // Fallback to Object.prototype methods
        Ok(match key {
            "hasOwnProperty" => self
                .get_native_func("Object.prototype.hasOwnProperty")
                .unwrap_or_default(),
            "toString" => self
                .get_native_func("Object.prototype.toString")
                .unwrap_or_default(),
            _ => Value::undefined(),
        })
    }

    #[inline]
    fn for_of_next_value(&mut self, iter_idx: usize) -> Option<Value> {
        let iter = self.for_of_iterators.get_mut(iter_idx)?;
        match iter {
            ForOfIterator::Array { arr_idx, index } => {
                let value = self
                    .arrays
                    .get(*arr_idx as usize)
                    .and_then(|arr| arr.get(*index).copied());
                if value.is_some() {
                    *index += 1;
                }
                value
            }
            ForOfIterator::Values { values, index } => {
                let value = values.get(*index).copied();
                if value.is_some() {
                    *index += 1;
                }
                value
            }
        }
    }

    #[inline]
    fn for_in_next_key(&mut self, iter_idx: usize) -> Option<Value> {
        let iter = self.for_in_iterators.get_mut(iter_idx)?;
        match iter {
            ForInIterator::ObjectKeys { keys, index } => {
                if *index >= keys.len() {
                    return None;
                }
                let key = keys.get(*index).copied();
                *index += 1;
                key
            }
            ForInIterator::Array { len, index } => {
                let current = if *index >= *len {
                    None
                } else {
                    let current = *index;
                    *index += 1;
                    Some(current)
                };
                current.and_then(|i| self.create_runtime_string_for_in_key(&i.to_string()).ok())
            }
            ForInIterator::Empty => None,
        }
    }

    #[inline]
    fn get_field_value(&mut self, obj: Value, prop_name: &str) -> InterpreterResult<Value> {
        if obj.is_array() {
            let arr_val = self.get_array_property(obj, prop_name);
            // If not a special property, try numeric index on dense array
            if arr_val.is_undefined() {
                if let Some(arr_idx) = obj.to_array_idx() {
                    if let Ok(index) = prop_name.parse::<usize>() {
                        if let Some(arr_data) = self.arrays.get(arr_idx as usize) {
                            return Ok(arr_data.get(index).copied().unwrap_or_default());
                        }
                    }
                }
            }
            Ok(arr_val)
        } else if let Some(obj_idx) = obj.to_object_idx() {
            self.object_get_property(obj_idx, prop_name)
        } else if let Some(builtin_idx) = obj.to_builtin_object_idx() {
            Ok(self.get_builtin_property(builtin_idx, prop_name))
        } else if let Some(typed_idx) = obj.to_typed_array_idx() {
            let ta_val = self.get_typed_array_property(typed_idx, prop_name);
            // If not a special property, try numeric index on typed array
            if ta_val.is_undefined() {
                if let Ok(index) = prop_name.parse::<usize>() {
                    if let Some(ta) = self.typed_arrays.get(typed_idx as usize) {
                        return Ok(ta.get(index).unwrap_or_default());
                    }
                }
            }
            Ok(ta_val)
        } else if let Some(ab_idx) = obj.to_array_buffer_idx() {
            Ok(self.get_array_buffer_property(ab_idx, prop_name))
        } else if let Some(err_idx) = obj.to_error_object_idx() {
            Ok(self.get_error_property(err_idx, prop_name))
        } else if let Some(regex_idx) = obj.to_regexp_object_idx() {
            Ok(self.get_regexp_property(regex_idx, prop_name))
        } else if obj.is_string() {
            Ok(self.get_string_property(obj, prop_name))
        } else if obj.is_number() {
            Ok(self.get_number_property(obj, prop_name))
        } else if obj.is_closure() || obj.to_func_ptr().is_some() {
            Ok(self.get_function_property(prop_name))
        } else {
            Ok(Value::undefined())
        }
    }

    #[inline]
    fn get_length_value(&mut self, obj: Value) -> Value {
        if let Some(arr_idx) = obj.to_array_idx() {
            self.get_array(arr_idx)
                .map(|arr| Value::int(arr.len() as i32))
                .unwrap_or_default()
        } else if let Some(typed_idx) = obj.to_typed_array_idx() {
            self.typed_arrays
                .get(typed_idx as usize)
                .map(|ta| Value::int(ta.length as i32))
                .unwrap_or_default()
        } else if obj.is_string() {
            obj.to_string_idx()
                .map(|idx| {
                    if idx >= Self::RUNTIME_STRING_OFFSET {
                        let runtime_idx = (idx - Self::RUNTIME_STRING_OFFSET) as usize;
                        self.runtime_strings
                            .get(runtime_idx)
                            .and_then(|_| self.runtime_string_as_str(runtime_idx))
                            .map(|s| Value::int(utf16_len(s) as i32))
                            .unwrap_or(Value::int(0))
                    } else {
                        self.get_string_by_idx(idx)
                            .map(|s| Value::int(utf16_len(s) as i32))
                            .unwrap_or(Value::int(0))
                    }
                })
                .unwrap_or(Value::int(0))
        } else {
            self.get_field_value(obj, "length").unwrap_or_default()
        }
    }

    #[inline]
    fn store_local_slot(&mut self, idx: usize, val: Value) {
        if idx == 0 {
            if let Some(frame) = self.call_stack.last_mut() {
                frame.local0_string_builder = None;
            }
        }
        let Some(frame) = self.call_stack.last() else {
            return;
        };
        let cell = if idx == 0 {
            frame.local0_cell
        } else {
            frame
                .local_cells
                .as_ref()
                .and_then(|lc| lc.get(idx).copied().flatten())
        };
        let frame_ptr = frame.frame_ptr;
        if let Some(cell_idx) = cell {
            self.var_cells[cell_idx as usize] = val;
        } else {
            self.stack.set_local_at(frame_ptr, idx, val);
        }
    }

    #[inline]
    fn load_local_slot(&mut self, idx: usize) -> Value {
        if idx == 0 {
            let should_materialize = self
                .call_stack
                .last()
                .is_some_and(|frame| frame.local0_string_builder.is_some());
            if should_materialize {
                if let Some(val) = self.materialize_local0_string_builder() {
                    return val;
                }
            }
        }

        let frame = self.call_stack.last().unwrap();
        let cell = if idx == 0 {
            frame.local0_cell
        } else {
            frame
                .local_cells
                .as_ref()
                .and_then(|lc| lc.get(idx).copied().flatten())
        };
        let frame_ptr = frame.frame_ptr;
        if let Some(cell_idx) = cell {
            self.var_cells[cell_idx as usize]
        } else {
            unsafe { self.stack.get_local_at_unchecked(frame_ptr, idx) }
        }
    }

    #[inline]
    fn ensure_captured_local_cell(&mut self, idx: usize) -> u32 {
        if idx == 0 {
            let should_materialize = self
                .call_stack
                .last()
                .is_some_and(|frame| frame.local0_string_builder.is_some());
            if should_materialize {
                let _ = self.materialize_local0_string_builder();
            }
        }

        let existing_cell = {
            let frame = self.call_stack.last().unwrap();
            frame
                .local_cells
                .as_ref()
                .and_then(|lc| lc.get(idx).copied().flatten())
                .or(if idx == 0 { frame.local0_cell } else { None })
        };
        if let Some(cell_idx) = existing_cell {
            return cell_idx;
        }

        let val = self.load_local_slot(idx);
        let cell_idx = self.alloc_var_cell(val);

        let frame = self.call_stack.last_mut().unwrap();
        let local_count = unsafe { (*frame.bytecode).local_count as usize };
        let local_cells = frame
            .local_cells
            .get_or_insert_with(|| vec![None; local_count]);
        if idx < local_cells.len() {
            local_cells[idx] = Some(cell_idx);
        }
        if idx == 0 {
            frame.local0_cell = Some(cell_idx);
        }

        cell_idx
    }

    #[inline]
    fn try_inc_local_slot_discard(&mut self, idx: usize) -> InterpreterResult<bool> {
        if idx != 0 {
            let (cell, frame_ptr, bytecode_ptr) = {
                let frame = self.call_stack.last().unwrap();
                let cell = frame
                    .local_cells
                    .as_ref()
                    .and_then(|lc| lc.get(idx).copied().flatten());
                (cell, frame.frame_ptr, frame.bytecode)
            };

            let val = if let Some(cell_idx) = cell {
                self.var_cells[cell_idx as usize]
            } else {
                unsafe { self.stack.get_local_at_unchecked(frame_ptr, idx) }
            };

            if val.is_string() {
                let bytecode = unsafe { &*bytecode_ptr };
                let mut out = if let Some(s) = self.get_string_content(val, bytecode) {
                    String::with_capacity(s.len() + 1)
                } else {
                    String::with_capacity(self.value_to_string_len_hint(val, bytecode) + 1)
                };
                self.append_value_to_string(&mut out, val, bytecode);
                out.push('1');
                self.bump_runtime_string_concat();
                let result = self.create_runtime_string_raw(out);
                if let Some(cell_idx) = cell {
                    self.var_cells[cell_idx as usize] = result;
                } else {
                    self.stack.set_local_at(frame_ptr, idx, result);
                }
                return Ok(true);
            }
        }

        let val = self.load_local_slot(idx);
        if val.is_string() {
            let frame = self.call_stack.last().unwrap();
            let bytecode = unsafe { &*frame.bytecode };
            let mut out = if let Some(s) = self.get_string_content(val, bytecode) {
                String::with_capacity(s.len() + 1)
            } else {
                String::with_capacity(self.value_to_string_len_hint(val, bytecode) + 1)
            };
            self.append_value_to_string(&mut out, val, bytecode);
            out.push('1');
            self.bump_runtime_string_concat();
            let result = self.create_runtime_string_raw(out);
            self.store_local_slot(idx, result);
            return Ok(true);
        }
        match self.try_op(self.op_add(val, Value::int(1)))? {
            Some(result) => {
                self.store_local_slot(idx, result);
                Ok(true)
            }
            None => Ok(false),
        }
    }

    #[inline]
    fn materialize_local0_string_builder(&mut self) -> Option<Value> {
        let (cell, frame_ptr, builder) = {
            let frame = self.call_stack.last_mut()?;
            let builder = frame.local0_string_builder.take()?;
            let cell = frame.local0_cell;
            (cell, frame.frame_ptr, builder)
        };

        self.bump_runtime_string_concat();
        let val = self.create_runtime_string_raw(builder);
        if let Some(cell_idx) = cell {
            self.var_cells[cell_idx as usize] = val;
        } else {
            self.stack.set_local_at(frame_ptr, 0, val);
        }
        Some(val)
    }

    #[inline]
    fn try_consume_sieve_style_local_update(&mut self, val: Value) -> bool {
        let Some(frame) = self.call_stack.last_mut() else {
            return false;
        };
        let bytecode = unsafe { &*frame.bytecode };
        let bc = &bytecode.bytecode;
        let pc = frame.pc;

        // Hot local-update shapes:
        //   Add; Dup; PutLoc0; Drop
        //   Add; Dup; PutLoc1; Drop
        //   Add; Dup; PutLoc3; Drop
        //   Add; Dup; PutLoc8 4; Drop
        let matched = if bc.get(pc).copied() == Some(OpCode::Dup as u8)
            && bc.get(pc + 1).copied() == Some(OpCode::PutLoc0 as u8)
            && bc.get(pc + 2).copied() == Some(OpCode::Drop as u8)
        {
            frame.pc += 3;
            Some(0usize)
        } else if bc.get(pc).copied() == Some(OpCode::Dup as u8)
            && bc.get(pc + 1).copied() == Some(OpCode::PutLoc1 as u8)
            && bc.get(pc + 2).copied() == Some(OpCode::Drop as u8)
        {
            frame.pc += 3;
            Some(1usize)
        } else if bc.get(pc).copied() == Some(OpCode::Dup as u8)
            && bc.get(pc + 1).copied() == Some(OpCode::PutLoc3 as u8)
            && bc.get(pc + 2).copied() == Some(OpCode::Drop as u8)
        {
            frame.pc += 3;
            Some(3usize)
        } else if bc.get(pc).copied() == Some(OpCode::Dup as u8)
            && bc.get(pc + 1).copied() == Some(OpCode::PutLoc8 as u8)
            && bc.get(pc + 2).copied() == Some(4)
            && bc.get(pc + 3).copied() == Some(OpCode::Drop as u8)
        {
            frame.pc += 4;
            Some(4usize)
        } else {
            None
        };

        if let Some(idx) = matched {
            self.store_local_slot(idx, val);
            true
        } else {
            false
        }
    }

    #[inline]
    fn try_consume_statement_local_store(&mut self, val: Value) -> bool {
        let Some(frame) = self.call_stack.last_mut() else {
            return false;
        };
        let bytecode = unsafe { &*frame.bytecode };
        let bc = &bytecode.bytecode;
        let pc = frame.pc;

        let target = match bc.get(pc).copied() {
            Some(x) if x == OpCode::PutLoc0 as u8 => {
                frame.pc += 1;
                Some(0usize)
            }
            Some(x) if x == OpCode::PutLoc1 as u8 => {
                frame.pc += 1;
                Some(1usize)
            }
            Some(x) if x == OpCode::PutLoc2 as u8 => {
                frame.pc += 1;
                Some(2usize)
            }
            Some(x) if x == OpCode::PutLoc3 as u8 => {
                frame.pc += 1;
                Some(3usize)
            }
            Some(x) if x == OpCode::PutLoc4 as u8 => {
                frame.pc += 1;
                Some(4usize)
            }
            Some(x) if x == OpCode::PutLoc8 as u8 => {
                let idx = bc.get(pc + 1).copied().map(|x| x as usize);
                if let Some(idx) = idx {
                    frame.pc += 2;
                    Some(idx)
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some(idx) = target {
            self.store_local_slot(idx, val);
            true
        } else {
            false
        }
    }

    /// Set a property on an object
    pub(crate) fn object_set_property(
        &mut self,
        obj_idx: u32,
        key: String,
        value: Value,
    ) -> InterpreterResult<()> {
        if let Some(obj) = self.get_object_mut(obj_idx) {
            // Check if property already exists
            for (k, v) in obj.properties.iter_mut() {
                if k == &key {
                    *v = value;
                    return Ok(());
                }
            }

            if let Some(accessor) = obj.accessors.iter().find(|a| a.key == key).cloned() {
                if accessor.setter.is_undefined() {
                    return Ok(());
                }
                let this_val = Value::object_idx(obj_idx);
                let _ = self.call_value(accessor.setter, this_val, &[value])?;
                return Ok(());
            }

            obj.properties.push((key, value));
        }
        Ok(())
    }

    pub(crate) fn object_define_accessor(
        &mut self,
        obj_idx: u32,
        key: String,
        getter: Value,
        setter: Value,
    ) {
        if let Some(obj) = self.get_object_mut(obj_idx) {
            obj.properties.retain(|(k, _)| k != &key);
            if let Some(existing) = obj.accessors.iter_mut().find(|a| a.key == key) {
                existing.getter = getter;
                existing.setter = setter;
            } else {
                obj.accessors.push(ObjectAccessor {
                    key,
                    getter,
                    setter,
                });
            }
        }
    }

    /// Get a mutable closure by index
    fn get_closure_mut(&mut self, idx: u32) -> Option<&mut ClosureData> {
        self.closures.get_mut(idx as usize)
    }

    /// Call a function value with the given `this` value and arguments
    ///
    /// This handles closures, function pointers, and function indices.
    pub fn call_value(
        &mut self,
        func: Value,
        this_val: Value,
        args: &[Value],
    ) -> InterpreterResult<Value> {
        // Save current call stack depth to return when we're back to this level
        let saved_target = self.nested_call_target_depth;
        self.nested_call_target_depth = Some(self.call_stack.len());

        let result = self.call_value_inner(func, this_val, args);

        // Restore the previous target depth
        self.nested_call_target_depth = saved_target;

        result
    }

    /// Inner implementation of call_value
    fn call_value_inner(
        &mut self,
        func: Value,
        this_val: Value,
        args: &[Value],
    ) -> InterpreterResult<Value> {
        // Handle closures
        if let Some(closure_idx) = func.to_closure_idx() {
            let closure = self.get_closure(closure_idx).ok_or_else(|| {
                InterpreterError::InternalError(format!("invalid closure index: {}", closure_idx))
            })?;
            let bytecode = unsafe { &*closure.bytecode };

            // Check recursion limit
            if self.call_stack.len() >= self.max_recursion {
                self.try_handle_runtime_error(InterpreterError::InternalError(
                    "maximum call stack size exceeded".to_string(),
                ))?;
                return Ok(Value::undefined());
            }

            let frame_ptr = self.stack.len();

            // Push arguments (pad with undefined if needed)
            for i in 0..bytecode.arg_count as usize {
                let arg = args.get(i).copied().unwrap_or_default();
                self.stack.push(arg);
            }

            // Allocate space for locals (beyond arguments)
            let extra_locals = bytecode.local_count.saturating_sub(bytecode.arg_count);
            for _ in 0..extra_locals {
                self.stack.push(Value::undefined());
            }

            // Create frame with closure
            let frame = CallFrame::new_closure(
                bytecode as *const _,
                frame_ptr,
                args.len().min(u16::MAX as usize) as u16,
                this_val,
                func,
                closure_idx as usize,
            );
            self.call_stack.push(frame);

            // Run the interpreter loop
            return self.run();
        }

        // Handle function pointers
        if let Some(ptr) = func.to_func_ptr() {
            let bytecode = unsafe { &*ptr };
            return self.call_function(bytecode, this_val, args);
        }

        self.try_handle_runtime_error(InterpreterError::TypeError("not a function".to_string()))?;
        Ok(Value::undefined())
    }

    /// Execute bytecode and return the result
    ///
    /// # Safety
    /// The bytecode pointer must be valid for the duration of execution.
    pub fn execute(&mut self, bytecode: &FunctionBytecode) -> InterpreterResult<Value> {
        self.maybe_gc();
        self.call_function(bytecode, Value::undefined(), &[])
    }

    #[cfg(feature = "dump")]
    pub fn reset_opcode_counts(&mut self) {
        self.opcode_counts = [0; 256];
    }

    #[cfg(feature = "dump")]
    pub fn opcode_counts(&self) -> &[u64; 256] {
        &self.opcode_counts
    }

    /// Call a function with the given `this` value and arguments
    pub fn call_function(
        &mut self,
        bytecode: &FunctionBytecode,
        this_val: Value,
        args: &[Value],
    ) -> InterpreterResult<Value> {
        // Check if GC should run
        self.maybe_gc();

        // Check recursion limit
        if self.call_stack.len() >= self.max_recursion {
            self.try_handle_runtime_error(InterpreterError::InternalError(
                "maximum call stack size exceeded".to_string(),
            ))?;
            return Ok(Value::undefined());
        }

        let frame_ptr = self.stack.len();

        // Push arguments (pad with undefined if needed)
        for i in 0..bytecode.arg_count as usize {
            let arg = args.get(i).copied().unwrap_or_default();
            self.stack.push(arg);
        }

        // Allocate space for locals (beyond arguments)
        let extra_locals = bytecode.local_count.saturating_sub(bytecode.arg_count);
        for _ in 0..extra_locals {
            self.stack.push(Value::undefined());
        }

        let frame = CallFrame::new(
            bytecode as *const _,
            frame_ptr,
            args.len().min(u16::MAX as usize) as u16,
            this_val,
            Value::undefined(), // Top-level call has no function value
        );
        self.call_stack.push(frame);

        // Run the interpreter loop
        self.run()
    }

    /// Try to route a runtime error through JS exception handlers.
    /// Returns Ok(()) if the error was caught (state has been updated to jump to catch block).
    /// Returns the original Err if no handler is available.
    fn try_handle_runtime_error(&mut self, err: InterpreterError) -> InterpreterResult<()> {
        if self.exception_handlers.is_empty() {
            return Err(err);
        }

        // Create an error object for the JS exception
        let (name, message) = match &err {
            InterpreterError::TypeError(msg) => ("TypeError".to_string(), msg.clone()),
            InterpreterError::ReferenceError(msg) => ("ReferenceError".to_string(), msg.clone()),
            InterpreterError::RangeError(msg) => ("RangeError".to_string(), msg.clone()),
            InterpreterError::InternalError(msg) => ("InternalError".to_string(), msg.clone()),
            InterpreterError::DivisionByZero => {
                ("RangeError".to_string(), "division by zero".to_string())
            }
            InterpreterError::StackOverflow => {
                ("InternalError".to_string(), "stack overflow".to_string())
            }
            _ => ("Error".to_string(), err.to_string()),
        };

        let err_obj = ErrorObject { name, message };
        self.maybe_gc();
        let (err_idx, is_new) = self.gc.alloc_slot(&mut self.gen_error_objects);
        if is_new {
            self.error_objects.push(err_obj);
        } else {
            self.error_objects[err_idx] = err_obj;
        }
        let exception = Value::error_object(err_idx as u32);

        self.route_exception_to_handler(exception)
    }

    #[inline]
    fn route_exception_to_handler(&mut self, exception: Value) -> InterpreterResult<()> {
        let handler = match self.exception_handlers.pop() {
            Some(handler) => handler,
            None => {
                let msg = format_value(self, exception);
                return Err(InterpreterError::UncaughtException(msg));
            }
        };

        self.call_stack.truncate(handler.frame_depth);
        self.stack
            .drop_n(self.stack.len().saturating_sub(handler.stack_depth));
        self.stack.push(exception);

        if let Some(frame) = self.call_stack.last_mut() {
            frame.pc = handler.catch_pc;
            Ok(())
        } else {
            let msg = format_value(self, exception);
            Err(InterpreterError::UncaughtException(msg))
        }
    }

    /// Helper: execute a fallible operation, routing errors through the
    /// exception handler so that try/catch can intercept runtime errors
    /// (TypeError, DivisionByZero, etc.) instead of letting them escape.
    /// Returns `Some(value)` on success, or `None` when an exception handler
    /// caught the error (the caller should `continue` the dispatch loop).
    fn try_op(&mut self, result: InterpreterResult<Value>) -> InterpreterResult<Option<Value>> {
        match result {
            Ok(v) => Ok(Some(v)),
            Err(e) => {
                self.try_handle_runtime_error(e)?;
                Ok(None)
            }
        }
    }

    /// Main interpreter loop
    fn run(&mut self) -> InterpreterResult<Value> {
        loop {
            // Get current frame
            let frame = self.call_stack.last_mut().ok_or_else(|| {
                InterpreterError::InternalError("no active call frame".to_string())
            })?;

            // Safety: bytecode pointer is valid for frame lifetime
            let bytecode = unsafe { &*frame.bytecode };
            let bc = &bytecode.bytecode;

            // Set current string constants for native functions to access
            self.current_string_constants = Some(&bytecode.string_constants as *const _);

            // Check if we've reached the end
            if frame.pc >= bc.len() {
                // Implicit return undefined
                return Ok(Value::undefined());
            }

            // Fetch opcode
            let opcode_byte = bc[frame.pc];
            frame.pc += 1;

            #[cfg(feature = "dump")]
            {
                self.opcode_counts[opcode_byte as usize] += 1;
            }

            // Decode and execute
            match opcode_byte {
                // Invalid
                op if op == OpCode::Invalid as u8 => {
                    return Err(InterpreterError::InvalidOpcode(op));
                }

                // Push integer constants
                op if op == OpCode::PushMinus1 as u8 => {
                    self.stack.push(Value::int(-1));
                }
                op if op == OpCode::Push0 as u8 => {
                    self.stack.push(Value::int(0));
                }
                op if op == OpCode::Push1 as u8 => {
                    self.stack.push(Value::int(1));
                }
                op if op == OpCode::Push2 as u8 => {
                    self.stack.push(Value::int(2));
                }
                op if op == OpCode::Push3 as u8 => {
                    self.stack.push(Value::int(3));
                }
                op if op == OpCode::Push4 as u8 => {
                    self.stack.push(Value::int(4));
                }
                op if op == OpCode::Push5 as u8 => {
                    self.stack.push(Value::int(5));
                }
                op if op == OpCode::Push6 as u8 => {
                    self.stack.push(Value::int(6));
                }
                op if op == OpCode::Push7 as u8 => {
                    self.stack.push(Value::int(7));
                }

                // Push 8-bit signed integer
                op if op == OpCode::PushI8 as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let val = bytecode.bytecode[frame.pc] as i8 as i32;
                    frame.pc += 1;
                    self.stack.push(Value::int(val));
                }

                // Push 16-bit signed integer
                op if op == OpCode::PushI16 as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let val = i16::from_le_bytes([bc[frame.pc], bc[frame.pc + 1]]) as i32;
                    frame.pc += 2;
                    self.stack.push(Value::int(val));
                }

                // Push constant from pool
                op if op == OpCode::PushConst as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let idx = u16::from_le_bytes([bc[frame.pc], bc[frame.pc + 1]]) as usize;
                    frame.pc += 2;
                    let val = bytecode.constants.get(idx).copied().unwrap_or_default();
                    self.stack.push(val);
                }

                // Push constant (8-bit index)
                op if op == OpCode::PushConst8 as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let idx = bytecode.bytecode[frame.pc] as usize;
                    frame.pc += 1;
                    let val = bytecode.constants.get(idx).copied().unwrap_or_default();
                    self.stack.push(val);
                }

                // Push undefined
                op if op == OpCode::Undefined as u8 => {
                    self.stack.push(Value::undefined());
                }

                // Push null
                op if op == OpCode::Null as u8 => {
                    self.stack.push(Value::null());
                }

                // Push false
                op if op == OpCode::PushFalse as u8 => {
                    self.stack.push(Value::bool(false));
                }

                // Push true
                op if op == OpCode::PushTrue as u8 => {
                    self.stack.push(Value::bool(true));
                }

                // Create empty object (Object opcode)
                op if op == OpCode::Object as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    // Skip the 16-bit class id (unused for now)
                    let _class_id = u16::from_le_bytes([bc[frame.pc], bc[frame.pc + 1]]);
                    frame.pc += 2;
                    let obj = self.create_object();
                    self.stack.push(obj);
                }

                // Push empty string
                op if op == OpCode::PushEmptyString as u8 => {
                    self.stack.push(Value::string(crate::value::STR_EMPTY));
                }

                // Stack manipulation: Drop
                op if op == OpCode::Drop as u8 => {
                    self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                }

                // Stack manipulation: Dup
                op if op == OpCode::Dup as u8 => {
                    self.stack.dup().ok_or(InterpreterError::StackUnderflow)?;
                }

                // Stack manipulation: Swap
                op if op == OpCode::Swap as u8 => {
                    self.stack.swap().ok_or(InterpreterError::StackUnderflow)?;
                }

                // Get local variable (16-bit index)
                op if op == OpCode::GetLoc as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let idx = u16::from_le_bytes([bc[frame.pc], bc[frame.pc + 1]]) as usize;
                    frame.pc += 2;
                    let cell = frame
                        .local_cells
                        .as_ref()
                        .and_then(|lc| lc.get(idx).copied().flatten());
                    let frame_ptr = frame.frame_ptr;
                    let val = if let Some(cell_idx) = cell {
                        self.var_cells[cell_idx as usize]
                    } else {
                        self.stack.get_local_at(frame_ptr, idx).unwrap_or_default()
                    };
                    self.stack.push(val);
                }

                // Set local variable (16-bit index)
                op if op == OpCode::PutLoc as u8 => {
                    let val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let idx = {
                        let frame = self.call_stack.last_mut().unwrap();
                        let bytecode = unsafe { &*frame.bytecode };
                        let bc = &bytecode.bytecode;
                        let idx = u16::from_le_bytes([bc[frame.pc], bc[frame.pc + 1]]) as usize;
                        frame.pc += 2;
                        idx
                    };
                    self.store_local_slot(idx, val);
                }

                // Get local 0-3 (optimized)
                op if op == OpCode::GetLoc0 as u8 => {
                    let should_materialize = self
                        .call_stack
                        .last()
                        .is_some_and(|frame| frame.local0_string_builder.is_some());
                    if should_materialize {
                        if let Some(val) = self.materialize_local0_string_builder() {
                            self.stack.push(val);
                            continue;
                        }
                    }
                    let frame = self.call_stack.last().unwrap();
                    let cell = frame
                        .local_cells
                        .as_ref()
                        .and_then(|lc| lc.first().copied().flatten());
                    let frame_ptr = frame.frame_ptr;
                    let val = if let Some(cell_idx) = cell {
                        self.var_cells[cell_idx as usize]
                    } else {
                        unsafe { self.stack.get_local_at_unchecked(frame_ptr, 0) }
                    };
                    self.stack.push(val);
                }

                op if op == OpCode::AppendConstStringToLoc0 as u8 => {
                    let (str_idx, bytecode_ptr) = {
                        let frame = self.call_stack.last_mut().unwrap();
                        let bytecode = unsafe { &*frame.bytecode };
                        let bc = &bytecode.bytecode;
                        let str_idx = u16::from_le_bytes([bc[frame.pc], bc[frame.pc + 1]]);
                        frame.pc += 2;
                        (str_idx, frame.bytecode)
                    };
                    let bytecode = unsafe { &*bytecode_ptr };
                    let suffix = self
                        .get_const_string_content(bytecode, str_idx)
                        .unwrap_or_default()
                        .to_string();

                    {
                        let frame = self.call_stack.last_mut().unwrap();
                        if let Some(buf) = frame.local0_string_builder.as_mut() {
                            buf.push_str(&suffix);
                            continue;
                        }
                    }

                    let lhs = if let Some(val) = self.materialize_local0_string_builder() {
                        val
                    } else {
                        let frame = self.call_stack.last().unwrap();
                        let cell = frame
                            .local_cells
                            .as_ref()
                            .and_then(|lc| lc.first().copied().flatten());
                        let frame_ptr = frame.frame_ptr;
                        if let Some(cell_idx) = cell {
                            self.var_cells[cell_idx as usize]
                        } else {
                            unsafe { self.stack.get_local_at_unchecked(frame_ptr, 0) }
                        }
                    };

                    let mut out = if let Some(lhs_str) = self.get_string_content(lhs, bytecode) {
                        String::with_capacity(lhs_str.len() + suffix.len())
                    } else if let Some(n) = lhs.to_i32() {
                        let mut buf = [0u8; 16];
                        let len = i32_to_str(&mut buf, n);
                        String::with_capacity(len + suffix.len())
                    } else {
                        String::with_capacity(
                            self.value_to_string_len_hint(lhs, bytecode) + suffix.len(),
                        )
                    };
                    self.append_value_to_string(&mut out, lhs, bytecode);
                    out.push_str(&suffix);
                    let frame = self.call_stack.last_mut().unwrap();
                    frame.local0_string_builder = Some(out);
                }
                op if op == OpCode::GetLoc1 as u8 => {
                    let frame = self.call_stack.last().unwrap();
                    let val = if frame.local_cells.is_none() {
                        unsafe { self.stack.get_local_at_unchecked(frame.frame_ptr, 1) }
                    } else {
                        self.load_local_slot(1)
                    };
                    self.stack.push(val);
                }
                op if op == OpCode::GetLoc2 as u8 => {
                    let frame = self.call_stack.last().unwrap();
                    let val = if frame.local_cells.is_none() {
                        unsafe { self.stack.get_local_at_unchecked(frame.frame_ptr, 2) }
                    } else {
                        self.load_local_slot(2)
                    };
                    self.stack.push(val);
                }
                op if op == OpCode::GetLoc3 as u8 => {
                    let frame = self.call_stack.last().unwrap();
                    let val = if frame.local_cells.is_none() {
                        unsafe { self.stack.get_local_at_unchecked(frame.frame_ptr, 3) }
                    } else {
                        self.load_local_slot(3)
                    };
                    self.stack.push(val);
                }
                op if op == OpCode::GetLoc4 as u8 => {
                    let frame = self.call_stack.last().unwrap();
                    let val = if frame.local_cells.is_none() {
                        unsafe { self.stack.get_local_at_unchecked(frame.frame_ptr, 4) }
                    } else {
                        self.load_local_slot(4)
                    };
                    self.stack.push(val);
                }

                // Set local 0-3 (optimized)
                op if op == OpCode::PutLoc0 as u8 => {
                    let val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    self.store_local_slot(0, val);
                }
                op if op == OpCode::PutLoc1 as u8 => {
                    let val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    self.store_local_slot(1, val);
                }
                op if op == OpCode::PutLoc2 as u8 => {
                    let val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    self.store_local_slot(2, val);
                }
                op if op == OpCode::PutLoc3 as u8 => {
                    let val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    self.store_local_slot(3, val);
                }
                op if op == OpCode::PutLoc4 as u8 => {
                    let val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    self.store_local_slot(4, val);
                }

                op if op == OpCode::IncLoc8Drop as u8 => {
                    let idx = {
                        let frame = self.call_stack.last_mut().unwrap();
                        let bytecode = unsafe { &*frame.bytecode };
                        let idx = bytecode.bytecode[frame.pc] as usize;
                        frame.pc += 1;
                        idx
                    };
                    if !self.try_inc_local_slot_discard(idx)? {
                        continue;
                    }
                }
                op if op == OpCode::IncLoc0Drop as u8 => {
                    if !self.try_inc_local_slot_discard(0)? {
                        continue;
                    }
                }
                op if op == OpCode::IncLoc1Drop as u8 => {
                    if !self.try_inc_local_slot_discard(1)? {
                        continue;
                    }
                }
                op if op == OpCode::IncLoc2Drop as u8 => {
                    if !self.try_inc_local_slot_discard(2)? {
                        continue;
                    }
                }
                op if op == OpCode::IncLoc3Drop as u8 => {
                    if !self.try_inc_local_slot_discard(3)? {
                        continue;
                    }
                }
                op if op == OpCode::IncLoc4Drop as u8 => {
                    if !self.try_inc_local_slot_discard(4)? {
                        continue;
                    }
                }

                // Get local (8-bit index)
                op if op == OpCode::GetLoc8 as u8 => {
                    let idx = {
                        let frame = self.call_stack.last_mut().unwrap();
                        let bytecode = unsafe { &*frame.bytecode };
                        let idx = bytecode.bytecode[frame.pc] as usize;
                        frame.pc += 1;
                        idx
                    };
                    let frame = self.call_stack.last().unwrap();
                    let cell = frame
                        .local_cells
                        .as_ref()
                        .and_then(|lc| lc.get(idx).copied().flatten());
                    let frame_ptr = frame.frame_ptr;
                    let val = if let Some(cell_idx) = cell {
                        self.var_cells[cell_idx as usize]
                    } else {
                        self.stack.get_local_at(frame_ptr, idx).unwrap_or_default()
                    };
                    self.stack.push(val);
                }

                // Set local (8-bit index)
                op if op == OpCode::PutLoc8 as u8 => {
                    let val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let idx = {
                        let frame = self.call_stack.last_mut().unwrap();
                        let bytecode = unsafe { &*frame.bytecode };
                        let idx = bytecode.bytecode[frame.pc] as usize;
                        frame.pc += 1;
                        idx
                    };
                    self.store_local_slot(idx, val);
                }

                // Get argument (16-bit index)
                op if op == OpCode::GetArg as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let idx = u16::from_le_bytes([bc[frame.pc], bc[frame.pc + 1]]) as usize;
                    frame.pc += 2;
                    let frame_ptr = frame.frame_ptr;
                    // Arguments are at the start of the frame
                    let val = self.stack.get_local_at(frame_ptr, idx).unwrap_or_default();
                    self.stack.push(val);
                }

                // Set argument (16-bit index)
                op if op == OpCode::PutArg as u8 => {
                    let val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let idx = u16::from_le_bytes([bc[frame.pc], bc[frame.pc + 1]]) as usize;
                    frame.pc += 2;
                    let frame_ptr = frame.frame_ptr;
                    self.stack.set_local_at(frame_ptr, idx, val);
                }

                // Get argument 0-3 (optimized)
                op if op == OpCode::GetArg0 as u8 => {
                    let frame = self.call_stack.last().unwrap();
                    let frame_ptr = frame.frame_ptr;
                    let val = self.stack.get_local_at(frame_ptr, 0).unwrap_or_default();
                    self.stack.push(val);
                }
                op if op == OpCode::GetArg1 as u8 => {
                    let frame = self.call_stack.last().unwrap();
                    let frame_ptr = frame.frame_ptr;
                    let val = self.stack.get_local_at(frame_ptr, 1).unwrap_or_default();
                    self.stack.push(val);
                }
                op if op == OpCode::GetArg2 as u8 => {
                    let frame = self.call_stack.last().unwrap();
                    let frame_ptr = frame.frame_ptr;
                    let val = self.stack.get_local_at(frame_ptr, 2).unwrap_or_default();
                    self.stack.push(val);
                }
                op if op == OpCode::GetArg3 as u8 => {
                    let frame = self.call_stack.last().unwrap();
                    let frame_ptr = frame.frame_ptr;
                    let val = self.stack.get_local_at(frame_ptr, 3).unwrap_or_default();
                    self.stack.push(val);
                }

                // Set argument 0-3 (optimized)
                op if op == OpCode::PutArg0 as u8 => {
                    let val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let frame = self.call_stack.last().unwrap();
                    let frame_ptr = frame.frame_ptr;
                    self.stack.set_local_at(frame_ptr, 0, val);
                }
                op if op == OpCode::PutArg1 as u8 => {
                    let val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let frame = self.call_stack.last().unwrap();
                    let frame_ptr = frame.frame_ptr;
                    self.stack.set_local_at(frame_ptr, 1, val);
                }
                op if op == OpCode::PutArg2 as u8 => {
                    let val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let frame = self.call_stack.last().unwrap();
                    let frame_ptr = frame.frame_ptr;
                    self.stack.set_local_at(frame_ptr, 2, val);
                }
                op if op == OpCode::PutArg3 as u8 => {
                    let val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let frame = self.call_stack.last().unwrap();
                    let frame_ptr = frame.frame_ptr;
                    self.stack.set_local_at(frame_ptr, 3, val);
                }

                // Push this value
                op if op == OpCode::PushThis as u8 => {
                    let frame = self.call_stack.last().unwrap();
                    self.stack.push(frame.this_val);
                }

                // Push current function (for self-reference/recursion)
                op if op == OpCode::ThisFunc as u8 => {
                    let frame = self.call_stack.last().unwrap();
                    // Push the function index that created this frame
                    self.stack.push(frame.this_func);
                }

                // Get captured variable (16-bit index)
                op if op == OpCode::GetVarRef as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let idx = u16::from_le_bytes([bc[frame.pc], bc[frame.pc + 1]]) as usize;
                    frame.pc += 2;

                    // Get the closure for this frame, look up cell index, read from var_cells
                    let closure_idx = frame.closure_idx;
                    let val = if let Some(closure_idx) = closure_idx {
                        if let Some(cell_idx) = self
                            .get_closure(closure_idx as u32)
                            .and_then(|c| c.get_cell_index(idx))
                        {
                            self.var_cells[cell_idx as usize]
                        } else {
                            Value::undefined()
                        }
                    } else {
                        Value::undefined()
                    };
                    self.stack.push(val);
                }

                // Set captured variable (16-bit index)
                op if op == OpCode::PutVarRef as u8 => {
                    let val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let idx = u16::from_le_bytes([bc[frame.pc], bc[frame.pc + 1]]) as usize;
                    frame.pc += 2;

                    // Write to the shared cell via closure's cell index
                    if let Some(closure_idx) = frame.closure_idx {
                        if let Some(cell_idx) = self
                            .get_closure(closure_idx as u32)
                            .and_then(|c| c.get_cell_index(idx))
                        {
                            self.var_cells[cell_idx as usize] = val;
                        }
                    }
                }

                // Arithmetic: Negate
                op if op == OpCode::Neg as u8 => {
                    let val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    match self.try_op(self.op_neg(val))? {
                        Some(result) => self.stack.push(result),
                        None => continue,
                    }
                }

                // Unary plus (ToNumber)
                op if op == OpCode::Plus as u8 => {
                    let val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let result = if val.is_int() || val.is_float() {
                        val
                    } else if let Some(b) = val.to_bool() {
                        Value::int(if b { 1 } else { 0 })
                    } else if val.is_null() {
                        Value::int(0)
                    } else if val.is_undefined() {
                        Value::nan()
                    } else if let Some(str_idx) = val.to_string_idx() {
                        if let Some(s) = self.get_string_by_idx(str_idx) {
                            let s = s.trim();
                            if s.is_empty() {
                                Value::int(0)
                            } else if let Ok(i) = s.parse::<i32>() {
                                Value::int(i)
                            } else if let Ok(f) = s.parse::<Float>() {
                                float_to_value(f)
                            } else {
                                Value::nan()
                            }
                        } else {
                            Value::nan()
                        }
                    } else {
                        Value::nan()
                    };
                    self.stack.push(result);
                }

                // Arithmetic: Add (also handles string concatenation)
                op if op == OpCode::Add as u8 => {
                    // SAFETY: Stack operations are valid for well-formed bytecode
                    let (b, a) = unsafe { self.stack.pop2_unchecked() };

                    // String concatenation: if either operand is a string, convert both to strings and concat
                    if a.is_string() || b.is_string() {
                        let frame = self.call_stack.last().unwrap();
                        let bytecode = unsafe { &*frame.bytecode };

                        let out = if let (Some(sa), Some(sb)) = (
                            self.get_string_content(a, bytecode),
                            self.get_string_content(b, bytecode),
                        ) {
                            let mut out = String::with_capacity(sa.len() + sb.len());
                            out.push_str(sa);
                            out.push_str(sb);
                            out
                        } else if let (Some(sa), Some(n)) =
                            (self.get_string_content(a, bytecode), b.to_i32())
                        {
                            let mut buf = [0u8; 16];
                            let len = i32_to_str(&mut buf, n);
                            let mut out = String::with_capacity(sa.len() + len);
                            out.push_str(sa);
                            // SAFETY: i32_to_str only writes ASCII decimal digits and optional '-'.
                            let s = unsafe { core::str::from_utf8_unchecked(&buf[..len]) };
                            out.push_str(s);
                            out
                        } else if let (Some(n), Some(sb)) =
                            (a.to_i32(), self.get_string_content(b, bytecode))
                        {
                            let mut buf = [0u8; 16];
                            let len = i32_to_str(&mut buf, n);
                            let mut out = String::with_capacity(len + sb.len());
                            // SAFETY: i32_to_str only writes ASCII decimal digits and optional '-'.
                            let s = unsafe { core::str::from_utf8_unchecked(&buf[..len]) };
                            out.push_str(s);
                            out.push_str(sb);
                            out
                        } else if let Some(sa) = self.get_string_content(a, bytecode) {
                            let len_b = self.value_to_string_len_hint(b, bytecode);
                            let mut out = String::with_capacity(sa.len() + len_b);
                            out.push_str(sa);
                            self.append_value_to_string(&mut out, b, bytecode);
                            out
                        } else if let Some(sb) = self.get_string_content(b, bytecode) {
                            let len_a = self.value_to_string_len_hint(a, bytecode);
                            let mut out = String::with_capacity(len_a + sb.len());
                            self.append_value_to_string(&mut out, a, bytecode);
                            out.push_str(sb);
                            out
                        } else {
                            let len_a = self.value_to_string_len_hint(a, bytecode);
                            let len_b = self.value_to_string_len_hint(b, bytecode);
                            let mut out = String::with_capacity(len_a + len_b);
                            self.append_value_to_string(&mut out, a, bytecode);
                            self.append_value_to_string(&mut out, b, bytecode);
                            out
                        };
                        self.bump_runtime_string_concat();
                        let result = self.create_runtime_string_raw(out);
                        self.stack.push(result);
                    } else {
                        match self.try_op(self.op_add(a, b))? {
                            Some(result) => {
                                if !self.try_consume_statement_local_store(result)
                                    && !self.try_consume_sieve_style_local_update(result)
                                {
                                    self.stack.push(result);
                                }
                            }
                            None => continue,
                        }
                    }
                }

                op if op == OpCode::AddConstStringLeft as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let str_idx = u16::from_le_bytes([bc[frame.pc], bc[frame.pc + 1]]);
                    frame.pc += 2;

                    let rhs = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let _lhs = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let lhs_str = self
                        .get_const_string_content(bytecode, str_idx)
                        .unwrap_or_default();
                    let len_rhs = self.value_to_string_len_hint(rhs, bytecode);
                    let mut out = String::with_capacity(lhs_str.len() + len_rhs);
                    out.push_str(lhs_str);
                    self.append_value_to_string(&mut out, rhs, bytecode);
                    self.bump_runtime_string_concat();
                    let result = self.create_runtime_string_raw(out);
                    self.stack.push(result);
                }

                op if op == OpCode::AddConstStringRight as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let str_idx = u16::from_le_bytes([bc[frame.pc], bc[frame.pc + 1]]);
                    frame.pc += 2;

                    let lhs = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let rhs_str = self
                        .get_const_string_content(bytecode, str_idx)
                        .unwrap_or_default()
                        .to_string();
                    self.bump_runtime_string_concat();
                    let left = self.runtime_string_part_from_value(lhs, bytecode);
                    let right = RuntimeStringPart::Owned(rhs_str);
                    let result = self.create_runtime_string_concat(left, right);
                    self.stack.push(result);
                }

                op if op == OpCode::AddConstStringSurround as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let left_idx = u16::from_le_bytes([bc[frame.pc], bc[frame.pc + 1]]);
                    let right_idx = u16::from_le_bytes([bc[frame.pc + 2], bc[frame.pc + 3]]);
                    frame.pc += 4;

                    let middle = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let _left = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let left_str = self
                        .get_const_string_content(bytecode, left_idx)
                        .unwrap_or_default();
                    let right_str = self
                        .get_const_string_content(bytecode, right_idx)
                        .unwrap_or_default();

                    let out = if let Some(mid_str) = self.get_string_content(middle, bytecode) {
                        let mut out =
                            String::with_capacity(left_str.len() + mid_str.len() + right_str.len());
                        out.push_str(left_str);
                        out.push_str(mid_str);
                        out.push_str(right_str);
                        out
                    } else if let Some(n) = middle.to_i32() {
                        let mut buf = [0u8; 16];
                        let len = i32_to_str(&mut buf, n);
                        let mut out = String::with_capacity(left_str.len() + len + right_str.len());
                        out.push_str(left_str);
                        // SAFETY: i32_to_str only writes ASCII decimal digits and optional '-'.
                        let s = unsafe { core::str::from_utf8_unchecked(&buf[..len]) };
                        out.push_str(s);
                        out.push_str(right_str);
                        out
                    } else {
                        let len_middle = self.value_to_string_len_hint(middle, bytecode);
                        let mut out =
                            String::with_capacity(left_str.len() + len_middle + right_str.len());
                        out.push_str(left_str);
                        self.append_value_to_string(&mut out, middle, bytecode);
                        out.push_str(right_str);
                        out
                    };
                    self.bump_runtime_string_concat();
                    let result = self.create_runtime_string_raw(out);
                    self.stack.push(result);
                }

                op if op == OpCode::AddConstStringSurroundValue as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let left_idx = u16::from_le_bytes([bc[frame.pc], bc[frame.pc + 1]]);
                    let mid_idx = u16::from_le_bytes([bc[frame.pc + 2], bc[frame.pc + 3]]);
                    frame.pc += 4;

                    let rhs = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let middle = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let _left = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let left_str = self
                        .get_const_string_content(bytecode, left_idx)
                        .unwrap_or_default();
                    let mid_str = self
                        .get_const_string_content(bytecode, mid_idx)
                        .unwrap_or_default();

                    let len_middle = self.value_to_string_len_hint(middle, bytecode);
                    let len_rhs = self.value_to_string_len_hint(rhs, bytecode);
                    let mut out = String::with_capacity(
                        left_str.len() + len_middle + mid_str.len() + len_rhs,
                    );
                    out.push_str(left_str);
                    self.append_value_to_string(&mut out, middle, bytecode);
                    out.push_str(mid_str);
                    self.append_value_to_string(&mut out, rhs, bytecode);
                    self.bump_runtime_string_concat();
                    let result = self.create_runtime_string_raw(out);
                    self.stack.push(result);
                }

                // Arithmetic: Subtract
                op if op == OpCode::Sub as u8 => {
                    // SAFETY: Stack operations are valid for well-formed bytecode
                    let (b, a) = unsafe { self.stack.pop2_unchecked() };
                    match self.try_op(self.op_sub(a, b))? {
                        Some(result) => self.stack.push(result),
                        None => continue,
                    }
                }

                // Arithmetic: Multiply
                op if op == OpCode::Mul as u8 => {
                    // SAFETY: Stack operations are valid for well-formed bytecode
                    let (b, a) = unsafe { self.stack.pop2_unchecked() };
                    match self.try_op(self.op_mul(a, b))? {
                        Some(result) => {
                            if !self.try_consume_statement_local_store(result) {
                                self.stack.push(result);
                            }
                        }
                        None => continue,
                    }
                }

                // Arithmetic: Divide
                op if op == OpCode::Div as u8 => {
                    // SAFETY: Stack operations are valid for well-formed bytecode
                    let (b, a) = unsafe { self.stack.pop2_unchecked() };
                    match self.try_op(self.op_div(a, b))? {
                        Some(result) => self.stack.push(result),
                        None => continue,
                    }
                }

                // Arithmetic: Modulo
                op if op == OpCode::Mod as u8 => {
                    // SAFETY: Stack operations are valid for well-formed bytecode
                    let (b, a) = unsafe { self.stack.pop2_unchecked() };
                    match self.try_op(self.op_mod(a, b))? {
                        Some(result) => self.stack.push(result),
                        None => continue,
                    }
                }

                // Exponentiation: a ** b
                op if op == OpCode::Pow as u8 => {
                    // SAFETY: Stack operations are valid for well-formed bytecode
                    let (b, a) = unsafe { self.stack.pop2_unchecked() };
                    match self.try_op(self.op_pow(a, b))? {
                        Some(result) => self.stack.push(result),
                        None => continue,
                    }
                }

                // Comparison: Less than
                op if op == OpCode::Lt as u8 => {
                    // SAFETY: Stack operations are valid for well-formed bytecode
                    let (b, a) = unsafe { self.stack.pop2_unchecked() };
                    let branch = {
                        let frame = self.call_stack.last_mut().unwrap();
                        let bytecode = unsafe { &*frame.bytecode };
                        let bc = &bytecode.bytecode;
                        if frame.pc < bc.len() {
                            let next = bc[frame.pc];
                            if (next == OpCode::IfFalse as u8 || next == OpCode::IfTrue as u8)
                                && frame.pc + 4 < bc.len()
                            {
                                let offset = i32::from_le_bytes([
                                    bc[frame.pc + 1],
                                    bc[frame.pc + 2],
                                    bc[frame.pc + 3],
                                    bc[frame.pc + 4],
                                ]);
                                frame.pc += 5;
                                Some((next == OpCode::IfTrue as u8, offset))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    };

                    // String lexicographic comparison
                    if a.is_string() && b.is_string() {
                        let frame = self.call_stack.last().unwrap();
                        let bytecode = unsafe { &*frame.bytecode };
                        let sa = self
                            .get_string_content(a, bytecode)
                            .unwrap_or_default()
                            .to_string();
                        let sb = self
                            .get_string_content(b, bytecode)
                            .unwrap_or_default()
                            .to_string();
                        let result = sa < sb;
                        if let Some((branch_on_true, offset)) = branch {
                            if result == branch_on_true {
                                let frame = self.call_stack.last_mut().unwrap();
                                frame.pc = (frame.pc as i32 + offset) as usize;
                            }
                        } else {
                            self.stack.push(Value::bool(result));
                        }
                    } else {
                        if let Some((branch_on_true, offset)) = branch {
                            let result = if let (Some(va), Some(vb)) = (a.to_i32(), b.to_i32()) {
                                va < vb
                            } else if let Some((fa, fb)) =
                                crate::vm::ops::to_numeric_pair(self, a, b)
                            {
                                !fa.is_nan() && !fb.is_nan() && fa < fb
                            } else {
                                let result = match self.try_op(self.op_lt(a, b))? {
                                    Some(result) => result,
                                    None => continue,
                                };
                                Self::value_to_bool(result)
                            };

                            if result == branch_on_true {
                                let frame = self.call_stack.last_mut().unwrap();
                                frame.pc = (frame.pc as i32 + offset) as usize;
                            }
                        } else {
                            let result = match self.try_op(self.op_lt(a, b))? {
                                Some(result) => result,
                                None => continue,
                            };
                            self.stack.push(result);
                        }
                    }
                }

                // Comparison: Less than or equal
                op if op == OpCode::Lte as u8 => {
                    // SAFETY: Stack operations are valid for well-formed bytecode
                    let (b, a) = unsafe { self.stack.pop2_unchecked() };
                    let branch = {
                        let frame = self.call_stack.last_mut().unwrap();
                        let bytecode = unsafe { &*frame.bytecode };
                        let bc = &bytecode.bytecode;
                        if frame.pc < bc.len() {
                            let next = bc[frame.pc];
                            if (next == OpCode::IfFalse as u8 || next == OpCode::IfTrue as u8)
                                && frame.pc + 4 < bc.len()
                            {
                                let offset = i32::from_le_bytes([
                                    bc[frame.pc + 1],
                                    bc[frame.pc + 2],
                                    bc[frame.pc + 3],
                                    bc[frame.pc + 4],
                                ]);
                                frame.pc += 5;
                                Some((next == OpCode::IfTrue as u8, offset))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    };

                    if a.is_string() && b.is_string() {
                        let frame = self.call_stack.last().unwrap();
                        let bytecode = unsafe { &*frame.bytecode };
                        let sa = self
                            .get_string_content(a, bytecode)
                            .unwrap_or_default()
                            .to_string();
                        let sb = self
                            .get_string_content(b, bytecode)
                            .unwrap_or_default()
                            .to_string();
                        let result = sa <= sb;
                        if let Some((branch_on_true, offset)) = branch {
                            if result == branch_on_true {
                                let frame = self.call_stack.last_mut().unwrap();
                                frame.pc = (frame.pc as i32 + offset) as usize;
                            }
                        } else {
                            self.stack.push(Value::bool(result));
                        }
                    } else {
                        if let Some((branch_on_true, offset)) = branch {
                            let result = if let (Some(va), Some(vb)) = (a.to_i32(), b.to_i32()) {
                                va <= vb
                            } else if let Some((fa, fb)) =
                                crate::vm::ops::to_numeric_pair(self, a, b)
                            {
                                !fa.is_nan() && !fb.is_nan() && fa <= fb
                            } else {
                                let result = match self.try_op(self.op_lte(a, b))? {
                                    Some(result) => result,
                                    None => continue,
                                };
                                Self::value_to_bool(result)
                            };

                            if result == branch_on_true {
                                let frame = self.call_stack.last_mut().unwrap();
                                frame.pc = (frame.pc as i32 + offset) as usize;
                            }
                        } else {
                            let result = match self.try_op(self.op_lte(a, b))? {
                                Some(result) => result,
                                None => continue,
                            };
                            self.stack.push(result);
                        }
                    }
                }

                // Comparison: Greater than
                op if op == OpCode::Gt as u8 => {
                    // SAFETY: Stack operations are valid for well-formed bytecode
                    let (b, a) = unsafe { self.stack.pop2_unchecked() };
                    if a.is_string() && b.is_string() {
                        let frame = self.call_stack.last().unwrap();
                        let bytecode = unsafe { &*frame.bytecode };
                        let sa = self
                            .get_string_content(a, bytecode)
                            .unwrap_or_default()
                            .to_string();
                        let sb = self
                            .get_string_content(b, bytecode)
                            .unwrap_or_default()
                            .to_string();
                        self.stack.push(Value::bool(sa > sb));
                    } else {
                        match self.try_op(self.op_gt(a, b))? {
                            Some(result) => self.stack.push(result),
                            None => continue,
                        }
                    }
                }

                // Comparison: Greater than or equal
                op if op == OpCode::Gte as u8 => {
                    // SAFETY: Stack operations are valid for well-formed bytecode
                    let (b, a) = unsafe { self.stack.pop2_unchecked() };
                    if a.is_string() && b.is_string() {
                        let frame = self.call_stack.last().unwrap();
                        let bytecode = unsafe { &*frame.bytecode };
                        let sa = self
                            .get_string_content(a, bytecode)
                            .unwrap_or_default()
                            .to_string();
                        let sb = self
                            .get_string_content(b, bytecode)
                            .unwrap_or_default()
                            .to_string();
                        self.stack.push(Value::bool(sa >= sb));
                    } else {
                        match self.try_op(self.op_gte(a, b))? {
                            Some(result) => self.stack.push(result),
                            None => continue,
                        }
                    }
                }

                // Comparison: Equal (==) 閳?abstract equality with type coercion
                op if op == OpCode::Eq as u8 => {
                    // SAFETY: Stack operations are valid for well-formed bytecode
                    let (b, a) = unsafe { self.stack.pop2_unchecked() };
                    // String == String: compare content
                    if a.is_string() && b.is_string() {
                        let frame = self.call_stack.last().unwrap();
                        let bytecode = unsafe { &*frame.bytecode };
                        let sa = self.get_string_content(a, bytecode).unwrap_or_default();
                        let sb = self.get_string_content(b, bytecode).unwrap_or_default();
                        self.stack.push(Value::bool(sa == sb));
                    } else {
                        match self.try_op(self.op_eq(a, b))? {
                            Some(result) => self.stack.push(result),
                            None => continue,
                        }
                    }
                }

                // Comparison: Not equal (!=) 閳?abstract inequality
                op if op == OpCode::Neq as u8 => {
                    // SAFETY: Stack operations are valid for well-formed bytecode
                    let (b, a) = unsafe { self.stack.pop2_unchecked() };
                    if a.is_string() && b.is_string() {
                        let frame = self.call_stack.last().unwrap();
                        let bytecode = unsafe { &*frame.bytecode };
                        let sa = self.get_string_content(a, bytecode).unwrap_or_default();
                        let sb = self.get_string_content(b, bytecode).unwrap_or_default();
                        self.stack.push(Value::bool(sa != sb));
                    } else {
                        match self.try_op(self.op_neq(a, b))? {
                            Some(result) => self.stack.push(result),
                            None => continue,
                        }
                    }
                }

                // Comparison: Strict equal (===) 鈥?no type coercion
                op if op == OpCode::StrictEq as u8 => {
                    let b = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    if a == b {
                        self.stack.push(Value::bool(!a.is_nan_value()));
                    } else if let (Some(ia), Some(ib)) = (a.to_i32(), b.to_i32()) {
                        self.stack.push(Value::bool(ia == ib));
                    } else if let (Some(ba), Some(bb)) = (a.to_bool(), b.to_bool()) {
                        self.stack.push(Value::bool(ba == bb));
                    } else if (a.is_null() && b.is_undefined()) || (a.is_undefined() && b.is_null())
                    {
                        self.stack.push(Value::bool(false));
                    // String === String: compare content
                    } else if a.is_string() && b.is_string() {
                        let frame = self.call_stack.last().unwrap();
                        let bytecode = unsafe { &*frame.bytecode };
                        let sa = self.get_string_content(a, bytecode).unwrap_or_default();
                        let sb = self.get_string_content(b, bytecode).unwrap_or_default();
                        self.stack.push(Value::bool(sa == sb));
                    } else {
                        let result = self.op_strict_eq(a, b).unwrap_or(Value::bool(false));
                        self.stack.push(result);
                    }
                }

                // Comparison: Strict not equal (!==)
                op if op == OpCode::StrictNeq as u8 => {
                    let b = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    if a == b {
                        self.stack.push(Value::bool(a.is_nan_value()));
                    } else if let (Some(ia), Some(ib)) = (a.to_i32(), b.to_i32()) {
                        self.stack.push(Value::bool(ia != ib));
                    } else if let (Some(ba), Some(bb)) = (a.to_bool(), b.to_bool()) {
                        self.stack.push(Value::bool(ba != bb));
                    } else if (a.is_null() && b.is_undefined()) || (a.is_undefined() && b.is_null())
                    {
                        self.stack.push(Value::bool(true));
                    } else if a.is_string() && b.is_string() {
                        let frame = self.call_stack.last().unwrap();
                        let bytecode = unsafe { &*frame.bytecode };
                        let sa = self.get_string_content(a, bytecode).unwrap_or_default();
                        let sb = self.get_string_content(b, bytecode).unwrap_or_default();
                        self.stack.push(Value::bool(sa != sb));
                    } else {
                        let result = self.op_strict_neq(a, b).unwrap_or(Value::bool(true));
                        self.stack.push(result);
                    }
                }

                // Logical NOT
                op if op == OpCode::LNot as u8 => {
                    let val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let result = Value::bool(!Self::value_to_bool(val));
                    self.stack.push(result);
                }

                // Bitwise NOT
                op if op == OpCode::Not as u8 => {
                    let val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    match self.try_op(self.op_bitwise_not(val))? {
                        Some(result) => self.stack.push(result),
                        None => continue,
                    }
                }

                // Bitwise AND
                op if op == OpCode::And as u8 => {
                    let b = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    match self.try_op(self.op_bitwise_and(a, b))? {
                        Some(result) => self.stack.push(result),
                        None => continue,
                    }
                }

                // Bitwise OR
                op if op == OpCode::Or as u8 => {
                    let b = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    match self.try_op(self.op_bitwise_or(a, b))? {
                        Some(result) => self.stack.push(result),
                        None => continue,
                    }
                }

                // Bitwise XOR
                op if op == OpCode::Xor as u8 => {
                    let b = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    match self.try_op(self.op_bitwise_xor(a, b))? {
                        Some(result) => self.stack.push(result),
                        None => continue,
                    }
                }

                // Left shift
                op if op == OpCode::Shl as u8 => {
                    let b = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    match self.try_op(self.op_shl(a, b))? {
                        Some(result) => self.stack.push(result),
                        None => continue,
                    }
                }

                // Arithmetic right shift
                op if op == OpCode::Sar as u8 => {
                    let b = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    match self.try_op(self.op_sar(a, b))? {
                        Some(result) => self.stack.push(result),
                        None => continue,
                    }
                }

                // Logical right shift
                op if op == OpCode::Shr as u8 => {
                    let b = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let a = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    match self.try_op(self.op_shr(a, b))? {
                        Some(result) => self.stack.push(result),
                        None => continue,
                    }
                }

                // Increment
                op if op == OpCode::Inc as u8 => {
                    let val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    match self.try_op(self.op_add(val, Value::int(1)))? {
                        Some(result) => self.stack.push(result),
                        None => continue,
                    }
                }

                // Decrement
                op if op == OpCode::Dec as u8 => {
                    let val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    match self.try_op(self.op_sub(val, Value::int(1)))? {
                        Some(result) => self.stack.push(result),
                        None => continue,
                    }
                }

                // Control flow: Goto
                op if op == OpCode::Goto as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let offset = unsafe {
                        i32::from_le_bytes([
                            *bc.get_unchecked(frame.pc),
                            *bc.get_unchecked(frame.pc + 1),
                            *bc.get_unchecked(frame.pc + 2),
                            *bc.get_unchecked(frame.pc + 3),
                        ])
                    };
                    frame.pc += 4;
                    // offset is relative to the end of this instruction (after the 4-byte offset)
                    frame.pc = (frame.pc as i32 + offset) as usize;
                }

                // Control flow: If false
                op if op == OpCode::IfFalse as u8 => {
                    let val = unsafe { self.stack.pop_unchecked() };
                    let is_truthy = Self::value_to_bool(val);
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let offset = unsafe {
                        i32::from_le_bytes([
                            *bc.get_unchecked(frame.pc),
                            *bc.get_unchecked(frame.pc + 1),
                            *bc.get_unchecked(frame.pc + 2),
                            *bc.get_unchecked(frame.pc + 3),
                        ])
                    };
                    frame.pc += 4;
                    if !is_truthy {
                        // offset is relative to the end of this instruction (after the 4-byte offset)
                        frame.pc = (frame.pc as i32 + offset) as usize;
                    }
                }

                // Control flow: If true
                op if op == OpCode::IfTrue as u8 => {
                    let val = unsafe { self.stack.pop_unchecked() };
                    let is_truthy = Self::value_to_bool(val);
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let offset = unsafe {
                        i32::from_le_bytes([
                            *bc.get_unchecked(frame.pc),
                            *bc.get_unchecked(frame.pc + 1),
                            *bc.get_unchecked(frame.pc + 2),
                            *bc.get_unchecked(frame.pc + 3),
                        ])
                    };
                    frame.pc += 4;
                    if is_truthy {
                        // offset is relative to the end of this instruction (after the 4-byte offset)
                        frame.pc = (frame.pc as i32 + offset) as usize;
                    }
                }

                // Return
                op if op == OpCode::Return as u8 => {
                    let result = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;

                    // Pop the current frame
                    let frame = self.call_stack.pop().ok_or_else(|| {
                        InterpreterError::InternalError("no call frame to return from".to_string())
                    })?;

                    // Promote compile-time strings before leaving this scope
                    let bytecode_ref = unsafe { &*frame.bytecode };
                    let result = self.promote_string(result, bytecode_ref);

                    // Clean up locals from the stack
                    let local_count = unsafe { (*frame.bytecode).local_count } as usize;
                    self.stack.drop_n(local_count);

                    // For constructor calls: if result is not an object, return 'this' instead
                    let final_result = if frame.is_constructor && !result.is_object() {
                        frame.this_val
                    } else {
                        result
                    };

                    // If there are no more frames, this is the final result
                    if self.call_stack.is_empty() {
                        return Ok(final_result);
                    }

                    // Check if we've reached the target depth for a nested call_value
                    if let Some(target_depth) = self.nested_call_target_depth {
                        if self.call_stack.len() == target_depth {
                            return Ok(final_result);
                        }
                    }

                    // Otherwise, push the result for the caller and continue the loop (no recursion!)
                    self.stack.push(final_result);
                }

                // Return undefined
                op if op == OpCode::ReturnUndef as u8 => {
                    let result = Value::undefined();

                    // Pop the current frame
                    let frame = self.call_stack.pop().ok_or_else(|| {
                        InterpreterError::InternalError("no call frame to return from".to_string())
                    })?;

                    // Clean up locals from the stack
                    let local_count = unsafe { (*frame.bytecode).local_count } as usize;
                    self.stack.drop_n(local_count);

                    // For constructor calls: if result is not an object, return 'this' instead
                    let final_result = if frame.is_constructor && !result.is_object() {
                        frame.this_val
                    } else {
                        result
                    };

                    // If there are no more frames, this is the final result
                    if self.call_stack.is_empty() {
                        return Ok(final_result);
                    }

                    // Check if we've reached the target depth for a nested call_value
                    if let Some(target_depth) = self.nested_call_target_depth {
                        if self.call_stack.len() == target_depth {
                            return Ok(final_result);
                        }
                    }

                    // Otherwise, push the result for the caller and continue the loop (no recursion!)
                    self.stack.push(final_result);
                }

                // Function closure creation (16-bit function index)
                op if op == OpCode::FClosure as u8 => {
                    let frame = self.call_stack.last().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let pc = frame.pc;
                    let closure_idx_current = frame.closure_idx;

                    let func_idx = u16::from_le_bytes([bc[pc], bc[pc + 1]]) as usize;

                    // Get the inner function bytecode
                    let inner_func = bytecode.inner_functions.get(func_idx).ok_or_else(|| {
                        InterpreterError::InternalError(format!(
                            "invalid function index in FClosure: {}",
                            func_idx
                        ))
                    })?;

                    // Capture variables into shared cells
                    let mut cell_indices = Vec::with_capacity(inner_func.captures.len());
                    for capture in &inner_func.captures {
                        let cell_idx = if capture.is_local {
                            // Once a local is captured, later closures and outer-frame
                            // writes must keep reusing the same shared cell.
                            self.ensure_captured_local_cell(capture.outer_index)
                        } else {
                            // Capture from outer's captures: reuse the same cell index.
                            if let Some(closure_idx) = closure_idx_current {
                                self.get_closure(closure_idx as u32)
                                    .and_then(|c| c.get_cell_index(capture.outer_index))
                                    .unwrap_or_else(|| self.alloc_var_cell(Value::undefined()))
                            } else {
                                self.alloc_var_cell(Value::undefined())
                            }
                        };
                        cell_indices.push(cell_idx);
                    }

                    // Update PC after we're done reading
                    let frame = self.call_stack.last_mut().unwrap();
                    frame.pc += 2;

                    // Create closure or simple function reference based on whether there are captures
                    let func_val = if !cell_indices.is_empty() {
                        self.create_closure(inner_func as *const _, cell_indices)
                    } else {
                        Value::func_ptr(inner_func as *const _)
                    };

                    self.stack.push(func_val);
                }

                // Function call (16-bit argc)
                op if op == OpCode::Call as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let argc = u16::from_le_bytes([bc[frame.pc], bc[frame.pc + 1]]) as usize;
                    frame.pc += 2;

                    // Args are already on stack in correct order: [func_val][arg0][arg1]...[argN-1]
                    // Remove func_val from below the args (offset = argc from top)
                    let func_val = self
                        .stack
                        .compact_call_args(argc)
                        .ok_or(InterpreterError::StackUnderflow)?;

                    // Check if this is a native function call
                    if let Some(native_idx) = func_val.to_native_func_idx() {
                        let result = match argc {
                            0 => self.call_native_func(native_idx, Value::undefined(), &[]),
                            1 => {
                                let a0 =
                                    self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                                let args = [a0];
                                self.call_native_func(native_idx, Value::undefined(), &args)
                            }
                            2 => {
                                let a1 =
                                    self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                                let a0 =
                                    self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                                let args = [a0, a1];
                                self.call_native_func(native_idx, Value::undefined(), &args)
                            }
                            3 => {
                                let a2 =
                                    self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                                let a1 =
                                    self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                                let a0 =
                                    self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                                let args = [a0, a1, a2];
                                self.call_native_func(native_idx, Value::undefined(), &args)
                            }
                            _ => {
                                let stack_len = self.stack.len();
                                let mut args = Vec::with_capacity(argc);
                                for i in 0..argc {
                                    args.push(self.stack.get_raw(stack_len - argc + i));
                                }
                                self.stack.drop_n(argc);
                                self.call_native_func(native_idx, Value::undefined(), &args)
                            }
                        };
                        if let Some(result) = self.try_op(result)? {
                            self.stack.push(result);
                        }
                        continue;
                    }

                    // Check if this is a builtin object called as a function
                    if let Some(builtin_idx) = func_val.to_builtin_object_idx() {
                        let result = match argc {
                            0 => self.call_builtin_as_function(builtin_idx, &[]),
                            1 => {
                                let a0 =
                                    self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                                let args = [a0];
                                self.call_builtin_as_function(builtin_idx, &args)
                            }
                            2 => {
                                let a1 =
                                    self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                                let a0 =
                                    self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                                let args = [a0, a1];
                                self.call_builtin_as_function(builtin_idx, &args)
                            }
                            3 => {
                                let a2 =
                                    self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                                let a1 =
                                    self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                                let a0 =
                                    self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                                let args = [a0, a1, a2];
                                self.call_builtin_as_function(builtin_idx, &args)
                            }
                            _ => {
                                let stack_len = self.stack.len();
                                let mut args = Vec::with_capacity(argc);
                                for i in 0..argc {
                                    args.push(self.stack.get_raw(stack_len - argc + i));
                                }
                                self.stack.drop_n(argc);
                                self.call_builtin_as_function(builtin_idx, &args)
                            }
                        }?;
                        self.stack.push(result);
                        continue;
                    }

                    // Promote compile-time string arguments to runtime strings in-place on stack
                    {
                        let stack_len = self.stack.len();
                        for i in 0..argc {
                            let slot = stack_len - argc + i;
                            let val = self.stack.get_raw(slot);
                            if val.is_string() {
                                let promoted = self.promote_string(val, bytecode);
                                self.stack.set_raw(slot, promoted);
                            }
                        }
                    }

                    // Determine if this is a closure or a regular function
                    let (callee_bytecode, callee_closure_idx): (&FunctionBytecode, Option<usize>) =
                        if let Some(closure_idx) = func_val.to_closure_idx() {
                            // Closure call - get bytecode from closure
                            let closure = self.get_closure(closure_idx).ok_or_else(|| {
                                InterpreterError::InternalError(format!(
                                    "invalid closure index: {}",
                                    closure_idx
                                ))
                            })?;
                            (unsafe { &*closure.bytecode }, Some(closure_idx as usize))
                        } else if let Some(ptr) = func_val.to_func_ptr() {
                            // Pointer-based function (from FClosure without captures or ThisFunc)
                            (unsafe { &*ptr }, None)
                        } else if let Some(idx) = func_val.to_func_idx() {
                            // Index-based function (legacy, shouldn't happen anymore)
                            let bc =
                                bytecode.inner_functions.get(idx as usize).ok_or_else(|| {
                                    InterpreterError::InternalError(format!(
                                        "invalid function index: {}",
                                        idx
                                    ))
                                })?;
                            (bc, None)
                        } else {
                            self.try_handle_runtime_error(InterpreterError::TypeError(
                                "not a function".to_string(),
                            ))?;
                            continue;
                        };

                    // Check recursion limit
                    if self.call_stack.len() >= self.max_recursion {
                        self.try_handle_runtime_error(InterpreterError::InternalError(
                            "maximum call stack size exceeded".to_string(),
                        ))?;
                        continue;
                    }

                    // Args are already on stack starting at callee_frame_ptr
                    let callee_frame_ptr = self.stack.len() - argc;

                    // Pad or truncate: add undefined for missing args
                    let expected = callee_bytecode.arg_count as usize;
                    if argc < expected {
                        for _ in 0..(expected - argc) {
                            self.stack.push(Value::undefined());
                        }
                    } else if argc > expected {
                        // Extra args: drop them
                        self.stack.drop_n(argc - expected);
                    }

                    // Allocate space for locals (beyond arguments)
                    let extra_locals = callee_bytecode
                        .local_count
                        .saturating_sub(callee_bytecode.arg_count);
                    for _ in 0..extra_locals {
                        self.stack.push(Value::undefined());
                    }

                    // Create frame - with closure_idx if this is a closure call
                    let callee_frame = if let Some(closure_idx) = callee_closure_idx {
                        CallFrame::new_closure(
                            callee_bytecode as *const _,
                            callee_frame_ptr,
                            argc.min(u16::MAX as usize) as u16,
                            Value::undefined(), // this value
                            func_val,           // the function value for self-reference
                            closure_idx,
                        )
                    } else {
                        CallFrame::new(
                            callee_bytecode as *const _,
                            callee_frame_ptr,
                            argc.min(u16::MAX as usize) as u16,
                            Value::undefined(), // this value
                            func_val,           // the function value for self-reference
                        )
                    };
                    self.call_stack.push(callee_frame);

                    // Continue execution in the new frame (run loop will pick it up)
                }

                op if op == OpCode::CallArrayPush1 as u8 => {
                    let mut arg = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let method_val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let this_val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;

                    if let Some(arr_idx) = this_val.to_array_idx() {
                        let discard_result = {
                            let frame = self.call_stack.last_mut().unwrap();
                            let bytecode = unsafe { &*frame.bytecode };
                            let bc = &bytecode.bytecode;
                            if frame.pc < bc.len() && bc[frame.pc] == OpCode::Drop as u8 {
                                frame.pc += 1;
                                true
                            } else {
                                false
                            }
                        };

                        if arg.is_string() {
                            let frame = self.call_stack.last().unwrap();
                            let bytecode = unsafe { &*frame.bytecode };
                            arg = self.promote_string(arg, bytecode);
                        }

                        let array = unsafe { self.get_array_mut_unchecked(arr_idx) };
                        array.push(arg);

                        if !discard_result {
                            let new_len = unsafe { self.get_array_unchecked(arr_idx) }.len() as i32;
                            self.stack.push(Value::int(new_len));
                        }
                        continue;
                    }

                    let args = [arg];
                    let result = if let Some(native_idx) = method_val.to_native_func_idx() {
                        let native_result = self.call_native_func(native_idx, this_val, &args);
                        if let Some(result) = self.try_op(native_result)? {
                            result
                        } else {
                            continue;
                        }
                    } else {
                        self.call_value(method_val, this_val, &args)?
                    };
                    self.stack.push(result);
                }

                op if op == OpCode::CallArrayMap1 as u8 => {
                    let callback = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let method_val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let this_val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;

                    if this_val.to_array_idx().is_some() {
                        let args = [callback];
                        let native_result = native_array_map(self, this_val, &args)
                            .map_err(InterpreterError::TypeError);
                        if let Some(result) = self.try_op(native_result)? {
                            self.stack.push(result);
                        }
                        continue;
                    }

                    let args = [callback];
                    let result = if let Some(native_idx) = method_val.to_native_func_idx() {
                        let native_result = self.call_native_func(native_idx, this_val, &args);
                        if let Some(result) = self.try_op(native_result)? {
                            result
                        } else {
                            continue;
                        }
                    } else {
                        self.call_value(method_val, this_val, &args)?
                    };
                    self.stack.push(result);
                }

                op if op == OpCode::CallArrayFilter1 as u8 => {
                    let callback = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let method_val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let this_val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;

                    if this_val.to_array_idx().is_some() {
                        let args = [callback];
                        let native_result = native_array_filter(self, this_val, &args)
                            .map_err(InterpreterError::TypeError);
                        if let Some(result) = self.try_op(native_result)? {
                            self.stack.push(result);
                        }
                        continue;
                    }

                    let args = [callback];
                    let result = if let Some(native_idx) = method_val.to_native_func_idx() {
                        let native_result = self.call_native_func(native_idx, this_val, &args);
                        if let Some(result) = self.try_op(native_result)? {
                            result
                        } else {
                            continue;
                        }
                    } else {
                        self.call_value(method_val, this_val, &args)?
                    };
                    self.stack.push(result);
                }

                op if op == OpCode::CallArrayReduce2 as u8 => {
                    let initial = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let callback = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let method_val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let this_val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;

                    if this_val.to_array_idx().is_some() {
                        let args = [callback, initial];
                        let native_result = native_array_reduce(self, this_val, &args)
                            .map_err(InterpreterError::TypeError);
                        if let Some(result) = self.try_op(native_result)? {
                            self.stack.push(result);
                        }
                        continue;
                    }

                    let args = [callback, initial];
                    let result = if let Some(native_idx) = method_val.to_native_func_idx() {
                        let native_result = self.call_native_func(native_idx, this_val, &args);
                        if let Some(result) = self.try_op(native_result)? {
                            result
                        } else {
                            continue;
                        }
                    } else {
                        self.call_value(method_val, this_val, &args)?
                    };
                    self.stack.push(result);
                }

                // CallConstructor - new operator: func args -> new_object
                op if op == OpCode::CallConstructor as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let argc = u16::from_le_bytes([bc[frame.pc], bc[frame.pc + 1]]) as usize;
                    frame.pc += 2;

                    // Remove the constructor from below the arguments without
                    // repeatedly shifting stack elements.
                    let func_val = self
                        .stack
                        .compact_call_args(argc)
                        .ok_or(InterpreterError::StackUnderflow)?;

                    // Check if this is a builtin Error constructor
                    if let Some(builtin_idx) = func_val.to_builtin_object_idx() {
                        let stack_len = self.stack.len();
                        let mut args = Vec::with_capacity(argc);
                        for i in 0..argc {
                            args.push(self.stack.get_raw(stack_len - argc + i));
                        }
                        self.stack.drop_n(argc);

                        let error_name = match builtin_idx {
                            BUILTIN_ERROR => Some("Error"),
                            BUILTIN_TYPE_ERROR => Some("TypeError"),
                            BUILTIN_REFERENCE_ERROR => Some("ReferenceError"),
                            BUILTIN_SYNTAX_ERROR => Some("SyntaxError"),
                            BUILTIN_RANGE_ERROR => Some("RangeError"),
                            BUILTIN_EVAL_ERROR => Some("EvalError"),
                            BUILTIN_URI_ERROR => Some("URIError"),
                            BUILTIN_INTERNAL_ERROR => Some("InternalError"),
                            _ => None,
                        };

                        if let Some(error_name) = error_name {
                            // Create an error object

                            // Get message from first argument (if present)
                            let message = if let Some(msg_val) = args.first() {
                                if let Some(str_idx) = msg_val.to_string_idx() {
                                    self.get_string_by_idx(str_idx)
                                        .map(|s| s.to_string())
                                        .unwrap_or_default()
                                } else if let Some(n) = msg_val.to_i32() {
                                    n.to_string()
                                } else {
                                    String::new()
                                }
                            } else {
                                String::new()
                            };

                            // Create and store the error object
                            let (error_idx, is_new) =
                                self.gc.alloc_slot(&mut self.gen_error_objects);
                            let obj = ErrorObject {
                                name: error_name.to_string(),
                                message,
                            };
                            if is_new {
                                self.error_objects.push(obj);
                            } else {
                                self.error_objects[error_idx] = obj;
                            }

                            // Push the error object value
                            self.stack.push(Value::error_object(error_idx as u32));
                            continue;
                        }

                        // Check if this is the RegExp constructor (requires std/regex)
                        #[cfg(feature = "std")]
                        if builtin_idx == BUILTIN_REGEXP {
                            // Get pattern from first argument
                            let pattern = if let Some(pattern_val) = args.first() {
                                if let Some(str_idx) = pattern_val.to_string_idx() {
                                    self.get_string_by_idx(str_idx)
                                        .map(|s| s.to_string())
                                        .unwrap_or_default()
                                } else {
                                    String::new()
                                }
                            } else {
                                String::new()
                            };

                            // Get flags from second argument (if present)
                            let flags = if let Some(flags_val) = args.get(1) {
                                if let Some(str_idx) = flags_val.to_string_idx() {
                                    self.get_string_by_idx(str_idx)
                                        .map(|s| s.to_string())
                                        .unwrap_or_default()
                                } else {
                                    String::new()
                                }
                            } else {
                                String::new()
                            };

                            // Parse flags
                            let global = flags.contains('g');
                            let ignore_case = flags.contains('i');
                            let multiline = flags.contains('m');

                            // Build regex pattern with flags
                            let mut regex_pattern = String::new();
                            if ignore_case || multiline {
                                regex_pattern.push_str("(?");
                                if ignore_case {
                                    regex_pattern.push('i');
                                }
                                if multiline {
                                    regex_pattern.push('m');
                                }
                                regex_pattern.push(')');
                            }
                            regex_pattern.push_str(&pattern);

                            // Compile the regex
                            match regex::Regex::new(&regex_pattern) {
                                Ok(regex) => {
                                    self.maybe_gc();
                                    let (regex_idx, is_new) =
                                        self.gc.alloc_slot(&mut self.gen_regex_objects);
                                    let obj = RegExpObject {
                                        regex,
                                        pattern,
                                        flags,
                                        global,
                                        ignore_case,
                                        multiline,
                                    };
                                    if is_new {
                                        self.regex_objects.push(obj);
                                    } else {
                                        self.regex_objects[regex_idx] = obj;
                                    }
                                    self.stack.push(Value::regexp_object(regex_idx as u32));
                                }
                                Err(e) => {
                                    // Invalid regex - return a SyntaxError
                                    return Err(InterpreterError::InternalError(format!(
                                        "Invalid regular expression: {}",
                                        e
                                    )));
                                }
                            }
                            continue;
                        }

                        // Check if this is a TypedArray constructor
                        let typed_kind = match builtin_idx {
                            BUILTIN_INT8_ARRAY => Some(TypedArrayKind::Int8),
                            BUILTIN_UINT8_ARRAY => Some(TypedArrayKind::Uint8),
                            BUILTIN_UINT8_CLAMPED_ARRAY => Some(TypedArrayKind::Uint8Clamped),
                            BUILTIN_INT16_ARRAY => Some(TypedArrayKind::Int16),
                            BUILTIN_UINT16_ARRAY => Some(TypedArrayKind::Uint16),
                            BUILTIN_INT32_ARRAY => Some(TypedArrayKind::Int32),
                            BUILTIN_UINT32_ARRAY => Some(TypedArrayKind::Uint32),
                            BUILTIN_FLOAT32_ARRAY => Some(TypedArrayKind::Float32),
                            BUILTIN_FLOAT64_ARRAY => Some(TypedArrayKind::Float64),
                            _ => None,
                        };

                        if let Some(kind) = typed_kind {
                            // Get length from first argument
                            let length = if let Some(len_val) = args.first() {
                                if let Some(n) = len_val.to_i32() {
                                    n.max(0) as usize
                                } else if len_val.is_array() {
                                    // Creating from an array
                                    if let Some(arr_idx) = len_val.to_array_idx() {
                                        self.arrays
                                            .get(arr_idx as usize)
                                            .map(|a| a.len())
                                            .unwrap_or(0)
                                    } else {
                                        0
                                    }
                                } else {
                                    0
                                }
                            } else {
                                0
                            };

                            // Create the typed array
                            let mut typed_arr = TypedArrayObject::new(kind, length);

                            // If created from an array, copy values
                            if let Some(src_val) = args.first() {
                                if let Some(arr_idx) = src_val.to_array_idx() {
                                    if let Some(arr) = self.arrays.get(arr_idx as usize) {
                                        for (i, v) in arr.iter().enumerate() {
                                            if i >= length {
                                                break;
                                            }
                                            typed_arr.set(i, *v);
                                        }
                                    }
                                }
                            }

                            self.maybe_gc();
                            let (typed_idx, is_new) =
                                self.gc.alloc_slot(&mut self.gen_typed_arrays);
                            if is_new {
                                self.typed_arrays.push(typed_arr);
                            } else {
                                self.typed_arrays[typed_idx] = typed_arr;
                            }
                            self.stack.push(Value::typed_array_object(typed_idx as u32));
                            continue;
                        }

                        // Check if this is an Array constructor: new Array(n)
                        if builtin_idx == BUILTIN_ARRAY {
                            let arr = if let Some(len) = args.first().and_then(|v| v.to_i32()) {
                                vec![Value::undefined(); len.max(0) as usize]
                            } else if !args.is_empty() {
                                // new Array(a, b, c) 鈫?[a, b, c]
                                args.to_vec()
                            } else {
                                Vec::new()
                            };
                            self.maybe_gc();
                            let (arr_idx, is_new) = self.gc.alloc_slot(&mut self.gen_arrays);
                            if is_new {
                                self.arrays.push(arr);
                            } else {
                                self.arrays[arr_idx] = arr;
                            }
                            self.stack.push(Value::array_idx(arr_idx as u32));
                            continue;
                        }

                        // Check if this is an ArrayBuffer constructor
                        if builtin_idx == BUILTIN_ARRAY_BUFFER {
                            let byte_length = args
                                .first()
                                .and_then(|v| v.to_i32())
                                .map(|n| n.max(0) as usize)
                                .unwrap_or(0);

                            let ab = ArrayBufferObject::new(byte_length);
                            self.maybe_gc();
                            let (ab_idx, is_new) = self.gc.alloc_slot(&mut self.gen_array_buffers);
                            if is_new {
                                self.array_buffers.push(ab);
                            } else {
                                self.array_buffers[ab_idx] = ab;
                            }
                            self.stack.push(Value::array_buffer_object(ab_idx as u32));
                            continue;
                        }
                    }

                    // Create a new object for 'this', storing the constructor reference for instanceof
                    let new_obj = self.create_object_with_constructor(func_val);

                    // Determine if this is a closure or a regular function
                    let (callee_bytecode, callee_closure_idx): (&FunctionBytecode, Option<usize>) =
                        if let Some(closure_idx) = func_val.to_closure_idx() {
                            let closure = self.get_closure(closure_idx).ok_or_else(|| {
                                InterpreterError::InternalError(format!(
                                    "invalid closure index: {}",
                                    closure_idx
                                ))
                            })?;
                            (unsafe { &*closure.bytecode }, Some(closure_idx as usize))
                        } else if let Some(ptr) = func_val.to_func_ptr() {
                            (unsafe { &*ptr }, None)
                        } else if let Some(idx) = func_val.to_func_idx() {
                            let bc =
                                bytecode.inner_functions.get(idx as usize).ok_or_else(|| {
                                    InterpreterError::InternalError(format!(
                                        "invalid function index: {}",
                                        idx
                                    ))
                                })?;
                            (bc, None)
                        } else {
                            return Err(InterpreterError::TypeError(
                                "not a constructor".to_string(),
                            ));
                        };

                    // Check recursion limit
                    if self.call_stack.len() >= self.max_recursion {
                        return Err(InterpreterError::InternalError(
                            "maximum call stack size exceeded".to_string(),
                        ));
                    }

                    let callee_frame_ptr = self.stack.len() - argc;

                    // Pad or truncate arguments in place on stack.
                    let expected = callee_bytecode.arg_count as usize;
                    if argc < expected {
                        for _ in 0..(expected - argc) {
                            self.stack.push(Value::undefined());
                        }
                    } else if argc > expected {
                        self.stack.drop_n(argc - expected);
                    }

                    // Allocate space for locals (beyond arguments)
                    let extra_locals = callee_bytecode
                        .local_count
                        .saturating_sub(callee_bytecode.arg_count);
                    for _ in 0..extra_locals {
                        self.stack.push(Value::undefined());
                    }

                    // Create frame with new object as 'this' - marked as constructor call
                    let callee_frame = if let Some(closure_idx) = callee_closure_idx {
                        CallFrame::new_closure_constructor(
                            callee_bytecode as *const _,
                            callee_frame_ptr,
                            argc.min(u16::MAX as usize) as u16,
                            new_obj, // 'this' is the new object
                            func_val,
                            closure_idx,
                        )
                    } else {
                        CallFrame::new_constructor(
                            callee_bytecode as *const _,
                            callee_frame_ptr,
                            argc.min(u16::MAX as usize) as u16,
                            new_obj, // 'this' is the new object
                            func_val,
                        )
                    };
                    self.call_stack.push(callee_frame);

                    // Continue execution in the new frame
                    // When the constructor returns, do_return handles returning 'this'
                    // if the return value isn't an object
                }

                // CallMethod - method call: obj method args... -> ret
                // Stack before: [obj, method, arg0, arg1, ...]
                op if op == OpCode::CallMethod as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let argc = u16::from_le_bytes([bc[frame.pc], bc[frame.pc + 1]]) as usize;
                    frame.pc += 2;

                    // Remove method and object from below the arguments so that
                    // native fast paths can consume arguments directly from the stack.
                    let (this_val, method_val) = self
                        .stack
                        .compact_method_call_args(argc)
                        .ok_or(InterpreterError::StackUnderflow)?;

                    // Check if this is a native function call
                    if let Some(native_idx) = method_val.to_native_func_idx() {
                        // Check for JSON.parse (cached for performance)
                        if argc == 1
                            && this_val.to_builtin_object_idx() == Some(BUILTIN_JSON)
                            && self.native_func_index.get("JSON.parse") == Some(&native_idx)
                        {
                            let a0 = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                            let args = [a0];
                            let result = native_json_parse(self, this_val, &args)
                                .map_err(Interpreter::classify_native_error);
                            if let Some(result) = self.try_op(result)? {
                                self.stack.push(result);
                            }
                            continue;
                        }

                        // Check for Array methods (cached for performance)
                        if argc == 0
                            && self.native_func_index.get("Array.prototype.push")
                                == Some(&native_idx)
                        {
                            if let Some(arr_idx) = this_val.to_array_idx() {
                                let discard_result = {
                                    let frame = self.call_stack.last_mut().unwrap();
                                    let bytecode = unsafe { &*frame.bytecode };
                                    let bc = &bytecode.bytecode;
                                    if frame.pc < bc.len() && bc[frame.pc] == OpCode::Drop as u8 {
                                        frame.pc += 1;
                                        true
                                    } else {
                                        false
                                    }
                                };
                                if argc == 1 {
                                    let mut arg =
                                        self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                                    if arg.is_string() {
                                        arg = self.promote_string(arg, bytecode);
                                    }
                                    let array = unsafe { self.get_array_mut_unchecked(arr_idx) };
                                    array.push(arg);
                                } else {
                                    let stack_len = self.stack.len();
                                    for i in 0..argc {
                                        let mut arg = self.stack.get_raw(stack_len - argc + i);
                                        if arg.is_string() {
                                            arg = self.promote_string(arg, bytecode);
                                        }
                                        // SAFETY: arr_idx is valid if this_val came from a live array value
                                        let array =
                                            unsafe { self.get_array_mut_unchecked(arr_idx) };
                                        array.push(arg);
                                    }
                                    self.stack.drop_n(argc);
                                }
                                if !discard_result {
                                    let new_len =
                                        unsafe { self.get_array_unchecked(arr_idx) }.len() as i32;
                                    self.stack.push(Value::int(new_len));
                                }
                                continue;
                            }
                        }

                        // Promote compile-time string arguments in-place on stack.
                        {
                            let stack_len = self.stack.len();
                            for i in 0..argc {
                                let slot = stack_len - argc + i;
                                let val = self.stack.get_raw(slot);
                                if val.is_string() {
                                    let promoted = self.promote_string(val, bytecode);
                                    self.stack.set_raw(slot, promoted);
                                }
                            }
                        }

                        let result = match argc {
                            0 => self.call_native_func(native_idx, this_val, &[]),
                            1 => {
                                let a0 =
                                    self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                                let args = [a0];
                                self.call_native_func(native_idx, this_val, &args)
                            }
                            2 => {
                                let a1 =
                                    self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                                let a0 =
                                    self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                                let args = [a0, a1];
                                self.call_native_func(native_idx, this_val, &args)
                            }
                            3 => {
                                let a2 =
                                    self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                                let a1 =
                                    self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                                let a0 =
                                    self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                                let args = [a0, a1, a2];
                                self.call_native_func(native_idx, this_val, &args)
                            }
                            _ => {
                                let stack_len = self.stack.len();
                                let mut args = Vec::with_capacity(argc);
                                for i in 0..argc {
                                    args.push(self.stack.get_raw(stack_len - argc + i));
                                }
                                self.stack.drop_n(argc);
                                self.call_native_func(native_idx, this_val, &args)
                            }
                        };
                        if let Some(result) = self.try_op(result)? {
                            self.stack.push(result);
                        }
                        continue;
                    }

                    // Promote compile-time string arguments to runtime strings before
                    // entering a user-defined method frame so parameter string values
                    // stay valid across bytecode/string-constant tables.
                    {
                        let stack_len = self.stack.len();
                        for i in 0..argc {
                            let slot = stack_len - argc + i;
                            let val = self.stack.get_raw(slot);
                            if val.is_string() {
                                let promoted = self.promote_string(val, bytecode);
                                self.stack.set_raw(slot, promoted);
                            }
                        }
                    }

                    // Determine if this is a closure or a regular function
                    let (callee_bytecode, callee_closure_idx): (&FunctionBytecode, Option<usize>) =
                        if let Some(closure_idx) = method_val.to_closure_idx() {
                            let closure = self.get_closure(closure_idx).ok_or_else(|| {
                                InterpreterError::InternalError(format!(
                                    "invalid closure index: {}",
                                    closure_idx
                                ))
                            })?;
                            (unsafe { &*closure.bytecode }, Some(closure_idx as usize))
                        } else if let Some(ptr) = method_val.to_func_ptr() {
                            (unsafe { &*ptr }, None)
                        } else if let Some(idx) = method_val.to_func_idx() {
                            let bc =
                                bytecode.inner_functions.get(idx as usize).ok_or_else(|| {
                                    InterpreterError::InternalError(format!(
                                        "invalid function index: {}",
                                        idx
                                    ))
                                })?;
                            (bc, None)
                        } else {
                            self.try_handle_runtime_error(InterpreterError::TypeError(
                                "not a function".to_string(),
                            ))?;
                            continue;
                        };

                    // Check recursion limit
                    if self.call_stack.len() >= self.max_recursion {
                        self.try_handle_runtime_error(InterpreterError::InternalError(
                            "maximum call stack size exceeded".to_string(),
                        ))?;
                        continue;
                    }

                    let callee_frame_ptr = self.stack.len() - argc;

                    // Pad or truncate: add undefined for missing args
                    let expected = callee_bytecode.arg_count as usize;
                    if argc < expected {
                        for _ in 0..(expected - argc) {
                            self.stack.push(Value::undefined());
                        }
                    } else if argc > expected {
                        self.stack.drop_n(argc - expected);
                    }

                    // Allocate space for locals (beyond arguments)
                    let extra_locals = callee_bytecode
                        .local_count
                        .saturating_sub(callee_bytecode.arg_count);
                    for _ in 0..extra_locals {
                        self.stack.push(Value::undefined());
                    }

                    // Create frame with the object as 'this'
                    let callee_frame = if let Some(closure_idx) = callee_closure_idx {
                        CallFrame::new_closure(
                            callee_bytecode as *const _,
                            callee_frame_ptr,
                            argc.min(u16::MAX as usize) as u16,
                            this_val, // Pass the object as 'this'
                            method_val,
                            closure_idx,
                        )
                    } else {
                        CallFrame::new(
                            callee_bytecode as *const _,
                            callee_frame_ptr,
                            argc.min(u16::MAX as usize) as u16,
                            this_val, // Pass the object as 'this'
                            method_val,
                        )
                    };
                    self.call_stack.push(callee_frame);
                }

                // TypeOf operator
                op if op == OpCode::TypeOf as u8 => {
                    use crate::value::{
                        STR_BOOLEAN, STR_FUNCTION, STR_NUMBER, STR_OBJECT, STR_STRING,
                        STR_UNDEFINED,
                    };

                    let val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let type_idx = if val.is_undefined() {
                        STR_UNDEFINED
                    } else if val.is_null() {
                        STR_OBJECT // typeof null === "object" (JavaScript quirk)
                    } else if val.is_bool() {
                        STR_BOOLEAN
                    } else if val.is_int() || val.is_float() {
                        STR_NUMBER
                    } else if val.is_string() {
                        STR_STRING
                    } else if val.is_func()
                        || val.to_func_ptr().is_some()
                        || val.is_closure()
                        || val.is_native_func()
                    {
                        STR_FUNCTION
                    } else {
                        // Objects, arrays, and all other pointers/objects
                        STR_OBJECT
                    };
                    self.stack.push(Value::string(type_idx));
                }

                // Nop
                op if op == OpCode::Nop as u8 => {
                    // Do nothing
                }

                // Print (built-in print statement)
                op if op == OpCode::Print as u8 => {
                    let val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let frame = self.call_stack.last().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };

                    // Convert value to string representation
                    let _output = if val.is_string() {
                        self.get_string_content(val, bytecode)
                            .unwrap_or_default()
                            .to_string()
                    } else if let Some(n) = val.to_i32() {
                        n.to_string()
                    } else if let Some(f) = val.to_f32() {
                        crate::value::format_float(f)
                    } else if val.is_bool() {
                        if val.to_bool().unwrap_or(false) {
                            "true"
                        } else {
                            "false"
                        }
                        .to_string()
                    } else if val.is_null() {
                        "null".to_string()
                    } else if val.is_undefined() {
                        "undefined".to_string()
                    } else if val.is_func() || val.to_func_ptr().is_some() {
                        "[function]".to_string()
                    } else {
                        "[object]".to_string()
                    };

                    #[cfg(feature = "std")]
                    println!("{}", _output);
                }

                // GetGlobal - look up global variable by name
                op if op == OpCode::GetGlobal as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let name_idx = u16::from_le_bytes([bc[frame.pc], bc[frame.pc + 1]]);
                    frame.pc += 2;

                    // Get the name from constant pool
                    let name = bytecode
                        .constants
                        .get(name_idx as usize)
                        .and_then(|v| {
                            if v.is_string() {
                                let str_idx = v.to_string_idx()?;
                                bytecode
                                    .string_constants
                                    .get(str_idx as usize)
                                    .map(|s| s.as_str())
                            } else {
                                None
                            }
                        })
                        .ok_or_else(|| {
                            InterpreterError::InternalError(format!(
                                "invalid global name constant: {}",
                                name_idx
                            ))
                        })?;

                    let val = self.lookup_global_value(name);

                    if let Some(v) = val {
                        self.stack.push(v);
                    } else {
                        return Err(InterpreterError::ReferenceError(format!(
                            "{} is not defined",
                            name
                        )));
                    }
                }

                // GetGlobalOrUndefined - like GetGlobal, but missing names become undefined.
                // Used to implement `typeof bareIdentifier` semantics.
                op if op == OpCode::GetGlobalOrUndefined as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let name_idx = u16::from_le_bytes([bc[frame.pc], bc[frame.pc + 1]]);
                    frame.pc += 2;

                    let name = bytecode
                        .constants
                        .get(name_idx as usize)
                        .and_then(|v| {
                            if v.is_string() {
                                let str_idx = v.to_string_idx()?;
                                bytecode
                                    .string_constants
                                    .get(str_idx as usize)
                                    .map(|s| s.as_str())
                            } else {
                                None
                            }
                        })
                        .ok_or_else(|| {
                            InterpreterError::InternalError(format!(
                                "invalid global name constant: {}",
                                name_idx
                            ))
                        })?;

                    let val = self
                        .lookup_global_value(name)
                        .unwrap_or_else(Value::undefined);
                    self.stack.push(val);
                }

                // SetGlobal - store top-level variable by name (16-bit constant index)
                op if op == OpCode::SetGlobal as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let name_idx = u16::from_le_bytes([bc[frame.pc], bc[frame.pc + 1]]);
                    frame.pc += 2;

                    let name = bytecode
                        .constants
                        .get(name_idx as usize)
                        .and_then(|v| {
                            if v.is_string() {
                                let str_idx = v.to_string_idx()?;
                                bytecode
                                    .string_constants
                                    .get(str_idx as usize)
                                    .map(|s| s.as_str())
                            } else {
                                None
                            }
                        })
                        .ok_or_else(|| {
                            InterpreterError::InternalError(format!(
                                "invalid global name constant: {}",
                                name_idx
                            ))
                        })?;

                    // Peek at stack top (SetGlobal does NOT consume the value;
                    // PutLoc follows and consumes it)
                    let val = self.stack.peek().unwrap_or_default();

                    // Update or insert into global_vars
                    self.global_vars.insert(String::from(name), val);
                }

                // Catch - set up exception handler
                op if op == OpCode::Catch as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let offset = i32::from_le_bytes([
                        bc[frame.pc],
                        bc[frame.pc + 1],
                        bc[frame.pc + 2],
                        bc[frame.pc + 3],
                    ]);
                    frame.pc += 4;

                    // Calculate catch PC (relative to end of instruction)
                    let catch_pc = (frame.pc as i32 + offset) as usize;

                    // Push exception handler
                    self.exception_handlers.push(ExceptionHandler {
                        frame_depth: self.call_stack.len(),
                        catch_pc,
                        stack_depth: self.stack.len(),
                    });
                }

                // DropCatch - remove exception handler
                op if op == OpCode::DropCatch as u8 => {
                    // Pop the top exception handler
                    self.exception_handlers.pop();
                }

                // Throw - throw exception
                op if op == OpCode::Throw as u8 => {
                    let exception = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    self.route_exception_to_handler(exception)?;
                }

                // ArrayFrom - create array from stack elements
                op if op == OpCode::ArrayFrom as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    // Read number of elements (16-bit)
                    let count = u16::from_le_bytes([bc[frame.pc], bc[frame.pc + 1]]) as usize;
                    frame.pc += 2;

                    // Pop elements from stack (they were pushed in order)
                    let mut elements = Vec::with_capacity(count);
                    for _ in 0..count {
                        elements.push(self.stack.pop().ok_or(InterpreterError::StackUnderflow)?);
                    }
                    elements.reverse(); // Elements were pushed left-to-right

                    // Create array and push reference
                    let arr_val = self.create_array(elements);
                    self.stack.push(arr_val);
                }

                // GetArrayEl - get array element: arr idx -> val
                op if op == OpCode::GetArrayEl as u8 => {
                    // SAFETY: Stack operations are valid for well-formed bytecode
                    let (idx, arr) = unsafe { self.stack.pop2_unchecked() };

                    let (branch_op, branch_offset) = {
                        let frame = self.call_stack.last_mut().unwrap();
                        let bytecode = unsafe { &*frame.bytecode };
                        let bc = &bytecode.bytecode;
                        let pc = frame.pc;
                        if pc + 4 < bc.len() {
                            let next = unsafe { *bc.get_unchecked(pc) };
                            if next == OpCode::IfFalse as u8 || next == OpCode::IfTrue as u8 {
                                let offset = i32::from_le_bytes([
                                    unsafe { *bc.get_unchecked(pc + 1) },
                                    unsafe { *bc.get_unchecked(pc + 2) },
                                    unsafe { *bc.get_unchecked(pc + 3) },
                                    unsafe { *bc.get_unchecked(pc + 4) },
                                ]);
                                frame.pc += 5;
                                (next, offset)
                            } else {
                                (0, 0)
                            }
                        } else {
                            (0, 0)
                        }
                    };

                    // Fast path: regular array with integer index
                    if let Some((arr_idx, index)) = Self::dense_array_access(arr, idx) {
                        // SAFETY: Array index is valid for arrays we created
                        let array = unsafe { self.get_array_unchecked(arr_idx) };
                        let val = if index < array.len() {
                            // SAFETY: We just checked index < len
                            unsafe { *array.get_unchecked(index) }
                        } else {
                            Value::undefined()
                        };
                        if branch_op != 0 {
                            let branch_on_true = branch_op == OpCode::IfTrue as u8;
                            let raw = val.raw().0;
                            let take_branch = if raw == crate::value::RawValue::TRUE.0 {
                                branch_on_true
                            } else if raw == crate::value::RawValue::FALSE.0
                                || raw == crate::value::RawValue::NULL.0
                                || raw == crate::value::RawValue::UNDEFINED.0
                            {
                                !branch_on_true
                            } else if val.is_int() {
                                (unsafe { val.to_i32_unchecked() != 0 }) == branch_on_true
                            } else {
                                Self::branch_matches_value(val, branch_on_true)
                            };
                            if take_branch {
                                let frame = self.call_stack.last_mut().unwrap();
                                frame.pc = (frame.pc as i32 + branch_offset) as usize;
                            }
                        } else {
                            self.stack.push(val);
                        }
                        continue;
                    }

                    // Check if it's a typed array
                    if let Some(typed_idx) = arr.to_typed_array_idx() {
                        let index = idx.to_i32().ok_or_else(|| {
                            InterpreterError::TypeError(
                                "typed array index must be a number".to_string(),
                            )
                        })? as usize;

                        let val = self
                            .typed_arrays
                            .get(typed_idx as usize)
                            .and_then(|ta| ta.get(index))
                            .unwrap_or_default();
                        if branch_op != 0 {
                            let branch_on_true = branch_op == OpCode::IfTrue as u8;
                            if Self::branch_matches_value(val, branch_on_true) {
                                let frame = self.call_stack.last_mut().unwrap();
                                frame.pc = (frame.pc as i32 + branch_offset) as usize;
                            }
                        } else {
                            self.stack.push(val);
                        }
                        continue;
                    }

                    // Slow path for non-array or non-integer index
                    let arr_idx = arr.to_array_idx().ok_or_else(|| {
                        InterpreterError::TypeError("cannot read property of non-array".to_string())
                    })?;

                    let array = self.get_array(arr_idx).ok_or_else(|| {
                        InterpreterError::InternalError("invalid array index".to_string())
                    })?;

                    let index = idx.to_i32().ok_or_else(|| {
                        InterpreterError::TypeError("array index must be a number".to_string())
                    })? as usize;

                    let val = array.get(index).copied().unwrap_or_default();
                    if branch_op != 0 {
                        let branch_on_true = branch_op == OpCode::IfTrue as u8;
                        if Self::branch_matches_value(val, branch_on_true) {
                            let frame = self.call_stack.last_mut().unwrap();
                            frame.pc = (frame.pc as i32 + branch_offset) as usize;
                        }
                    } else {
                        self.stack.push(val);
                    }
                }

                op if op == OpCode::GetArrayElDiscard as u8 => {
                    let (idx, arr) = unsafe { self.stack.pop2_unchecked() };

                    let idx_raw = idx.raw().0;
                    let arr_raw = arr.raw().0;
                    if (idx_raw & 1) == 0
                        && (arr_raw & 0x1f) == crate::value::SpecialTag::CatchOffset as u64
                        && (((arr_raw >> 5) as i32) & crate::value::ARRAY_INDEX_MARKER) != 0
                    {
                        continue;
                    }

                    if let Some(typed_idx) = arr.to_typed_array_idx() {
                        if idx.is_int() {
                            let _ = typed_idx;
                            continue;
                        }

                        let index = idx.to_i32().ok_or_else(|| {
                            InterpreterError::TypeError(
                                "typed array index must be a number".to_string(),
                            )
                        })? as usize;

                        let _ = self
                            .typed_arrays
                            .get(typed_idx as usize)
                            .and_then(|ta| ta.get(index))
                            .unwrap_or_default();
                        continue;
                    }

                    let arr_idx = arr.to_array_idx().ok_or_else(|| {
                        InterpreterError::TypeError("cannot read property of non-array".to_string())
                    })?;

                    let array = self.get_array(arr_idx).ok_or_else(|| {
                        InterpreterError::InternalError("invalid array index".to_string())
                    })?;

                    let index = idx.to_i32().ok_or_else(|| {
                        InterpreterError::TypeError("array index must be a number".to_string())
                    })? as usize;

                    let _ = array.get(index).copied().unwrap_or_default();
                }

                // GetArrayEl2 - get array element, keep object: arr idx -> arr val
                op if op == OpCode::GetArrayEl2 as u8 => {
                    let idx = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let arr = self.stack.peek().ok_or(InterpreterError::StackUnderflow)?;

                    // Check if it's a typed array
                    if let Some(typed_idx) = arr.to_typed_array_idx() {
                        let index = idx.to_i32().ok_or_else(|| {
                            InterpreterError::TypeError(
                                "typed array index must be a number".to_string(),
                            )
                        })? as usize;

                        let val = self
                            .typed_arrays
                            .get(typed_idx as usize)
                            .and_then(|ta| ta.get(index))
                            .unwrap_or_default();
                        self.stack.push(val);
                        continue;
                    }

                    // Get the array
                    let arr_idx = arr.to_array_idx().ok_or_else(|| {
                        InterpreterError::TypeError("cannot read property of non-array".to_string())
                    })?;

                    let array = self.get_array(arr_idx).ok_or_else(|| {
                        InterpreterError::InternalError("invalid array index".to_string())
                    })?;

                    // Get the element
                    let index = idx.to_i32().ok_or_else(|| {
                        InterpreterError::TypeError("array index must be a number".to_string())
                    })? as usize;

                    let val = array.get(index).copied().unwrap_or_default();
                    self.stack.push(val);
                }

                // PutArrayEl - set array element: arr idx val -> val
                op if op == OpCode::PutArrayEl as u8 => {
                    let discard_result = {
                        let frame = self.call_stack.last_mut().unwrap();
                        let bytecode = unsafe { &*frame.bytecode };
                        let bc = &bytecode.bytecode;
                        if frame.pc < bc.len() && bc[frame.pc] == OpCode::Drop as u8 {
                            frame.pc += 1;
                            true
                        } else {
                            false
                        }
                    };

                    // SAFETY: Stack operations are valid for well-formed bytecode
                    let (val, idx, arr) = unsafe { self.stack.pop3_unchecked() };

                    // Fast path: regular array with integer index within bounds
                    if let Some((arr_idx, index)) = Self::dense_array_access(arr, idx) {
                        // SAFETY: Array index is valid for arrays we created
                        let array = unsafe { self.get_array_mut_unchecked(arr_idx) };
                        if index < array.len() {
                            unsafe { *array.get_unchecked_mut(index) = val };
                        } else {
                            // Extend array if index is out of bounds
                            array.resize(index + 1, Value::undefined());
                            array[index] = val;
                        }
                        if !discard_result {
                            self.stack.push(val);
                        }
                        continue;
                    }

                    // Check if it's a typed array
                    if let Some(typed_idx) = arr.to_typed_array_idx() {
                        let index = idx.to_i32().ok_or_else(|| {
                            InterpreterError::TypeError(
                                "typed array index must be a number".to_string(),
                            )
                        })? as usize;

                        if let Some(ta) = self.typed_arrays.get_mut(typed_idx as usize) {
                            ta.set(index, val);
                        }
                        if !discard_result {
                            self.stack.push(val);
                        }
                        continue;
                    }

                    // Slow path for non-array or non-integer index
                    let arr_idx = arr.to_array_idx().ok_or_else(|| {
                        InterpreterError::TypeError("cannot set property of non-array".to_string())
                    })?;

                    let index = idx.to_i32().ok_or_else(|| {
                        InterpreterError::TypeError("array index must be a number".to_string())
                    })? as usize;

                    let array = self.get_array_mut(arr_idx).ok_or_else(|| {
                        InterpreterError::InternalError("invalid array index".to_string())
                    })?;

                    // Extend array if index is out of bounds
                    if index >= array.len() {
                        array.resize(index + 1, Value::undefined());
                    }
                    array[index] = val;

                    // Push the assigned value back (assignment is an expression)
                    if !discard_result {
                        self.stack.push(val);
                    }
                }

                op if op == OpCode::PutArrayElFalseDrop as u8 => {
                    let (idx, arr) = unsafe { self.stack.pop2_unchecked() };
                    let val = Value::bool(false);

                    if let Some((arr_idx, index)) = Self::dense_array_access(arr, idx) {
                        let array = unsafe { self.get_array_mut_unchecked(arr_idx) };
                        if index < array.len() {
                            unsafe { *array.get_unchecked_mut(index) = val };
                        } else {
                            array.resize(index + 1, Value::undefined());
                            array[index] = val;
                        }
                        continue;
                    }

                    if let Some(typed_idx) = arr.to_typed_array_idx() {
                        let index = idx.to_i32().ok_or_else(|| {
                            InterpreterError::TypeError(
                                "typed array index must be a number".to_string(),
                            )
                        })? as usize;

                        if let Some(ta) = self.typed_arrays.get_mut(typed_idx as usize) {
                            ta.set(index, val);
                        }
                        continue;
                    }

                    let arr_idx = arr.to_array_idx().ok_or_else(|| {
                        InterpreterError::TypeError("cannot set property of non-array".to_string())
                    })?;

                    let index = idx.to_i32().ok_or_else(|| {
                        InterpreterError::TypeError("array index must be a number".to_string())
                    })? as usize;

                    let array = self.get_array_mut(arr_idx).ok_or_else(|| {
                        InterpreterError::InternalError("invalid array index".to_string())
                    })?;

                    if index >= array.len() {
                        array.resize(index + 1, Value::undefined());
                    }
                    array[index] = val;
                }

                // GetLength - get `.length` property through a dedicated fast path.
                op if op == OpCode::GetLength as u8 => {
                    let obj = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let val = self.get_length_value(obj);
                    self.stack.push(val);
                }

                // GetLength2 - get `.length` but keep the base object on stack.
                op if op == OpCode::GetLength2 as u8 => {
                    let obj = self.stack.peek().ok_or(InterpreterError::StackUnderflow)?;
                    let val = self.get_length_value(obj);
                    self.stack.push(val);
                }

                // GetField - get object property: obj -> value
                op if op == OpCode::GetField as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let str_idx = u16::from_le_bytes([bc[frame.pc], bc[frame.pc + 1]]) as usize;
                    frame.pc += 2;

                    let obj = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;

                    // TypeError: Cannot read properties of null/undefined
                    if obj.is_null() || obj.is_undefined() {
                        let prop_name = bytecode
                            .string_constants
                            .get(str_idx)
                            .map(|s| s.as_str())
                            .unwrap_or("?");
                        let type_name = if obj.is_null() { "null" } else { "undefined" };
                        self.try_handle_runtime_error(InterpreterError::TypeError(format!(
                            "Cannot read properties of {} (reading '{}')",
                            type_name, prop_name
                        )))?;
                        continue;
                    }

                    // Get property name from string constants
                    let prop_name = bytecode.string_constants.get(str_idx).ok_or_else(|| {
                        InterpreterError::InternalError(format!(
                            "invalid string index: {}",
                            str_idx
                        ))
                    })?;

                    let val = if let Some(obj_idx) = obj.to_object_idx() {
                        self.object_get_property(obj_idx, prop_name)?
                    } else {
                        self.get_field_value(obj, prop_name)?
                    };
                    self.stack.push(val);
                }

                // GetField2 - get object property but keep object: obj -> obj value
                // Used for method calls where we need the object as 'this'
                op if op == OpCode::GetField2 as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let str_idx = u16::from_le_bytes([bc[frame.pc], bc[frame.pc + 1]]) as usize;
                    frame.pc += 2;

                    // Peek at the object (don't pop - we need to keep it for 'this')
                    let obj = self.stack.peek().ok_or(InterpreterError::StackUnderflow)?;

                    // TypeError: Cannot read properties of null/undefined
                    if obj.is_null() || obj.is_undefined() {
                        let prop_name = bytecode
                            .string_constants
                            .get(str_idx)
                            .map(|s| s.as_str())
                            .unwrap_or("?");
                        let type_name = if obj.is_null() { "null" } else { "undefined" };
                        self.try_handle_runtime_error(InterpreterError::TypeError(format!(
                            "Cannot read properties of {} (reading '{}')",
                            type_name, prop_name
                        )))?;
                        continue;
                    }

                    // Get property name from string constants
                    let prop_name = bytecode.string_constants.get(str_idx).ok_or_else(|| {
                        InterpreterError::InternalError(format!(
                            "invalid string index: {}",
                            str_idx
                        ))
                    })?;

                    let val = if let Some(obj_idx) = obj.to_object_idx() {
                        self.object_get_property(obj_idx, prop_name)?
                    } else {
                        self.get_field_value(obj, prop_name)?
                    };

                    // Push the property value (object is still on stack below it)
                    self.stack.push(val);
                }

                op if op == OpCode::GetFieldChain4 as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let i0 = u16::from_le_bytes([bc[frame.pc], bc[frame.pc + 1]]) as usize;
                    let i1 = u16::from_le_bytes([bc[frame.pc + 2], bc[frame.pc + 3]]) as usize;
                    let i2 = u16::from_le_bytes([bc[frame.pc + 4], bc[frame.pc + 5]]) as usize;
                    let i3 = u16::from_le_bytes([bc[frame.pc + 6], bc[frame.pc + 7]]) as usize;
                    frame.pc += 8;
                    let p0 = bytecode.string_constants.get(i0).ok_or_else(|| {
                        InterpreterError::InternalError(format!("invalid string index: {}", i0))
                    })?;
                    let p1 = bytecode.string_constants.get(i1).ok_or_else(|| {
                        InterpreterError::InternalError(format!("invalid string index: {}", i1))
                    })?;
                    let p2 = bytecode.string_constants.get(i2).ok_or_else(|| {
                        InterpreterError::InternalError(format!("invalid string index: {}", i2))
                    })?;
                    let p3 = bytecode.string_constants.get(i3).ok_or_else(|| {
                        InterpreterError::InternalError(format!("invalid string index: {}", i3))
                    })?;

                    let mut current = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;

                    macro_rules! step {
                        ($prop:expr) => {{
                            if current.is_null() || current.is_undefined() {
                                let type_name = if current.is_null() {
                                    "null"
                                } else {
                                    "undefined"
                                };
                                self.try_handle_runtime_error(InterpreterError::TypeError(
                                    format!(
                                        "Cannot read properties of {} (reading '{}')",
                                        type_name, $prop
                                    ),
                                ))?;
                                continue;
                            }

                            current = if let Some(obj_idx) = current.to_object_idx() {
                                self.object_get_property(obj_idx, $prop)?
                            } else {
                                self.get_field_value(current, $prop)?
                            };
                        }};
                    }

                    step!(p0);
                    step!(p1);
                    step!(p2);
                    step!(p3);

                    self.stack.push(current);
                }

                // GetFieldDyn - get property by dynamic key: obj key -> val
                op if op == OpCode::GetFieldDyn as u8 => {
                    let key = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let obj = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;

                    let prop_name = if let Some(idx) = key.to_string_idx() {
                        if idx >= Self::RUNTIME_STRING_OFFSET {
                            let runtime_idx = (idx - Self::RUNTIME_STRING_OFFSET) as usize;
                            self.runtime_string_as_str(runtime_idx)
                                .map(|s| s.to_string())
                                .unwrap_or_default()
                        } else {
                            self.get_string_by_idx(idx)
                                .map(|s| s.to_string())
                                .unwrap_or_default()
                        }
                    } else {
                        key.to_string()
                    };

                    let val = self.get_field_value(obj, &prop_name)?;
                    self.stack.push(val);
                }

                // PutFieldDyn - set property by dynamic key: obj key val -> val
                op if op == OpCode::PutFieldDyn as u8 => {
                    let val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let key = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let obj = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;

                    let prop_name = if let Some(idx) = key.to_string_idx() {
                        if idx >= Self::RUNTIME_STRING_OFFSET {
                            let runtime_idx = (idx - Self::RUNTIME_STRING_OFFSET) as usize;
                            self.runtime_string_as_str(runtime_idx)
                                .map(|s| s.to_string())
                                .unwrap_or_default()
                        } else {
                            self.get_string_by_idx(idx)
                                .map(|s| s.to_string())
                                .unwrap_or_default()
                        }
                    } else {
                        key.to_string()
                    };

                    if obj.is_array() {
                        if let Some(arr_idx) = obj.to_array_idx() {
                            if let Ok(index) = prop_name.parse::<usize>() {
                                if let Some(arr_data) = self.arrays.get_mut(arr_idx as usize) {
                                    if index >= arr_data.len() {
                                        arr_data.resize(index + 1, Value::undefined());
                                    }
                                    arr_data[index] = val;
                                }
                            }
                            // Non-numeric keys on arrays are silently ignored
                        }
                    } else if let Some(typed_idx) = obj.to_typed_array_idx() {
                        if let Ok(index) = prop_name.parse::<usize>() {
                            if let Some(ta) = self.typed_arrays.get_mut(typed_idx as usize) {
                                ta.set(index, val);
                            }
                        }
                        // Non-numeric keys on typed arrays are silently ignored
                    } else if let Some(obj_idx) = obj.to_object_idx() {
                        self.object_set_property(obj_idx, prop_name, val)?;
                    }
                    self.stack.push(val);
                }

                op if op == OpCode::SwitchCaseI8 as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let case_val = bc[frame.pc] as i8 as i32;
                    let offset = i32::from_le_bytes([
                        bc[frame.pc + 1],
                        bc[frame.pc + 2],
                        bc[frame.pc + 3],
                        bc[frame.pc + 4],
                    ]);
                    frame.pc += 5;

                    let top = self.stack.peek().ok_or(InterpreterError::StackUnderflow)?;
                    let is_match = if let Some(i) = top.to_i32() {
                        i == case_val
                    } else if let Some(f) = top.to_f32() {
                        !f.is_nan() && f == case_val as Float
                    } else {
                        false
                    };

                    if is_match {
                        let frame = self.call_stack.last_mut().unwrap();
                        frame.pc = (frame.pc as i32 + offset) as usize;
                    }
                }

                // GetArrayPush2 - get `.push`, keep object: obj -> obj value
                op if op == OpCode::GetArrayPush2 as u8 => {
                    let obj = self.stack.peek().ok_or(InterpreterError::StackUnderflow)?;

                    if obj.is_null() || obj.is_undefined() {
                        let type_name = if obj.is_null() { "null" } else { "undefined" };
                        self.try_handle_runtime_error(InterpreterError::TypeError(format!(
                            "Cannot read properties of {} (reading 'push')",
                            type_name
                        )))?;
                        continue;
                    }

                    let val = if obj.is_array() {
                        // Try to get native function from cache
                        if let Some(&idx) = self.native_func_index.get("Array.prototype.push") {
                            Value::native_func(idx)
                        } else {
                            self.get_field_value(obj, "push").unwrap_or_default()
                        }
                    } else {
                        self.get_field_value(obj, "push")?
                    };
                    self.stack.push(val);
                }

                // PutField - set object property: obj val -> val
                op if op == OpCode::PutField as u8 => {
                    let frame = self.call_stack.last_mut().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };
                    let bc = &bytecode.bytecode;
                    let str_idx = u16::from_le_bytes([bc[frame.pc], bc[frame.pc + 1]]) as usize;
                    frame.pc += 2;

                    let val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let obj = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;

                    // Get property name from string constants
                    let prop_name = bytecode
                        .string_constants
                        .get(str_idx)
                        .ok_or_else(|| {
                            InterpreterError::InternalError(format!(
                                "invalid string index: {}",
                                str_idx
                            ))
                        })?
                        .clone();

                    // Promote compile-time strings to runtime strings so they
                    // remain valid when the object is accessed from other scopes.
                    let val = self.promote_string(val, bytecode);

                    // Set property on object
                    if let Some(obj_idx) = obj.to_object_idx() {
                        self.object_set_property(obj_idx, prop_name, val)?;
                    }
                    // Push the assigned value back (assignment is an expression)
                    self.stack.push(val);
                }

                // In operator: prop in obj -> bool
                op if op == OpCode::In as u8 => {
                    let frame = self.call_stack.last().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };

                    let obj = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let prop = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;

                    let result = if let Some(obj_idx) = obj.to_object_idx() {
                        // Check if property exists in object
                        // Convert prop to string, checking bytecode string constants
                        let prop_name = if prop.is_string() {
                            if let Some(str_idx) = prop.to_string_idx() {
                                // Check built-in strings first
                                use crate::value::{
                                    STR_BOOLEAN, STR_EMPTY, STR_FUNCTION, STR_NUMBER, STR_OBJECT,
                                    STR_STRING, STR_UNDEFINED,
                                };
                                match str_idx {
                                    STR_UNDEFINED => Some("undefined".to_string()),
                                    STR_OBJECT => Some("object".to_string()),
                                    STR_BOOLEAN => Some("boolean".to_string()),
                                    STR_NUMBER => Some("number".to_string()),
                                    STR_FUNCTION => Some("function".to_string()),
                                    STR_STRING => Some("string".to_string()),
                                    STR_EMPTY => Some(String::new()),
                                    _ => {
                                        if str_idx >= Self::RUNTIME_STRING_OFFSET {
                                            self.runtime_strings
                                                .get(
                                                    (str_idx - Self::RUNTIME_STRING_OFFSET)
                                                        as usize,
                                                )
                                                .and_then(|_| self.get_string_by_idx(str_idx))
                                                .map(|s| s.to_string())
                                        } else {
                                            // Compile-time string constant
                                            bytecode.string_constants.get(str_idx as usize).cloned()
                                        }
                                    }
                                }
                            } else {
                                None
                            }
                        } else {
                            prop.to_i32().map(|n| n.to_string())
                        };

                        if let Some(name) = prop_name {
                            let obj_props = self.get_object(obj_idx);
                            let exists = obj_props
                                .map(|props| {
                                    props.properties.iter().any(|(k, _)| k == &name)
                                        || props.accessors.iter().any(|a| a.key == name)
                                })
                                .unwrap_or(false);
                            Value::bool(exists)
                        } else {
                            Value::bool(false)
                        }
                    } else if let Some(arr_idx) = obj.to_array_idx() {
                        // Check if index exists in array
                        if let Some(idx) = prop.to_i32() {
                            let arr = self.get_array(arr_idx);
                            let exists = arr
                                .map(|a| idx >= 0 && (idx as usize) < a.len())
                                .unwrap_or(false);
                            Value::bool(exists)
                        } else {
                            Value::bool(false)
                        }
                    } else {
                        Value::bool(false)
                    };
                    self.stack.push(result);
                }

                // Delete operator: obj prop -> bool
                op if op == OpCode::Delete as u8 => {
                    let frame = self.call_stack.last().unwrap();
                    let bytecode = unsafe { &*frame.bytecode };

                    let prop = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let obj = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;

                    let result = if let Some(obj_idx) = obj.to_object_idx() {
                        // Convert prop to string, checking bytecode string constants
                        let prop_name = if prop.is_string() {
                            if let Some(str_idx) = prop.to_string_idx() {
                                use crate::value::{
                                    STR_BOOLEAN, STR_EMPTY, STR_FUNCTION, STR_NUMBER, STR_OBJECT,
                                    STR_STRING, STR_UNDEFINED,
                                };
                                match str_idx {
                                    STR_UNDEFINED => Some("undefined".to_string()),
                                    STR_OBJECT => Some("object".to_string()),
                                    STR_BOOLEAN => Some("boolean".to_string()),
                                    STR_NUMBER => Some("number".to_string()),
                                    STR_FUNCTION => Some("function".to_string()),
                                    STR_STRING => Some("string".to_string()),
                                    STR_EMPTY => Some(String::new()),
                                    _ => {
                                        if str_idx >= Self::RUNTIME_STRING_OFFSET {
                                            self.runtime_strings
                                                .get(
                                                    (str_idx - Self::RUNTIME_STRING_OFFSET)
                                                        as usize,
                                                )
                                                .and_then(|_| self.get_string_by_idx(str_idx))
                                                .map(|s| s.to_string())
                                        } else {
                                            bytecode.string_constants.get(str_idx as usize).cloned()
                                        }
                                    }
                                }
                            } else {
                                None
                            }
                        } else {
                            prop.to_i32().map(|n| n.to_string())
                        };

                        // Delete property from object
                        if let Some(name) = prop_name {
                            if let Some(obj_props) = self.get_object_mut(obj_idx) {
                                let orig_props_len = obj_props.properties.len();
                                let orig_accessors_len = obj_props.accessors.len();
                                obj_props.properties.retain(|(k, _)| k != &name);
                                obj_props.accessors.retain(|a| a.key != name);
                                Value::bool(
                                    obj_props.properties.len() < orig_props_len
                                        || obj_props.accessors.len() < orig_accessors_len,
                                )
                            } else {
                                Value::bool(false)
                            }
                        } else {
                            Value::bool(false)
                        }
                    } else if let Some(arr_idx) = obj.to_array_idx() {
                        // For arrays, set element to undefined (don't actually remove)
                        if let Some(idx) = prop.to_i32() {
                            if let Some(arr) = self.get_array_mut(arr_idx) {
                                if idx >= 0 && (idx as usize) < arr.len() {
                                    arr[idx as usize] = Value::undefined();
                                    Value::bool(true)
                                } else {
                                    Value::bool(true) // Deleting non-existent index returns true
                                }
                            } else {
                                Value::bool(false)
                            }
                        } else {
                            Value::bool(false)
                        }
                    } else {
                        Value::bool(true) // delete on non-object returns true
                    };
                    self.stack.push(result);
                }

                // InstanceOf operator: obj ctor -> bool
                op if op == OpCode::InstanceOf as u8 => {
                    let ctor = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let obj = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;

                    let result = if let Some(obj_idx) = obj.to_object_idx() {
                        // Get the constructor stored when the object was created
                        if let Some(obj_instance) = self.get_object(obj_idx) {
                            if let Some(stored_ctor) = obj_instance.constructor {
                                // Compare if the stored constructor matches the right operand
                                // For closures, compare the closure indices
                                if let (Some(stored_idx), Some(ctor_idx)) =
                                    (stored_ctor.to_closure_idx(), ctor.to_closure_idx())
                                {
                                    // Same closure instance
                                    Value::bool(stored_idx == ctor_idx)
                                } else {
                                    // For non-closure functions, compare raw values
                                    Value::bool(stored_ctor.0 == ctor.0)
                                }
                            } else {
                                // Object was not created with new
                                Value::bool(false)
                            }
                        } else {
                            Value::bool(false)
                        }
                    } else if obj.is_error_object() {
                        // Error instanceof: match by error type name
                        if let Some(err_idx) = obj.to_error_object_idx() {
                            let err_name = self
                                .error_objects
                                .get(err_idx as usize)
                                .map(|e| e.name.as_str())
                                .unwrap_or("");
                            let ctor_name =
                                ctor.to_builtin_object_idx().and_then(|bidx| match bidx {
                                    crate::vm::types::BUILTIN_ERROR => Some("Error"),
                                    crate::vm::types::BUILTIN_TYPE_ERROR => Some("TypeError"),
                                    crate::vm::types::BUILTIN_REFERENCE_ERROR => {
                                        Some("ReferenceError")
                                    }
                                    crate::vm::types::BUILTIN_RANGE_ERROR => Some("RangeError"),
                                    crate::vm::types::BUILTIN_SYNTAX_ERROR => Some("SyntaxError"),
                                    crate::vm::types::BUILTIN_URI_ERROR => Some("URIError"),
                                    _ => None,
                                });
                            if let Some(name) = ctor_name {
                                // "Error" matches all errors; specific types match exactly
                                Value::bool(name == "Error" || err_name == name)
                            } else {
                                Value::bool(false)
                            }
                        } else {
                            Value::bool(false)
                        }
                    } else {
                        // Left operand is not an object
                        Value::bool(false)
                    };
                    self.stack.push(result);
                }

                // ForInStart - Start for-in iteration: obj -> iter
                op if op == OpCode::ForInStart as u8 => {
                    let obj = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;

                    // Create iterator based on value type
                    let iter = if let Some(obj_idx) = obj.to_object_idx() {
                        if let Some(obj_instance) = self.get_object(obj_idx) {
                            let mut keys = Vec::with_capacity(obj_instance.properties.len());
                            let prop_names: Vec<String> = obj_instance
                                .properties
                                .iter()
                                .map(|(k, _)| k.clone())
                                .collect();
                            for key in &prop_names {
                                keys.push(self.create_runtime_string_for_in_key(key)?);
                            }
                            ForInIterator::from_object_keys(keys)
                        } else {
                            ForInIterator::empty()
                        }
                    } else if let Some(arr_idx) = obj.to_array_idx() {
                        if let Some(arr) = self.get_array(arr_idx) {
                            ForInIterator::from_array_len(arr.len())
                        } else {
                            ForInIterator::empty()
                        }
                    } else {
                        // For non-objects/arrays, create empty iterator
                        ForInIterator::empty()
                    };

                    // Store iterator and push reference
                    self.maybe_gc();
                    let (iter_idx, is_new) = self.gc.alloc_slot(&mut self.gen_for_in_iterators);
                    if is_new {
                        self.for_in_iterators.push(iter);
                    } else {
                        self.for_in_iterators[iter_idx] = iter;
                    }
                    self.stack.push(Value::iterator_idx(iter_idx as u32));
                }

                // ForInNext - Get next for-in key: iter -> key done
                op if op == OpCode::ForInNext as u8 => {
                    let iter_val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;

                    if let Some(iter_idx) = iter_val.to_iterator_idx() {
                        if self.for_in_iterators.get(iter_idx as usize).is_some() {
                            if let Some(key) = self.for_in_next_key(iter_idx as usize) {
                                // Push key and false (not done)
                                self.stack.push(key);
                                self.stack.push(Value::bool(false)); // not done
                            } else {
                                // Push undefined and true (done)
                                self.stack.push(Value::undefined());
                                self.stack.push(Value::bool(true)); // done
                            }
                        } else {
                            // Invalid iterator, push done
                            self.stack.push(Value::undefined());
                            self.stack.push(Value::bool(true));
                        }
                    } else {
                        // Not an iterator, push done
                        self.stack.push(Value::undefined());
                        self.stack.push(Value::bool(true));
                    }
                }

                // ForOfStart - Start for-of iteration: obj -> iter
                op if op == OpCode::ForOfStart as u8 => {
                    let obj = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;

                    // Create iterator based on value type
                    let iter = if let Some(arr_idx) = obj.to_array_idx() {
                        if self.get_array(arr_idx).is_some() {
                            ForOfIterator::from_array_idx(arr_idx)
                        } else {
                            ForOfIterator::empty()
                        }
                    } else if let Some(obj_idx) = obj.to_object_idx() {
                        if let Some(obj_instance) = self.get_object(obj_idx) {
                            ForOfIterator::from_object(obj_instance)
                        } else {
                            ForOfIterator::empty()
                        }
                    } else {
                        // For non-objects/arrays, create empty iterator
                        ForOfIterator::empty()
                    };

                    // Store iterator and push reference
                    self.maybe_gc();
                    let (iter_idx, is_new) = self.gc.alloc_slot(&mut self.gen_for_of_iterators);
                    if is_new {
                        self.for_of_iterators.push(iter);
                    } else {
                        self.for_of_iterators[iter_idx] = iter;
                    }
                    self.stack.push(Value::for_of_iterator_idx(iter_idx as u32));
                }

                // ForOfNext - Get next for-of value: iter -> value done
                op if op == OpCode::ForOfNext as u8 => {
                    let iter_val = self.stack.pop().ok_or(InterpreterError::StackUnderflow)?;
                    let branch = {
                        let frame = self.call_stack.last_mut().unwrap();
                        let bytecode = unsafe { &*frame.bytecode };
                        let bc = &bytecode.bytecode;
                        if frame.pc < bc.len() {
                            let next = bc[frame.pc];
                            if (next == OpCode::IfFalse as u8 || next == OpCode::IfTrue as u8)
                                && frame.pc + 4 < bc.len()
                            {
                                let offset = i32::from_le_bytes([
                                    bc[frame.pc + 1],
                                    bc[frame.pc + 2],
                                    bc[frame.pc + 3],
                                    bc[frame.pc + 4],
                                ]);
                                frame.pc += 5;
                                Some((next == OpCode::IfTrue as u8, offset))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    };

                    let next_value = if let Some(iter_idx) = iter_val.to_for_of_iterator_idx() {
                        if self.for_of_iterators.get(iter_idx as usize).is_some() {
                            self.for_of_next_value(iter_idx as usize)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    if let Some((branch_on_true, offset)) = branch {
                        let done = next_value.is_none();
                        self.stack.push(next_value.unwrap_or(Value::undefined()));
                        if done == branch_on_true {
                            let frame = self.call_stack.last_mut().unwrap();
                            frame.pc = (frame.pc as i32 + offset) as usize;
                        }
                    } else if let Some(val) = next_value {
                        // Push value and false (not done)
                        self.stack.push(val);
                        self.stack.push(Value::bool(false));
                    } else {
                        // Push undefined and true (done)
                        self.stack.push(Value::undefined());
                        self.stack.push(Value::bool(true));
                    }
                }

                // Unknown opcode
                op => {
                    return Err(InterpreterError::InvalidOpcode(op));
                }
            }
        }
    }

    // Helper: Convert value to boolean (static method to avoid borrow issues)
    pub(crate) fn value_to_bool(val: Value) -> bool {
        if val.is_bool() {
            val.to_bool().unwrap_or(false)
        } else if val.is_int() {
            val.to_i32().map(|n| n != 0).unwrap_or(false)
        } else if let Some(f) = val.to_f32() {
            f != 0.0 && !f.is_nan()
        } else if val.is_null() || val.is_undefined() {
            false
        } else if val.is_string() {
            // Empty string is falsy, non-empty string is truthy
            val.to_string_idx()
                .map(|idx| idx != crate::value::STR_EMPTY)
                .unwrap_or(false)
        } else {
            // Objects, arrays, closures, etc. are truthy
            true
        }
    }

    #[inline]
    fn branch_matches_value(val: Value, branch_on_true: bool) -> bool {
        let truthy = if val.is_bool() {
            val.to_bool().unwrap_or(false)
        } else if val.is_null() || val.is_undefined() {
            false
        } else if val.is_int() {
            val.to_i32().map(|n| n != 0).unwrap_or(false)
        } else if let Some(f) = val.to_f32() {
            f != 0.0 && !f.is_nan()
        } else if val.is_string() {
            val.to_string_idx()
                .map(|idx| idx != crate::value::STR_EMPTY)
                .unwrap_or(false)
        } else {
            true
        };
        truthy == branch_on_true
    }

    /// Convert a value to a string for property access
    fn value_to_string(&self, val: &Value) -> Option<String> {
        if val.is_string() {
            // Get string from string constants or runtime strings
            let str_idx = val.to_string_idx()?;
            // Check if it's a built-in string
            use crate::value::{
                STR_BOOLEAN, STR_EMPTY, STR_FUNCTION, STR_NUMBER, STR_OBJECT, STR_STRING,
                STR_UNDEFINED,
            };
            match str_idx {
                STR_UNDEFINED => Some("undefined".to_string()),
                STR_OBJECT => Some("object".to_string()),
                STR_BOOLEAN => Some("boolean".to_string()),
                STR_NUMBER => Some("number".to_string()),
                STR_FUNCTION => Some("function".to_string()),
                STR_STRING => Some("string".to_string()),
                STR_EMPTY => Some(String::new()),
                _ => {
                    // Check runtime strings first (high indices)
                    if str_idx >= Self::RUNTIME_STRING_OFFSET {
                        self.runtime_strings
                            .get((str_idx - Self::RUNTIME_STRING_OFFSET) as usize)
                            .and_then(|_| self.get_string_by_idx(str_idx))
                            .map(|s| s.to_string())
                    } else {
                        // It's a compile-time string - we need bytecode access
                        // For now, return None (caller should handle)
                        None
                    }
                }
            }
        } else if let Some(n) = val.to_i32() {
            Some(n.to_string())
        } else {
            val.to_f32().map(crate::value::format_float)
        }
    }

    // Arithmetic, comparison, and bitwise operators moved to src/vm/ops.rs

    // =========================================================================
    // Native function support
    // =========================================================================

    /// Register a native function and return its index
    pub fn register_native(&mut self, name: &'static str, func: NativeFn, arity: u8) -> u32 {
        let idx = self.native_functions.len() as u32;
        self.native_functions
            .push(NativeFunction { name, func, arity });
        self.native_func_index.insert(name, idx);
        idx
    }

    /// Get a native function value by name
    pub fn get_native_func(&self, name: &str) -> Option<Value> {
        self.native_func_index
            .get(name)
            .copied()
            .map(Value::native_func)
    }

    // Native function dispatch and registration moved to src/vm/natives.rs

    /// Convert a value to boolean
    pub(crate) fn to_boolean(&self, val: Value) -> bool {
        if val.is_undefined() || val.is_null() {
            false
        } else if let Some(b) = val.to_bool() {
            b
        } else if let Some(n) = val.to_i32() {
            n != 0
        } else if let Some(f) = val.to_f32() {
            f != 0.0 && !f.is_nan()
        } else if let Some(str_idx) = val.to_string_idx() {
            // Empty string is falsy
            if let Some(s) = self.get_string_by_idx(str_idx) {
                !s.is_empty()
            } else {
                true
            }
        } else {
            // Objects, arrays, closures are truthy
            true
        }
    }

    /// Convert a value to number
    pub(crate) fn to_number(&self, val: Value) -> Value {
        if val.is_int() || val.is_float() {
            val
        } else if let Some(b) = val.to_bool() {
            Value::int(if b { 1 } else { 0 })
        } else if val.is_null() {
            Value::int(0)
        } else if val.is_undefined() {
            Value::nan()
        } else if let Some(str_idx) = val.to_string_idx() {
            if let Some(s) = self.get_string_by_idx(str_idx) {
                let s = s.trim();
                if s.is_empty() {
                    return Value::int(0);
                }
                if let Ok(i) = s.parse::<i32>() {
                    Value::int(i)
                } else if let Ok(f) = s.parse::<crate::value::Float>() {
                    crate::value::float_to_value(f)
                } else {
                    Value::nan()
                }
            } else {
                Value::nan()
            }
        } else {
            Value::nan()
        }
    }

    /// Convert a value to string
    pub(crate) fn stringify_value(&mut self, val: Value) -> Value {
        let s = if val.is_undefined() {
            "undefined".to_string()
        } else if val.is_null() {
            "null".to_string()
        } else if let Some(b) = val.to_bool() {
            b.to_string()
        } else if let Some(n) = val.to_i32() {
            n.to_string()
        } else if let Some(f) = val.to_f32() {
            crate::value::format_float(f)
        } else if val.to_string_idx().is_some() {
            // Already a string - return as-is
            return val;
        } else if val.is_array() {
            "[object Array]".to_string()
        } else if val.is_object() {
            "[object Object]".to_string()
        } else if val.is_closure() {
            "[object Function]".to_string()
        } else {
            "".to_string()
        };
        self.create_runtime_string(s)
    }
}

// Native function implementations moved to src/vm/natives.rs

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

// Most tests moved to tests/vm_tests.rs.
// test_recursion_limit remains here because it accesses pub(crate) fields
// (call_stack) that are not visible from outside the crate.

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vm::gc::SLOT_FREE;
    use crate::vm::natives::native_gc;

    #[test]
    fn test_recursion_limit() {
        let mut interp = Interpreter::with_config(1024, 5); // Max 5 calls deep

        // Fill up call stack
        let fb = FunctionBytecode::new(0, 0);
        for _ in 0..5 {
            interp.call_stack.push(CallFrame::new(
                &fb as *const _,
                0,
                0,
                Value::undefined(),
                Value::undefined(),
            ));
        }

        // Next call should fail
        let result = interp.call_function(&fb, Value::undefined(), &[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_gc_collects_unrooted_self_referential_object() {
        let mut interp = Interpreter::new();

        let obj = interp.create_object();
        let obj_idx = obj.to_object_idx().unwrap() as usize;
        interp.objects[obj_idx]
            .properties
            .push(("self".to_string(), obj));

        interp.gc_collect();

        assert_eq!(interp.gen_objects[obj_idx], SLOT_FREE);
    }

    #[test]
    fn test_gc_reuses_freed_object_slot() {
        let mut interp = Interpreter::new();

        let first = interp.create_object();
        let first_idx = first.to_object_idx().unwrap() as usize;

        interp.gc_collect();
        assert_eq!(interp.gen_objects[first_idx], SLOT_FREE);

        let second = interp.create_object();
        let second_idx = second.to_object_idx().unwrap() as usize;

        assert_eq!(second_idx, first_idx);
        assert_ne!(interp.gen_objects[second_idx], SLOT_FREE);
    }

    #[test]
    fn test_native_gc_collects_and_increments_count() {
        let mut interp = Interpreter::new();

        let obj = interp.create_object();
        let obj_idx = obj.to_object_idx().unwrap() as usize;
        interp.objects[obj_idx]
            .properties
            .push(("self".to_string(), obj));

        let before = interp.gc_count;
        let result = native_gc(&mut interp, Value::undefined(), &[]).unwrap();

        assert!(result.is_undefined());
        assert_eq!(interp.gc_count, before + 1);
        assert_eq!(interp.gen_objects[obj_idx], SLOT_FREE);
    }
}
