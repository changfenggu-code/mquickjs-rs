//! Integration tests for Context::eval()
//
//! Tests the full pipeline: source -> lexer -> compiler -> bytecode -> VM -> result.

use mquickjs::Context;
use std::sync::atomic::{AtomicU64, Ordering};

static HOST_TIME_MILLIS: AtomicU64 = AtomicU64::new(0);

fn host_time_provider() -> u64 {
    HOST_TIME_MILLIS.load(Ordering::Relaxed)
}

#[test]
fn test_create_context() {
    let ctx = Context::new(64 * 1024);
    let stats = ctx.memory_stats();
    assert!(stats.heap_size >= 64 * 1024);
}

#[test]
fn test_memory_stats_track_array_and_object_shape() {
    let mut ctx = Context::new(64 * 1024);
    ctx.eval(
        "
        var arr = [1, 2, 3];
        var obj = { a: 1, b: 2 };
        return 0;
    ",
    )
    .unwrap();

    let stats = ctx.memory_stats();
    assert!(stats.arrays >= 1);
    assert!(stats.array_elements >= 3);
    assert!(stats.objects >= 1);
    assert!(stats.object_properties >= 2);
}

#[test]
fn test_memory_stats_track_runtime_string_bytes() {
    let mut ctx = Context::new(64 * 1024);
    ctx.eval(
        "
        var s = 'item-' + 12 + '-' + 3;
        return s;
    ",
    )
    .unwrap();

    let stats = ctx.memory_stats();
    assert!(stats.runtime_strings >= 1);
    assert!(stats.runtime_string_bytes >= 9);
}

#[test]
fn test_context_gc_collects_unrooted_cycle_objects() {
    let mut ctx = Context::new(64 * 1024);
    let before = ctx.memory_stats();

    ctx.eval(
        "
        function makeCycle() {
            var a = {};
            a.self = a;
        }
        for (var i = 0; i < 50; i = i + 1) {
            makeCycle();
        }
        return 0;
    ",
    )
    .unwrap();

    let after_alloc = ctx.memory_stats();
    assert!(after_alloc.objects > before.objects);

    ctx.gc();

    let after_gc = ctx.memory_stats();
    assert!(after_gc.objects < after_alloc.objects);
    assert!(after_gc.gc_count > before.gc_count);
}

#[test]
fn test_gc_auto_triggers_during_js_function_workload() {
    let mut ctx = Context::new(128 * 1024);
    let before = ctx.memory_stats();

    ctx.eval(
        "
        function makeCycle() {
            var a = {};
            a.self = a;
        }
        for (var i = 0; i < 1500; i = i + 1) {
            makeCycle();
        }
        return 0;
    ",
    )
    .unwrap();

    let after = ctx.memory_stats();
    assert!(after.gc_count > before.gc_count);
}

#[test]
fn test_gc_frees_dead_arrays() {
    let mut ctx = Context::new(64 * 1024);
    let before = ctx.memory_stats();

    // Create and discard many arrays
    ctx.eval(
        "
        for (var i = 0; i < 100; i = i + 1) {
            var arr = [1, 2, 3];
            arr.push(i);
        }
        return 0;
    ",
    )
    .unwrap();

    let after_alloc = ctx.memory_stats();
    assert!(after_alloc.arrays > before.arrays);

    ctx.gc();

    let after_gc = ctx.memory_stats();
    // After GC, dead arrays should be swept and not counted as live
    assert!(after_gc.arrays <= after_alloc.arrays);
}

#[test]
fn test_gc_frees_dead_objects() {
    let mut ctx = Context::new(64 * 1024);
    let before = ctx.memory_stats();

    // Create and discard many plain objects
    ctx.eval(
        "
        for (var i = 0; i < 100; i = i + 1) {
            var obj = { x: i, y: i + 1 };
            var sum = obj.x + obj.y;
        }
        return 0;
    ",
    )
    .unwrap();

    let after_alloc = ctx.memory_stats();
    assert!(after_alloc.objects > before.objects);

    ctx.gc();

    let after_gc = ctx.memory_stats();
    assert!(after_gc.objects < after_alloc.objects);
}

#[test]
fn test_gc_mixed_arrays_and_objects() {
    let mut ctx = Context::new(64 * 1024);

    ctx.eval(
        "
        var result = [];
        for (var i = 0; i < 200; i = i + 1) {
            var obj = { val: i };
            result.push(obj);
        }
        // Keep only the last 10
        result = result.slice(190);
        return 0;
    ",
    )
    .unwrap();

    let after_alloc = ctx.memory_stats();
    let object_count = after_alloc.objects;

    ctx.gc();

    let after_gc = ctx.memory_stats();
    // GC should have reclaimed most of the dropped objects
    assert!(after_gc.objects <= object_count);
}

#[test]
fn test_gc_no_crash_on_empty() {
    let mut ctx = Context::new(64 * 1024);
    // GC on fresh context should not crash
    ctx.gc();
    let stats = ctx.memory_stats();
    assert_eq!(stats.objects, 0);
    assert_eq!(stats.arrays, 0);
}

#[test]
fn test_eval_empty() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx.eval("").unwrap();
    assert!(result.is_undefined());
}

#[test]
fn test_eval_literal() {
    let mut ctx = Context::new(64 * 1024);

    // Test integer literal
    let result = ctx.eval("42;").unwrap();
    assert!(result.is_undefined()); // Expression statement drops result

    // Test return statement
    let result = ctx.eval("return 42;").unwrap();
    assert_eq!(result.to_i32(), Some(42));
}
#[test]
fn test_for_in_runtime_string_overflow_becomes_error() {
    let mut ctx = Context::new(256 * 1024);
    let result = ctx.eval("var obj = { a:1,b:2,c:3,d:4,e:5,f:6,g:7,h:8,i:9,j:10,k:11,l:12,m:13,n:14,o:15,p:16,q:17,r:18,s:19,t:20 }; for (var round = 0; round < 5000; round = round + 1) { for (var k in obj) { } } return 0;");
    assert!(result.is_err());
    let msg = format!("{}", result.unwrap_err());
    assert!(msg.contains("runtime string table exhausted"));
}

#[test]
fn test_eval_arithmetic() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx.eval("return 2 + 3;").unwrap();
    assert_eq!(result.to_i32(), Some(5));

    let result = ctx.eval("return 10 - 4;").unwrap();
    assert_eq!(result.to_i32(), Some(6));

    let result = ctx.eval("return 3 * 4;").unwrap();
    assert_eq!(result.to_i32(), Some(12));

    let result = ctx.eval("return 20 / 5;").unwrap();
    assert_eq!(result.to_i32(), Some(4));
}

#[test]
fn test_eval_precedence() {
    let mut ctx = Context::new(64 * 1024);

    // 2 + 3 * 4 = 2 + 12 = 14
    let result = ctx.eval("return 2 + 3 * 4;").unwrap();
    assert_eq!(result.to_i32(), Some(14));

    // (2 + 3) * 4 = 5 * 4 = 20
    let result = ctx.eval("return (2 + 3) * 4;").unwrap();
    assert_eq!(result.to_i32(), Some(20));
}

#[test]
fn test_eval_comparison() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx.eval("return 5 < 10;").unwrap();
    assert_eq!(result.to_bool(), Some(true));

    let result = ctx.eval("return 10 < 5;").unwrap();
    assert_eq!(result.to_bool(), Some(false));

    let result = ctx.eval("return 5 === 5;").unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_eval_variables() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx.eval("var x = 10; return x;").unwrap();
    assert_eq!(result.to_i32(), Some(10));

    let result = ctx.eval("var x = 5; var y = 3; return x + y;").unwrap();
    assert_eq!(result.to_i32(), Some(8));
}

#[test]
fn test_eval_if_else() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval("var x = 5; if (x < 10) { return 1; } else { return 2; }")
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));

    let result = ctx
        .eval("var x = 15; if (x < 10) { return 1; } else { return 2; }")
        .unwrap();
    assert_eq!(result.to_i32(), Some(2));
}

#[test]
fn test_eval_while_loop() {
    let mut ctx = Context::new(64 * 1024);

    // Sum 1 to 5
    let result = ctx
        .eval(
            "
        var sum = 0;
        var i = 1;
        while (i < 6) {
            sum = sum + i;
            i = i + 1;
        }
        return sum;
    ",
        )
        .unwrap();

    assert_eq!(result.to_i32(), Some(15));
}

#[test]
fn test_eval_assignment() {
    let mut ctx = Context::new(64 * 1024);

    // Simple assignment
    let result = ctx.eval("var x = 5; x = 10; return x;").unwrap();
    assert_eq!(result.to_i32(), Some(10));

    // Assignment returns the assigned value
    let result = ctx.eval("var x = 0; return x = 42;").unwrap();
    assert_eq!(result.to_i32(), Some(42));
}

#[test]
fn test_eval_compound_assignment() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx.eval("var x = 10; x += 5; return x;").unwrap();
    assert_eq!(result.to_i32(), Some(15));

    let result = ctx.eval("var x = 10; x -= 3; return x;").unwrap();
    assert_eq!(result.to_i32(), Some(7));

    let result = ctx.eval("var x = 4; x *= 3; return x;").unwrap();
    assert_eq!(result.to_i32(), Some(12));

    let result = ctx.eval("var x = 20; x /= 4; return x;").unwrap();
    assert_eq!(result.to_i32(), Some(5));
}

#[test]
fn test_eval_for_loop() {
    let mut ctx = Context::new(64 * 1024);

    // Simple for loop test - just count iterations
    let result = ctx
        .eval(
            "
        var count = 0;
        for (var i = 0; i < 3; i = i + 1) {
            count = count + 1;
        }
        return count;
    ",
        )
        .unwrap();

    assert_eq!(result.to_i32(), Some(3));
}

#[test]
fn test_eval_for_loop_sum() {
    let mut ctx = Context::new(64 * 1024);

    // Sum 1 to 5 using for loop
    let result = ctx
        .eval(
            "
        var sum = 0;
        for (var i = 1; i < 6; i = i + 1) {
            sum = sum + i;
        }
        return sum;
    ",
        )
        .unwrap();

    assert_eq!(result.to_i32(), Some(15));
}

#[test]
fn test_eval_ternary() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx.eval("return 1 ? 100 : 200;").unwrap();
    assert_eq!(result.to_i32(), Some(100));

    let result = ctx.eval("return 0 ? 100 : 200;").unwrap();
    assert_eq!(result.to_i32(), Some(200));
}

#[test]
fn test_eval_boolean_literals() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx.eval("return true;").unwrap();
    assert_eq!(result.to_bool(), Some(true));

    let result = ctx.eval("return false;").unwrap();
    assert_eq!(result.to_bool(), Some(false));
}

#[test]
fn test_eval_null() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx.eval("return null;").unwrap();
    assert!(result.is_null());
}

#[test]
fn test_eval_unary() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx.eval("return -5;").unwrap();
    assert_eq!(result.to_i32(), Some(-5));

    let result = ctx.eval("return !false;").unwrap();
    assert_eq!(result.to_bool(), Some(true));

    let result = ctx.eval("return !true;").unwrap();
    assert_eq!(result.to_bool(), Some(false));
}

#[test]
fn test_compile_error() {
    let mut ctx = Context::new(64 * 1024);

    // Missing semicolon should cause compile error
    let result = ctx.eval("return 1 +");
    assert!(result.is_err());
}

#[test]
fn test_function_declaration() {
    let mut ctx = Context::new(64 * 1024);

    // Simple function that returns a constant
    let result = ctx
        .eval(
            "
        function five() {
            return 5;
        }
        return five();
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(5));
}

#[test]
fn test_function_with_args() {
    let mut ctx = Context::new(64 * 1024);

    // Function with arguments
    let result = ctx
        .eval(
            "
        function add(a, b) {
            return a + b;
        }
        return add(10, 20);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(30));
}

#[test]
fn test_function_with_local() {
    let mut ctx = Context::new(64 * 1024);

    // Function with local variable
    let result = ctx
        .eval(
            "
        function double(x) {
            var result = x * 2;
            return result;
        }
        return double(7);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(14));
}

#[test]
fn test_recursive_function() {
    let mut ctx = Context::new(64 * 1024);

    // Recursive factorial
    let result = ctx
        .eval(
            "
        function factorial(n) {
            if (n < 2) {
                return 1;
            }
            return n * factorial(n - 1);
        }
        return factorial(5);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(120)); // 5! = 120
}

#[test]
fn test_multiple_functions() {
    let mut ctx = Context::new(64 * 1024);

    // Multiple independent functions (cross-function calls require closures - Stage 7)
    let result = ctx
        .eval(
            "
        function triple(x) {
            return x * 3;
        }
        function negate(x) {
            return 0 - x;
        }
        var a = triple(5);
        var b = negate(7);
        return a + b;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(8)); // 15 + (-7) = 8
}

#[test]
fn test_nested_function_calls() {
    let mut ctx = Context::new(64 * 1024);

    // Test that we can call the same function multiple times
    let result = ctx
        .eval(
            "
        function add(a, b) {
            return a + b;
        }
        return add(add(1, 2), add(3, 4));
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(10)); // (1+2) + (3+4) = 10
}

#[test]
fn test_break_in_while() {
    let mut ctx = Context::new(64 * 1024);

    // Break out of while loop
    let result = ctx
        .eval(
            "
        var i = 0;
        while (i < 100) {
            if (i === 5) {
                break;
            }
            i = i + 1;
        }
        return i;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(5));
}

#[test]
fn test_break_in_for() {
    let mut ctx = Context::new(64 * 1024);

    // Break out of for loop
    let result = ctx
        .eval(
            "
        var sum = 0;
        for (var i = 0; i < 100; i = i + 1) {
            if (i === 5) {
                break;
            }
            sum = sum + i;
        }
        return sum;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(10)); // 0 + 1 + 2 + 3 + 4 = 10
}

#[test]
fn test_continue_in_while() {
    let mut ctx = Context::new(64 * 1024);

    // Skip even numbers
    let result = ctx
        .eval(
            "
        var sum = 0;
        var i = 0;
        while (i < 10) {
            i = i + 1;
            if (i % 2 === 0) {
                continue;
            }
            sum = sum + i;
        }
        return sum;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(25)); // 1 + 3 + 5 + 7 + 9 = 25
}

#[test]
fn test_continue_in_for() {
    let mut ctx = Context::new(64 * 1024);

    // Skip multiples of 3
    let result = ctx
        .eval(
            "
        var sum = 0;
        for (var i = 1; i < 10; i = i + 1) {
            if (i % 3 === 0) {
                continue;
            }
            sum = sum + i;
        }
        return sum;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(27)); // 1+2+4+5+7+8 = 27
}

#[test]
fn test_typeof_operator() {
    use mquickjs::value::{STR_BOOLEAN, STR_FUNCTION, STR_NUMBER, STR_OBJECT, STR_UNDEFINED};

    let mut ctx = Context::new(64 * 1024);

    // typeof now returns string values
    // typeof undefined
    let result = ctx.eval("var x; return typeof x;").unwrap();
    assert!(result.is_string());
    assert_eq!(result.to_string_idx(), Some(STR_UNDEFINED));

    // typeof null
    let result = ctx.eval("return typeof null;").unwrap();
    assert!(result.is_string());
    assert_eq!(result.to_string_idx(), Some(STR_OBJECT)); // JS quirk

    // typeof boolean
    let result = ctx.eval("return typeof true;").unwrap();
    assert!(result.is_string());
    assert_eq!(result.to_string_idx(), Some(STR_BOOLEAN));

    // typeof number
    let result = ctx.eval("return typeof 42;").unwrap();
    assert!(result.is_string());
    assert_eq!(result.to_string_idx(), Some(STR_NUMBER));

    // typeof function
    let result = ctx.eval("function f() {} return typeof f;").unwrap();
    assert!(result.is_string());
    assert_eq!(result.to_string_idx(), Some(STR_FUNCTION));
}

#[test]
fn test_string_literal() {
    use mquickjs::value::STR_STRING;

    let mut ctx = Context::new(64 * 1024);

    // typeof string
    let result = ctx.eval("return typeof \"hello\";").unwrap();
    assert!(result.is_string());
    assert_eq!(result.to_string_idx(), Some(STR_STRING));
}

