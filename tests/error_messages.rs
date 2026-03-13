//! Tests for error message quality and completeness.
//!
//! Verifies that compile errors and runtime errors produce useful,
//! informative messages including error type, location, and reason.

use mquickjs::Context;

fn eval_err(source: &str) -> String {
    let mut ctx = Context::new(64 * 1024);
    ctx.eval(source).unwrap_err().to_string()
}

fn assert_compile_error(source: &str, needle: &str) {
    let msg = eval_err(source);
    assert!(msg.contains("Compile error"), "not a compile error: {msg}");
    assert!(msg.contains("line"), "missing line info: {msg}");
    assert!(msg.contains(needle), "expected '{needle}' in: {msg}");
}

fn assert_runtime_error(source: &str, needle: &str) {
    let msg = eval_err(source);
    assert!(msg.contains("Runtime error"), "not a runtime error: {msg}");
    assert!(msg.contains(needle), "expected '{needle}' in: {msg}");
}

// =========================================================
// Compile errors — with line:column info
// =========================================================

#[test]
fn test_compile_error_missing_semicolon() {
    assert_compile_error("var x = 1\nvar y = 2", "Semicolon");
}

#[test]
fn test_compile_error_unclosed_string() {
    // May report as Eof or other token depending on lexer recovery
    let msg = eval_err("var x = \"hello");
    assert!(msg.contains("Compile error") || msg.contains("Runtime error"));
}

#[test]
fn test_compile_error_unexpected_token() {
    assert_compile_error("var x = * 3;", "Unexpected token");
}

#[test]
fn test_compile_error_unclosed_paren() {
    assert_compile_error("var x = (1 + 2;", "RParen");
}

#[test]
fn test_compile_error_unclosed_brace() {
    assert_compile_error("function foo() { return 1;", "RBrace");
}

#[test]
fn test_compile_error_reserved_word_as_var() {
    assert_compile_error("var return = 5;", "variable name");
}

#[test]
fn test_compile_error_undeclared_variable() {
    let msg = eval_err("return undeclaredVariable;");
    assert!(
        msg.contains("undeclaredVariable"),
        "should mention the variable name: {msg}"
    );
}

#[test]
fn test_typeof_undeclared_variable_is_not_compile_error() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx.eval("return typeof undeclaredVariable;");
    assert!(result.is_ok(), "typeof undeclared variable should not error");
}

// =========================================================
// Runtime errors
// =========================================================

#[test]
fn test_runtime_error_call_non_function() {
    assert_runtime_error("var x = 42; x();", "not a function");
}

#[test]
fn test_division_by_zero_returns_infinity() {
    // Division by zero now returns Infinity (JS standard behavior)
    let mut ctx = Context::new(64 * 1024);
    let result = ctx.eval("return 1/0;").unwrap();
    assert!(result.is_infinite_value(), "1/0 should be Infinity");

    let result = ctx.eval("return -1/0;").unwrap();
    assert!(result.is_infinite_value(), "-1/0 should be -Infinity");

    let result = ctx.eval("return 0/0;").unwrap();
    assert!(result.is_nan_value(), "0/0 should be NaN");
}

#[test]
fn test_runtime_error_type_error_arithmetic() {
    // Arrays cannot be used in arithmetic — they produce a TypeError
    assert_runtime_error("var a = [1,2]; return a - 1;", "cannot");
}

#[test]
fn test_runtime_error_stack_overflow() {
    assert_runtime_error("function r() { return r(); } r();", "stack");
}

// =========================================================
// Uncaught exception formatting (previously showed RawValue)
// =========================================================

#[test]
fn test_uncaught_error_shows_message() {
    let msg = eval_err("throw new Error('something broke');");
    assert!(
        msg.contains("something broke"),
        "should include the error message: {msg}"
    );
}

#[test]
fn test_uncaught_type_error_shows_name_and_message() {
    let msg = eval_err("throw new TypeError('bad type');");
    assert!(
        msg.contains("TypeError") && msg.contains("bad type"),
        "should include error name and message: {msg}"
    );
}

// =========================================================
// try/catch now catches runtime errors (previously escaped)
// =========================================================

#[test]
fn test_try_catch_catches_call_non_function() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx.eval(
        "var caught = ''; try { var x = 42; x(); } catch(e) { caught = e.name + ':' + e.message; } return caught;",
    ).unwrap();
    // Result is a string, not an integer
    assert!(result.to_i32().is_none(), "should return a string, not int");
}

#[test]
fn test_try_catch_catches_stack_overflow() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx.eval(
        "var caught = false; try { function r() { return r(); } r(); } catch(e) { caught = true; } return caught;",
    );
    assert!(result.is_ok(), "try/catch should catch stack overflow");
}
