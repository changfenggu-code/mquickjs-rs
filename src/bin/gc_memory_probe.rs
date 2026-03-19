use mquickjs::Context;

struct Case {
    name: &'static str,
    source: &'static str,
}

fn print_case(case: &Case) {
    let mut ctx = Context::new(256 * 1024);

    let before = ctx.memory_stats();
    ctx.eval(case.source).expect("eval");
    let after_alloc = ctx.memory_stats();
    ctx.gc();
    let after_gc = ctx.memory_stats();

    println!("== {} ==", case.name);
    println!(
        "gc_count: {} -> {} -> {}",
        before.gc_count, after_alloc.gc_count, after_gc.gc_count
    );
    println!(
        "objects: {} -> {} -> {}",
        before.objects, after_alloc.objects, after_gc.objects
    );
    println!(
        "arrays: {} -> {} -> {}",
        before.arrays, after_alloc.arrays, after_gc.arrays
    );
    println!(
        "runtime_strings: {} -> {} -> {}",
        before.runtime_strings, after_alloc.runtime_strings, after_gc.runtime_strings
    );
    println!(
        "estimated_object_bytes: {} -> {} -> {}",
        before.estimated_object_bytes,
        after_alloc.estimated_object_bytes,
        after_gc.estimated_object_bytes
    );
    println!();
}

fn main() {
    let cases = [
        Case {
            name: "object_cycles",
            source: r#"
                function makeCycle() {
                    var a = {};
                    a.self = a;
                }
                for (var i = 0; i < 200; i = i + 1) {
                    makeCycle();
                }
                return 0;
            "#,
        },
        Case {
            name: "transient_arrays",
            source: r#"
                function makeArray() {
                    var a = [1, 2, 3, 4, 5, 6, 7, 8];
                    return a.length;
                }
                for (var i = 0; i < 400; i = i + 1) {
                    makeArray();
                }
                return 0;
            "#,
        },
        Case {
            name: "auto_gc_cycles",
            source: r#"
                function makeCycle() {
                    var a = {};
                    a.self = a;
                }
                for (var i = 0; i < 1500; i = i + 1) {
                    makeCycle();
                }
                return 0;
            "#,
        },
    ];

    for case in &cases {
        print_case(case);
    }
}