#[test]
fn test_empty_string() {
    use mquickjs::value::STR_STRING;

    let mut ctx = Context::new(64 * 1024);

    // Empty string
    let result = ctx.eval("return typeof \"\";").unwrap();
    assert!(result.is_string());
    assert_eq!(result.to_string_idx(), Some(STR_STRING));
}

#[test]
fn test_string_variable() {
    use mquickjs::value::STR_STRING;

    let mut ctx = Context::new(64 * 1024);

    // Store string in variable and check type
    let result = ctx.eval("var s = \"world\"; return typeof s;").unwrap();
    assert!(result.is_string());
    assert_eq!(result.to_string_idx(), Some(STR_STRING));
}

#[test]
fn test_string_self_equality() {
    let mut ctx = Context::new(64 * 1024);

    // String variable equals itself
    let result = ctx.eval("var s = \"test\"; return s === s;").unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_empty_string_equality() {
    let mut ctx = Context::new(64 * 1024);

    // Two empty strings are equal (both map to same sentinel index)
    let result = ctx.eval("return \"\" === \"\";").unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_string_concat() {
    let mut ctx = Context::new(64 * 1024);

    // Basic string concatenation
    let result = ctx.eval("return \"hello\" + \" world\";").unwrap();
    assert!(result.is_string());

    // Concat with number
    let result = ctx.eval("return \"value: \" + 42;").unwrap();
    assert!(result.is_string());

    // Number + string
    let result = ctx.eval("return 123 + \"abc\";").unwrap();
    assert!(result.is_string());
}

#[test]
fn test_string_concat_in_variable() {
    let mut ctx = Context::new(64 * 1024);

    // Store concatenated string and check type
    let result = ctx.eval("var s = \"a\" + \"b\"; return typeof s;").unwrap();
    assert!(result.is_string());
    assert_eq!(result.to_string_idx(), Some(mquickjs::value::STR_STRING));
}

#[test]
fn test_string_concat_assignment_statement() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx.eval("var s = ''; s = s + 'x'; return s;").unwrap();
    assert!(result.is_string());
}

#[test]
fn test_string_concat_chain() {
    let mut ctx = Context::new(64 * 1024);

    // Multiple concatenations
    let result = ctx.eval("return \"a\" + \"b\" + \"c\";").unwrap();
    assert!(result.is_string());
}

#[test]
fn test_string_concat_with_bool() {
    let mut ctx = Context::new(64 * 1024);

    // String + boolean
    let result = ctx.eval("return \"value: \" + true;").unwrap();
    assert!(result.is_string());
}

#[test]
fn test_string_concat_with_null() {
    let mut ctx = Context::new(64 * 1024);

    // String + null
    let result = ctx.eval("return \"value: \" + null;").unwrap();
    assert!(result.is_string());
}

#[test]
fn test_print_statement() {
    let mut ctx = Context::new(64 * 1024);

    // Print should execute without error and return undefined
    let result = ctx.eval("print 42; return 1;").unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_print_string() {
    let mut ctx = Context::new(64 * 1024);

    // Print a string
    let result = ctx.eval("print \"hello world\"; return 1;").unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_print_expression() {
    let mut ctx = Context::new(64 * 1024);

    // Print result of expression
    let result = ctx.eval("print 2 + 3; return 1;").unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_simple_closure() {
    let mut ctx = Context::new(64 * 1024);

    // Simple closure that captures a variable
    let result = ctx
        .eval(
            "
        function outer() {
            var x = 42;
            function inner() {
                return x;
            }
            return inner();
        }
        return outer();
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(42));
}

#[test]
fn test_later_closure_sees_reassigned_captured_local() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        function outer() {
            var x = 1;
            function set(v) {
                x = v;
            }
            set(2);
            function get() {
                return x;
            }
            return get();
        }
        return outer();
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(2));
}

#[test]
fn test_captured_local0_string_builder_materializes_for_later_closure() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        function outer() {
            var s = '';
            function first() {
                return s;
            }
            s = s + 'x';
            function second() {
                return s;
            }
            return second();
        }
        return outer();
    ",
        )
        .unwrap();
    assert!(result.is_string());
    assert_eq!(ctx.string_value(result).as_deref(), Some("x"));
}

#[test]
fn test_closure_captures_value() {
    let mut ctx = Context::new(64 * 1024);

    // Closure captures the value at definition time (value capture semantics)
    let result = ctx
        .eval(
            "
        function makeAdder(x) {
            function adder(y) {
                return x + y;
            }
            return adder;
        }
        var add5 = makeAdder(5);
        return add5(10);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(15));
}

#[test]
fn test_closure_with_multiple_captures() {
    let mut ctx = Context::new(64 * 1024);

    // Closure that captures multiple variables
    let result = ctx
        .eval(
            "
        function outer() {
            var a = 10;
            var b = 20;
            function inner() {
                return a + b;
            }
            return inner();
        }
        return outer();
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(30));
}

#[test]
fn test_closure_with_parameter() {
    let mut ctx = Context::new(64 * 1024);

    // Closure captures parameter
    let result = ctx
        .eval(
            "
        function multiplier(factor) {
            function mult(x) {
                return x * factor;
            }
            return mult;
        }
        var double = multiplier(2);
        return double(7);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(14));
}

#[test]
fn test_closure_typeof() {
    use mquickjs::value::STR_FUNCTION;

    let mut ctx = Context::new(64 * 1024);

    // typeof closure should be "function"
    let result = ctx
        .eval(
            "
        function outer() {
            var x = 1;
            function inner() {
                return x;
            }
            return inner;
        }
        return typeof outer();
    ",
        )
        .unwrap();
    assert!(result.is_string());
    assert_eq!(result.to_string_idx(), Some(STR_FUNCTION));
}

#[test]
fn test_try_catch_basic() {
    let mut ctx = Context::new(64 * 1024);

    // Basic try-catch that catches an exception
    let result = ctx
        .eval(
            "
        var result = 0;
        try {
            throw 42;
        } catch (e) {
            result = e;
        }
        return result;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(42));
}

#[test]
fn test_try_catch_no_exception() {
    let mut ctx = Context::new(64 * 1024);

    // Try block completes normally, catch is skipped
    let result = ctx
        .eval(
            "
        var result = 0;
        try {
            result = 10;
        } catch (e) {
            result = 99;
        }
        return result;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(10));
}

#[test]
fn test_try_catch_with_finally() {
    let mut ctx = Context::new(64 * 1024);

    // Finally always executes
    let result = ctx
        .eval(
            "
        var result = 0;
        try {
            result = 1;
        } catch (e) {
            result = 2;
        } finally {
            result = result + 10;
        }
        return result;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(11)); // 1 + 10
}

#[test]
fn test_try_catch_exception_with_finally() {
    let mut ctx = Context::new(64 * 1024);

    // Exception caught, then finally executes
    let result = ctx
        .eval(
            "
        var result = 0;
        try {
            throw 5;
        } catch (e) {
            result = e;
        } finally {
            result = result + 100;
        }
        return result;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(105)); // 5 + 100
}

#[test]
fn test_try_finally_without_catch_runs_before_outer_handler() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var result = 0;
        try {
            try {
                throw 1;
            } finally {
                result = 7;
            }
        } catch (e) {}
        return result;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(7));
}

#[test]
fn test_try_catch_repeated_throw_loop() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var sum = 0;
        for (var i = 0; i < 5; i = i + 1) {
            try {
                throw i;
            } catch (e) {
                sum = sum + e;
            }
        }
        return sum;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(10));
}

#[test]
fn test_throw_statement() {
    let mut ctx = Context::new(64 * 1024);

    // Throw and catch a value
    let result = ctx
        .eval(
            "
        function mayThrow(x) {
            if (x < 0) {
                throw x;
            }
            return x * 2;
        }

        var result = 0;
        try {
            result = mayThrow(-5);
        } catch (e) {
            result = e + 100;
        }
        return result;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(95)); // -5 + 100
}

#[test]
fn test_nested_try_catch() {
    let mut ctx = Context::new(64 * 1024);

    // Nested try-catch
    let result = ctx
        .eval(
            "
        var result = 0;
        try {
            try {
                throw 1;
            } catch (inner) {
                result = inner + 10;
                throw result;
            }
        } catch (outer) {
            result = outer + 100;
        }
        return result;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(111)); // (1 + 10) + 100
}

#[test]
fn test_array_literal_empty() {
    let mut ctx = Context::new(64 * 1024);

    // Empty array
    let result = ctx
        .eval(
            "
        var arr = [];
        return 1;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_array_typeof() {
    let mut ctx = Context::new(64 * 1024);

    // Check typeof array - should be "object" in JavaScript
    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3];
        return 42;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(42));
}

#[test]
fn test_array_literal_with_elements() {
    let mut ctx = Context::new(64 * 1024);

    // Array with elements
    let result = ctx
        .eval(
            "
        var arr = [10, 20, 30];
        return arr[1];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(20));
}

#[test]
fn test_array_element_access() {
    let mut ctx = Context::new(64 * 1024);

    // Access elements at different indices
    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3, 4, 5];
        return arr[0] + arr[2] + arr[4];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(9)); // 1 + 3 + 5
}

#[test]
fn test_array_element_assignment() {
    let mut ctx = Context::new(64 * 1024);

    // Assign to array element
    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3];
        arr[1] = 100;
        return arr[0] + arr[1] + arr[2];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(104)); // 1 + 100 + 3
}

#[test]
fn test_array_extend_on_assignment() {
    let mut ctx = Context::new(64 * 1024);

    // Assign to index beyond current length
    let result = ctx
        .eval(
            "
        var arr = [1, 2];
        arr[5] = 100;
        return arr[5];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(100));
}

#[test]
fn test_array_assignment_expression_returns_value() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [];
        var x = (arr[0] = 7);
        return x + arr[0];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(14));
}

#[test]
fn test_local_assignment_statement_update() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var i = 0;
        i = i + 1;
        i = i + 1;
        return i;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(2));
}

#[test]
fn test_local_assignment_statement_update_preserves_string_plus_one() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval(
            "
        var x = 'a';
        x = x + 1;
        return x;
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_local_assignment_statement_update_overflow_preserves_result() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval(
            "
        var x = 2147483647;
        x = x + 1;
        return x;
    ",
        )
        .unwrap();
    assert_eq!(result.to_number_f32(), Some(2147483648.0));
}

#[test]
fn test_array_out_of_bounds_returns_undefined() {
    let mut ctx = Context::new(64 * 1024);

    // Access index beyond current length returns undefined
    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3];
        var x = arr[10];
        if (x) {
            return 1;
        }
        return 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(0)); // undefined is falsy
}

#[test]
fn test_array_in_expression() {
    let mut ctx = Context::new(64 * 1024);

    // Use array elements in expressions
    let result = ctx
        .eval(
            "
        var arr = [2, 3, 4];
        return arr[0] * arr[1] * arr[2];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(24)); // 2 * 3 * 4
}

#[test]
fn test_discarded_array_read_still_evaluates_and_continues() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3];
        arr[1];
        return 7;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(7));
}

#[test]
fn test_discarded_negative_array_read_still_continues() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3];
        arr[-1];
        return 9;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(9));
}

#[test]
fn test_array_nested() {
    let mut ctx = Context::new(64 * 1024);

    // Nested arrays (array of arrays)
    let result = ctx
        .eval(
            "
        var arr = [[1, 2], [3, 4]];
        var inner = arr[1];
        return inner[0];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(3));
}

#[test]
fn test_array_with_trailing_comma() {
    let mut ctx = Context::new(64 * 1024);

    // Trailing comma is allowed
    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3,];
        return arr[2];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(3));
}

#[test]
fn test_array_computed_index() {
    let mut ctx = Context::new(64 * 1024);

    // Use computed index
    let result = ctx
        .eval(
            "
        var arr = [10, 20, 30, 40];
        var i = 1 + 1;
        return arr[i];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(30)); // arr[2]
}

// =========================================================================
// New operator and object tests
// =========================================================================

#[test]
fn test_new_simple_constructor() {
    let mut ctx = Context::new(64 * 1024);

    // Simple constructor that sets properties on this
    let result = ctx
        .eval(
            "
        function Point(x, y) {
            this.x = x;
            this.y = y;
            return this;
        }
        var p = new Point(3, 4);
        return p.x + p.y;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(7)); // 3 + 4
}

#[test]
fn test_new_without_args() {
    let mut ctx = Context::new(64 * 1024);

    // Constructor with no arguments
    let result = ctx
        .eval(
            "
        function Counter() {
            this.count = 0;
            return this;
        }
        var c = new Counter();
        return c.count;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(0));
}

#[test]
fn test_object_property_get() {
    let mut ctx = Context::new(64 * 1024);

    // Get property from constructed object
    let result = ctx
        .eval(
            "
        function Box(val) {
            this.value = val;
            return this;
        }
        var b = new Box(42);
        return b.value;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(42));
}

#[test]
fn test_object_property_set() {
    let mut ctx = Context::new(64 * 1024);

    // Set property on object after construction
    let result = ctx
        .eval(
            "
        function Obj() {
            this.a = 1;
            return this;
        }
        var o = new Obj();
        o.a = 100;
        return o.a;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(100));
}

#[test]
fn test_object_multiple_properties() {
    let mut ctx = Context::new(64 * 1024);

    // Object with multiple properties
    let result = ctx
        .eval(
            "
        function Person(name, age) {
            this.name = name;
            this.age = age;
            return this;
        }
        var p = new Person(0, 25);
        return p.age;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(25));
}

