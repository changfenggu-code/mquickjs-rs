#[cfg(not(feature = "dump"))]
fn main() {
    eprintln!("enable --features dump to run hotspot profiling");
}

#[cfg(feature = "dump")]
mod dump_main {
    use mquickjs::vm::opcode::OpCode;
    use mquickjs::Context;

    fn opcode_name(byte: u8) -> &'static str {
        match byte {
            x if x == OpCode::PushConst as u8 => "PushConst",
            x if x == OpCode::PushConst8 as u8 => "PushConst8",
            x if x == OpCode::Push0 as u8 => "Push0",
            x if x == OpCode::Push1 as u8 => "Push1",
            x if x == OpCode::Push2 as u8 => "Push2",
            x if x == OpCode::Push3 as u8 => "Push3",
            x if x == OpCode::Push4 as u8 => "Push4",
            x if x == OpCode::Push5 as u8 => "Push5",
            x if x == OpCode::Push6 as u8 => "Push6",
            x if x == OpCode::Push7 as u8 => "Push7",
            x if x == OpCode::PushI8 as u8 => "PushI8",
            x if x == OpCode::PushI16 as u8 => "PushI16",
            x if x == OpCode::GetArrayEl as u8 => "GetArrayEl",
            x if x == OpCode::PutArrayEl as u8 => "PutArrayEl",
            x if x == OpCode::GetLoc as u8 => "GetLoc",
            x if x == OpCode::GetLoc8 as u8 => "GetLoc8",
            x if x == OpCode::GetLoc0 as u8 => "GetLoc0",
            x if x == OpCode::GetLoc1 as u8 => "GetLoc1",
            x if x == OpCode::GetLoc2 as u8 => "GetLoc2",
            x if x == OpCode::GetLoc3 as u8 => "GetLoc3",
            x if x == OpCode::PutLoc as u8 => "PutLoc",
            x if x == OpCode::PutLoc8 as u8 => "PutLoc8",
            x if x == OpCode::PutLoc0 as u8 => "PutLoc0",
            x if x == OpCode::PutLoc1 as u8 => "PutLoc1",
            x if x == OpCode::PutLoc2 as u8 => "PutLoc2",
            x if x == OpCode::PutLoc3 as u8 => "PutLoc3",
            x if x == OpCode::Goto as u8 => "Goto",
            x if x == OpCode::IfFalse as u8 => "IfFalse",
            x if x == OpCode::IfTrue as u8 => "IfTrue",
            x if x == OpCode::Lt as u8 => "Lt",
            x if x == OpCode::Lte as u8 => "Lte",
            x if x == OpCode::Mul as u8 => "Mul",
            x if x == OpCode::Add as u8 => "Add",
            x if x == OpCode::Drop as u8 => "Drop",
            x if x == OpCode::Dup as u8 => "Dup",
            x if x == OpCode::CallMethod as u8 => "CallMethod",
            x if x == OpCode::Return as u8 => "Return",
            x if x == OpCode::ReturnUndef as u8 => "ReturnUndef",
            _ => "Other",
        }
    }

    fn print_top(ctx: &Context, limit: usize) {
        let mut pairs: Vec<(usize, u64)> = ctx
            .opcode_counts()
            .iter()
            .copied()
            .enumerate()
            .filter(|(_, count)| *count > 0)
            .collect();
        pairs.sort_by(|a, b| b.1.cmp(&a.1));
        for (idx, count) in pairs.into_iter().take(limit) {
            println!("  {:<14} {}", opcode_name(idx as u8), count);
        }
    }

    fn dump_bytecode(name: &str, bytecode: &mquickjs::FunctionBytecode) {
        println!("== bytecode: {} ==", name);
        for (i, op) in bytecode.bytecode.iter().copied().enumerate() {
            println!("{:4}: {:<14} 0x{:02x}", i, opcode_name(op), op);
        }
        for (idx, inner) in bytecode.inner_functions.iter().enumerate() {
            dump_bytecode(&format!("{}::inner{}", name, idx), inner);
        }
    }

    pub fn run() {
        let sieve = r#"
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

        let mut ctx = Context::new(256 * 1024);
        let bytecode = ctx.compile(sieve).expect("compile");
        dump_bytecode("top", &bytecode);
        ctx.reset_opcode_counts();
        let result = ctx.execute(&bytecode).expect("execute");
        println!("result: {:?}", result);
        print_top(&ctx, 16);
    }
}

#[cfg(feature = "dump")]
fn main() {
    dump_main::run();
}
