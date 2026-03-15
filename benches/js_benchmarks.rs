use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mquickjs::Context;

fn bench_fib(c: &mut Criterion) {
    // Use iterative fibonacci to avoid stack overflow
    let code = r#"
        function fib(n) {
            if (n <= 1) return n;
            var a = 0;
            var b = 1;
            for (var i = 2; i <= n; i = i + 1) {
                var c = a + b;
                a = b;
                b = c;
            }
            return b;
        }
        var sum = 0;
        for (var i = 0; i < 1000; i = i + 1) {
            sum = sum + fib(30);
        }
        return sum;
    "#;

    c.bench_function("fib_iter 1k", |b| {
        b.iter(|| {
            let mut ctx = Context::new(64 * 1024);
            black_box(ctx.eval(code).unwrap())
        })
    });
}

fn bench_loop(c: &mut Criterion) {
    let code = r#"
        var sum = 0;
        for (var i = 0; i < 10000; i = i + 1) {
            sum = sum + i;
        }
        return sum;
    "#;

    c.bench_function("loop 10k", |b| {
        b.iter(|| {
            let mut ctx = Context::new(64 * 1024);
            black_box(ctx.eval(code).unwrap())
        })
    });
}

fn bench_array_push(c: &mut Criterion) {
    let code = r#"
        var arr = [];
        for (var i = 0; i < 10000; i = i + 1) {
            arr.push(i);
        }
        return arr.length;
    "#;

    c.bench_function("array push 10k", |b| {
        b.iter(|| {
            let mut ctx = Context::new(256 * 1024);
            black_box(ctx.eval(code).unwrap())
        })
    });
}

fn bench_object_create(c: &mut Criterion) {
    let code = r#"
        function Point(x, y) {
            this.x = x;
            this.y = y;
        }
        var points = [];
        for (var i = 0; i < 1000; i = i + 1) {
            points.push(new Point(i, i * 2));
        }
        return points.length;
    "#;

    c.bench_function("object create 1k", |b| {
        b.iter(|| {
            let mut ctx = Context::new(256 * 1024);
            black_box(ctx.eval(code).unwrap())
        })
    });
}

fn bench_closure(c: &mut Criterion) {
    let code = r#"
        function makeAdder(x) {
            function adder(y) {
                return x + y;
            }
            return adder;
        }
        var sum = 0;
        for (var i = 0; i < 1000; i = i + 1) {
            var add = makeAdder(i);
            sum = sum + add(i);
        }
        return sum;
    "#;

    c.bench_function("closure 1k", |b| {
        b.iter(|| {
            let mut ctx = Context::new(256 * 1024);
            black_box(ctx.eval(code).unwrap())
        })
    });
}

fn bench_string_concat(c: &mut Criterion) {
    let code = r#"
        var s = "";
        for (var i = 0; i < 1000; i = i + 1) {
            s = s + "x";
        }
        return s.length;
    "#;

    c.bench_function("string concat 1k", |b| {
        b.iter(|| {
            let mut ctx = Context::new(64 * 1024);
            black_box(ctx.eval(code).unwrap())
        })
    });
}

fn bench_json_parse(c: &mut Criterion) {
    let code = r#"
        var data = '{"name": "test", "value": 42, "items": [1, 2, 3]}';
        var sum = 0;
        for (var i = 0; i < 1000; i = i + 1) {
            var obj = JSON.parse(data);
            sum = sum + obj.value;
        }
        return sum;
    "#;

    c.bench_function("json parse 1k", |b| {
        b.iter(|| {
            let mut ctx = Context::new(256 * 1024);
            black_box(ctx.eval(code).unwrap())
        })
    });
}

fn bench_sieve(c: &mut Criterion) {
    let code = r#"
        function sieve(n) {
            var primes = [];
            for (var i = 0; i <= n; i = i + 1) {
                primes.push(true);
            }
            primes[0] = false;
            primes[1] = false;
            for (var i = 2; i * i <= n; i = i + 1) {
                if (primes[i]) {
                    for (var j = i * i; j <= n; j = j + i) {
                        primes[j] = false;
                    }
                }
            }
            var count = 0;
            for (var i = 0; i <= n; i = i + 1) {
                if (primes[i]) count = count + 1;
            }
            return count;
        }
        return sieve(10000);
    "#;

    c.bench_function("sieve 10k", |b| {
        b.iter(|| {
            let mut ctx = Context::new(256 * 1024);
            black_box(ctx.eval(code).unwrap())
        })
    });
}

