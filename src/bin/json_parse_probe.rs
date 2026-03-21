use mquickjs::Context;
use std::time::Instant;

struct Case {
    name: &'static str,
    source: &'static str,
    mem_size: usize,
}

fn run_case(case: &Case, iterations: u32) {
    let compiler = Context::new(case.mem_size);
    let bytecode = compiler.compile(case.source).expect("compile");

    let start = Instant::now();
    for _ in 0..iterations {
        let mut ctx = Context::new(case.mem_size);
        let _ = ctx.execute(&bytecode).expect("execute");
    }
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;
    println!("{} avg_ms={:.3}", case.name, elapsed_ms);
}

fn main() {
    let cases = [
        Case {
            name: "json_parse",
            mem_size: 256 * 1024,
            source: r#"
                var data = '{"name": "test", "value": 42, "items": [1, 2, 3]}';
                var sum = 0;
                for (var i = 0; i < 1000; i = i + 1) {
                    var obj = JSON.parse(data);
                    sum = sum + obj.value;
                }
                return sum;
            "#,
        },
        Case {
            name: "json_parse_only",
            mem_size: 256 * 1024,
            source: r#"
                var data = '{"name": "test", "value": 42, "items": [1, 2, 3]}';
                var count = 0;
                for (var i = 0; i < 1000; i = i + 1) {
                    JSON.parse(data);
                    count = count + 1;
                }
                return count;
            "#,
        },
        Case {
            name: "json_parse_property_read",
            mem_size: 256 * 1024,
            source: r#"
                var data = '{"name": "test", "value": 42, "items": [1, 2, 3]}';
                var sum = 0;
                for (var i = 0; i < 1000; i = i + 1) {
                    var obj = JSON.parse(data);
                    sum = sum + obj.value;
                }
                return sum;
            "#,
        },
    ];

    let iterations = 20;
    for case in &cases {
        run_case(case, iterations);
    }
}
