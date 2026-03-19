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
            x if x == OpCode::GetArrayElDiscard as u8 => "GetArrayElDiscard",
            x if x == OpCode::GetField as u8 => "GetField",
            x if x == OpCode::GetField2 as u8 => "GetField2",
            x if x == OpCode::GetArrayPush2 as u8 => "GetArrayPush2",
            x if x == OpCode::GetLength as u8 => "GetLength",
            x if x == OpCode::PutArrayEl as u8 => "PutArrayEl",
            x if x == OpCode::GetLoc as u8 => "GetLoc",
            x if x == OpCode::GetLoc8 as u8 => "GetLoc8",
            x if x == OpCode::GetLoc0 as u8 => "GetLoc0",
            x if x == OpCode::GetLoc1 as u8 => "GetLoc1",
            x if x == OpCode::GetLoc2 as u8 => "GetLoc2",
            x if x == OpCode::GetLoc3 as u8 => "GetLoc3",
            x if x == OpCode::GetLoc4 as u8 => "GetLoc4",
            x if x == OpCode::PutLoc as u8 => "PutLoc",
            x if x == OpCode::PutLoc8 as u8 => "PutLoc8",
            x if x == OpCode::PutLoc0 as u8 => "PutLoc0",
            x if x == OpCode::PutLoc1 as u8 => "PutLoc1",
            x if x == OpCode::PutLoc2 as u8 => "PutLoc2",
            x if x == OpCode::PutLoc3 as u8 => "PutLoc3",
            x if x == OpCode::PutLoc4 as u8 => "PutLoc4",
            x if x == OpCode::IncLoc8Drop as u8 => "IncLoc8Drop",
            x if x == OpCode::IncLoc0Drop as u8 => "IncLoc0Drop",
            x if x == OpCode::IncLoc1Drop as u8 => "IncLoc1Drop",
            x if x == OpCode::IncLoc2Drop as u8 => "IncLoc2Drop",
            x if x == OpCode::IncLoc3Drop as u8 => "IncLoc3Drop",
            x if x == OpCode::IncLoc4Drop as u8 => "IncLoc4Drop",
            x if x == OpCode::Goto as u8 => "Goto",
            x if x == OpCode::IfFalse as u8 => "IfFalse",
            x if x == OpCode::IfTrue as u8 => "IfTrue",
            x if x == OpCode::Lt as u8 => "Lt",
            x if x == OpCode::Lte as u8 => "Lte",
            x if x == OpCode::Mul as u8 => "Mul",
            x if x == OpCode::Add as u8 => "Add",
            x if x == OpCode::AddConstStringLeft as u8 => "AddConstStringLeft",
            x if x == OpCode::AddConstStringRight as u8 => "AddConstStringRight",
            x if x == OpCode::AddConstStringSurround as u8 => "AddConstStringSurround",
            x if x == OpCode::AddConstStringSurroundValue as u8 => "AddConstStringSurroundValue",
            x if x == OpCode::AppendConstStringToLoc0 as u8 => "AppendConstStringToLoc0",
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

    fn run_case(name: &str, source: &str, _mem_size: usize) {
        let mut ctx = Context::new(256 * 1024);
        let bytecode = ctx.compile(source).expect("compile");
        ctx.reset_opcode_counts();
        let result = ctx.execute(&bytecode).expect("execute");
        let strings = ctx.runtime_string_source_stats();
        println!("== {} ==", name);
        println!("result: {:?}", result);
        println!(
            "runtime strings: total={} concat={} for_in_key={} json={} object_keys={} object_entries={} error={} type={} other={}",
            strings.total,
            strings.concat,
            strings.for_in_key,
            strings.json,
            strings.object_keys,
            strings.object_entries,
            strings.error_string,
            strings.type_string,
            strings.other
        );
        print_top(&ctx, 16);
        println!();
    }

    pub fn run() {
        let method_chain = include_str!("../../benches/scripts/method_chain.js");
        let string_concat = r#"
            var s = "";
            for (var i = 0; i < 1000; i = i + 1) {
                s = s + "x";
            }
            return s.length;
        "#;
        let string_local_update_only = r#"
            var s = "";
            for (var i = 0; i < 1000; i = i + 1) {
                s = "x";
            }
            return s.length;
        "#;
        let string_concat_ephemeral = r#"
            var s = "";
            var total = 0;
            for (var i = 0; i < 1000; i = i + 1) {
                var t = s + "x";
                total = total + t.length;
            }
            return total;
        "#;
        let dense_array_bool_read_branch =
            include_str!("../../benches/scripts/dense_array_bool_read_branch.js");
        let dense_array_false_write_only =
            include_str!("../../benches/scripts/dense_array_false_write_only.js");
        let dense_array_bool_read_hot =
            include_str!("../../benches/scripts/dense_array_bool_read_hot.js");
        let dense_array_bool_condition_only_hot =
            include_str!("../../benches/scripts/dense_array_bool_condition_only_hot.js");
        let dense_array_bool_condition_only_hot_arg0 =
            include_str!("../../benches/scripts/dense_array_bool_condition_only_hot_arg0.js");
        let dense_array_bool_condition_only_hot_local1 =
            include_str!("../../benches/scripts/dense_array_bool_condition_only_hot_local1.js");
        let dense_array_read_only_hot =
            include_str!("../../benches/scripts/dense_array_read_only_hot.js");
        let dense_array_read_only_hot_arg0 =
            include_str!("../../benches/scripts/dense_array_read_only_hot_arg0.js");
        let dense_array_read_only_hot_local1 =
            include_str!("../../benches/scripts/dense_array_read_only_hot_local1.js");
        let dense_array_loop_only_hot =
            include_str!("../../benches/scripts/dense_array_loop_only_hot.js");
        let dense_array_false_write_then_read_hot =
            include_str!("../../benches/scripts/dense_array_false_write_then_read_hot.js");
        let sieve = include_str!("../../benches/scripts/sieve.js");
        let runtime_string_pressure =
            include_str!("../../benches/scripts/runtime_string_pressure.js");

        run_case("method_chain", method_chain, 256 * 1024);
        run_case("string_concat", string_concat, 64 * 1024);
        run_case(
            "string_local_update_only",
            string_local_update_only,
            64 * 1024,
        );
        run_case(
            "string_concat_ephemeral",
            string_concat_ephemeral,
            64 * 1024,
        );
        run_case(
            "dense_array_bool_read_branch",
            dense_array_bool_read_branch,
            256 * 1024,
        );
        run_case(
            "dense_array_false_write_only",
            dense_array_false_write_only,
            256 * 1024,
        );
        run_case(
            "dense_array_bool_read_hot",
            dense_array_bool_read_hot,
            256 * 1024,
        );
        run_case(
            "dense_array_bool_condition_only_hot",
            dense_array_bool_condition_only_hot,
            256 * 1024,
        );
        run_case(
            "dense_array_bool_condition_only_hot_arg0",
            dense_array_bool_condition_only_hot_arg0,
            256 * 1024,
        );
        run_case(
            "dense_array_bool_condition_only_hot_local1",
            dense_array_bool_condition_only_hot_local1,
            256 * 1024,
        );
        run_case(
            "dense_array_read_only_hot",
            dense_array_read_only_hot,
            256 * 1024,
        );
        run_case(
            "dense_array_read_only_hot_arg0",
            dense_array_read_only_hot_arg0,
            256 * 1024,
        );
        run_case(
            "dense_array_read_only_hot_local1",
            dense_array_read_only_hot_local1,
            256 * 1024,
        );
        run_case(
            "dense_array_loop_only_hot",
            dense_array_loop_only_hot,
            256 * 1024,
        );
        run_case(
            "dense_array_false_write_then_read_hot",
            dense_array_false_write_then_read_hot,
            256 * 1024,
        );
        run_case("sieve", sieve, 256 * 1024);
        run_case(
            "runtime_string_pressure",
            runtime_string_pressure,
            256 * 1024,
        );
    }
}

#[cfg(feature = "dump")]
fn main() {
    dump_main::run();
}