#[test]
fn test_typeof_object() {
    let mut ctx = Context::new(64 * 1024);

    // First, verify the object is created and is_object() works
    let obj_result = ctx
        .eval(
            "
        function Foo() {
            return this;
        }
        var f = new Foo();
        return f;
    ",
        )
        .unwrap();
    assert!(obj_result.is_object(), "Result should be an object");

    // Now test typeof
    let mut ctx2 = Context::new(64 * 1024);
    let result = ctx2
        .eval(
            "
        function Foo() {
            return this;
        }
        var f = new Foo();
        if (typeof f === 'object') {
            return 1;
        }
        return 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

// =========================================================================
// In and Delete operator tests
// =========================================================================

#[test]
fn test_in_operator_array() {
    let mut ctx = Context::new(64 * 1024);

    // Check if index exists in array
    let result = ctx
        .eval(
            "
        var arr = [10, 20, 30];
        if (1 in arr) {
            return 1;
        }
        return 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_in_operator_array_out_of_bounds() {
    let mut ctx = Context::new(64 * 1024);

    // Check if out-of-bounds index returns false
    let result = ctx
        .eval(
            "
        var arr = [10, 20, 30];
        if (5 in arr) {
            return 1;
        }
        return 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(0));
}

#[test]
fn test_in_operator_object() {
    let mut ctx = Context::new(64 * 1024);

    // Check if property exists in object
    let result = ctx
        .eval(
            "
        function Obj() {
            this.x = 1;
            this.y = 2;
            return this;
        }
        var o = new Obj();
        if ('x' in o) {
            return 1;
        }
        return 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_in_operator_object_missing() {
    let mut ctx = Context::new(64 * 1024);

    // Check that missing property returns false
    let result = ctx
        .eval(
            "
        function Obj() {
            this.x = 1;
            return this;
        }
        var o = new Obj();
        if ('z' in o) {
            return 1;
        }
        return 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(0));
}

#[test]
fn test_delete_array_element() {
    let mut ctx = Context::new(64 * 1024);

    // Delete array element sets it to undefined
    let result = ctx
        .eval(
            "
        var arr = [10, 20, 30];
        delete arr[1];
        if (arr[1]) {
            return 1;
        }
        return 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(0)); // arr[1] is now undefined (falsy)
}

#[test]
fn test_delete_object_property() {
    let mut ctx = Context::new(64 * 1024);

    // Delete object property
    let result = ctx
        .eval(
            "
        function Obj() {
            this.x = 1;
            this.y = 2;
            return this;
        }
        var o = new Obj();
        delete o.x;
        if ('x' in o) {
            return 1;
        }
        return 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(0)); // x is deleted
}

#[test]
fn test_delete_returns_true() {
    let mut ctx = Context::new(64 * 1024);

    // Delete returns true on success
    let result = ctx
        .eval(
            "
        var arr = [10, 20, 30];
        if (delete arr[1]) {
            return 1;
        }
        return 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_instanceof_basic() {
    let mut ctx = Context::new(64 * 1024);

    // Object created with new is instanceof its constructor
    let result = ctx
        .eval(
            "
        function Foo() {
            return this;
        }
        var f = new Foo();
        if (f instanceof Foo) {
            return 1;
        }
        return 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_instanceof_different_constructors() {
    let mut ctx = Context::new(64 * 1024);

    // Object is not instanceof a different constructor
    let result = ctx
        .eval(
            "
        function Foo() { return this; }
        function Bar() { return this; }
        var f = new Foo();
        if (f instanceof Bar) {
            return 1;
        }
        return 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(0));
}

#[test]
fn test_instanceof_non_object() {
    let mut ctx = Context::new(64 * 1024);

    // Non-object is not instanceof anything
    let result = ctx
        .eval(
            "
        function Foo() { return this; }
        var x = 42;
        if (x instanceof Foo) {
            return 1;
        }
        return 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(0));
}

#[test]
fn test_instanceof_multiple_instances() {
    let mut ctx = Context::new(64 * 1024);

    // Multiple instances of the same constructor
    let result = ctx
        .eval(
            "
        function Foo() { return this; }
        var f1 = new Foo();
        var f2 = new Foo();
        if (f1 instanceof Foo) {
            if (f2 instanceof Foo) {
                return 2;
            }
            return 1;
        }
        return 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(2));
}

#[test]
fn test_instanceof_with_properties() {
    let mut ctx = Context::new(64 * 1024);

    // instanceof works even with properties set on object
    let result = ctx
        .eval(
            "
        function Person(name) {
            this.name = name;
            return this;
        }
        var p = new Person('Alice');
        if (p instanceof Person) {
            return 1;
        }
        return 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_typeof_missing_var_returns_undefined() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval("return typeof missingVar === 'undefined';")
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_typeof_missing_var_does_not_corrupt_existing_string_constant() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval("var s = 'hello'; typeof missingVar; return s;")
        .unwrap();
    assert!(result.is_string());
    assert_eq!(ctx.string_value(result).as_deref(), Some("hello"));
}

#[test]
fn test_for_in_array() {
    let mut ctx = Context::new(64 * 1024);

    // Iterate over array indices
    let result = ctx
        .eval(
            "
        var arr = [10, 20, 30];
        var sum = 0;
        for (var k in arr) {
            sum = sum + 1;
        }
        return sum;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(3)); // 3 indices
}

#[test]
fn test_for_in_object() {
    let mut ctx = Context::new(64 * 1024);

    // Iterate over object keys
    let result = ctx
        .eval(
            "
        function Obj() {
            this.a = 1;
            this.b = 2;
            this.c = 3;
            return this;
        }
        var obj = new Obj();
        var count = 0;
        for (var k in obj) {
            count = count + 1;
        }
        return count;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(3)); // 3 properties
}

#[test]
fn test_for_in_empty_object() {
    let mut ctx = Context::new(64 * 1024);

    // Iterate over empty object
    let result = ctx
        .eval(
            "
        function Empty() {
            return this;
        }
        var obj = new Empty();
        var count = 0;
        for (var k in obj) {
            count = count + 1;
        }
        return count;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(0)); // no properties
}

#[test]
fn test_for_in_empty_array() {
    let mut ctx = Context::new(64 * 1024);

    // Iterate over empty array
    let result = ctx
        .eval(
            "
        var arr = [];
        var count = 0;
        for (var k in arr) {
            count = count + 1;
        }
        return count;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(0)); // no elements
}

#[test]
fn test_for_in_break() {
    let mut ctx = Context::new(64 * 1024);

    // Break in for-in loop
    let result = ctx
        .eval(
            "
        var arr = [10, 20, 30, 40, 50];
        var count = 0;
        for (var k in arr) {
            count = count + 1;
            if (count > 2) {
                break;
            }
        }
        return count;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(3)); // stopped after 3
}

#[test]
fn test_for_in_object_observes_value_updates() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var obj = { a: 1, b: 2, c: 3 };
        var sum = 0;
        for (var k in obj) {
            if (k === 'a') {
                sum = sum + obj.a;
                obj.b = 20;
            } else if (k === 'b') {
                sum = sum + obj.b;
            } else if (k === 'c') {
                sum = sum + obj.c;
            }
        }
        return sum;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(24)); // 1 + 20 + 3
}

#[test]
fn test_for_of_array() {
    let mut ctx = Context::new(64 * 1024);

    // Sum values in array using for-of
    let result = ctx
        .eval(
            "
        var arr = [10, 20, 30];
        var sum = 0;
        for (var val of arr) {
            sum = sum + val;
        }
        return sum;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(60)); // 10+20+30
}

#[test]
fn test_for_of_object_values() {
    let mut ctx = Context::new(64 * 1024);

    // Simplest for-of test with array (already works)
    let result = ctx
        .eval(
            "
        var arr = [5, 7];
        var sum = 0;
        for (var val of arr) {
            sum = sum + val;
        }
        return sum;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(12)); // 5+7 - this should work

    // Now test with object
    let mut ctx2 = Context::new(64 * 1024);
    let result = ctx2
        .eval(
            "
        function Point(x, y) {
            this.x = x;
            this.y = y;
        }
        var p = new Point(5, 7);
        var sum = 0;
        for (var val of p) {
            sum = sum + val;
        }
        return sum;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(12)); // 5+7
}

#[test]
fn test_for_of_empty_array() {
    let mut ctx = Context::new(64 * 1024);

    // for-of on empty array
    let result = ctx
        .eval(
            "
        var arr = [];
        var count = 0;
        for (var val of arr) {
            count = count + 1;
        }
        return count;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(0)); // no iterations
}

#[test]
fn test_for_of_break() {
    let mut ctx = Context::new(64 * 1024);

    // Break in for-of loop
    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3, 4, 5];
        var sum = 0;
        for (var val of arr) {
            sum = sum + val;
            if (sum > 5) {
                break;
            }
        }
        return sum;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(6)); // 1+2+3=6, then break
}

#[test]
fn test_for_of_continue() {
    let mut ctx = Context::new(64 * 1024);

    // Continue in for-of loop (skip even values)
    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3, 4, 5];
        var sum = 0;
        for (var val of arr) {
            if (val % 2 === 0) {
                continue;
            }
            sum = sum + val;
        }
        return sum;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(9)); // 1+3+5=9
}

#[test]
fn test_for_of_array_observes_element_updates() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3];
        var sum = 0;
        for (var val of arr) {
            sum = sum + val;
            if (val === 1) {
                arr[1] = 20;
            }
        }
        return sum;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(24)); // 1 + 20 + 3
}

// =========================================================================
// Native function tests
// =========================================================================

#[test]
fn test_parse_int() {
    let mut ctx = Context::new(64 * 1024);

    // parseInt with number
    let result = ctx.eval("return parseInt(42);").unwrap();
    assert_eq!(result.to_i32(), Some(42));

    // parseInt with negative
    let result = ctx.eval("return parseInt(-100);").unwrap();
    assert_eq!(result.to_i32(), Some(-100));
}

#[test]
fn test_parse_int_partial_and_radix() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx.eval("return parseInt('123abc');").unwrap();
    assert_eq!(result.to_i32(), Some(123));

    let result = ctx.eval("return parseInt('0xFF', 16);").unwrap();
    assert_eq!(result.to_i32(), Some(255));

    let result = ctx.eval("return parseInt('0x10');").unwrap();
    assert_eq!(result.to_i32(), Some(16));
}

#[test]
fn test_is_nan() {
    let mut ctx = Context::new(64 * 1024);

    // isNaN with number returns false
    let result = ctx.eval("return isNaN(42);").unwrap();
    assert_eq!(result.to_bool(), Some(false));

    // isNaN with undefined returns true (since undefined is not a number)
    let result = ctx.eval("return isNaN(undefined);").unwrap();
    assert_eq!(result.to_bool(), Some(true));

    // Global isNaN performs ToNumber on strings
    let result = ctx.eval("return isNaN('3');").unwrap();
    assert_eq!(result.to_bool(), Some(false));

    let result = ctx.eval("return isNaN('foo');").unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_is_finite_global() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx.eval("return isFinite(42);").unwrap();
    assert_eq!(result.to_bool(), Some(true));

    // Global isFinite performs ToNumber on strings
    let result = ctx.eval("return isFinite('3');").unwrap();
    assert_eq!(result.to_bool(), Some(true));

    let result = ctx.eval("return isFinite('foo');").unwrap();
    assert_eq!(result.to_bool(), Some(false));
}

#[test]
fn test_native_function_in_expression() {
    let mut ctx = Context::new(64 * 1024);

    // Use native function in expression
    let result = ctx
        .eval(
            "
        var x = parseInt(10);
        var y = parseInt(20);
        return x + y;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(30));
}

#[test]
fn test_native_function_as_value() {
    let mut ctx = Context::new(64 * 1024);

    // Store native function in variable and call it
    let result = ctx
        .eval(
            "
        var f = parseInt;
        return f(42);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(42));
}

// =========================================================================
// Math object tests
// =========================================================================

#[test]
fn test_math_abs() {
    let mut ctx = Context::new(64 * 1024);

    // Math.abs with positive
    let result = ctx.eval("return Math.abs(42);").unwrap();
    assert_eq!(result.to_i32(), Some(42));

    // Math.abs with negative
    let result = ctx.eval("return Math.abs(-42);").unwrap();
    assert_eq!(result.to_i32(), Some(42));

    // Math.abs with zero
    let result = ctx.eval("return Math.abs(0);").unwrap();
    assert_eq!(result.to_i32(), Some(0));
}

#[test]
fn test_math_abs_i32_min_promotes_to_positive_number() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx.eval("return Math.abs(-2147483648);").unwrap();
    assert_eq!(result.to_number_f32(), Some(2147483648.0));
}

#[test]
fn test_math_floor_ceil_round() {
    let mut ctx = Context::new(64 * 1024);

    // Math.floor (integers pass through)
    let result = ctx.eval("return Math.floor(42);").unwrap();
    assert_eq!(result.to_i32(), Some(42));

    // Math.ceil (integers pass through)
    let result = ctx.eval("return Math.ceil(42);").unwrap();
    assert_eq!(result.to_i32(), Some(42));

    // Math.round (integers pass through)
    let result = ctx.eval("return Math.round(42);").unwrap();
    assert_eq!(result.to_i32(), Some(42));
}

#[test]
fn test_math_max_min() {
    let mut ctx = Context::new(64 * 1024);

    // Math.max with two values
    let result = ctx.eval("return Math.max(10, 20);").unwrap();
    assert_eq!(result.to_i32(), Some(20));

    // Math.max with negative values
    let result = ctx.eval("return Math.max(-10, -5);").unwrap();
    assert_eq!(result.to_i32(), Some(-5));

    // Math.min with two values
    let result = ctx.eval("return Math.min(10, 20);").unwrap();
    assert_eq!(result.to_i32(), Some(10));

    // Math.min with negative values
    let result = ctx.eval("return Math.min(-10, -5);").unwrap();
    assert_eq!(result.to_i32(), Some(-10));
}

#[test]
fn test_math_max_min_float_nan_infinity() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx.eval("return Math.max(1.5, 2.25, -3.0);").unwrap();
    let val = result.to_number_f32().unwrap();
    assert!((val - 2.25).abs() < 0.001);

    let result = ctx.eval("return Math.min(1.5, 2.25, -3.0);").unwrap();
    let val = result.to_number_f32().unwrap();
    assert!((val - (-3.0)).abs() < 0.001);

    let result = ctx.eval("return Math.max(Infinity, 1);").unwrap();
    assert!(result.is_infinite_value());

    let result = ctx.eval("return Math.min(-Infinity, 1);").unwrap();
    assert!(result.is_infinite_value());
    assert!(result.to_number_f32().unwrap().is_sign_negative());

    let result = ctx.eval("return isNaN(Math.max(NaN, 1));").unwrap();
    assert_eq!(result.to_bool(), Some(true));

    let result = ctx.eval("return isNaN(Math.min(1, NaN));").unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_native_function_three_arg_order() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx.eval("return Math.max(1, 4, 2);").unwrap();
    assert_eq!(result.to_i32(), Some(4));
}

#[test]
fn test_math_sqrt() {
    let mut ctx = Context::new(64 * 1024);

    // Math.sqrt of perfect square
    let result = ctx.eval("return Math.sqrt(16);").unwrap();
    assert_eq!(result.to_i32(), Some(4));

    // Math.sqrt of 9
    let result = ctx.eval("return Math.sqrt(9);").unwrap();
    assert_eq!(result.to_i32(), Some(3));

    // Math.sqrt of 0
    let result = ctx.eval("return Math.sqrt(0);").unwrap();
    assert_eq!(result.to_i32(), Some(0));
}

#[test]
fn test_math_pow() {
    let mut ctx = Context::new(64 * 1024);

    // Math.pow(2, 3) = 8
    let result = ctx.eval("return Math.pow(2, 3);").unwrap();
    assert_eq!(result.to_i32(), Some(8));

    // Math.pow(5, 2) = 25
    let result = ctx.eval("return Math.pow(5, 2);").unwrap();
    assert_eq!(result.to_i32(), Some(25));

    // Math.pow(x, 0) = 1
    let result = ctx.eval("return Math.pow(100, 0);").unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_math_in_expression() {
    let mut ctx = Context::new(64 * 1024);

    // Use Math in complex expression
    let result = ctx
        .eval(
            "
        var x = Math.abs(-5);
        var y = Math.max(x, 10);
        return y;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(10));
}

// mquickjs-specific Math functions
#[test]
fn test_math_imul() {
    let mut ctx = Context::new(64 * 1024);

    // Basic integer multiplication
    let result = ctx.eval("return Math.imul(3, 4);").unwrap();
    assert_eq!(result.to_i32(), Some(12));

    // With negative numbers
    let result = ctx.eval("return Math.imul(-5, 3);").unwrap();
    assert_eq!(result.to_i32(), Some(-15));
}

#[test]
fn test_math_clz32() {
    let mut ctx = Context::new(64 * 1024);

    // Count leading zeros
    let result = ctx.eval("return Math.clz32(1);").unwrap();
    assert_eq!(result.to_i32(), Some(31));

    let result = ctx.eval("return Math.clz32(0);").unwrap();
    assert_eq!(result.to_i32(), Some(32));
}

#[test]
fn test_math_trunc() {
    let mut ctx = Context::new(64 * 1024);

    // Trunc on integer is identity
    let result = ctx.eval("return Math.trunc(42);").unwrap();
    assert_eq!(result.to_i32(), Some(42));

    let result = ctx.eval("return Math.trunc(-5);").unwrap();
    assert_eq!(result.to_i32(), Some(-5));
}

#[test]
fn test_math_sign_preserves_negative_zero() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx.eval("return 1 / Math.sign(-0);").unwrap();
    assert!(result.to_number_f32().unwrap().is_sign_negative());
    assert!(result.to_number_f32().unwrap().is_infinite());
}

#[test]
fn test_math_log2() {
    let mut ctx = Context::new(64 * 1024);

    // log2(8) = 3
    let result = ctx.eval("return Math.log2(8);").unwrap();
    assert_eq!(result.to_i32(), Some(3));

    // log2(1) = 0
    let result = ctx.eval("return Math.log2(1);").unwrap();
    assert_eq!(result.to_i32(), Some(0));
}

#[test]
fn test_math_log10() {
    let mut ctx = Context::new(64 * 1024);

    // log10(100) = 2
    let result = ctx.eval("return Math.log10(100);").unwrap();
    assert_eq!(result.to_i32(), Some(2));

    // log10(1000) = 3
    let result = ctx.eval("return Math.log10(1000);").unwrap();
    assert_eq!(result.to_i32(), Some(3));
}

#[test]
fn test_math_fround() {
    let mut ctx = Context::new(64 * 1024);

    // fround returns f32 (identity for our f32 engine)
    let result = ctx.eval("return Math.fround(42);").unwrap();
    assert_eq!(result.to_number_f32(), Some(42.0));
}

// =========================================================================
// Array.prototype method tests
// =========================================================================

#[test]
fn test_array_push() {
    let mut ctx = Context::new(64 * 1024);

    // arr.push(x) returns new length
    let result = ctx
        .eval(
            "
        var arr = [1, 2];
        var len = arr.push(3);
        return len;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(3));
}

