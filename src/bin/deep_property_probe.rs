use mquickjs::Context;
use std::time::Instant;

const SCRIPT: &str = r#"
var root = { a: { b: { c: { d: 1 } } } };
var sum = 0;

for (var i = 0; i < 200000; i = i + 1) {
    sum = sum + root.a.b.c.d;
}

return sum;
"#;

fn main() {
    let compiler = Context::new(128 * 1024);
    let bytecode = compiler.compile(SCRIPT).expect("compile");

    let iterations = 20u32;
    let start = Instant::now();
    for _ in 0..iterations {
        let mut ctx = Context::new(128 * 1024);
        let _ = ctx.execute(&bytecode).expect("execute");
    }
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;
    println!("deep_property_probe avg_ms={:.3}", elapsed_ms);
}
