use mquickjs::Context;
use std::time::Instant;

struct Case {
    name: &'static str,
    mem_size: usize,
    source: &'static str,
}

fn mean_us(case: &Case, iters: usize) -> f64 {
    let compiler = Context::new(case.mem_size);
    let bytecode = compiler.compile(case.source).expect("compile");

    let start = Instant::now();
    for _ in 0..iters {
        let mut ctx = Context::new(case.mem_size);
        let _ = ctx.execute(&bytecode).expect("execute");
    }
    let elapsed = start.elapsed().as_secs_f64();
    elapsed * 1_000_000.0 / iters as f64
}

fn main() {
    let cases = [
        Case {
            name: "loop_only",
            mem_size: 256 * 1024,
            source: include_str!("../../benches/scripts/dense_array_loop_only_hot.js"),
        },
        Case {
            name: "read_only_local0",
            mem_size: 256 * 1024,
            source: include_str!("../../benches/scripts/dense_array_read_only_hot.js"),
        },
        Case {
            name: "read_only_local1",
            mem_size: 256 * 1024,
            source: include_str!("../../benches/scripts/dense_array_read_only_hot_local1.js"),
        },
        Case {
            name: "read_only_arg0",
            mem_size: 256 * 1024,
            source: include_str!("../../benches/scripts/dense_array_read_only_hot_arg0.js"),
        },
        Case {
            name: "condition_only_local0",
            mem_size: 256 * 1024,
            source: include_str!("../../benches/scripts/dense_array_bool_condition_only_hot.js"),
        },
        Case {
            name: "condition_only_local1",
            mem_size: 256 * 1024,
            source: include_str!(
                "../../benches/scripts/dense_array_bool_condition_only_hot_local1.js"
            ),
        },
        Case {
            name: "condition_only_arg0",
            mem_size: 256 * 1024,
            source: include_str!(
                "../../benches/scripts/dense_array_bool_condition_only_hot_arg0.js"
            ),
        },
        Case {
            name: "read_hot",
            mem_size: 256 * 1024,
            source: include_str!("../../benches/scripts/dense_array_bool_read_hot.js"),
        },
        Case {
            name: "read_branch_10k",
            mem_size: 256 * 1024,
            source: include_str!("../../benches/scripts/dense_array_bool_read_branch.js"),
        },
    ];

    let iters = 20;
    let mut values = Vec::with_capacity(cases.len());
    println!("dense-array layer analysis ({iters} iterations each)");
    for case in &cases {
        let us = mean_us(case, iters);
        values.push((case.name, us));
        println!("{:<24} {:>10.2} us", case.name, us);
    }

    println!();
    for window in values.windows(2) {
        let (left_name, left) = window[0];
        let (right_name, right) = window[1];
        println!(
            "{:<24} -> {:<24} delta {:>10.2} us",
            left_name,
            right_name,
            right - left
        );
    }
}