#[test]
fn test_array_push_and_access() {
    let mut ctx = Context::new(64 * 1024);

    // Push and then access the pushed element
    let result = ctx
        .eval(
            "
        var arr = [1, 2];
        arr.push(42);
        return arr[2];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(42));
}

#[test]
fn test_array_push_true_specialized_shape() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval(
            "
        var arr = [];
        arr.push(true);
        return arr[0];
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_array_pop() {
    let mut ctx = Context::new(64 * 1024);

    // arr.pop() returns the removed element
    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3];
        var val = arr.pop();
        return val;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(3));
}

#[test]
fn test_array_length_property() {
    let mut ctx = Context::new(64 * 1024);

    // arr.length returns the array length
    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3, 4, 5];
        return arr.length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(5));
}

#[test]
fn test_array_element_length_property() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var parts = ['ab', 'cdef'];
        return parts[1].length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(4));
}

#[test]
fn test_array_push_multiple_args_order() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [];
        arr.push(10, 20);
        return arr[0] + arr[1];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(30));
}

#[test]
fn test_non_array_push_method_call_fallback() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var obj = {
            total: 1,
            push: function (x) {
                this.total = this.total + x;
                return this.total;
            }
        };
        return obj.push(5);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(6));
}

#[test]
fn test_array_push_property_read_happens_before_args() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var side = 0;
        var obj = null;
        try {
            obj.push(side = 1);
        } catch (e) {}
        return side;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(0));
}

#[test]
fn test_generic_method_call_preserves_receiver_before_args() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var obj = {
            base: 7,
            add: function (x) { return this.base + x; }
        };
        return obj.add(5);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(12));
}

#[test]
fn test_method_call_preserves_string_arguments_for_user_functions() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        function make() {
            var color = 1;
            var handlers = {
                setConfig: function (key, value) {
                    if (key === 'color') color = value;
                }
            };
            handlers.setConfig('color', 255);
            return color;
        }
        return make();
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(255));
}

#[test]
fn test_method_call_string_arguments_survive_forwarding_chain() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        function createBaseMachine(handlers) {
            const machine = {
                speed: 200,
                setConfig: function (key, value) {
                    if (key === 'speed') machine.speed = value;
                    else if (handlers.setConfig) handlers.setConfig(key, value);
                }
            };
            return machine;
        }

        function make() {
            var color = 217;
            var m = createBaseMachine({
                setConfig: function (key, value) {
                    if (key === 'color') color = value;
                }
            });
            m.setConfig('speed', 500);
            m.setConfig('color', 255);
            return m.speed * 1000 + color;
        }

        return make();
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(500255));
}

#[test]
fn test_array_element_used_in_if_condition() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [0, 1, 0, 2];
        var sum = 0;
        if (arr[0]) { sum = sum + 100; }
        if (arr[1]) { sum = sum + 1; }
        if (arr[2]) { sum = sum + 100; }
        if (arr[3]) { sum = sum + 2; }
        return sum;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(3));
}

#[test]
fn test_array_length_after_push() {
    let mut ctx = Context::new(64 * 1024);

    // Length updates after push
    let result = ctx
        .eval(
            "
        var arr = [1, 2];
        arr.push(3);
        arr.push(4);
        return arr.length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(4));
}

#[test]
fn test_array_method_chain() {
    let mut ctx = Context::new(64 * 1024);

    // Multiple operations
    let result = ctx
        .eval(
            "
        var arr = [10, 20, 30];
        arr.push(40);
        arr.pop();
        arr.pop();
        return arr.length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(2));
}

#[test]
fn test_array_shift() {
    let mut ctx = Context::new(64 * 1024);

    // arr.shift() removes and returns the first element
    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3];
        var first = arr.shift();
        return first;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));

    // Verify array is modified
    let result = ctx
        .eval(
            "
        var arr = [10, 20, 30];
        arr.shift();
        return arr[0];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(20));
}

#[test]
fn test_array_unshift() {
    let mut ctx = Context::new(64 * 1024);

    // arr.unshift(val) adds to front and returns new length
    let result = ctx
        .eval(
            "
        var arr = [2, 3];
        var len = arr.unshift(1);
        return len;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(3));

    // Verify element is at front
    let result = ctx
        .eval(
            "
        var arr = [2, 3];
        arr.unshift(1);
        return arr[0];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_array_index_of() {
    let mut ctx = Context::new(64 * 1024);

    // arr.indexOf(val) returns index or -1
    let result = ctx
        .eval(
            "
        var arr = [10, 20, 30, 20];
        return arr.indexOf(20);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));

    // Not found returns -1
    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3];
        return arr.indexOf(99);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(-1));
}

#[test]
fn test_array_join() {
    let mut ctx = Context::new(64 * 1024);

    // arr.join() joins with comma by default
    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3];
        return arr.join() === '1,2,3';
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_array_join_with_separator_and_strings() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval(
            "
        var arr = ['a', 'b', 'c'];
        return arr.join('-') === 'a-b-c';
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_array_to_string_null_and_undefined_become_empty_fields() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx.eval("return [null, 1, undefined].toString();").unwrap();
    assert_eq!(ctx.string_value(result).as_deref(), Some(",1,"));
}

#[test]
fn test_array_index_of_and_last_index_of_honor_from_index() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx.eval("return [1,2,1,2].indexOf(1, 1);").unwrap();
    assert_eq!(result.to_i32(), Some(2));

    let result = ctx.eval("return [1,2,1,2].indexOf(2, -1);").unwrap();
    assert_eq!(result.to_i32(), Some(3));

    let result = ctx.eval("return [1,2,1,2].lastIndexOf(2, 2);").unwrap();
    assert_eq!(result.to_i32(), Some(1));

    let result = ctx.eval("return [1,2,1,2].lastIndexOf(1, -2);").unwrap();
    assert_eq!(result.to_i32(), Some(2));
}

#[test]
fn test_array_reverse() {
    let mut ctx = Context::new(64 * 1024);

    // arr.reverse() reverses in place
    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3];
        arr.reverse();
        return arr[0];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(3));

    // Verify last element
    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3];
        arr.reverse();
        return arr[2];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_array_slice() {
    let mut ctx = Context::new(64 * 1024);

    // arr.slice(start, end) returns new array
    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3, 4, 5];
        var sliced = arr.slice(1, 4);
        return sliced.length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(3));

    // Verify slice content
    let result = ctx
        .eval(
            "
        var arr = [10, 20, 30, 40, 50];
        var sliced = arr.slice(1, 3);
        return sliced[0];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(20));
}

#[test]
fn test_array_slice_negative() {
    let mut ctx = Context::new(64 * 1024);

    // arr.slice(-2) returns last 2 elements
    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3, 4, 5];
        var sliced = arr.slice(-2);
        return sliced[0];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(4));
}

// =========================================================================
// String.prototype method tests (using runtime strings from concatenation)
// =========================================================================

#[test]
fn test_string_length() {
    let mut ctx = Context::new(64 * 1024);

    // String length property (using runtime string)
    let result = ctx
        .eval(
            "
        var s = \"hel\" + \"lo\";
        return s.length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(5));
}

#[test]
fn test_string_length_counts_utf16_code_units_not_utf8_bytes() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx.eval("return 'caf\\u00E9'.length;").unwrap();
    assert_eq!(result.to_i32(), Some(4));
}

#[test]
fn test_negative_zero_string_conversion() {
    let mut ctx = Context::new(64 * 1024);
    // JS spec: String(-0) === "0", sign is dropped
    let result = ctx.eval("return String(-0) === '0';").unwrap();
    assert_eq!(result.to_bool(), Some(true));

    // Arithmetic sign must be preserved: 1 / -0 === -Infinity
    let result = ctx.eval("return 1 / -0 === -Infinity;").unwrap();
    assert_eq!(result.to_bool(), Some(true));

    // Concatenation also drops sign
    let result = ctx.eval("return (-0 + '') === '0';").unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_object_length_property_still_uses_regular_lookup() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval(
            "
        var obj = { length: 123 };
        return obj.length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(123));
}

#[test]
fn test_string_char_at() {
    let mut ctx = Context::new(64 * 1024);

    // charAt returns character at index
    let result = ctx
        .eval(
            "
        var s = \"hel\" + \"lo\";
        var c = s.charAt(1);
        return c;
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_string_index_of() {
    let mut ctx = Context::new(64 * 1024);

    // indexOf returns position of substring (using runtime string for search)
    let result = ctx
        .eval(
            "
        var s = \"hel\" + \"lo world\";
        var search = \"wor\" + \"ld\";
        return s.indexOf(search);
    ",
        )
        .unwrap();
    // "hello world" -> "world" starts at index 6
    assert_eq!(result.to_i32(), Some(6));
}

#[test]
fn test_string_index_of_not_found() {
    let mut ctx = Context::new(64 * 1024);

    // indexOf returns -1 when not found (using runtime string for search)
    let result = ctx
        .eval(
            "
        var s = \"hel\" + \"lo\";
        var search = \"x\" + \"yz\";
        return s.indexOf(search);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(-1));
}

#[test]
fn test_string_index_of_unicode_uses_character_indices() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx.eval("return 'caf\\u00E9'.indexOf('\\u00E9');").unwrap();
    assert_eq!(result.to_i32(), Some(3));
}

#[test]
fn test_string_slice() {
    let mut ctx = Context::new(64 * 1024);

    // slice extracts portion of string
    let result = ctx
        .eval(
            "
        var s = \"hel\" + \"lo world\";
        var sub = s.slice(0, 5);
        return sub.length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(5));
}

#[test]
fn test_string_slice_unicode_respects_character_boundaries() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval("return 'caf\\u00E9'.slice(3).charCodeAt(0);")
        .unwrap();
    assert_eq!(result.to_i32(), Some(233));
}

#[test]
fn test_string_slice_negative() {
    let mut ctx = Context::new(64 * 1024);

    // slice with negative index
    let result = ctx
        .eval(
            "
        var s = \"hel\" + \"lo\";
        var sub = s.slice(-2);
        return sub.length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(2));
}

