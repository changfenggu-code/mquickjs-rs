use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mquickjs::Context;

fn bench_compiled(c: &mut Criterion, name: &str, mem_size: usize, code: &str) {
    let compiler_ctx = Context::new(mem_size);
    let bytecode = compiler_ctx.compile(code).unwrap();

    c.bench_function(name, |b| {
        b.iter(|| {
            let mut ctx = Context::new(mem_size);
            black_box(ctx.execute(&bytecode).unwrap())
        })
    });
}

fn bench_fib(c: &mut Criterion) {
    let code = include_str!("../workloads/fib_iter.js");

    bench_compiled(c, "fib_iter 1k", 64 * 1024, code);
}

fn bench_loop(c: &mut Criterion) {
    let code = include_str!("../workloads/loop_10k.js");

    bench_compiled(c, "loop 10k", 64 * 1024, code);
}

fn bench_array_push(c: &mut Criterion) {
    let code = include_str!("../workloads/array_push_10k.js");

    bench_compiled(c, "array push 10k", 256 * 1024, code);
}

fn bench_object_create(c: &mut Criterion) {
    let code = include_str!("../workloads/object_create_1k.js");

    bench_compiled(c, "object create 1k", 256 * 1024, code);
}

fn bench_closure(c: &mut Criterion) {
    let code = include_str!("../workloads/closure_1k.js");

    bench_compiled(c, "closure 1k", 256 * 1024, code);
}

fn bench_string_concat(c: &mut Criterion) {
    let code = include_str!("../workloads/string_concat_1k.js");

    bench_compiled(c, "string concat 1k", 64 * 1024, code);
}

fn bench_string_concat_local_update_only(c: &mut Criterion) {
    let code = include_str!("../workloads/string_concat_local_update_only.js");

    bench_compiled(c, "string local update only 1k", 64 * 1024, code);
}

fn bench_string_concat_ephemeral(c: &mut Criterion) {
    let code = include_str!("../workloads/string_concat_ephemeral.js");

    bench_compiled(c, "string concat ephemeral 1k", 64 * 1024, code);
}

fn bench_json_parse_only(c: &mut Criterion) {
    let code = include_str!("../workloads/json_parse_only.js");

    bench_compiled(c, "json parse only 1k", 256 * 1024, code);
}

fn bench_json_parse_property_read(c: &mut Criterion) {
    let code = include_str!("../workloads/json_parse_property_read.js");

    bench_compiled(c, "json parse property read 1k", 256 * 1024, code);
}

fn bench_math_max_3arg(c: &mut Criterion) {
    let code = include_str!("../workloads/math_max_3arg_10k.js");

    bench_compiled(c, "math max 3arg 10k", 64 * 1024, code);
}

fn bench_sieve(c: &mut Criterion) {
    let code = include_str!("../workloads/sieve_10k.js");

    bench_compiled(c, "sieve 10k", 256 * 1024, code);
}

fn bench_dense_array_bool_read_branch(c: &mut Criterion) {
    let code = include_str!("../workloads/dense_array_bool_read_branch_10k.js");

    bench_compiled(c, "dense array bool read branch 10k", 256 * 1024, code);
}

fn bench_dense_array_false_write_only(c: &mut Criterion) {
    let code = include_str!("../workloads/dense_array_false_write_only_10k.js");

    bench_compiled(c, "dense array false write only 10k", 256 * 1024, code);
}

fn bench_dense_array_bool_read_hot(c: &mut Criterion) {
    let code = include_str!("../workloads/dense_array_bool_read_hot.js");
    bench_compiled(c, "dense array bool read hot", 256 * 1024, code);
}

fn bench_dense_array_bool_condition_only_hot(c: &mut Criterion) {
    let code = include_str!("../workloads/dense_array_bool_condition_only_hot.js");
    bench_compiled(c, "dense array bool condition only hot", 256 * 1024, code);
}

fn bench_dense_array_bool_condition_only_hot_arg0(c: &mut Criterion) {
    let code = include_str!("../workloads/dense_array_bool_condition_only_hot_arg0.js");
    bench_compiled(
        c,
        "dense array bool condition only hot arg0",
        256 * 1024,
        code,
    );
}

fn bench_dense_array_bool_condition_only_hot_local1(c: &mut Criterion) {
    let code = include_str!("../workloads/dense_array_bool_condition_only_hot_local1.js");
    bench_compiled(
        c,
        "dense array bool condition only hot local1",
        256 * 1024,
        code,
    );
}

