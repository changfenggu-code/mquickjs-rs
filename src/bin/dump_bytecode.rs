use mquickjs::runtime::FunctionBytecode;
use mquickjs::vm::opcode::{OpCode, OpFormat, OPCODE_INFO};
use mquickjs::Context;
use std::env;
use std::fs;
use std::path::Path;

fn opcode_name(byte: u8) -> String {
    if (byte as usize) < OpCode::COUNT {
        // SAFETY: compiled bytecode must contain valid opcodes emitted by the compiler.
        let op: OpCode = unsafe { core::mem::transmute(byte) };
        format!("{op:?}")
    } else {
        format!("Invalid({byte})")
    }
}

fn format_operand(opcode: u8, bc: &[u8], pc: usize, bytecode: &FunctionBytecode) -> String {
    let info = &OPCODE_INFO[opcode as usize];
    match info.format {
        OpFormat::None | OpFormat::NoneInt | OpFormat::NoneLoc | OpFormat::NoneArg => String::new(),
        OpFormat::U8 | OpFormat::Const8 | OpFormat::Loc8 => {
            format!("{}", bc[pc + 1])
        }
        OpFormat::I8 => format!("{}", bc[pc + 1] as i8),
        OpFormat::U16
        | OpFormat::Const16
        | OpFormat::Loc
        | OpFormat::Label16
        | OpFormat::NPop
        | OpFormat::Arg
        | OpFormat::VarRef => {
            let val = u16::from_le_bytes([bc[pc + 1], bc[pc + 2]]);
            format!("{val}")
        }
        OpFormat::I16 => {
            let val = i16::from_le_bytes([bc[pc + 1], bc[pc + 2]]);
            format!("{val}")
        }
        OpFormat::Label => {
            let val = i32::from_le_bytes([bc[pc + 1], bc[pc + 2], bc[pc + 3], bc[pc + 4]]);
            format!("{val:+}")
        }
        OpFormat::U32 | OpFormat::I32 | OpFormat::Value => {
            let end = pc + info.size as usize;
            format!("{:02x?}", &bc[pc + 1..end])
        }
        OpFormat::Label8 => format!("{}", bc[pc + 1] as i8),
        OpFormat::NPopX | OpFormat::NoneVarRef => String::new(),
    }
    .pipe(|operand| match opcode {
        x if x == OpCode::GetField as u8
            || x == OpCode::GetField2 as u8
            || x == OpCode::PutField as u8
            || x == OpCode::GetGlobal as u8
            || x == OpCode::GetGlobalOrUndefined as u8
            || x == OpCode::SetGlobal as u8 =>
        {
            let idx = u16::from_le_bytes([bc[pc + 1], bc[pc + 2]]) as usize;
            let extra = bytecode
                .string_constants
                .get(idx)
                .map(|s| format!(" ; \"{}\"", s))
                .unwrap_or_default();
            format!("{operand}{extra}")
        }
        x if x == OpCode::PushConst as u8 || x == OpCode::PushConst8 as u8 => operand,
        _ => operand,
    })
}

trait Pipe: Sized {
    fn pipe<F, T>(self, f: F) -> T
    where
        F: FnOnce(Self) -> T,
    {
        f(self)
    }
}
impl<T> Pipe for T {}

fn dump_function(label: &str, bytecode: &FunctionBytecode, depth: usize) {
    let indent = "  ".repeat(depth);
    println!(
        "{}== {} (locals={}, args={}, stack={}) ==",
        indent, label, bytecode.local_count, bytecode.arg_count, bytecode.stack_size
    );
    let bc = &bytecode.bytecode;
    let mut pc = 0usize;
    while pc < bc.len() {
        let opcode = bc[pc];
        let info = &OPCODE_INFO[opcode as usize];
        let name = opcode_name(opcode);
        let operand = format_operand(opcode, bc, pc, bytecode);
        if operand.is_empty() {
            println!("{}{:04}  {}", indent, pc, name);
        } else {
            println!("{}{:04}  {:<26} {}", indent, pc, name, operand);
        }
        pc += info.size as usize;
    }
    for (idx, inner) in bytecode.inner_functions.iter().enumerate() {
        let name = inner.name.as_deref().unwrap_or("<anon>");
        dump_function(&format!("inner[{idx}] {name}"), inner, depth + 1);
    }
}

fn load_source(name: &str) -> Result<String, String> {
    match name {
        "sieve" => Ok(include_str!("../../benches/scripts/sieve.js").to_string()),
        "dense_array_bool_read_branch" => {
            Ok(include_str!("../../benches/scripts/dense_array_bool_read_branch.js").to_string())
        }
        "dense_array_false_write_only" => {
            Ok(include_str!("../../benches/scripts/dense_array_false_write_only.js").to_string())
        }
        "dense_array_bool_read_hot" => {
            Ok(include_str!("../../benches/scripts/dense_array_bool_read_hot.js").to_string())
        }
        "dense_array_false_write_then_read_hot" => Ok(include_str!(
            "../../benches/scripts/dense_array_false_write_then_read_hot.js"
        )
        .to_string()),
        path if Path::new(path).exists() => {
            fs::read_to_string(path).map_err(|e| format!("failed to read {path}: {e}"))
        }
        other => Err(format!("unknown case or file path: {other}")),
    }
}

fn main() {
    let mut args = env::args().skip(1);
    let Some(name) = args.next() else {
        eprintln!(
            "usage: cargo run --bin dump_bytecode -- <case-name|path>\nknown cases: sieve, dense_array_bool_read_branch, dense_array_false_write_only, dense_array_bool_read_hot, dense_array_false_write_then_read_hot"
        );
        std::process::exit(2);
    };

    let source = load_source(&name).unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });

    let ctx = Context::new(256 * 1024);
    let bytecode = ctx.compile(&source).unwrap_or_else(|e| {
        eprintln!("compile failed: {e}");
        std::process::exit(1);
    });

    dump_function("top-level", &bytecode, 0);
}
