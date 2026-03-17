//! Unit tests for the value stack and opcode table.
//!
//! Migrated from src/vm/stack.rs and src/vm/opcode.rs.

use mquickjs::vm::opcode::{OpCode, OPCODE_INFO};
use mquickjs::vm::Stack;
use mquickjs::Value;

// ---------------------------------------------------------------------------
// Stack tests
// ---------------------------------------------------------------------------

#[test]
fn test_push_pop() {
    let mut stack = Stack::new(16);

    stack.push(Value::int(1));
    stack.push(Value::int(2));
    stack.push(Value::int(3));

    assert_eq!(stack.len(), 3);
    assert_eq!(stack.pop().unwrap().to_i32(), Some(3));
    assert_eq!(stack.pop().unwrap().to_i32(), Some(2));
    assert_eq!(stack.pop().unwrap().to_i32(), Some(1));
    assert!(stack.is_empty());
}

#[test]
fn test_peek() {
    let mut stack = Stack::new(16);

    stack.push(Value::int(1));
    stack.push(Value::int(2));

    assert_eq!(stack.peek().unwrap().to_i32(), Some(2));
    assert_eq!(stack.peek_at(0).unwrap().to_i32(), Some(2));
    assert_eq!(stack.peek_at(1).unwrap().to_i32(), Some(1));
    assert!(stack.peek_at(2).is_none());
}

#[test]
fn test_dup() {
    let mut stack = Stack::new(16);

    stack.push(Value::int(42));
    stack.dup();

    assert_eq!(stack.len(), 2);
    assert_eq!(stack.pop().unwrap().to_i32(), Some(42));
    assert_eq!(stack.pop().unwrap().to_i32(), Some(42));
}

#[test]
fn test_swap() {
    let mut stack = Stack::new(16);

    stack.push(Value::int(1));
    stack.push(Value::int(2));
    stack.swap();

    assert_eq!(stack.pop().unwrap().to_i32(), Some(1));
    assert_eq!(stack.pop().unwrap().to_i32(), Some(2));
}

#[test]
fn test_locals() {
    let mut stack = Stack::new(16);

    stack.push_frame(3);
    assert_eq!(stack.len(), 3);

    stack.set_local(0, Value::int(10));
    stack.set_local(1, Value::int(20));
    stack.set_local(2, Value::int(30));

    assert_eq!(stack.get_local(0).unwrap().to_i32(), Some(10));
    assert_eq!(stack.get_local(1).unwrap().to_i32(), Some(20));
    assert_eq!(stack.get_local(2).unwrap().to_i32(), Some(30));
}

#[test]
fn test_compact_call_args() {
    let mut stack = Stack::new(16);
    stack.push(Value::int(99));
    stack.push(Value::int(1));
    stack.push(Value::int(2));

    let func = stack.compact_call_args(2).unwrap();
    assert_eq!(func.to_i32(), Some(99));
    assert_eq!(stack.len(), 2);
    assert_eq!(stack.get_raw(0).to_i32(), Some(1));
    assert_eq!(stack.get_raw(1).to_i32(), Some(2));
}

#[test]
fn test_compact_method_call_args() {
    let mut stack = Stack::new(16);
    stack.push(Value::int(7));
    stack.push(Value::int(8));
    stack.push(Value::int(1));
    stack.push(Value::int(2));

    let (this_val, method_val) = stack.compact_method_call_args(2).unwrap();
    assert_eq!(this_val.to_i32(), Some(7));
    assert_eq!(method_val.to_i32(), Some(8));
    assert_eq!(stack.len(), 2);
    assert_eq!(stack.get_raw(0).to_i32(), Some(1));
    assert_eq!(stack.get_raw(1).to_i32(), Some(2));
}

// ---------------------------------------------------------------------------
// OpCode table tests
// ---------------------------------------------------------------------------

#[test]
fn test_opcode_count() {
    assert_eq!(OPCODE_INFO.len(), OpCode::COUNT);
}

#[test]
fn test_opcode_sizes() {
    // Verify some known opcode sizes
    assert_eq!(OPCODE_INFO[OpCode::Drop as usize].size, 1);
    assert_eq!(OPCODE_INFO[OpCode::PushI8 as usize].size, 2);
    assert_eq!(OPCODE_INFO[OpCode::PushConst as usize].size, 3);
    assert_eq!(OPCODE_INFO[OpCode::Goto as usize].size, 5);
}