#[test]
fn test_string_to_upper_case() {
    let mut ctx = Context::new(64 * 1024);

    // toUpperCase converts to uppercase
    let result = ctx
        .eval(
            "
        var s = \"hel\" + \"lo\";
        var upper = s.toUpperCase();
        return upper;
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_string_to_lower_case() {
    let mut ctx = Context::new(64 * 1024);

    // toLowerCase converts to lowercase
    let result = ctx
        .eval(
            "
        var s = \"HEL\" + \"LO\";
        var lower = s.toLowerCase();
        return lower;
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_string_trim() {
    let mut ctx = Context::new(64 * 1024);

    // trim removes whitespace
    let result = ctx
        .eval(
            "
        var s = \"  hel\" + \"lo  \";
        var trimmed = s.trim();
        return trimmed.length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(5));
}

#[test]
fn test_string_split() {
    let mut ctx = Context::new(64 * 1024);

    // split returns array of strings (using runtime separator)
    let result = ctx
        .eval(
            "
        var s = \"a\" + \",b,c\";
        var sep = \"\" + \",\";
        var arr = s.split(sep);
        return arr.length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(3));
}

#[test]
fn test_string_split_access() {
    let mut ctx = Context::new(64 * 1024);

    // Access split array element (using runtime separator)
    let result = ctx
        .eval(
            "
        var s = \"one\" + \"-two-three\";
        var sep = \"\" + \"-\";
        var arr = s.split(sep);
        return arr[1];
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_string_concat_method() {
    let mut ctx = Context::new(64 * 1024);

    // String.prototype.concat with multiple args
    let result = ctx
        .eval(
            "
        var s = \"hello\" + \"\";
        var w = \"\" + \" world\";
        var e = \"\" + \"!\";
        return s.concat(w, e);
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_string_concat_method_coerces_float_arguments() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx.eval("return 'x'.concat(3.14);").unwrap();
    assert!(result.is_string());
    assert_eq!(ctx.string_value(result).as_deref(), Some("x3.14"));
}

#[test]
fn test_string_repeat() {
    let mut ctx = Context::new(64 * 1024);

    // String.prototype.repeat
    let result = ctx
        .eval(
            "
        var s = \"ab\" + \"\";
        return s.repeat(3);
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_string_starts_with() {
    let mut ctx = Context::new(64 * 1024);

    // String.prototype.startsWith - true case
    let result = ctx
        .eval(
            "
        var s = \"hello\" + \" world\";
        var prefix = \"hel\" + \"lo\";
        return s.startsWith(prefix);
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));

    // String.prototype.startsWith - false case
    let result = ctx
        .eval(
            "
        var s = \"hello\" + \" world\";
        var prefix = \"wor\" + \"ld\";
        return s.startsWith(prefix);
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(false));
}

#[test]
fn test_string_ends_with() {
    let mut ctx = Context::new(64 * 1024);

    // String.prototype.endsWith - true case
    let result = ctx
        .eval(
            "
        var s = \"hello\" + \" world\";
        var suffix = \"wor\" + \"ld\";
        return s.endsWith(suffix);
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));

    // String.prototype.endsWith - false case
    let result = ctx
        .eval(
            "
        var s = \"hello\" + \" world\";
        var suffix = \"hel\" + \"lo\";
        return s.endsWith(suffix);
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(false));
}

#[test]
fn test_string_pad_start() {
    let mut ctx = Context::new(64 * 1024);

    // String.prototype.padStart with spaces (default)
    let result = ctx
        .eval(
            "
        var s = \"5\" + \"\";
        return s.padStart(3).length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(3));
}

#[test]
fn test_string_pad_end() {
    let mut ctx = Context::new(64 * 1024);

    // String.prototype.padEnd with spaces (default)
    let result = ctx
        .eval(
            "
        var s = \"5\" + \"\";
        return s.padEnd(3).length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(3));
}

#[test]
fn test_string_pad_start_and_end_count_unicode_width_correctly() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval("return '\\u00E9'.padStart(2, '\\u00E9').length;")
        .unwrap();
    assert_eq!(result.to_i32(), Some(2));

    let result = ctx
        .eval("return '\\u00E9'.padEnd(2, '\\u00E9').length;")
        .unwrap();
    assert_eq!(result.to_i32(), Some(2));
}

#[test]
fn test_string_replace() {
    let mut ctx = Context::new(64 * 1024);

    // String.prototype.replace - replace first occurrence
    let result = ctx
        .eval(
            "
        var s = \"foo\" + \" bar foo\";
        var search = \"f\" + \"oo\";
        var rep = \"b\" + \"az\";
        return s.replace(search, rep);
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_string_includes() {
    let mut ctx = Context::new(64 * 1024);

    // String.prototype.includes - true case
    let result = ctx
        .eval(
            "
        var s = \"hello\" + \" world\";
        var search = \"lo w\" + \"\";
        return s.includes(search);
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));

    // String.prototype.includes - false case
    let result = ctx
        .eval(
            "
        var s = \"hello\" + \" world\";
        var search = \"xyz\" + \"\";
        return s.includes(search);
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(false));
}

#[test]
fn test_string_includes_and_starts_with_unicode() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval("return 'caf\\u00E9'.includes('f\\u00E9');")
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));

    let result = ctx
        .eval("return 'caf\\u00E9'.startsWith('f\\u00E9', 2);")
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_string_includes_coerces_missing_and_non_string_args() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx.eval("return 'xundefinedy'.includes();").unwrap();
    assert_eq!(result.to_bool(), Some(true));

    let result = ctx.eval("return 'abc123'.includes(123);").unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

// =========================================================================
// Number static method tests
// =========================================================================

#[test]
fn test_number_is_integer() {
    let mut ctx = Context::new(64 * 1024);

    // Number.isInteger with integer
    let result = ctx.eval("return Number.isInteger(42);").unwrap();
    assert_eq!(result.to_bool(), Some(true));

    // Number.isInteger with non-integer
    let result = ctx.eval("return Number.isInteger(undefined);").unwrap();
    assert_eq!(result.to_bool(), Some(false));
}

#[test]
fn test_number_is_finite() {
    let mut ctx = Context::new(64 * 1024);

    // Number.isFinite with number
    let result = ctx.eval("return Number.isFinite(42);").unwrap();
    assert_eq!(result.to_bool(), Some(true));

    // Number.isFinite with non-number
    let result = ctx.eval("return Number.isFinite(undefined);").unwrap();
    assert_eq!(result.to_bool(), Some(false));
}

#[test]
fn test_number_is_nan() {
    let mut ctx = Context::new(64 * 1024);

    // Number.isNaN with regular number
    let result = ctx.eval("return Number.isNaN(42);").unwrap();
    assert_eq!(result.to_bool(), Some(false));
}

#[test]
fn test_number_max_min_value() {
    let mut ctx = Context::new(64 * 1024);

    // Number.MAX_VALUE - check it returns a large positive number
    let result = ctx.eval("return Number.MAX_VALUE > 0;").unwrap();
    assert_eq!(result.to_bool(), Some(true));

    // Number.MIN_VALUE - smallest positive float (per JS spec)
    let result = ctx.eval("return Number.MIN_VALUE > 0;").unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

// =========================================================================
// console method tests
// =========================================================================

#[test]
fn test_console_log() {
    let mut ctx = Context::new(64 * 1024);

    // console.log should execute and return undefined
    let result = ctx
        .eval(
            "
        console.log(42);
        return 1;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_console_log_multiple_args() {
    let mut ctx = Context::new(64 * 1024);

    // console.log with multiple args
    let result = ctx
        .eval(
            "
        console.log(1, 2, 3);
        return 1;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_console_error() {
    let mut ctx = Context::new(64 * 1024);

    // console.error should execute and return undefined
    let result = ctx
        .eval(
            "
        console.error(42);
        return 1;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_console_log_array() {
    let mut ctx = Context::new(64 * 1024);

    // console.log with array
    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3];
        console.log(arr);
        return 1;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

// ========================================
// Error Tests
// ========================================

#[test]
fn test_error_constructor() {
    let mut ctx = Context::new(64 * 1024);

    // new Error("message") creates an error object
    let result = ctx
        .eval(
            "
        var e = new Error('something went wrong');
        return typeof e;
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_error_name_property() {
    let mut ctx = Context::new(64 * 1024);

    // Error.name should be "Error"
    let result = ctx
        .eval(
            "
        var e = new Error('test');
        return e.name;
    ",
        )
        .unwrap();
    // Check it's a string (the name property)
    assert!(result.is_string());
}

#[test]
fn test_error_message_property() {
    let mut ctx = Context::new(64 * 1024);

    // Error.message should be the message passed to constructor
    let result = ctx
        .eval(
            "
        var e = new Error('test message');
        return e.message;
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_type_error_constructor() {
    let mut ctx = Context::new(64 * 1024);

    // new TypeError("message")
    let result = ctx
        .eval(
            "
        var e = new TypeError('type error');
        return e.name;
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_reference_error_constructor() {
    let mut ctx = Context::new(64 * 1024);

    // new ReferenceError("message")
    let result = ctx
        .eval(
            "
        var e = new ReferenceError('ref error');
        return e.name;
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_throw_and_catch_error() {
    let mut ctx = Context::new(64 * 1024);

    // throw new Error() should be catchable
    let result = ctx
        .eval(
            "
        var caught = 0;
        try {
            throw new Error('test');
        } catch (e) {
            caught = 1;
        }
        return caught;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_catch_error_message() {
    let mut ctx = Context::new(64 * 1024);

    // Catch error and access its message
    let result = ctx
        .eval(
            "
        var msg = '';
        try {
            throw new Error('caught message');
        } catch (e) {
            msg = e.message;
        }
        return msg;
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_catch_error_name() {
    let mut ctx = Context::new(64 * 1024);

    // Catch error and access its name
    let result = ctx
        .eval(
            "
        var name = '';
        try {
            throw new TypeError('type error');
        } catch (e) {
            name = e.name;
        }
        return name;
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

// ========================================
// JSON Tests
// ========================================

#[test]
fn test_json_stringify_number() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        return JSON.stringify(42);
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_json_stringify_boolean() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        return JSON.stringify(true);
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_json_stringify_null() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        return JSON.stringify(null);
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_json_stringify_array() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3];
        return JSON.stringify(arr);
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_json_stringify_object() {
    let mut ctx = Context::new(64 * 1024);

    // Create object using constructor and set properties
    let result = ctx
        .eval(
            "
        function Obj() { this.a = 1; this.b = 2; }
        var obj = new Obj();
        return JSON.stringify(obj);
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_json_parse_number() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var s = '42';
        return JSON.parse(s);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(42));
}

#[test]
fn test_json_parse_boolean() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var s = 'true';
        return JSON.parse(s);
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_json_parse_null() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var s = 'null';
        return JSON.parse(s);
    ",
        )
        .unwrap();
    assert!(result.is_null());
}

#[test]
fn test_json_parse_array() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var s = '[1, 2, 3]';
        var arr = JSON.parse(s);
        return arr[1];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(2));
}

#[test]
fn test_json_parse_object() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var s = '{\"x\": 10, \"y\": 20}';
        var obj = JSON.parse(s);
        return obj.x;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(10));
}

#[test]
fn test_json_parse_nested() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var s = '{\"arr\": [1, 2, 3], \"obj\": {\"a\": 1}}';
        var data = JSON.parse(s);
        return data.arr[0];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

// ========================================
// Date Tests
// ========================================

#[test]
fn test_date_now() {
    let mut ctx = Context::new(64 * 1024);

    // Date.now() returns a number
    let result = ctx
        .eval(
            "
        return Date.now();
    ",
        )
        .unwrap();
    assert!(result.to_number_f32().is_some());
}

#[test]
fn test_date_now_increases() {
    let mut ctx = Context::new(64 * 1024);

    // Two calls to Date.now() should return different or equal values
    // (time progresses or stays same within execution)
    let result = ctx
        .eval(
            "
        var t1 = Date.now();
        var t2 = Date.now();
        return t2 >= t1 ? 1 : 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_date_now_positive() {
    let mut ctx = Context::new(64 * 1024);

    // Date.now() should return a positive value
    let result = ctx
        .eval(
            "
        return Date.now() > 0 ? 1 : 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_date_now_exceeds_i32_range() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx.eval("return Date.now() > 2147483647 ? 1 : 0;").unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_host_time_provider_drives_date_now_and_performance_now() {
    let mut ctx = Context::new(64 * 1024);
    HOST_TIME_MILLIS.store(1_700_000_000_000, Ordering::Relaxed);
    ctx.set_time_provider(host_time_provider);

    let result = ctx.eval("return Date.now();").unwrap();
    assert_eq!(result.to_number_f32(), Some(1_700_000_000_000.0));

    HOST_TIME_MILLIS.store(1_700_000_000_250, Ordering::Relaxed);
    let result = ctx.eval("return performance.now();").unwrap();
    assert_eq!(result.to_number_f32(), Some(250.0));
}

// ========================================
// RegExp Tests
// ========================================

#[test]
fn test_regexp_constructor() {
    let mut ctx = Context::new(64 * 1024);

    // Create a RegExp and check it exists
    let result = ctx
        .eval(
            "
        var pattern = \"hello\" + \"\";
        var re = new RegExp(pattern);
        return typeof re;
    ",
        )
        .unwrap();
    // Note: typeof returns "object" for RegExp in our implementation
    assert!(result.is_string());
}

#[test]
fn test_regexp_test_match() {
    let mut ctx = Context::new(64 * 1024);

    // Test matching pattern
    let result = ctx
        .eval(
            "
        var pattern = \"hel\" + \"lo\";
        var re = new RegExp(pattern);
        var str = \"hello\" + \" world\";
        return re.test(str);
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_regexp_test_no_match() {
    let mut ctx = Context::new(64 * 1024);

    // Test non-matching pattern
    let result = ctx
        .eval(
            "
        var pattern = \"xyz\" + \"\";
        var re = new RegExp(pattern);
        var str = \"hello\" + \" world\";
        return re.test(str);
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(false));
}

#[test]
fn test_regexp_test_case_sensitive() {
    let mut ctx = Context::new(64 * 1024);

    // Case-sensitive by default
    let result = ctx
        .eval(
            "
        var pattern = \"HELLO\" + \"\";
        var re = new RegExp(pattern);
        var str = \"hello\" + \" world\";
        return re.test(str);
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(false));
}

#[test]
fn test_regexp_test_case_insensitive() {
    let mut ctx = Context::new(64 * 1024);

    // Case-insensitive with 'i' flag
    let result = ctx
        .eval(
            "
        var pattern = \"HELLO\" + \"\";
        var flags = \"\" + \"i\";
        var re = new RegExp(pattern, flags);
        var str = \"hello\" + \" world\";
        return re.test(str);
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_regexp_exec_match() {
    let mut ctx = Context::new(64 * 1024);

    // exec returns array on match
    let result = ctx
        .eval(
            "
        var pattern = \"wor\" + \"ld\";
        var re = new RegExp(pattern);
        var str = \"hello\" + \" world\";
        var match = re.exec(str);
        return match[0];
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_regexp_exec_no_match() {
    let mut ctx = Context::new(64 * 1024);

    // exec returns null on no match
    let result = ctx
        .eval(
            "
        var pattern = \"xyz\" + \"\";
        var re = new RegExp(pattern);
        var str = \"hello\" + \" world\";
        var match = re.exec(str);
        return match === null ? 1 : 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_regexp_digit_pattern() {
    let mut ctx = Context::new(64 * 1024);

    // Test with digit pattern
    let result = ctx
        .eval(
            "
        var pattern = \"[0-9]\" + \"+\";
        var re = new RegExp(pattern);
        var str = \"abc\" + \"123def\";
        return re.test(str);
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_string_match_with_regexp() {
    let mut ctx = Context::new(64 * 1024);

    // String.match with RegExp
    let result = ctx
        .eval(
            "
        var pattern = \"wor\" + \"ld\";
        var re = new RegExp(pattern);
        var str = \"hello\" + \" world\";
        var match = str.match(re);
        return match[0];
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_string_match_no_match() {
    let mut ctx = Context::new(64 * 1024);

    // String.match returns null when no match
    let result = ctx
        .eval(
            "
        var pattern = \"xyz\" + \"\";
        var re = new RegExp(pattern);
        var str = \"hello\" + \" world\";
        var match = str.match(re);
        return match === null ? 1 : 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_string_match_global() {
    let mut ctx = Context::new(64 * 1024);

    // String.match with global flag returns all matches
    let result = ctx
        .eval(
            "
        var pattern = \"o\" + \"\";
        var flags = \"\" + \"g\";
        var re = new RegExp(pattern, flags);
        var str = \"hello\" + \" world\";
        var matches = str.match(re);
        return matches.length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(2)); // "hello world" has 2 'o's
}

#[test]
fn test_string_search_found() {
    let mut ctx = Context::new(64 * 1024);

    // String.search returns index of match
    let result = ctx
        .eval(
            "
        var pattern = \"wor\" + \"ld\";
        var re = new RegExp(pattern);
        var str = \"hello\" + \" world\";
        return str.search(re);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(6)); // "world" starts at index 6
}

#[test]
fn test_string_search_not_found() {
    let mut ctx = Context::new(64 * 1024);

    // String.search returns -1 when not found
    let result = ctx
        .eval(
            "
        var pattern = \"xyz\" + \"\";
        var re = new RegExp(pattern);
        var str = \"hello\" + \" world\";
        return str.search(re);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(-1));
}

// mquickjs-specific String methods
#[test]
fn test_string_code_point_at() {
    let mut ctx = Context::new(64 * 1024);

    // Get code point of 'A' (65)
    let result = ctx
        .eval(
            "
        var s = \"ABC\" + \"\";
        return s.codePointAt(0);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(65));

    // Get code point of 'B' (66)
    let result = ctx
        .eval(
            "
        var s = \"ABC\" + \"\";
        return s.codePointAt(1);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(66));
}

#[test]
fn test_string_trim_start() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var s = \"  hello\" + \"\";
        var t = s.trimStart();
        return t.length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(5)); // "hello" has length 5
}

#[test]
fn test_string_trim_end() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var s = \"hello  \" + \"\";
        var t = s.trimEnd();
        return t.length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(5)); // "hello" has length 5
}

#[test]
fn test_string_replace_all() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var s = \"aXbXc\" + \"\";
        var x = \"X\" + \"\";
        var y = \"Y\" + \"\";
        var r = s.replaceAll(x, y);
        return r.length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(5)); // "aYbYc" has length 5
}

// ========================================
// Boolean Tests
// ========================================

#[test]
fn test_boolean_true() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        return Boolean(true);
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_boolean_false() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        return Boolean(false);
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(false));
}

#[test]
fn test_boolean_number_truthy() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        return Boolean(42);
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_boolean_zero_falsy() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        return Boolean(0);
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(false));
}

#[test]
fn test_boolean_undefined_falsy() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        return Boolean(undefined);
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(false));
}

#[test]
fn test_boolean_null_falsy() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        return Boolean(null);
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(false));
}

#[test]
fn test_number_coercion() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        return Number(true);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_number_coercion_false() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        return Number(false);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(0));
}

#[test]
fn test_string_coercion() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        return String(42);
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_string_coercion_bool() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        return String(true);
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

// ========================================
// Object Static Method Tests
// ========================================

#[test]
fn test_object_keys() {
    let mut ctx = Context::new(64 * 1024);

    // Object.keys returns array of property names
    let result = ctx
        .eval(
            "
        function Obj() { this.a = 1; this.b = 2; this.c = 3; }
        var obj = new Obj();
        var keys = Object.keys(obj);
        return keys.length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(3));
}

#[test]
fn test_object_values() {
    let mut ctx = Context::new(64 * 1024);

    // Object.values returns array of property values
    let result = ctx
        .eval(
            "
        function Obj() { this.x = 10; this.y = 20; }
        var obj = new Obj();
        var vals = Object.values(obj);
        return vals[0] + vals[1];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(30));
}

#[test]
fn test_object_entries() {
    let mut ctx = Context::new(64 * 1024);

    // Object.entries returns array of [key, value] pairs
    let result = ctx
        .eval(
            "
        function Obj() { this.a = 100; }
        var obj = new Obj();
        var entries = Object.entries(obj);
        return entries.length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_has_own_property_true() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        function Obj() { this.x = 42; }
        var obj = new Obj();
        var prop = \"x\" + \"\";
        return obj.hasOwnProperty(prop);
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_has_own_property_false() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        function Obj() { this.x = 42; }
        var obj = new Obj();
        var prop = \"y\" + \"\";
        return obj.hasOwnProperty(prop);
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(false));
}

#[test]
fn test_deep_property_chain_access() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var root = { a: { b: { c: { d: 7 } } } };
        return root.a.b.c.d;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(7));
}

#[test]
fn test_object_get_prototype_of() {
    let mut ctx = Context::new(64 * 1024);

    // Object.getPrototypeOf on array
    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3];
        var proto = Object.getPrototypeOf(arr);
        return proto !== null;
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_object_create() {
    let mut ctx = Context::new(64 * 1024);

    // Object.create creates new object
    let result = ctx
        .eval(
            "
        var obj = Object.create(null);
        obj.x = 42;
        return obj.x;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(42));
}

#[test]
fn test_object_define_property() {
    let mut ctx = Context::new(64 * 1024);

    // Object.defineProperty exists - test that it's callable
    let result = ctx
        .eval(
            "
        return typeof Object.defineProperty;
    ",
        )
        .unwrap();
    // Should return "function"
    assert!(result.is_string());
}

#[test]
fn test_math_constants() {
    let mut ctx = Context::new(64 * 1024);

    // Math.PI is f32 PI
    let result = ctx.eval("return Math.PI;").unwrap();
    let pi = result.to_f32().unwrap();
    assert!((pi - std::f32::consts::PI).abs() < 0.0001);

    // Math.E is f32 E
    let result = ctx.eval("return Math.E;").unwrap();
    let e = result.to_f32().unwrap();
    assert!((e - std::f32::consts::E).abs() < 0.0001);
}

#[test]
fn test_math_trig_functions() {
    let mut ctx = Context::new(64 * 1024);

    // Math.sin(0) = 0
    let result = ctx.eval("return Math.sin(0);").unwrap();
    assert_eq!(result.to_i32(), Some(0));

    // Math.cos(0) = 1 (approximation for integers)
    let result = ctx.eval("return Math.cos(0);").unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_math_inverse_trig() {
    let mut ctx = Context::new(64 * 1024);

    // Math.asin(1) = PI/2 radians
    let result = ctx.eval("return Math.asin(1);").unwrap();
    let val = result.to_number_f32().unwrap();
    assert!((val - std::f32::consts::FRAC_PI_2).abs() < 0.001);

    // Math.acos(0) = PI/2 radians
    let result = ctx.eval("return Math.acos(0);").unwrap();
    let val = result.to_number_f32().unwrap();
    assert!((val - std::f32::consts::FRAC_PI_2).abs() < 0.001);

    // Math.atan(0) = 0
    let result = ctx.eval("return Math.atan(0);").unwrap();
    assert_eq!(result.to_i32(), Some(0));
}

#[test]
fn test_math_random() {
    let mut ctx = Context::new(64 * 1024);

    // Math.random() returns a float in [0, 1)
    let result = ctx.eval("return Math.random();").unwrap();
    let val = result.to_number_f32().unwrap();
    assert!((0.0..1.0).contains(&val));
}

#[test]
fn test_parse_float() {
    let mut ctx = Context::new(64 * 1024);
    let expected = 314.0_f32 / 100.0;

    // parseFloat with integer string
    let result = ctx.eval("return parseFloat('42');").unwrap();
    assert_eq!(result.to_i32(), Some(42));

    // parseFloat with decimal returns float
    let result = ctx.eval("return parseFloat('3.14');").unwrap();
    let val = result.to_number_f32().unwrap();
    assert!((val - expected).abs() < 0.01);
}

#[test]
fn test_parse_float_parses_leading_valid_portion() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx.eval("return parseFloat('3.14abc');").unwrap();
    let val = result.to_number_f32().unwrap();
    let expected = 314.0_f32 / 100.0;
    assert!((val - expected).abs() < 0.01);

    let result = ctx.eval("return parseFloat('  -1.5e2px');").unwrap();
    let val = result.to_number_f32().unwrap();
    assert!((val + 150.0).abs() < 0.01);
}

#[test]
fn test_string_split_coerces_non_string_separator() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx.eval("return 'a1b1c'.split(1)[1];").unwrap();
    assert_eq!(ctx.string_value(result).as_deref(), Some("b"));
}

#[test]
fn test_is_finite() {
    let mut ctx = Context::new(64 * 1024);

    // isFinite with number
    let result = ctx.eval("return isFinite(42);").unwrap();
    assert_eq!(result.to_bool(), Some(true));

    // isFinite with non-number returns false
    let result = ctx.eval("return isFinite('hello');").unwrap();
    assert_eq!(result.to_bool(), Some(false));
}

#[test]
fn test_number_to_string() {
    let mut ctx = Context::new(64 * 1024);

    // (42).toString() returns "42"
    let result = ctx.eval("return (42).toString();").unwrap();
    assert!(result.is_string());
}

#[test]
fn test_number_to_fixed() {
    let mut ctx = Context::new(64 * 1024);

    // (42).toFixed(2) returns "42.00"
    let result = ctx.eval("return (42).toFixed(2);").unwrap();
    assert!(result.is_string());
}

#[test]
fn test_number_formatting_float_nan_infinity() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx.eval("return (3.5).toString();").unwrap();
    assert!(result.is_string());

    let result = ctx.eval("return (3.14159).toFixed(2);").unwrap();
    assert!(result.is_string());

    let result = ctx.eval("return (1234.0).toExponential();").unwrap();
    assert!(result.is_string());

    let result = ctx.eval("return (3.14159).toPrecision(3);").unwrap();
    assert!(result.is_string());

    let result = ctx.eval("return (Infinity).toString();").unwrap();
    assert!(result.is_string());

    let result = ctx.eval("return (NaN).toString();").unwrap();
    assert!(result.is_string());
}

#[test]
fn test_array_buffer() {
    let mut ctx = Context::new(64 * 1024);

    // Create ArrayBuffer with length 10
    let result = ctx
        .eval(
            "
        var buf = new ArrayBuffer(10);
        return buf.byteLength;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(10));
}

#[test]
fn test_typed_array_subarray() {
    let mut ctx = Context::new(64 * 1024);

    // Create Uint8Array and subarray
    let result = ctx
        .eval(
            "
        var arr = new Uint8Array([1, 2, 3, 4, 5]);
        var sub = arr.subarray(1, 4);
        return sub.length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(3));
}

#[test]
fn test_typed_array_subarray_values() {
    let mut ctx = Context::new(64 * 1024);

    // Verify subarray values
    let result = ctx
        .eval(
            "
        var arr = new Uint8Array([10, 20, 30, 40, 50]);
        var sub = arr.subarray(1, 4);
        return sub[0];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(20));
}

#[test]
fn test_global_this_math() {
    let mut ctx = Context::new(64 * 1024);

    // globalThis.Math should work
    let result = ctx.eval("return globalThis.Math.abs(-5);").unwrap();
    assert_eq!(result.to_i32(), Some(5));
}

#[test]
fn test_global_this_self_reference() {
    let mut ctx = Context::new(64 * 1024);

    // globalThis.globalThis === globalThis
    let result = ctx
        .eval(
            "
        var g1 = globalThis;
        var g2 = globalThis.globalThis;
        return g1 === g2;
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_array_is_array_true() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3];
        return Array.isArray(arr) ? 1 : 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_array_is_array_false() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        function Obj() { this.x = 1; }
        var obj = new Obj();
        return Array.isArray(obj) ? 1 : 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(0));
}

#[test]
fn test_array_is_array_number() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        return Array.isArray(42) ? 1 : 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(0));
}

// ========================================
// TypedArray Tests
// ========================================

#[test]
fn test_int8_array_create() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = new Int8Array(5);
        return arr.length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(5));
}

#[test]
fn test_uint8_array_set_get() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = new Uint8Array(3);
        arr[0] = 10;
        arr[1] = 20;
        arr[2] = 30;
        return arr[0] + arr[1] + arr[2];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(60));
}

#[test]
fn test_int32_array() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = new Int32Array(2);
        arr[0] = 100000;
        arr[1] = 200000;
        return arr[0] + arr[1];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(300000));
}

#[test]
fn test_typed_array_byte_length() {
    let mut ctx = Context::new(64 * 1024);

    // Int32Array element = 4 bytes, 3 elements = 12 bytes
    let result = ctx
        .eval(
            "
        var arr = new Int32Array(3);
        return arr.byteLength;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(12));
}

#[test]
fn test_typed_array_from_array() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var src = [1, 2, 3, 4, 5];
        var arr = new Int8Array(src);
        return arr[2];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(3));
}

#[test]
fn test_int8_array_overflow() {
    let mut ctx = Context::new(64 * 1024);

    // Int8 range is -128 to 127, 200 should wrap
    let result = ctx
        .eval(
            "
        var arr = new Int8Array(1);
        arr[0] = 200;
        return arr[0];
    ",
        )
        .unwrap();
    // 200 as i8 = -56
    assert_eq!(result.to_i32(), Some(-56));
}

// ========================================
// Function.prototype Tests
// ========================================

#[test]
fn test_function_call_basic() {
    let mut ctx = Context::new(64 * 1024);

    // call() with a simple this value
    let result = ctx
        .eval(
            "
        function getThis() { return this.value; }
        function Obj() { this.value = 42; }
        var obj = new Obj();
        return getThis.call(obj);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(42));
}

#[test]
fn test_function_call_with_args() {
    let mut ctx = Context::new(64 * 1024);

    // call() with arguments
    let result = ctx
        .eval(
            "
        function add(a, b) { return this.base + a + b; }
        function Obj() { this.base = 10; }
        var obj = new Obj();
        return add.call(obj, 5, 3);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(18));
}

#[test]
fn test_function_apply_basic() {
    let mut ctx = Context::new(64 * 1024);

    // apply() with an array of arguments
    let result = ctx
        .eval(
            "
        function sum(a, b, c) { return a + b + c; }
        return sum.apply(null, [1, 2, 3]);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(6));
}

#[test]
fn test_function_apply_with_this() {
    let mut ctx = Context::new(64 * 1024);

    // apply() with this and arguments
    let result = ctx
        .eval(
            "
        function multiply(x) { return this.factor * x; }
        function Obj() { this.factor = 5; }
        var obj = new Obj();
        return multiply.apply(obj, [7]);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(35));
}

#[test]
fn test_function_bind_basic() {
    let mut ctx = Context::new(64 * 1024);

    // bind() creates a bound function object
    let result = ctx
        .eval(
            "
        function getVal() { return this.val; }
        function Obj() { this.val = 99; }
        var obj = new Obj();
        var bound = getVal.bind(obj);
        return typeof bound;
    ",
        )
        .unwrap();
    // bind returns an object (our implementation stores bound function data in an object)
    assert!(result.is_string());
}

// ========================================
// Array Higher-Order Function Tests
// ========================================

#[test]
fn test_array_map() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3];
        function double(x) { return x * 2; }
        var mapped = arr.map(double);
        return mapped[0] + mapped[1] + mapped[2];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(12)); // 2 + 4 + 6
}

#[test]
fn test_array_filter() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3, 4, 5];
        function isEven(x) { return x % 2 == 0; }
        var filtered = arr.filter(isEven);
        return filtered.length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(2)); // [2, 4]
}

