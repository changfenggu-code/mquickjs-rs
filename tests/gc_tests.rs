//! Unit tests for the GC allocator and collector.
//!
//! Migrated from src/gc/allocator.rs and src/gc/collector.rs.

use mquickjs::gc::{BlockHeader, GcRef, Heap, MemoryTag};
use mquickjs::Value;

// ---------------------------------------------------------------------------
// Allocator tests
// ---------------------------------------------------------------------------

#[test]
fn test_heap_creation() {
    let heap = Heap::new(1024);
    assert_eq!(heap.total_size, 1024);
    assert_eq!(heap.heap_used(), 0);
    assert!(heap.free_space() > 0);
}

#[test]
fn test_alloc() {
    let mut heap = Heap::new(4096);

    let ptr = heap.alloc(64, MemoryTag::Object);
    assert!(ptr.is_some());
    assert!(heap.heap_used() > 0);

    unsafe {
        let header = heap.get_header(ptr.unwrap());
        assert_eq!(header.tag(), MemoryTag::Object);
        assert!(!header.is_marked());
    }
}

#[test]
fn test_alloc_zeroed() {
    let mut heap = Heap::new(4096);

    let ptr = heap.alloc_zeroed(32, MemoryTag::String).unwrap();

    // Check that memory is zeroed
    unsafe {
        for i in 0..32 {
            assert_eq!(*ptr.add(i), 0);
        }
    }
}

#[test]
fn test_out_of_memory() {
    let mut heap = Heap::new(1024);

    // Try to allocate more than available
    let ptr = heap.alloc(2048, MemoryTag::Object);
    assert!(ptr.is_none());
}

#[test]
fn test_block_iterator() {
    let mut heap = Heap::new(4096);

    heap.alloc(32, MemoryTag::Object);
    heap.alloc(64, MemoryTag::String);
    heap.alloc(16, MemoryTag::Float64);

    let blocks: Vec<_> = heap.iter_blocks().collect();
    assert_eq!(blocks.len(), 3);
}

#[test]
fn test_stack_operations() {
    let mut heap = Heap::new(4096);

    let initial_free = heap.free_space();

    let ptr = heap.stack_push(4);
    assert!(ptr.is_some());
    assert!(heap.free_space() < initial_free);

    heap.stack_pop(4);
    assert_eq!(heap.free_space(), initial_free);
}

#[test]
fn test_header_mark_bit() {
    let mut header = BlockHeader::new(MemoryTag::Object, 8);
    assert!(!header.is_marked());

    header.set_marked(true);
    assert!(header.is_marked());
    assert_eq!(header.tag(), MemoryTag::Object);
    assert_eq!(header.size_words(), 8);

    header.set_marked(false);
    assert!(!header.is_marked());
}

// ---------------------------------------------------------------------------
// Collector tests
// ---------------------------------------------------------------------------

#[test]
fn test_gc_ref() {
    let mut r = GcRef::new(Value::int(42));
    assert_eq!(r.get().to_i32(), Some(42));

    r.set(Value::int(100));
    assert_eq!(r.get().to_i32(), Some(100));
}

#[test]
fn test_collect_empty_heap() {
    let mut heap = Heap::new(4096);
    heap.collect();
    // Should not crash on empty heap
}

#[test]
fn test_collect_with_objects() {
    let mut heap = Heap::new(4096);

    // Allocate some objects
    heap.alloc(32, MemoryTag::Object);
    heap.alloc(64, MemoryTag::String);
    heap.alloc(16, MemoryTag::Float64);

    let used_before = heap.heap_used();

    heap.collect();

    // Since we mark everything, nothing should be freed
    assert_eq!(heap.heap_used(), used_before);
}