fn bench_dense_array_read_only_hot(c: &mut Criterion) {
    let code = include_str!("../workloads/dense_array_read_only_hot.js");
    bench_compiled(c, "dense array read only hot", 256 * 1024, code);
}

fn bench_dense_array_read_only_hot_arg0(c: &mut Criterion) {
    let code = include_str!("../workloads/dense_array_read_only_hot_arg0.js");
    bench_compiled(c, "dense array read only hot arg0", 256 * 1024, code);
}

fn bench_dense_array_read_only_hot_local1(c: &mut Criterion) {
    let code = include_str!("../workloads/dense_array_read_only_hot_local1.js");
    bench_compiled(c, "dense array read only hot local1", 256 * 1024, code);
}

fn bench_dense_array_loop_only_hot(c: &mut Criterion) {
    let code = include_str!("../workloads/dense_array_loop_only_hot.js");
    bench_compiled(c, "dense array loop only hot", 256 * 1024, code);
}

fn bench_dense_array_false_write_then_read_hot(c: &mut Criterion) {
    let code = include_str!("../workloads/dense_array_false_write_then_read_hot.js");
    bench_compiled(c, "dense array false write then read hot", 256 * 1024, code);
}

fn bench_recursion(c: &mut Criterion) {
    let code = include_str!("../workloads/recursion_100x100.js");

    bench_compiled(c, "recursion 100x100", 128 * 1024, code);
}

fn bench_switch(c: &mut Criterion) {
    let code = include_str!("../workloads/switch_1k.js");

    bench_compiled(c, "switch 1k", 64 * 1024, code);
}

fn bench_do_while(c: &mut Criterion) {
    let code = include_str!("../workloads/do_while_10k.js");

    bench_compiled(c, "do...while 10k", 64 * 1024, code);
}

fn bench_method_chain(c: &mut Criterion) {
    let code = include_str!("../workloads/method_chain.js");

    bench_compiled(c, "method_chain 5k", 256 * 1024, code);
}

fn bench_runtime_string_pressure(c: &mut Criterion) {
    let code = include_str!("../workloads/runtime_string_pressure.js");

    bench_compiled(c, "runtime_string_pressure 4k", 256 * 1024, code);
}

fn bench_for_of_array(c: &mut Criterion) {
    let code = include_str!("../workloads/for_of_array.js");

    bench_compiled(c, "for_of_array 20k", 256 * 1024, code);
}

fn bench_deep_property(c: &mut Criterion) {
    let code = include_str!("../workloads/deep_property.js");

    bench_compiled(c, "deep_property 200k", 128 * 1024, code);
}

fn bench_try_catch(c: &mut Criterion) {
    let code = include_str!("../workloads/try_catch.js");

    bench_compiled(c, "try_catch 5k", 256 * 1024, code);
}

fn bench_for_in_object(c: &mut Criterion) {
    let code = include_str!("../workloads/for_in_object.js");

    bench_compiled(c, "for_in_object 20x2000", 256 * 1024, code);
}

fn bench_gc_transient_arrays(c: &mut Criterion) {
    let code = include_str!("../workloads/gc_transient_arrays_2500.js");

    bench_compiled(c, "gc transient arrays 2500", 256 * 1024, code);
}

criterion_group!(
    benches,
    bench_fib,
    bench_loop,
    bench_array_push,
    bench_object_create,
    bench_closure,
    bench_string_concat,
    bench_string_concat_local_update_only,
    bench_string_concat_ephemeral,
    bench_json_parse_only,
    bench_json_parse_property_read,
    bench_math_max_3arg,
    bench_sieve,
    bench_dense_array_bool_read_branch,
    bench_dense_array_false_write_only,
    bench_dense_array_bool_read_hot,
    bench_dense_array_bool_condition_only_hot,
    bench_dense_array_bool_condition_only_hot_arg0,
    bench_dense_array_bool_condition_only_hot_local1,
    bench_dense_array_read_only_hot,
    bench_dense_array_read_only_hot_arg0,
    bench_dense_array_read_only_hot_local1,
    bench_dense_array_loop_only_hot,
    bench_dense_array_false_write_then_read_hot,
    bench_recursion,
    bench_switch,
    bench_do_while,
    bench_method_chain,
    bench_runtime_string_pressure,
    bench_for_of_array,
    bench_deep_property,
    bench_try_catch,
    bench_for_in_object,
    bench_gc_transient_arrays,
);

criterion_main!(benches);