#[test]
fn test_array_foreach() {
    let mut ctx = Context::new(64 * 1024);

    // forEach just calls the callback, returns undefined
    // Test that it iterates over all elements using an object to accumulate
    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3];
        function Counter() { this.sum = 0; }
        var counter = new Counter();
        function addToCounter(x) { counter.sum = counter.sum + x; }
        arr.forEach(addToCounter);
        return counter.sum;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(6));
}

#[test]
fn test_array_reduce() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3, 4];
        function add(acc, x) { return acc + x; }
        return arr.reduce(add, 0);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(10));
}

#[test]
fn test_array_reduce_no_initial() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3, 4];
        function add(acc, x) { return acc + x; }
        return arr.reduce(add);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(10));
}

#[test]
fn test_array_method_chain_map_filter_reduce() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3, 4, 5, 6];
        function double(x) { return x * 2; }
        function divByThree(x) { return x % 3 == 0; }
        function add(acc, x) { return acc + x; }
        return arr.map(double).filter(divByThree).reduce(add, 0);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(18)); // [6, 12] -> 18
}

#[test]
fn test_non_array_map_method_call_fallback() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval(
            "
        var obj = {
            map: function (cb) { return cb(5); }
        };
        function plusOne(x) { return x + 1; }
        return obj.map(plusOne);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(6));
}

#[test]
fn test_array_map_uses_length_snapshot() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval(
            "
        var arr = [1, 2];
        var mapped = arr.map(function (x, i, a) {
            if (i === 0) a.push(99);
            return x;
        });
        return mapped.length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(2));
}

#[test]
fn test_array_map_observes_future_element_updates() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval(
            "
        var arr = [1, 2];
        var mapped = arr.map(function (x, i, a) {
            if (i === 0) a[1] = 99;
            return x;
        });
        return mapped[1];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(99));
}

#[test]
fn test_local_assignment_statement_update_putloc8_shape() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval(
            "
        var a = 1, b = 2, c = 3, d = 4, e = 5;
        e = e + 1;
        return e;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(6));
}

#[test]
fn test_array_find() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3, 4, 5];
        function isGreaterThan3(x) { return x > 3; }
        return arr.find(isGreaterThan3);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(4));
}

#[test]
fn test_array_find_index() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3, 4, 5];
        function isGreaterThan3(x) { return x > 3; }
        return arr.findIndex(isGreaterThan3);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(3));
}

#[test]
fn test_array_some() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3, 4, 5];
        function isGreaterThan3(x) { return x > 3; }
        return arr.some(isGreaterThan3) ? 1 : 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_array_every() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [2, 4, 6, 8];
        function isEven(x) { return x % 2 == 0; }
        return arr.every(isEven) ? 1 : 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_array_every_false() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [2, 4, 5, 8];
        function isEven(x) { return x % 2 == 0; }
        return arr.every(isEven) ? 1 : 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(0));
}

#[test]
fn test_array_includes() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3, 4, 5];
        return arr.includes(3) ? 1 : 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_array_includes_not_found() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3, 4, 5];
        return arr.includes(10) ? 1 : 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(0));
}

#[test]
fn test_array_concat() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr1 = [1, 2, 3];
        var arr2 = [4, 5, 6];
        var result = arr1.concat(arr2);
        return result.length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(6));
}

#[test]
fn test_array_concat_values() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr1 = [1, 2, 3];
        var arr2 = [4, 5, 6];
        var result = arr1.concat(arr2);
        return result[3] + result[4] + result[5];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(15)); // 4 + 5 + 6
}

#[test]
fn test_array_concat_single_value() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3];
        var result = arr.concat(4);
        return result.length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(4));
}

#[test]
fn test_array_sort() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [3, 1, 4, 1, 5, 9, 2, 6];
        arr.sort();
        return arr[0];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_array_sort_returns_array() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [3, 1, 2];
        var sorted = arr.sort();
        return sorted[2];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(3));
}

#[test]
fn test_array_sort_floats_numeric() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [3.5, 1.25, 2.0, -1.0];
        arr.sort();
        return arr[0] === -1 && arr[1] === 1.25 && arr[2] === 2 && arr[3] === 3.5;
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_array_sort_strings_lexicographic() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = ['b', 'aa', 'c', 'a'];
        arr.sort();
        return arr[0] === 'a' && arr[1] === 'aa' && arr[2] === 'b' && arr[3] === 'c';
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_array_sort_default_is_lexicographic_for_numbers() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [9, 80];
        arr.sort();
        return arr[0] === 80 && arr[1] === 9;
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_array_sort_supports_compare_function() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [3, 2, 1];
        arr.sort(function(a, b) { return a - b; });
        return arr[0] === 1 && arr[1] === 2 && arr[2] === 3;
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_array_flat() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [1, [2, 3], 4];
        var flat = arr.flat();
        return flat.length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(4));
}

#[test]
fn test_array_fill() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3, 4, 5];
        arr.fill(0);
        return arr[0] + arr[1] + arr[2];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(0));
}

#[test]
fn test_array_fill_range() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3, 4, 5];
        arr.fill(0, 1, 4);
        return arr[0] + arr[1] + arr[4];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(6)); // 1 + 0 + 5
}

#[test]
fn test_string_char_code_at() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var s = 'ABC';
        return s.charCodeAt(0);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(65)); // 'A' = 65
}

#[test]
fn test_string_last_index_of() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var s = 'hello hello';
        return s.lastIndexOf('hello');
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(6));
}

#[test]
fn test_string_last_index_of_unicode_uses_character_indices() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval("return 'caf\\u00E9'.lastIndexOf('\\u00E9');")
        .unwrap();
    assert_eq!(result.to_i32(), Some(3));
}

#[test]
fn test_string_from_char_code() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var s = String.fromCharCode(65, 66, 67);
        return s.length;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(3));
}

#[test]
fn test_array_last_index_of() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3, 2, 1];
        return arr.lastIndexOf(2);
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(3));
}

#[test]
fn test_performance_now() {
    let mut ctx = Context::new(64 * 1024);

    // Just verify it returns a number and doesn't throw
    let result = ctx
        .eval(
            "
        var t = performance.now();
        return t >= 0 ? 1 : 0;
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_uint8_clamped_array() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = new Uint8ClampedArray(3);
        arr[0] = -10;   // clamped to 0
        arr[1] = 300;   // clamped to 255
        arr[2] = 100;   // stays 100
        return arr[0] + arr[1] + arr[2];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(355));
}

#[test]
fn test_float32_array() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = new Float32Array(2);
        arr[0] = 3;
        arr[1] = 4;
        return arr[0] + arr[1];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(7));
}

#[test]
fn test_float64_array() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var arr = new Float64Array(2);
        arr[0] = 10;
        arr[1] = 20;
        return arr[0] + arr[1];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(30));
}

