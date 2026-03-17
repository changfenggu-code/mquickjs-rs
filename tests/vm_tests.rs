//! Unit tests for the bytecode interpreter (VM).
//!
//! Migrated from src/vm/interpreter.rs.
//! Tests that require pub(crate) access (e.g. test_recursion_limit) remain in-source.

use mquickjs::vm::{Interpreter, OpCode};
use mquickjs::{FunctionBytecode, Value};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn make_bytecode(bytecode: Vec<u8>) -> FunctionBytecode {
    let mut fb = FunctionBytecode::new(0, 4);
    fb.bytecode = bytecode;
    fb
}

// ---------------------------------------------------------------------------
// Stack / arithmetic
// ---------------------------------------------------------------------------

#[test]
fn test_push_integers() {
    let mut interp = Interpreter::new();

    // Push 3, Push 2, Add, Return
    let bc = make_bytecode(vec![
        OpCode::Push3 as u8,
        OpCode::Push2 as u8,
        OpCode::Add as u8,
        OpCode::Return as u8,
    ]);

    let result = interp.execute(&bc).unwrap();
    assert_eq!(result.to_i32(), Some(5));
}

#[test]
fn test_push_i8() {
    let mut interp = Interpreter::new();

    // PushI8 10, PushI8 -5, Add, Return
    let bc = make_bytecode(vec![
        OpCode::PushI8 as u8,
        10u8,
        OpCode::PushI8 as u8,
        (-5i8) as u8,
        OpCode::Add as u8,
        OpCode::Return as u8,
    ]);

    let result = interp.execute(&bc).unwrap();
    assert_eq!(result.to_i32(), Some(5));
}

#[test]
fn test_arithmetic() {
    let mut interp = Interpreter::new();

    // 10 - (3 * 2) = 4
    let bc = make_bytecode(vec![
        OpCode::PushI8 as u8,
        10,
        OpCode::Push3 as u8,
        OpCode::Push2 as u8,
        OpCode::Mul as u8,
        OpCode::Sub as u8,
        OpCode::Return as u8,
    ]);

    let result = interp.execute(&bc).unwrap();
    assert_eq!(result.to_i32(), Some(4));
}

// ---------------------------------------------------------------------------
// Local variables
// ---------------------------------------------------------------------------

#[test]
fn test_local_variables() {
    let mut interp = Interpreter::new();

    // var x = 5; var y = 3; return x + y;
    let bc = make_bytecode(vec![
        OpCode::Push5 as u8,
        OpCode::PutLoc0 as u8,
        OpCode::Push3 as u8,
        OpCode::PutLoc1 as u8,
        OpCode::GetLoc0 as u8,
        OpCode::GetLoc1 as u8,
        OpCode::Add as u8,
        OpCode::Return as u8,
    ]);

    let result = interp.execute(&bc).unwrap();
    assert_eq!(result.to_i32(), Some(8));
}

// ---------------------------------------------------------------------------
// Comparisons / control flow
// ---------------------------------------------------------------------------

#[test]
fn test_comparison() {
    let mut interp = Interpreter::new();

    // 5 < 10 => true
    let bc = make_bytecode(vec![
        OpCode::Push5 as u8,
        OpCode::PushI8 as u8,
        10,
        OpCode::Lt as u8,
        OpCode::Return as u8,
    ]);

    let result = interp.execute(&bc).unwrap();
    assert!(result.to_bool().unwrap());
}

#[test]
fn test_conditional_jump() {
    let mut interp = Interpreter::new();

    // if (false) { return 1; } return 2;
    // 0: PushFalse
    // 1: IfFalse +2  (5 bytes: opcode + 4-byte offset)
    // 6: Push1
    // 7: Return
    // 8: Push2
    // 9: Return
    let bc = make_bytecode(vec![
        OpCode::PushFalse as u8, // 0
        OpCode::IfFalse as u8,   // 1
        2,
        0,
        0,
        0,                    // 2-5: offset = 2
        OpCode::Push1 as u8,  // 6
        OpCode::Return as u8, // 7
        OpCode::Push2 as u8,  // 8
        OpCode::Return as u8, // 9
    ]);

    let result = interp.execute(&bc).unwrap();
    assert_eq!(result.to_i32(), Some(2));
}

// ---------------------------------------------------------------------------
// Bitwise / logical
// ---------------------------------------------------------------------------

#[test]
fn test_bitwise_operations() {
    let mut interp = Interpreter::new();

    // 5 & 3 = 1
    let bc = make_bytecode(vec![
        OpCode::Push5 as u8,
        OpCode::Push3 as u8,
        OpCode::And as u8,
        OpCode::Return as u8,
    ]);

    let result = interp.execute(&bc).unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_logical_not() {
    let mut interp = Interpreter::new();

    // !false = true
    let bc = make_bytecode(vec![
        OpCode::PushFalse as u8,
        OpCode::LNot as u8,
        OpCode::Return as u8,
    ]);

    let result = interp.execute(&bc).unwrap();
    assert!(result.to_bool().unwrap());
}

// ---------------------------------------------------------------------------
// Return
// ---------------------------------------------------------------------------

#[test]
fn test_return_undefined() {
    let mut interp = Interpreter::new();

    let bc = make_bytecode(vec![OpCode::ReturnUndef as u8]);

    let result = interp.execute(&bc).unwrap();
    assert!(result.is_undefined());
}

// ---------------------------------------------------------------------------
// Function calls
// ---------------------------------------------------------------------------

#[test]
fn test_function_with_args() {
    let mut interp = Interpreter::new();

    // function add(a, b) { return a + b; }  called with (10, 20)
    let mut fb = FunctionBytecode::new(2, 2);
    fb.bytecode = vec![
        OpCode::GetArg0 as u8,
        OpCode::GetArg1 as u8,
        OpCode::Add as u8,
        OpCode::Return as u8,
    ];

    let result = interp
        .call_function(&fb, Value::undefined(), &[Value::int(10), Value::int(20)])
        .unwrap();
    assert_eq!(result.to_i32(), Some(30));
}

#[test]
fn test_function_with_this() {
    let mut interp = Interpreter::new();

    // function getThis() { return this; }
    let mut fb = FunctionBytecode::new(0, 0);
    fb.bytecode = vec![OpCode::PushThis as u8, OpCode::Return as u8];

    let this_val = Value::int(42);
    let result = interp.call_function(&fb, this_val, &[]).unwrap();
    assert_eq!(result.to_i32(), Some(42));
}

#[test]
fn test_function_missing_args() {
    let mut interp = Interpreter::new();

    // function add(a, b) { return b; }  called with only 1 arg — b should be undefined
    let mut fb = FunctionBytecode::new(2, 2);
    fb.bytecode = vec![
        OpCode::GetArg1 as u8, // Get b (should be undefined)
        OpCode::Return as u8,
    ];

    let result = interp
        .call_function(&fb, Value::undefined(), &[Value::int(10)])
        .unwrap();
    assert!(result.is_undefined());
}
