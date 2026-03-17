//! Unit tests for the bytecode compiler.
//!
//! Migrated from src/parser/compiler.rs.

use mquickjs::parser::compiler::{CompileError, CompiledFunction, Compiler};
use mquickjs::vm::OpCode;

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn compile_expr(source: &str) -> Result<CompiledFunction, CompileError> {
    // Wrap expression in a statement so it compiles as a program
    let full_source = format!("{};", source);
    Compiler::new(&full_source).compile()
}

// ---------------------------------------------------------------------------
// Integer literals
// ---------------------------------------------------------------------------

#[test]
fn test_compile_integers() {
    let func = compile_expr("42").unwrap();
    // Should emit: PushI8 42, Drop, ReturnUndef
    assert!(!func.bytecode.is_empty());
}

#[test]
fn test_compile_small_integers() {
    // Test optimized integer opcodes (0-7)
    // Note: -1 is parsed as unary minus + 1, so it produces Push1, Neg
    for i in 0..=7 {
        let func = compile_expr(&i.to_string()).unwrap();
        let expected = match i {
            0 => OpCode::Push0 as u8,
            1 => OpCode::Push1 as u8,
            2 => OpCode::Push2 as u8,
            3 => OpCode::Push3 as u8,
            4 => OpCode::Push4 as u8,
            5 => OpCode::Push5 as u8,
            6 => OpCode::Push6 as u8,
            7 => OpCode::Push7 as u8,
            _ => unreachable!(),
        };
        assert_eq!(func.bytecode[0], expected);
    }
}

#[test]
fn test_compile_negative_one() {
    // -1 is parsed as unary minus + 1
    let func = compile_expr("-1").unwrap();
    // Should emit: Push1, Neg, Drop, ReturnUndef
    assert_eq!(func.bytecode[0], OpCode::Push1 as u8);
    assert_eq!(func.bytecode[1], OpCode::Neg as u8);
}

// ---------------------------------------------------------------------------
// Literals
// ---------------------------------------------------------------------------

#[test]
fn test_compile_boolean() {
    let func = compile_expr("true").unwrap();
    assert_eq!(func.bytecode[0], OpCode::PushTrue as u8);

    let func = compile_expr("false").unwrap();
    assert_eq!(func.bytecode[0], OpCode::PushFalse as u8);
}

#[test]
fn test_compile_null() {
    let func = compile_expr("null").unwrap();
    assert_eq!(func.bytecode[0], OpCode::Null as u8);
}

// ---------------------------------------------------------------------------
// Arithmetic / operators
// ---------------------------------------------------------------------------

#[test]
fn test_compile_addition() {
    let func = compile_expr("1 + 2").unwrap();
    // Should emit: Push1, Push2, Add, Drop, ReturnUndef
    assert_eq!(func.bytecode[0], OpCode::Push1 as u8);
    assert_eq!(func.bytecode[1], OpCode::Push2 as u8);
    assert_eq!(func.bytecode[2], OpCode::Add as u8);
}

#[test]
fn test_compile_precedence() {
    // 1 + 2 * 3 should be 1 + (2 * 3)
    let func = compile_expr("1 + 2 * 3").unwrap();
    // Should emit: Push1, Push2, Push3, Mul, Add
    assert_eq!(func.bytecode[0], OpCode::Push1 as u8);
    assert_eq!(func.bytecode[1], OpCode::Push2 as u8);
    assert_eq!(func.bytecode[2], OpCode::Push3 as u8);
    assert_eq!(func.bytecode[3], OpCode::Mul as u8);
    assert_eq!(func.bytecode[4], OpCode::Add as u8);
}

#[test]
fn test_compile_parentheses() {
    // (1 + 2) * 3
    let func = compile_expr("(1 + 2) * 3").unwrap();
    // Should emit: Push1, Push2, Add, Push3, Mul
    assert_eq!(func.bytecode[0], OpCode::Push1 as u8);
    assert_eq!(func.bytecode[1], OpCode::Push2 as u8);
    assert_eq!(func.bytecode[2], OpCode::Add as u8);
    assert_eq!(func.bytecode[3], OpCode::Push3 as u8);
    assert_eq!(func.bytecode[4], OpCode::Mul as u8);
}