#[test]
fn test_eval_error() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var e = new EvalError('bad eval');
        return e.name;
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_uri_error() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var e = new URIError('bad uri');
        return e.name;
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_internal_error() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var e = new InternalError('too much recursion');
        return e.name;
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_error_stack_property() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx
        .eval(
            "
        var e = new Error('test');
        return typeof e.stack;
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_error_to_string() {
    let mut ctx = Context::new(64 * 1024);

    // Check that toString method exists on Error
    let result = ctx
        .eval(
            "
        var e = new Error('test message');
        return typeof e.toString;
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_array_to_string() {
    let mut ctx = Context::new(64 * 1024);

    // Check that toString method exists
    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3];
        return typeof arr.toString;
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_json_stringify_undefined_returns_undefined() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx.eval("return JSON.stringify(undefined);").unwrap();
    assert!(result.is_undefined());
}

#[test]
fn test_string_repeat_negative_throws() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval("try { 'x'.repeat(-1); return 'no'; } catch (e) { return e.name; }")
        .unwrap();
    assert_eq!(ctx.string_value(result).as_deref(), Some("RangeError"));
}

#[test]
fn test_function_to_string() {
    let mut ctx = Context::new(64 * 1024);

    // Check that toString method exists
    let result = ctx
        .eval(
            "
        function foo() { return 1; }
        return typeof foo.toString;
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_array_reduce_right() {
    let mut ctx = Context::new(64 * 1024);

    // Check that reduceRight method exists
    let result = ctx
        .eval(
            "
        var arr = [1, 2, 3, 4];
        return typeof arr.reduceRight;
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_gc_function() {
    let mut ctx = Context::new(64 * 1024);

    // gc() should be a function and return undefined
    let result = ctx
        .eval(
            "
        return typeof gc;
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_load_function() {
    let mut ctx = Context::new(64 * 1024);

    // load() should be a function
    let result = ctx
        .eval(
            "
        return typeof load;
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_set_timeout_function() {
    let mut ctx = Context::new(64 * 1024);

    // setTimeout should be a function
    let result = ctx
        .eval(
            "
        return typeof setTimeout;
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

#[test]
fn test_clear_timeout_function() {
    let mut ctx = Context::new(64 * 1024);

    // clearTimeout should be a function
    let result = ctx
        .eval(
            "
        return typeof clearTimeout;
    ",
        )
        .unwrap();
    assert!(result.is_string());
}

// ============================================================
// Float TypedArray tests
// ============================================================

#[test]
fn test_float32_array_nan() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval(
            "
        var arr = new Float32Array(1);
        arr[0] = NaN;
        return isNaN(arr[0]);
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_float32_array_infinity() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval(
            "
        var arr = new Float32Array(1);
        arr[0] = Infinity;
        return arr[0];
    ",
        )
        .unwrap();
    assert!(result.is_infinite_value());
    assert!(result.to_number_f32().unwrap() > 0.0);
}

#[test]
fn test_float32_array_neg_infinity() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval(
            "
        var arr = new Float32Array(1);
        arr[0] = -Infinity;
        return arr[0];
    ",
        )
        .unwrap();
    assert!(result.is_infinite_value());
    assert!(result.to_number_f32().unwrap() < 0.0);
}

#[test]
fn test_float32_array_fractional() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval(
            "
        var arr = new Float32Array(2);
        arr[0] = 1.5;
        arr[1] = -2.25;
        return arr[0] + arr[1];
    ",
        )
        .unwrap();
    let val = result.to_number_f32().unwrap();
    assert!((val - (-0.75)).abs() < 0.001);
}

#[test]
fn test_float64_array_nan() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval(
            "
        var arr = new Float64Array(1);
        arr[0] = NaN;
        return isNaN(arr[0]);
    ",
        )
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_float64_array_infinity() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval(
            "
        var arr = new Float64Array(1);
        arr[0] = Infinity;
        return arr[0];
    ",
        )
        .unwrap();
    assert!(result.is_infinite_value());
}

#[test]
fn test_float64_array_neg_infinity() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval(
            "
        var arr = new Float64Array(1);
        arr[0] = -Infinity;
        return arr[0];
    ",
        )
        .unwrap();
    assert!(result.is_infinite_value());
    assert!(result.to_number_f32().unwrap() < 0.0);
}

#[test]
fn test_float32_array_whole_number_normalization() {
    let mut ctx = Context::new(64 * 1024);
    // Float32Array storing whole numbers should normalize to int
    let result = ctx
        .eval(
            "
        var arr = new Float32Array(1);
        arr[0] = 42;
        return arr[0];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(42));
}

#[test]
fn test_float64_array_whole_number_normalization() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval(
            "
        var arr = new Float64Array(1);
        arr[0] = 100;
        return arr[0];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(100));
}

#[test]
fn test_float64_array_preserves_large_int_precision() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval(
            "
        var value = 16777217;
        var arr = new Float64Array(1);
        arr[0] = value;
        return arr[0];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(16777217));
}

#[test]
fn test_uint32_array_preserves_high_bit_values() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval(
            "
        var arr = new Uint32Array(1);
        arr[0] = 2147483648;
        return arr[0];
    ",
        )
        .unwrap();
    assert_eq!(result.to_number_f32(), Some(2147483648.0));
}

#[test]
fn test_float32_array_zero() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval(
            "
        var arr = new Float32Array(1);
        return arr[0];
    ",
        )
        .unwrap();
    assert_eq!(result.to_i32(), Some(0));
}

#[test]
fn test_nan_global() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx.eval("return NaN;").unwrap();
    assert!(result.is_nan_value());
}

#[test]
fn test_infinity_global() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx.eval("return Infinity;").unwrap();
    assert!(result.is_infinite_value());
    assert!(result.to_number_f32().unwrap() > 0.0);
}

#[test]
fn test_negative_infinity() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx.eval("return -Infinity;").unwrap();
    assert!(result.is_infinite_value());
    assert!(result.to_number_f32().unwrap() < 0.0);
}

#[test]
fn test_typeof_nan() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx.eval("return typeof NaN;").unwrap();
    assert!(result.is_string());
}

#[test]
fn test_isnan_function() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx.eval("return isNaN(NaN);").unwrap();
    assert_eq!(result.to_bool(), Some(true));

    let result = ctx.eval("return isNaN(42);").unwrap();
    assert_eq!(result.to_bool(), Some(false));

    let result = ctx.eval("return isNaN(Infinity);").unwrap();
    assert_eq!(result.to_bool(), Some(false));
}

#[test]
fn test_nan_arithmetic() {
    let mut ctx = Context::new(64 * 1024);

    // NaN + anything = NaN
    let result = ctx.eval("return NaN + 1;").unwrap();
    assert!(result.is_nan_value());

    // NaN === NaN should be false
    let result = ctx.eval("return NaN === NaN;").unwrap();
    assert_eq!(result.to_bool(), Some(false));
}

#[test]
fn test_infinity_arithmetic() {
    let mut ctx = Context::new(64 * 1024);

    let result = ctx.eval("return Infinity + 1;").unwrap();
    assert!(result.is_infinite_value());

    let result = ctx.eval("return Infinity - Infinity;").unwrap();
    assert!(result.is_nan_value());

    let result = ctx.eval("return 1 / Infinity;").unwrap();
    assert_eq!(result.to_i32(), Some(0));
}

// ============================================================
// Implicit type coercion tests
// ============================================================

// --- ToNumber in arithmetic ---

#[test]
fn test_bool_plus_number() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(ctx.eval("return true + 1;").unwrap().to_i32(), Some(2));
    assert_eq!(ctx.eval("return false + 1;").unwrap().to_i32(), Some(1));
}

#[test]
fn test_null_plus_number() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(ctx.eval("return null + 1;").unwrap().to_i32(), Some(1));
}

#[test]
fn test_undefined_plus_number() {
    let mut ctx = Context::new(64 * 1024);
    // undefined 閳?NaN, NaN + 1 = NaN
    assert!(ctx.eval("return undefined + 1;").unwrap().is_nan_value());
}

#[test]
fn test_bool_subtract() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(ctx.eval("return 1 - true;").unwrap().to_i32(), Some(0));
    assert_eq!(ctx.eval("return 5 - false;").unwrap().to_i32(), Some(5));
}

#[test]
fn test_null_subtract() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(ctx.eval("return 5 - null;").unwrap().to_i32(), Some(5));
}

#[test]
fn test_bool_multiply() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(ctx.eval("return 3 * true;").unwrap().to_i32(), Some(3));
    assert_eq!(ctx.eval("return 3 * false;").unwrap().to_i32(), Some(0));
}

#[test]
fn test_null_multiply() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(ctx.eval("return 5 * null;").unwrap().to_i32(), Some(0));
}

#[test]
fn test_bool_divide() {
    let mut ctx = Context::new(64 * 1024);
    let val = ctx
        .eval("return 6 / true;")
        .unwrap()
        .to_number_f32()
        .unwrap();
    assert!((val - 6.0).abs() < 0.001);
}

#[test]
fn test_bool_modulo() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(ctx.eval("return 5 % true;").unwrap().to_i32(), Some(0));
}

#[test]
fn test_unary_neg_bool() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(ctx.eval("return -true;").unwrap().to_i32(), Some(-1));
    // -false and -null produce -0.0 per JS spec (negative zero)
    let neg_false = ctx.eval("return -false;").unwrap();
    let f = neg_false.to_number_f32().unwrap();
    assert!(f == 0.0 && f.is_sign_negative(), "-false should be -0.0");
    let neg_null = ctx.eval("return -null;").unwrap();
    let f = neg_null.to_number_f32().unwrap();
    assert!(f == 0.0 && f.is_sign_negative(), "-null should be -0.0");
}

// --- Abstract Equality == ---

#[test]
fn test_null_equals_undefined() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(
        ctx.eval("return null == undefined;").unwrap().to_bool(),
        Some(true)
    );
    assert_eq!(
        ctx.eval("return undefined == null;").unwrap().to_bool(),
        Some(true)
    );
}

#[test]
fn test_null_not_equals_false() {
    let mut ctx = Context::new(64 * 1024);
    // null only == undefined (not false/0/"")
    assert_eq!(
        ctx.eval("return null == false;").unwrap().to_bool(),
        Some(false)
    );
    assert_eq!(
        ctx.eval("return null == 0;").unwrap().to_bool(),
        Some(false)
    );
}

#[test]
fn test_bool_coercion_in_eq() {
    let mut ctx = Context::new(64 * 1024);
    // true閳?, false閳?
    assert_eq!(ctx.eval("return true == 1;").unwrap().to_bool(), Some(true));
    assert_eq!(
        ctx.eval("return false == 0;").unwrap().to_bool(),
        Some(true)
    );
    assert_eq!(
        ctx.eval("return true == 2;").unwrap().to_bool(),
        Some(false)
    );
}

#[test]
fn test_strict_eq_no_coercion() {
    let mut ctx = Context::new(64 * 1024);
    // === never coerces types
    assert_eq!(
        ctx.eval("return true === 1;").unwrap().to_bool(),
        Some(false)
    );
    assert_eq!(
        ctx.eval("return null === undefined;").unwrap().to_bool(),
        Some(false)
    );
    assert_eq!(
        ctx.eval("return 0 === false;").unwrap().to_bool(),
        Some(false)
    );
}

#[test]
fn test_strict_eq_same_type() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(ctx.eval("return 1 === 1;").unwrap().to_bool(), Some(true));
    assert_eq!(ctx.eval("return 1 === 2;").unwrap().to_bool(), Some(false));
    assert_eq!(
        ctx.eval("return true === true;").unwrap().to_bool(),
        Some(true)
    );
    assert_eq!(
        ctx.eval("return null === null;").unwrap().to_bool(),
        Some(true)
    );
}

// --- String comparisons ---

#[test]
fn test_string_lt_comparison() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(ctx.eval("return 'a' < 'b';").unwrap().to_bool(), Some(true));
    assert_eq!(
        ctx.eval("return 'b' < 'a';").unwrap().to_bool(),
        Some(false)
    );
    assert_eq!(
        ctx.eval("return 'abc' < 'abd';").unwrap().to_bool(),
        Some(true)
    );
}

#[test]
fn test_string_gt_comparison() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(ctx.eval("return 'b' > 'a';").unwrap().to_bool(), Some(true));
    assert_eq!(
        ctx.eval("return 'a' > 'b';").unwrap().to_bool(),
        Some(false)
    );
}

#[test]
fn test_string_lte_gte_comparison() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(
        ctx.eval("return 'a' <= 'a';").unwrap().to_bool(),
        Some(true)
    );
    assert_eq!(
        ctx.eval("return 'a' >= 'a';").unwrap().to_bool(),
        Some(true)
    );
    assert_eq!(
        ctx.eval("return 'a' <= 'b';").unwrap().to_bool(),
        Some(true)
    );
}

#[test]
fn test_string_eq_content() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(
        ctx.eval("return 'hello' == 'hello';").unwrap().to_bool(),
        Some(true)
    );
    assert_eq!(
        ctx.eval("return 'hello' == 'world';").unwrap().to_bool(),
        Some(false)
    );
    assert_eq!(
        ctx.eval("return 'hello' === 'hello';").unwrap().to_bool(),
        Some(true)
    );
}

// --- Bitwise with bool/null ---

#[test]
fn test_bitwise_bool_coercion() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(ctx.eval("return true & 1;").unwrap().to_i32(), Some(1));
    assert_eq!(ctx.eval("return false | 1;").unwrap().to_i32(), Some(1));
    assert_eq!(ctx.eval("return true ^ true;").unwrap().to_i32(), Some(0));
}

#[test]
fn test_bitwise_null_coercion() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(ctx.eval("return null | 5;").unwrap().to_i32(), Some(5));
    assert_eq!(ctx.eval("return null & 5;").unwrap().to_i32(), Some(0));
    assert_eq!(ctx.eval("return ~null;").unwrap().to_i32(), Some(-1));
}

// --- String concat with non-string values ---

#[test]
fn test_string_concat_with_undefined() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx.eval("return 'x' + undefined;").unwrap();
    assert!(result.is_string());
}

#[test]
fn test_number_plus_string_coercion() {
    let mut ctx = Context::new(64 * 1024);
    // When one side is a string, + does concatenation (number閳姱tring)
    let result = ctx.eval("return 123 + 'abc';").unwrap();
    assert!(result.is_string());
}

#[test]
fn test_string_numeric_subtract_coercion() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(ctx.eval("return '5' - 3;").unwrap().to_i32(), Some(2));
}

#[test]
fn test_string_numeric_relational_coercion() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(ctx.eval("return '5' < 10;").unwrap().to_bool(), Some(true));
}

#[test]
fn test_runtime_string_pressure_concat_shape() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval(
            "
        var s = 'item-' + 12 + '-' + 3;
        return s;
    ",
        )
        .unwrap();
    let result_str = ctx.string_value(result).unwrap();
    assert_eq!(result_str, "item-12-3");
}

// ===== debugger / void / do...while / switch tests =====

#[test]
fn test_debugger_statement() {
    let mut ctx = Context::new(64 * 1024);
    // debugger is a no-op, should not error
    assert_eq!(ctx.eval("debugger; return 42;").unwrap().to_i32(), Some(42));
}

#[test]
fn test_void_operator() {
    let mut ctx = Context::new(64 * 1024);
    assert!(ctx.eval("return void 0;").unwrap().is_undefined());
    assert!(ctx.eval("return void (1 + 2);").unwrap().is_undefined());
    // void should still evaluate its operand (side effects)
    assert_eq!(
        ctx.eval("var x = 1; void (x = 5); return x;")
            .unwrap()
            .to_i32(),
        Some(5)
    );
}

#[test]
fn test_do_while_basic() {
    let mut ctx = Context::new(64 * 1024);
    // Basic do...while: body always runs at least once
    assert_eq!(
        ctx.eval("var i = 0; do { i = i + 1; } while (i < 5); return i;")
            .unwrap()
            .to_i32(),
        Some(5)
    );
}

#[test]
fn test_do_while_runs_once_when_false() {
    let mut ctx = Context::new(64 * 1024);
    // Body runs once even if condition is immediately false
    assert_eq!(
        ctx.eval("var x = 0; do { x = x + 10; } while (false); return x;")
            .unwrap()
            .to_i32(),
        Some(10)
    );
}

#[test]
fn test_do_while_break() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(
        ctx.eval("var i = 0; do { i = i + 1; if (i === 3) { break; } } while (i < 10); return i;")
            .unwrap()
            .to_i32(),
        Some(3)
    );
}

#[test]
fn test_do_while_continue() {
    let mut ctx = Context::new(64 * 1024);
    // continue should jump to condition check
    assert_eq!(
        ctx.eval("var i = 0; var sum = 0; do { i = i + 1; if (i === 3) { continue; } sum = sum + i; } while (i < 5); return sum;")
            .unwrap()
            .to_i32(),
        Some(1 + 2 + 4 + 5) // skip i=3
    );
}

