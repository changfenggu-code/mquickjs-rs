use mquickjs::Context;
use std::time::Instant;

struct Case {
    name: &'static str,
    source: &'static str,
    mem_size: usize,
}

fn run_case(case: &Case) {
    let mut ctx = Context::new(case.mem_size);

    let before = ctx.memory_stats();
    let start = Instant::now();
    ctx.eval(case.source).expect("eval");
    let eval_elapsed = start.elapsed().as_secs_f64() * 1000.0;
    let after_eval = ctx.memory_stats();

    let gc_start = Instant::now();
    ctx.gc();
    let gc_elapsed = gc_start.elapsed().as_secs_f64() * 1000.0;
    let after_gc = ctx.memory_stats();

    println!("== {} ==", case.name);
    println!("eval_ms: {:.3}", eval_elapsed);
    println!("gc_ms: {:.3}", gc_elapsed);
    println!(
        "gc_count: {} -> {} -> {}",
        before.gc_count, after_eval.gc_count, after_gc.gc_count
    );
    println!(
        "objects: {} -> {} -> {}",
        before.objects, after_eval.objects, after_gc.objects
    );
    println!(
        "arrays: {} -> {} -> {}",
        before.arrays, after_eval.arrays, after_gc.arrays
    );
    println!(
        "estimated_object_bytes: {} -> {} -> {}",
        before.estimated_object_bytes,
        after_eval.estimated_object_bytes,
        after_gc.estimated_object_bytes
    );
    println!();
}

fn main() {
    let cases = [
        Case {
            name: "manual_gc_object_cycles",
            mem_size: 256 * 1024,
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
            name: "auto_gc_object_cycles",
            mem_size: 256 * 1024,
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
        Case {
            name: "auto_gc_transient_arrays",
            mem_size: 256 * 1024,
            source: r#"
                function makeArray() {
                    var a = [1, 2, 3, 4, 5, 6, 7, 8];
                    return a.length;
                }
                for (var i = 0; i < 2500; i = i + 1) {
                    makeArray();
                }
                return 0;
            "#,
        },
    ];

    for case in &cases {
        run_case(case);
    }
}