#[test]
fn test_compile_unary_minus() {
    let func = compile_expr("-5").unwrap();
    // Should emit: Push5, Neg
    assert_eq!(func.bytecode[0], OpCode::Push5 as u8);
    assert_eq!(func.bytecode[1], OpCode::Neg as u8);
}

#[test]
fn test_compile_comparison() {
    let func = compile_expr("1 < 2").unwrap();
    assert_eq!(func.bytecode[0], OpCode::Push1 as u8);
    assert_eq!(func.bytecode[1], OpCode::Push2 as u8);
    assert_eq!(func.bytecode[2], OpCode::Lt as u8);
}

// ---------------------------------------------------------------------------
// Variables
// ---------------------------------------------------------------------------

#[test]
fn test_compile_var_declaration() {
    let source = "var x = 10;";
    let func = Compiler::new(source).compile().unwrap();
    // Should declare local and initialize it
    assert_eq!(func.local_count, 1);
}

#[test]
fn test_compile_var_usage() {
    let source = "var x = 10; x;";
    let func = Compiler::new(source).compile().unwrap();
    // Check that GetLoc0 is emitted for x
    assert!(func.bytecode.contains(&(OpCode::GetLoc0 as u8)));
}

// ---------------------------------------------------------------------------
// Control flow
// ---------------------------------------------------------------------------

#[test]
fn test_compile_if_statement() {
    let source = "var x = 1; if (x) { x; }";
    let func = Compiler::new(source).compile().unwrap();
    // Should contain IfFalse jump
    assert!(func.bytecode.contains(&(OpCode::IfFalse as u8)));
}

#[test]
fn test_compile_while_loop() {
    let source = "var i = 0; while (i < 5) { i; }";
    let func = Compiler::new(source).compile().unwrap();
    // Should contain IfFalse and Goto
    assert!(func.bytecode.contains(&(OpCode::IfFalse as u8)));
    assert!(func.bytecode.contains(&(OpCode::Goto as u8)));
}

#[test]
fn test_compile_ternary() {
    let func = compile_expr("1 ? 2 : 3").unwrap();
    // Should contain IfFalse and Goto for branches
    assert!(func.bytecode.contains(&(OpCode::IfFalse as u8)));
    assert!(func.bytecode.contains(&(OpCode::Goto as u8)));
}

#[test]
fn test_compile_add_const_string_left_specialization() {
    let source = "var x = 1; 'a' + x;";
    let func = Compiler::new(source).compile().unwrap();
    assert!(func.bytecode.contains(&(OpCode::AddConstStringLeft as u8)));
}

#[test]
fn test_compile_add_const_string_right_specialization() {
    let source = "var x = 1; x + 'a';";
    let func = Compiler::new(source).compile().unwrap();
    assert!(func.bytecode.contains(&(OpCode::AddConstStringRight as u8)));
}

#[test]
fn test_compile_add_const_string_surround_specialization() {
    let source = "var x = 1; 'a' + x + 'b';";
    let func = Compiler::new(source).compile().unwrap();
    assert!(func
        .bytecode
        .contains(&(OpCode::AddConstStringSurround as u8)));
}

#[test]
fn test_compile_adjacent_string_concat_folds_to_const() {
    let func = compile_expr("'a' + 'b'").unwrap();
    assert!(!func.bytecode.contains(&(OpCode::Add as u8)));
    assert!(!func.bytecode.contains(&(OpCode::AddConstStringLeft as u8)));
    assert!(!func.bytecode.contains(&(OpCode::AddConstStringRight as u8)));
    assert!(!func
        .bytecode
        .contains(&(OpCode::AddConstStringSurround as u8)));
}