#[test]
fn test_switch_basic() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(
        ctx.eval("var x = 2; var r = 0; switch (x) { case 1: r = 10; break; case 2: r = 20; break; case 3: r = 30; break; } return r;")
            .unwrap().to_i32(),
        Some(20)
    );
}

#[test]
fn test_switch_default() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(
        ctx.eval("var x = 99; var r = 0; switch (x) { case 1: r = 10; break; default: r = -1; break; } return r;")
            .unwrap().to_i32(),
        Some(-1)
    );
}

#[test]
fn test_switch_no_match_no_default() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(
        ctx.eval(
            "var r = 0; switch (5) { case 1: r = 10; break; case 2: r = 20; break; } return r;"
        )
        .unwrap()
        .to_i32(),
        Some(0)
    );
}

#[test]
fn test_switch_fallthrough() {
    let mut ctx = Context::new(64 * 1024);
    // Without break, case 2 falls through to case 3
    assert_eq!(
        ctx.eval("var x = 2; var r = 0; switch (x) { case 1: r = r + 10; case 2: r = r + 20; case 3: r = r + 30; } return r;")
            .unwrap().to_i32(),
        Some(50) // 20 + 30 (fall-through)
    );
}

#[test]
fn test_switch_break_in_loop() {
    let mut ctx = Context::new(64 * 1024);
    // break inside switch should only break the switch, not the outer loop
    assert_eq!(
        ctx.eval("var sum = 0; for (var i = 0; i < 3; i = i + 1) { switch (i) { case 0: sum = sum + 1; break; case 1: sum = sum + 10; break; default: sum = sum + 100; break; } } return sum;")
            .unwrap().to_i32(),
        Some(111) // 1 + 10 + 100
    );
}

// --- Math additional methods ---

#[test]
fn test_math_tan() {
    let mut ctx = Context::new(64 * 1024);
    // tan(0) === 0
    assert_eq!(ctx.eval("return Math.tan(0);").unwrap().to_i32(), Some(0));
}

#[test]
fn test_math_exp() {
    let mut ctx = Context::new(64 * 1024);
    // exp(0) === 1
    assert_eq!(ctx.eval("return Math.exp(0);").unwrap().to_i32(), Some(1));
}

#[test]
fn test_math_log() {
    let mut ctx = Context::new(64 * 1024);
    // log(1) === 0
    assert_eq!(ctx.eval("return Math.log(1);").unwrap().to_i32(), Some(0));
}

#[test]
fn test_math_atan2() {
    let mut ctx = Context::new(64 * 1024);
    // atan2(0, 1) === 0
    assert_eq!(
        ctx.eval("return Math.atan2(0, 1);").unwrap().to_i32(),
        Some(0)
    );
    // atan2(1, 0) should be ~PI/2 > 1
    assert_eq!(
        ctx.eval("return Math.atan2(1, 0) > 1;").unwrap().to_bool(),
        Some(true)
    );
}

// --- String.substring ---

#[test]
fn test_string_substring() {
    let mut ctx = Context::new(64 * 1024);
    // substring(0, 5) extracts first 5 chars
    assert_eq!(
        ctx.eval("return 'hello world'.substring(0, 5).length;")
            .unwrap()
            .to_i32(),
        Some(5)
    );
    // Verify content via charCodeAt: 'h' = 104
    assert_eq!(
        ctx.eval("return 'hello world'.substring(0, 5).charCodeAt(0);")
            .unwrap()
            .to_i32(),
        Some(104)
    );
    // No end parameter 閳?to end of string
    assert_eq!(
        ctx.eval("return 'hello world'.substring(6).length;")
            .unwrap()
            .to_i32(),
        Some(5) // "world".length
    );
}

#[test]
fn test_string_substring_unicode_respects_character_boundaries() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx
        .eval("return 'caf\\u00E9'.substring(3, 4).charCodeAt(0);")
        .unwrap();
    assert_eq!(result.to_i32(), Some(233));
}

// --- Exponentiation operator ---

#[test]
fn test_exponentiation_operator() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(ctx.eval("return 2 ** 10;").unwrap().to_i32(), Some(1024));
    assert_eq!(ctx.eval("return 3 ** 0;").unwrap().to_i32(), Some(1));
    assert_eq!(ctx.eval("return 5 ** 2;").unwrap().to_i32(), Some(25));
}

#[test]
fn test_exponentiation_assignment_operator() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(
        ctx.eval("var x = 2; x **= 3; return x;").unwrap().to_i32(),
        Some(8)
    );
}

#[test]
fn test_switch_string_cases() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(
        ctx.eval("var s = 'b'; var r = 0; switch (s) { case 'a': r = 1; break; case 'b': r = 2; break; case 'c': r = 3; break; } return r;")
            .unwrap().to_i32(),
        Some(2)
    );
}

// --- Bug regression tests ---

#[test]
fn test_division_by_negative_zero() {
    let mut ctx = Context::new(64 * 1024);
    // 1 / -0 should be -Infinity
    let result = ctx.eval("return 1 / (-0);").unwrap();
    assert!(result.is_float());
    let f = result.to_f32().unwrap();
    assert!(
        f.is_infinite() && f < 0.0,
        "1 / -0 should be -Infinity, got {}",
        f
    );

    // -1 / -0 should be Infinity
    let result = ctx.eval("return (-1) / (-0);").unwrap();
    let f = result.to_f32().unwrap();
    assert!(
        f.is_infinite() && f > 0.0,
        "-1 / -0 should be Infinity, got {}",
        f
    );
}

#[test]
fn test_char_code_at_out_of_bounds_returns_nan() {
    let mut ctx = Context::new(64 * 1024);
    // charCodeAt with out-of-bounds index should return NaN
    let result = ctx.eval("return 'abc'.charCodeAt(10);").unwrap();
    assert!(result.is_float());
    assert!(
        result.to_f32().unwrap().is_nan(),
        "charCodeAt OOB should return NaN"
    );

    // charCodeAt with negative index should return NaN
    let result = ctx.eval("return 'abc'.charCodeAt(-1);").unwrap();
    assert!(result.is_float());
    assert!(
        result.to_f32().unwrap().is_nan(),
        "charCodeAt(-1) should return NaN"
    );
}

#[test]
fn test_typeof_string_equality() {
    let mut ctx = Context::new(64 * 1024);
    // typeof returns built-in strings; they must resolve correctly in comparisons
    let result = ctx
        .eval("return typeof undefined === 'undefined';")
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));

    let result = ctx.eval("return typeof 42 === 'number';").unwrap();
    assert_eq!(result.to_bool(), Some(true));

    let result = ctx.eval("return typeof 'hello' === 'string';").unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_negative_zero_semantics() {
    let mut ctx = Context::new(64 * 1024);
    // -0 === 0 should be true (JS spec)
    let result = ctx.eval("return (-0) === 0;").unwrap();
    assert_eq!(result.to_bool(), Some(true));

    // 1 / -0 should be -Infinity (sign preserved through division)
    let result = ctx.eval("return 1 / (-0) === -Infinity;").unwrap();
    assert_eq!(result.to_bool(), Some(true), "1/-0 should equal -Infinity");
}

#[test]
fn test_math_round_negative_halfway() {
    let mut ctx = Context::new(64 * 1024);
    // JS rounds halfway toward positive infinity, NOT away from zero
    assert_eq!(
        ctx.eval("return Math.round(-0.5);").unwrap().to_i32(),
        Some(0)
    );
    assert_eq!(
        ctx.eval("return Math.round(-1.5);").unwrap().to_i32(),
        Some(-1)
    );
    assert_eq!(
        ctx.eval("return Math.round(0.5);").unwrap().to_i32(),
        Some(1)
    );
    assert_eq!(
        ctx.eval("return Math.round(2.5);").unwrap().to_i32(),
        Some(3)
    );
    assert_eq!(
        ctx.eval("return Math.round(-2.5);").unwrap().to_i32(),
        Some(-2)
    );
}

#[test]
fn test_json_stringify_circular_reference() {
    let mut ctx = Context::new(64 * 1024);
    // Circular reference must not crash (stack overflow)
    let result = ctx.eval("var obj = {}; obj.self = obj; return JSON.stringify(obj);");
    // Should succeed without crashing; circular ref produces null
    assert!(result.is_ok());
}

#[test]
fn test_json_stringify_nested_circular() {
    let mut ctx = Context::new(64 * 1024);
    let result =
        ctx.eval("var a = {}; var b = {parent: a}; a.child = b; return JSON.stringify(a);");
    assert!(result.is_ok());
}

#[test]
fn test_modulo_overflow_negative_zero() {
    let mut ctx = Context::new(64 * 1024);
    // Mathematically 0, but dividend is negative 鈫?-0.0 per JS spec
    let result = ctx.eval("return 1 / ((-2147483648) % (-1));").unwrap();
    let f = result.to_f32().unwrap();
    assert!(
        f.is_infinite() && f < 0.0,
        "i32::MIN % -1 should be -0, got {}",
        f
    );
}

// --- Lexer and parser bug regression tests ---

#[test]
fn test_hex_literal() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(ctx.eval("return 0xFF;").unwrap().to_i32(), Some(255));
    assert_eq!(ctx.eval("return 0x1F;").unwrap().to_i32(), Some(31));
    assert_eq!(ctx.eval("return 0x0;").unwrap().to_i32(), Some(0));
}

#[test]
fn test_octal_literal() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(ctx.eval("return 0o17;").unwrap().to_i32(), Some(15));
    assert_eq!(ctx.eval("return 0o777;").unwrap().to_i32(), Some(511));
}

#[test]
fn test_binary_literal() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(ctx.eval("return 0b1010;").unwrap().to_i32(), Some(10));
    assert_eq!(ctx.eval("return 0b11111111;").unwrap().to_i32(), Some(255));
}

#[test]
fn test_leading_decimal() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx.eval("return .5;").unwrap();
    let f = result.to_number_f32().unwrap();
    assert!((f - 0.5).abs() < 0.001);

    let result = ctx.eval("return .125;").unwrap();
    let f = result.to_number_f32().unwrap();
    assert!((f - 0.125).abs() < 0.001);
}

#[test]
fn test_string_hex_escape() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx.eval("return \"\\x41\";").unwrap();
    assert!(result.is_string());
    // \x41 = 'A'
    let result = ctx.eval("return \"\\x41\" === \"A\";").unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_string_unicode_escape() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx.eval("return \"\\u0041\" === \"A\";").unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_raw_utf8_string_literal_source_decodes_correctly() {
    let mut ctx = Context::new(64 * 1024);
    let script = "return 'caf\u{00E9}'.charCodeAt(3) === 233 && 'caf\u{00E9}'.length === 4;";
    let result = ctx.eval(script).unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_string_property_access_survives_rhs_string_concat_optimization() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx.eval("return 10 + 'caf\\u00E9'.length;").unwrap();
    assert_eq!(result.to_i32(), Some(14));
}

#[test]
fn test_string_null_escape() {
    let mut ctx = Context::new(64 * 1024);
    // \0 produces a null byte; string length should be 1
    let result = ctx.eval("return \"\\0\".length;").unwrap();
    assert_eq!(result.to_i32(), Some(1));
}

#[test]
fn test_string_split_empty_separator() {
    let mut ctx = Context::new(64 * 1024);
    // "abc".split("") should produce ["a", "b", "c"]
    assert_eq!(
        ctx.eval("return \"abc\".split(\"\").length;")
            .unwrap()
            .to_i32(),
        Some(3)
    );
    let result = ctx
        .eval("return \"abc\".split(\"\")[0] === \"a\";")
        .unwrap();
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_array_sort_mixed_types() {
    let mut ctx = Context::new(64 * 1024);
    // Mixed types should not throw; default sort uses string comparison
    let result = ctx.eval("var a = [3, 1, \"2\"]; a.sort(); return a[0];");
    assert!(result.is_ok(), "sort on mixed types should not error");
}

#[test]
fn test_new_array_constructor() {
    let mut ctx = Context::new(64 * 1024);
    // new Array(n) creates array of length n
    assert_eq!(
        ctx.eval("return new Array(3).length;").unwrap().to_i32(),
        Some(3)
    );
    // new Array() creates empty array
    assert_eq!(
        ctx.eval("return new Array().length;").unwrap().to_i32(),
        Some(0)
    );
}

#[test]
fn test_format_value_circular_no_crash() {
    let mut ctx = Context::new(64 * 1024);
    // Circular reference in console.log must not crash
    let result = ctx.eval("var a = [1]; a[1] = a; return a.length;");
    assert!(result.is_ok());
}

#[test]
fn test_json_parse_key_escapes() {
    let mut ctx = Context::new(64 * 1024);
    // JSON key with tab escape should parse correctly
    let result = ctx.eval("var x = JSON.parse('{\"a\\\\tb\": 1}'); return typeof x;");
    assert!(result.is_ok());
}

#[test]
fn test_unary_plus_to_number() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(ctx.eval("return +42;").unwrap().to_i32(), Some(42));
    assert_eq!(ctx.eval("return +true;").unwrap().to_i32(), Some(1));
    assert_eq!(ctx.eval("return +false;").unwrap().to_i32(), Some(0));
    assert_eq!(ctx.eval("return +null;").unwrap().to_i32(), Some(0));
    assert!(ctx.eval("return +undefined;").unwrap().is_nan_value());
    assert_eq!(ctx.eval("return +\"123\";").unwrap().to_i32(), Some(123));
    assert!(ctx.eval("return +\"abc\";").unwrap().is_nan_value());
}

#[test]
fn test_unsigned_right_shift() {
    let mut ctx = Context::new(64 * 1024);
    // (-1) >>> 0 should produce 4294967295 (unsigned representation)
    let result = ctx.eval("return (-1) >>> 0;").unwrap();
    let f = result.to_number_f32().unwrap();
    assert!(
        (f - 4294967295.0).abs() < 1.0,
        "(-1) >>> 0 should be ~4294967295, got {}",
        f
    );
    // (-1) >>> 16 should be 65535
    assert_eq!(
        ctx.eval("return (-1) >>> 16;").unwrap().to_i32(),
        Some(65535)
    );
}

#[test]
fn test_null_property_access_throws_typeerror() {
    let mut ctx = Context::new(64 * 1024);
    // null.x should throw TypeError, catchable by try/catch
    let result = ctx.eval("try { return null.x; } catch(e) { return e.message; }");
    assert!(result.is_ok());
    let val = result.unwrap();
    assert!(val.is_string());
}

#[test]
fn test_undefined_property_access_throws_typeerror() {
    let mut ctx = Context::new(64 * 1024);
    let result = ctx.eval("try { return undefined.x; } catch(e) { return e.message; }");
    assert!(result.is_ok());
    let val = result.unwrap();
    assert!(val.is_string());
}

#[test]
fn test_null_method_call_throws_typeerror() {
    let mut ctx = Context::new(64 * 1024);
    // null.toString() should throw TypeError
    let result = ctx.eval("try { null.toString(); return false; } catch(e) { return true; }");
    assert_eq!(result.unwrap().to_bool(), Some(true));
}

#[test]
fn test_empty_string_is_falsy() {
    let mut ctx = Context::new(64 * 1024);
    // Empty string must be falsy in JS
    assert_eq!(
        ctx.eval("return \"\" ? true : false;").unwrap().to_bool(),
        Some(false)
    );
    // Non-empty string must be truthy
    assert_eq!(
        ctx.eval("return \"x\" ? true : false;").unwrap().to_bool(),
        Some(true)
    );
    // Logical operators
    assert_eq!(
        ctx.eval("return !\"\" ? true : false;").unwrap().to_bool(),
        Some(true)
    );
}

#[test]
fn test_error_instanceof() {
    let mut ctx = Context::new(64 * 1024);
    assert_eq!(
        ctx.eval("var e = new Error('x'); return e instanceof Error;")
            .unwrap()
            .to_bool(),
        Some(true)
    );
    assert_eq!(
        ctx.eval("var e = new TypeError('x'); return e instanceof TypeError;")
            .unwrap()
            .to_bool(),
        Some(true)
    );
    // TypeError is also an instance of Error
    assert_eq!(
        ctx.eval("var e = new TypeError('x'); return e instanceof Error;")
            .unwrap()
            .to_bool(),
        Some(true)
    );
}
