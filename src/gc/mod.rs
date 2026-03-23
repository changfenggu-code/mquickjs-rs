//! Garbage collector module
//!
//! # Current Status
//!
//! **Plan B (generation-based mark-sweep) is currently active** in `src/vm/gc.rs`.
//! Plan C (mark-compact) is a future plan and exists as stubs in this module.
//!
//! ## Active Implementation: Plan B
//! - Uses generation-based mark-sweep garbage collection
//! - Heap-allocated work list prevents stack overflow during marking
//! - Vec-indexed storage in Interpreter (no raw pointers)
//! - Adaptive GC trigger based on survival rate
//! - Handles cycles automatically
//! - Optimized for no_std embedded environments (ESP32)
//!
//! ## Future Implementation: Plan C
//! - Exists as stubs in `src/gc/` (allocator.rs, collector.rs)
//! - Will implement mark-compact GC for better memory compaction
//! - Not currently used - kept for future development
//!
//! ## Comparison with QuickJS
//! Unlike QuickJS which uses reference counting, MQuickJS uses tracing GC which:
//! - Provides smaller object headers (no reference count)
//! - Eliminates memory fragmentation (with future Plan C compaction)
//! - Handles cycles automatically
//! - Better suited for embedded no_std environments

mod allocator;
mod collector;

pub use allocator::{BlockHeader, Heap, MemoryTag};
pub use collector::GcRef;

impl Heap {
    /// Run garbage collection
    pub fn collect(&mut self) {
        collector::collect(self);
    }
}