fn bench_recursion(c: &mut Criterion) {
    // Benchmark recursion with a simpler function
    let code = r#"
        function sum(n) {
            if (n <= 0) return 0;
            return n + sum(n - 1);
        }
        var total = 0;
        for (var i = 0; i < 100; i = i + 1) {
            total = total + sum(100);
        }
        return total;
    "#;

    c.bench_function("recursion 100x100", |b| {
        b.iter(|| {
            let mut ctx = Context::new(128 * 1024);
            black_box(ctx.eval(code).unwrap())
        })
    });
}

fn bench_switch(c: &mut Criterion) {
    let code = r#"
        var sum = 0;
        for (var i = 0; i < 1000; i = i + 1) {
            switch (i % 10) {
                case 0: sum = sum + 10; break;
                case 1: sum = sum + 20; break;
                case 2: sum = sum + 30; break;
                case 3: sum = sum + 40; break;
                case 4: sum = sum + 50; break;
                case 5: sum = sum + 60; break;
                case 6: sum = sum + 70; break;
                case 7: sum = sum + 80; break;
                case 8: sum = sum + 90; break;
                default: sum = sum + 5;
            }
        }
        return sum;
    "#;

    c.bench_function("switch 1k", |b| {
        b.iter(|| {
            let mut ctx = Context::new(64 * 1024);
            black_box(ctx.eval(code).unwrap())
        })
    });
}

fn bench_do_while(c: &mut Criterion) {
    let code = r#"
        var i = 0;
        do {
            i = i + 1;
        } while (i < 10000);
        return i;
    "#;

    c.bench_function("do...while 10k", |b| {
        b.iter(|| {
            let mut ctx = Context::new(64 * 1024);
            black_box(ctx.eval(code).unwrap())
        })
    });
}

fn bench_method_chain(c: &mut Criterion) {
    let code = r#"
        var arr = [];
        for (var i = 0; i < 5000; i = i + 1) {
            arr.push(i);
        }
        function double(x) { return x * 2; }
        function divByThree(x) { return x % 3 === 0; }
        function add(acc, x) { return acc + x; }
        return arr.map(double).filter(divByThree).reduce(add, 0);
    "#;

    c.bench_function("method_chain 5k", |b| {
        b.iter(|| {
            let mut ctx = Context::new(256 * 1024);
            black_box(ctx.eval(code).unwrap())
        })
    });
}

fn bench_runtime_string_pressure(c: &mut Criterion) {
    let code = r#"
        var parts = [];
        for (var i = 0; i < 4000; i = i + 1) {
            parts.push("item-" + i + "-" + (i % 17));
        }
        var total = 0;
        for (var i = 0; i < parts.length; i = i + 1) {
            total = total + parts[i].length;
        }
        return total;
    "#;

    c.bench_function("runtime_string_pressure 4k", |b| {
        b.iter(|| {
            let mut ctx = Context::new(256 * 1024);
            black_box(ctx.eval(code).unwrap())
        })
    });
}

fn bench_for_of_array(c: &mut Criterion) {
    let code = r#"
        var arr = [];
        for (var i = 0; i < 20000; i = i + 1) {
            arr.push(i);
        }
        var sum = 0;
        for (var value of arr) {
            sum = sum + value;
        }
        return sum;
    "#;

    c.bench_function("for_of_array 20k", |b| {
        b.iter(|| {
            let mut ctx = Context::new(256 * 1024);
            black_box(ctx.eval(code).unwrap())
        })
    });
}

fn bench_deep_property(c: &mut Criterion) {
    let code = r#"
        var root = { a: { b: { c: { d: 1 } } } };
        var sum = 0;
        for (var i = 0; i < 200000; i = i + 1) {
            sum = sum + root.a.b.c.d;
        }
        return sum;
    "#;

    c.bench_function("deep_property 200k", |b| {
        b.iter(|| {
            let mut ctx = Context::new(128 * 1024);
            black_box(ctx.eval(code).unwrap())
        })
    });
}

criterion_group!(
    benches,
    bench_fib,
    bench_loop,
    bench_array_push,
    bench_object_create,
    bench_closure,
    bench_string_concat,
    bench_json_parse,
    bench_sieve,
    bench_recursion,
    bench_switch,
    bench_do_while,
    bench_method_chain,
    bench_runtime_string_pressure,
    bench_for_of_array,
    bench_deep_property,
);

criterion_main!(benches);
