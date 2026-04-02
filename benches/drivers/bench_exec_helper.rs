use mquickjs::Context;
use std::env;
use std::fs;
use std::process;
use std::time::Instant;

fn parse_usize(label: &str, value: &str) -> usize {
    value.parse::<usize>().unwrap_or_else(|err| {
        eprintln!("invalid {label} '{value}': {err}");
        process::exit(2);
    })
}

fn main() {
    let mut args = env::args().skip(1);
    let Some(script_path) = args.next() else {
        eprintln!("usage: cargo run --release --bin bench_exec_helper -- <script-path> <iterations> <mem-bytes>");
        process::exit(2);
    };
    let Some(iterations) = args.next() else {
        eprintln!("missing iterations");
        process::exit(2);
    };
    let Some(mem_size) = args.next() else {
        eprintln!("missing mem-bytes");
        process::exit(2);
    };
    if args.next().is_some() {
        eprintln!("unexpected extra arguments");
        process::exit(2);
    }

    let iterations = parse_usize("iterations", &iterations);
    let mem_size = parse_usize("mem-bytes", &mem_size);
    if iterations == 0 {
        eprintln!("iterations must be > 0");
        process::exit(2);
    }

    let source = fs::read_to_string(&script_path).unwrap_or_else(|err| {
        eprintln!("failed to read {script_path}: {err}");
        process::exit(1);
    });
    let compiler_ctx = Context::new(mem_size);
    let bytecode = compiler_ctx.compile(&source).unwrap_or_else(|err| {
        eprintln!("compile failed for {script_path}: {err}");
        process::exit(1);
    });

    let start = Instant::now();
    for _ in 0..iterations {
        let mut ctx = Context::new(mem_size);
        ctx.execute(&bytecode).unwrap_or_else(|err| {
            eprintln!("execute failed for {script_path}: {err}");
            process::exit(1);
        });
    }

    println!("{:.6}", start.elapsed().as_secs_f64() / iterations as f64);
}
