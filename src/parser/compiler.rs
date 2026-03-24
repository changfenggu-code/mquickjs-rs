//! JavaScript compiler
//!
//! Generates bytecode from source code in a single pass.
//! Uses precedence climbing for expression parsing.

use super::lexer::{Lexer, SourcePos, Token};
use crate::value::Value;
use crate::vm::opcode::OpCode;
use alloc::{format, string::String, string::ToString, vec::Vec};

/// Maximum number of local variables
const MAX_LOCALS: usize = 256;

/// Maximum number of constants
const MAX_CONSTANTS: usize = 65536;

/// Local variable info
#[derive(Debug, Clone)]
struct Local {
    name: String,
    depth: u32,
}

/// Captured variable info (for closures)
#[derive(Debug, Clone)]
struct Capture {
    /// Name of the captured variable
    name: String,
    /// Index in the outer function's locals (or captures)
    outer_index: usize,
    /// Whether this captures from outer's locals (true) or outer's captures (false)
    is_local: bool,
}

/// Jump patch location
#[derive(Debug, Clone, Copy)]
struct JumpPatch {
    /// Offset in bytecode where the jump target needs to be patched
    offset: usize,
}

/// Loop/switch context for break/continue
#[derive(Debug, Clone)]
struct LoopContext {
    /// Continue jump target (start of loop or increment section)
    continue_target: usize,
    /// Break jump patches (to be patched after loop/switch)
    break_patches: Vec<JumpPatch>,
    /// Continue jump patches (for do...while where target is unknown at body compile time)
    continue_patches: Vec<JumpPatch>,
    /// Scope depth when loop started (for proper cleanup)
    scope_depth: u32,
    /// True if this is a switch statement (continue should skip past it)
    is_switch: bool,
}

/// Compiler state
pub struct Compiler<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
    previous_token: Token,
    bytecode: Vec<u8>,
    constants: Vec<Value>,
    /// String constant pool
    string_constants: Vec<String>,
    /// Local variables in current scope
    locals: Vec<Local>,
    /// Maximum number of locals ever used (for frame allocation)
    max_locals: usize,
    /// Current scope depth
    scope_depth: u32,
    /// Current source position
    current_pos: SourcePos,
    /// Had error during compilation
    had_error: bool,
    /// Panic mode (suppress cascading errors)
    panic_mode: bool,
    /// Compiled inner functions
    functions: Vec<CompiledFunction>,
    /// Loop context stack for break/continue
    loop_stack: Vec<LoopContext>,
    /// Captured variables from outer scopes (for closures)
    captures: Vec<Capture>,
    /// Outer function's locals (for resolving captures during inner function compilation)
    outer_locals: Option<Vec<Local>>,
    /// Outer function's captures (for resolving nested captures)
    outer_captures: Option<Vec<Capture>>,
    /// Last variable reference (for postfix ++ / --): (is_local, index)
    last_var_ref: Option<(bool, usize)>,
    /// If the most recently parsed expression was a pure compile-time string,
    /// record its string index and the start offset of its emitted bytecode.
    last_expr_string_const: Option<(u16, usize)>,
    /// If the most recently parsed expression was a pure boolean literal,
    /// record its value and the start offset of its emitted bytecode.
    last_expr_bool_const: Option<(bool, usize)>,
    /// If the most recently parsed expression ended with `AddConstStringSurround`,
    /// record its const string indices and opcode start offset.
    last_expr_concat_surround: Option<(u16, u16, usize)>,
}

impl<'a> Compiler<'a> {
    /// Create a new compiler for the given source
    pub fn new(source: &'a str) -> Self {
        let mut lexer = Lexer::new(source);
        let current_token = lexer.next_token();

        Compiler {
            lexer,
            current_token,
            previous_token: Token::Eof,
            bytecode: Vec::new(),
            constants: Vec::new(),
            string_constants: Vec::new(),
            locals: Vec::new(),
            max_locals: 0,
            scope_depth: 0,
            current_pos: SourcePos::default(),
            had_error: false,
            panic_mode: false,
            functions: Vec::new(),
            loop_stack: Vec::new(),
            captures: Vec::new(),
            outer_locals: None,
            outer_captures: None,
            last_var_ref: None,
            last_expr_string_const: None,
            last_expr_bool_const: None,
            last_expr_concat_surround: None,
        }
    }

    fn peek_token(&self) -> Token {
        let mut lexer = self.lexer.clone();
        lexer.next_token()
    }

    /// Compile the source and return bytecode
    pub fn compile(mut self) -> Result<CompiledFunction, CompileError> {
        // Parse statements until EOF
        while !self.check(&Token::Eof) {
            self.statement()?;
        }

        // Emit implicit return undefined
        self.emit_op(OpCode::ReturnUndef);

        if self.had_error {
            Err(self.syntax_error("Compilation failed"))
        } else {
            // Convert captures to CaptureInfo
            let captures: Vec<CaptureInfo> = self
                .captures
                .iter()
                .map(|c| CaptureInfo {
                    outer_index: c.outer_index,
                    is_local: c.is_local,
                })
                .collect();

            Ok(CompiledFunction {
                bytecode: self.bytecode,
                constants: self.constants,
                string_constants: self.string_constants,
                local_count: self.max_locals,
                arg_count: 0, // Top-level script has no arguments
                functions: self.functions,
                captures,
            })
        }
    }

    // =========================================================================
    // Token handling
    // =========================================================================

    /// Advance to the next token
    fn advance(&mut self) {
        self.previous_token = core::mem::replace(&mut self.current_token, Token::Eof);
        self.current_pos = self.lexer.position();

        loop {
            self.current_token = self.lexer.next_token();
            if !matches!(self.current_token, Token::Error(_)) {
                break;
            }
            // Report lexer error and continue
            if let Token::Error(msg) = &self.current_token {
                self.error(&msg.clone());
            }
        }
    }

    /// Check if current token matches expected
    fn check(&self, expected: &Token) -> bool {
        core::mem::discriminant(&self.current_token) == core::mem::discriminant(expected)
    }

    /// Consume token if it matches, return true if matched
    fn match_token(&mut self, expected: &Token) -> bool {
        if self.check(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Expect a specific token, advance if matched
    fn expect(&mut self, expected: Token) -> Result<(), CompileError> {
        if self.check(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(self.unexpected_token_error(format!("{:?}", expected)))
        }
    }

    /// Create an UnexpectedToken error with current position
    fn unexpected_token_error(&self, expected: String) -> CompileError {
        CompileError::UnexpectedToken {
            expected,
            found: format!("{:?}", self.current_token),
            line: self.current_pos.line,
            column: self.current_pos.column,
        }
    }

    /// Create a SyntaxError with current position
    fn syntax_error(&self, message: impl Into<String>) -> CompileError {
        CompileError::SyntaxError {
            message: message.into(),
            line: self.current_pos.line,
            column: self.current_pos.column,
        }
    }

    /// Report an error
    fn error(&mut self, message: &str) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;
        self.had_error = true;
        #[cfg(not(feature = "std"))]
        let _ = message;
        #[cfg(feature = "std")]
        eprintln!("[line {}] Error: {}", self.current_pos.line, message);
    }

    /// Check if current token is an assignment operator
    fn is_assignment_op(&self) -> bool {
        matches!(
            self.current_token,
            Token::Eq
                | Token::PlusEq
                | Token::MinusEq
                | Token::StarEq
                | Token::SlashEq
                | Token::PercentEq
                | Token::AmpEq
                | Token::PipeEq
                | Token::CaretEq
                | Token::LtLtEq
                | Token::GtGtEq
                | Token::GtGtGtEq
                | Token::StarStarEq
        )
    }

    /// Synchronize after error
    fn synchronize(&mut self) {
        self.panic_mode = false;

        while !self.check(&Token::Eof) {
            // Stop at statement boundary
            if matches!(self.previous_token, Token::Semicolon) {
                return;
            }

            // Stop before certain keywords
            match self.current_token {
                Token::Function
                | Token::Var
                | Token::Let
                | Token::Const
                | Token::For
                | Token::If
                | Token::While
                | Token::Return => return,
                _ => {}
            }

            self.advance();
        }
    }

    // =========================================================================
    // Bytecode emission
    // =========================================================================

    /// Emit a single opcode
    fn emit_op(&mut self, op: OpCode) {
        self.bytecode.push(op as u8);
    }

    /// Emit a single byte
    fn emit_byte(&mut self, byte: u8) {
        self.bytecode.push(byte);
    }

    /// Emit two bytes
    fn emit_bytes(&mut self, b1: u8, b2: u8) {
        self.bytecode.push(b1);
        self.bytecode.push(b2);
    }

    /// Emit a 16-bit value (little-endian)
    fn emit_u16(&mut self, val: u16) {
        self.bytecode.push((val & 0xff) as u8);
        self.bytecode.push((val >> 8) as u8);
    }

    /// Emit a 32-bit value (little-endian)
    fn emit_u32(&mut self, val: u32) {
        self.bytecode.push((val & 0xff) as u8);
        self.bytecode.push(((val >> 8) & 0xff) as u8);
        self.bytecode.push(((val >> 16) & 0xff) as u8);
        self.bytecode.push((val >> 24) as u8);
    }

    /// Emit a signed 32-bit value (little-endian)
    fn emit_i32(&mut self, val: i32) {
        self.emit_u32(val as u32);
    }

    /// Emit an integer constant, using optimized opcodes when possible
    fn emit_int(&mut self, val: i32) {
        match val {
            -1 => self.emit_op(OpCode::PushMinus1),
            0 => self.emit_op(OpCode::Push0),
            1 => self.emit_op(OpCode::Push1),
            2 => self.emit_op(OpCode::Push2),
            3 => self.emit_op(OpCode::Push3),
            4 => self.emit_op(OpCode::Push4),
            5 => self.emit_op(OpCode::Push5),
            6 => self.emit_op(OpCode::Push6),
            7 => self.emit_op(OpCode::Push7),
            v if v >= i8::MIN as i32 && v <= i8::MAX as i32 => {
                self.emit_op(OpCode::PushI8);
                self.emit_byte(v as i8 as u8);
            }
            v if v >= i16::MIN as i32 && v <= i16::MAX as i32 => {
                self.emit_op(OpCode::PushI16);
                self.emit_u16(v as i16 as u16);
            }
            _ => {
                let idx = self.add_constant(Value::int(val));
                self.emit_const(idx);
            }
        }
    }

    /// Emit a constant load instruction
    fn emit_const(&mut self, index: u16) {
        if index < 256 {
            self.emit_op(OpCode::PushConst8);
            self.emit_byte(index as u8);
        } else {
            self.emit_op(OpCode::PushConst);
            self.emit_u16(index);
        }
    }

    fn plain_string_const_index(&mut self, s: String) -> u16 {
        let idx = self.string_constants.len() as u16;
        self.string_constants.push(s);
        idx
    }

    fn string_const_content(&self, idx: u16) -> Option<&str> {
        if let Some(s) = crate::value::get_builtin_string(idx) {
            Some(s)
        } else {
            self.string_constants.get(idx as usize).map(|s| s.as_str())
        }
    }

    fn rewrite_last_add_const_string_right(&mut self, right_idx: u16) -> bool {
        let len = self.bytecode.len();
        if len < 3 || self.bytecode[len - 3] != OpCode::AddConstStringRight as u8 {
            return false;
        }
        let existing_idx = u16::from_le_bytes([self.bytecode[len - 2], self.bytecode[len - 1]]);
        let Some(existing) = self.string_const_content(existing_idx) else {
            return false;
        };
        let Some(right) = self.string_const_content(right_idx) else {
            return false;
        };
        let combined_idx = self.plain_string_const_index(format!("{}{}", existing, right));
        self.bytecode[len - 2] = (combined_idx & 0xff) as u8;
        self.bytecode[len - 1] = (combined_idx >> 8) as u8;
        true
    }

    fn rewrite_last_add_const_string_surround(&mut self, right_idx: u16) -> bool {
        let len = self.bytecode.len();
        if len < 5 || self.bytecode[len - 5] != OpCode::AddConstStringSurround as u8 {
            return false;
        }
        let left_idx = u16::from_le_bytes([self.bytecode[len - 4], self.bytecode[len - 3]]);
        let existing_right_idx =
            u16::from_le_bytes([self.bytecode[len - 2], self.bytecode[len - 1]]);
        let Some(existing_right) = self.string_const_content(existing_right_idx) else {
            return false;
        };
        let Some(right) = self.string_const_content(right_idx) else {
            return false;
        };
        let combined_right_idx =
            self.plain_string_const_index(format!("{}{}", existing_right, right));
        self.bytecode[len - 4] = (left_idx & 0xff) as u8;
        self.bytecode[len - 3] = (left_idx >> 8) as u8;
        self.bytecode[len - 2] = (combined_right_idx & 0xff) as u8;
        self.bytecode[len - 1] = (combined_right_idx >> 8) as u8;
        true
    }

    fn try_merge_trailing_concat_with_right_const(&mut self, right_idx: u16) -> bool {
        let len = self.bytecode.len();
        if len >= 3 && self.bytecode[len - 3] == OpCode::AddConstStringLeft as u8 {
            let left_idx = u16::from_le_bytes([self.bytecode[len - 2], self.bytecode[len - 1]]);
            self.bytecode.truncate(len - 3);
            self.emit_op(OpCode::AddConstStringSurround);
            self.emit_u16(left_idx);
            self.emit_u16(right_idx);
            return true;
        }

        self.rewrite_last_add_const_string_right(right_idx)
            || self.rewrite_last_add_const_string_surround(right_idx)
    }

    fn try_rewrite_discarded_local0_string_concat(&mut self) -> bool {
        let len = self.bytecode.len();
        if len < 6
            || self.bytecode[len - 1] != OpCode::PutLoc0 as u8
            || self.bytecode[len - 2] != OpCode::Dup as u8
            || self.bytecode[len - 6] != OpCode::GetLoc0 as u8
            || self.bytecode[len - 5] != OpCode::AddConstStringRight as u8
        {
            return false;
        }

        let str_idx = u16::from_le_bytes([self.bytecode[len - 4], self.bytecode[len - 3]]);
        self.bytecode.truncate(len - 6);
        self.emit_op(OpCode::AppendConstStringToLoc0);
        self.emit_u16(str_idx);
        true
    }

    fn try_rewrite_discarded_put_array_false(&mut self) -> bool {
        let len = self.bytecode.len();
        if len < 2
            || self.bytecode[len - 1] != OpCode::PutArrayEl as u8
            || self.bytecode[len - 2] != OpCode::PushFalse as u8
        {
            return false;
        }

        self.bytecode.truncate(len - 2);
        self.emit_op(OpCode::PutArrayElFalseDrop);
        true
    }

    fn try_rewrite_discarded_get_array_el(&mut self) -> bool {
        let len = self.bytecode.len();
        if len < 1 || self.bytecode[len - 1] != OpCode::GetArrayEl as u8 {
            return false;
        }

        self.bytecode.truncate(len - 1);
        self.emit_op(OpCode::GetArrayElDiscard);
        true
    }

    fn try_rewrite_get_field_chain4(&mut self) -> bool {
        let len = self.bytecode.len();
        if len < 12 {
            return false;
        }

        let tail = &self.bytecode;
        let base = len - 12;
        if tail[base] != OpCode::GetField as u8
            || tail[base + 3] != OpCode::GetField as u8
            || tail[base + 6] != OpCode::GetField as u8
            || tail[base + 9] != OpCode::GetField as u8
        {
            return false;
        }

        let a = u16::from_le_bytes([tail[base + 1], tail[base + 2]]);
        let b = u16::from_le_bytes([tail[base + 4], tail[base + 5]]);
        let c = u16::from_le_bytes([tail[base + 7], tail[base + 8]]);
        let d = u16::from_le_bytes([tail[base + 10], tail[base + 11]]);

        self.bytecode.truncate(base);
        self.emit_op(OpCode::GetFieldChain4);
        self.emit_u16(a);
        self.emit_u16(b);
        self.emit_u16(c);
        self.emit_u16(d);
        true
    }

    fn bytecode_tail_is_getglobal_name(&self, name: &str) -> bool {
        let len = self.bytecode.len();
        if len < 3 || self.bytecode[len - 3] != OpCode::GetGlobal as u8 {
            return false;
        }
        let idx = u16::from_le_bytes([self.bytecode[len - 2], self.bytecode[len - 1]]) as usize;
        self.string_constants.get(idx).is_some_and(|s| s == name)
    }

    fn try_rewrite_discarded_local_inc_one(&mut self) -> bool {
        let len = self.bytecode.len();
        if len < 5 {
            return false;
        }

        let tail = &self.bytecode;

        let rewrite_short = |this: &mut Self, start: usize, op: OpCode| {
            this.bytecode.truncate(start);
            this.emit_op(op);
        };

        if tail[len - 1] == OpCode::PutLoc0 as u8
            && tail[len - 2] == OpCode::Dup as u8
            && tail[len - 3] == OpCode::Add as u8
            && tail[len - 4] == OpCode::Push1 as u8
            && tail[len - 5] == OpCode::GetLoc0 as u8
        {
            rewrite_short(self, len - 5, OpCode::IncLoc0Drop);
            return true;
        }
        if tail[len - 1] == OpCode::PutLoc1 as u8
            && tail[len - 2] == OpCode::Dup as u8
            && tail[len - 3] == OpCode::Add as u8
            && tail[len - 4] == OpCode::Push1 as u8
            && tail[len - 5] == OpCode::GetLoc1 as u8
        {
            rewrite_short(self, len - 5, OpCode::IncLoc1Drop);
            return true;
        }
        if tail[len - 1] == OpCode::PutLoc2 as u8
            && tail[len - 2] == OpCode::Dup as u8
            && tail[len - 3] == OpCode::Add as u8
            && tail[len - 4] == OpCode::Push1 as u8
            && tail[len - 5] == OpCode::GetLoc2 as u8
        {
            rewrite_short(self, len - 5, OpCode::IncLoc2Drop);
            return true;
        }
        if tail[len - 1] == OpCode::PutLoc3 as u8
            && tail[len - 2] == OpCode::Dup as u8
            && tail[len - 3] == OpCode::Add as u8
            && tail[len - 4] == OpCode::Push1 as u8
            && tail[len - 5] == OpCode::GetLoc3 as u8
        {
            rewrite_short(self, len - 5, OpCode::IncLoc3Drop);
            return true;
        }
        if tail[len - 1] == OpCode::PutLoc4 as u8
            && tail[len - 2] == OpCode::Dup as u8
            && tail[len - 3] == OpCode::Add as u8
            && tail[len - 4] == OpCode::Push1 as u8
            && tail[len - 5] == OpCode::GetLoc4 as u8
        {
            rewrite_short(self, len - 5, OpCode::IncLoc4Drop);
            return true;
        }

        if len >= 7
            && tail[len - 1] == tail[len - 6]
            && tail[len - 2] == OpCode::PutLoc8 as u8
            && tail[len - 3] == OpCode::Dup as u8
            && tail[len - 4] == OpCode::Add as u8
            && tail[len - 5] == OpCode::Push1 as u8
            && tail[len - 7] == OpCode::GetLoc8 as u8
        {
            let idx = tail[len - 1];
            self.bytecode.truncate(len - 7);
            self.emit_op(OpCode::IncLoc8Drop);
            self.emit_byte(idx);
            return true;
        }

        false
    }

    /// Emit local variable get
    ///
    /// Captured locals still compile to local slot ops in the owning function.
    /// The interpreter redirects those ops through shared cells at runtime.
    fn emit_get_local(&mut self, index: usize) {
        match index {
            0 => self.emit_op(OpCode::GetLoc0),
            1 => self.emit_op(OpCode::GetLoc1),
            2 => self.emit_op(OpCode::GetLoc2),
            3 => self.emit_op(OpCode::GetLoc3),
            4 => self.emit_op(OpCode::GetLoc4),
            i if i < 256 => {
                self.emit_op(OpCode::GetLoc8);
                self.emit_byte(i as u8);
            }
            i => {
                self.emit_op(OpCode::GetLoc);
                self.emit_u16(i as u16);
            }
        }
    }

    /// Emit local variable set
    ///
    /// Captured locals still compile to local slot ops in the owning function.
    /// The interpreter redirects those ops through shared cells at runtime.
    fn emit_set_local(&mut self, index: usize) {
        match index {
            0 => self.emit_op(OpCode::PutLoc0),
            1 => self.emit_op(OpCode::PutLoc1),
            2 => self.emit_op(OpCode::PutLoc2),
            3 => self.emit_op(OpCode::PutLoc3),
            4 => self.emit_op(OpCode::PutLoc4),
            i if i < 256 => {
                self.emit_op(OpCode::PutLoc8);
                self.emit_byte(i as u8);
            }
            i => {
                self.emit_op(OpCode::PutLoc);
                self.emit_u16(i as u16);
            }
        }
    }

    /// Emit get capture instruction
    fn emit_get_capture(&mut self, index: usize) {
        self.emit_op(OpCode::GetVarRef);
        self.emit_u16(index as u16);
    }

    /// Emit set capture instruction
    fn emit_set_capture(&mut self, index: usize) {
        self.emit_op(OpCode::PutVarRef);
        self.emit_u16(index as u16);
    }

    /// Emit get global instruction for builtin functions
    fn emit_get_global(&mut self, name: &str) {
        // Add the name as a string constant
        let str_idx = self.string_constants.len() as u16;
        self.string_constants.push(name.to_string());
        let const_idx = self.add_constant(Value::string(str_idx));
        self.emit_op(OpCode::GetGlobal);
        self.emit_u16(const_idx);
    }

    /// Emit a jump instruction and return the patch location
    fn emit_jump(&mut self, op: OpCode) -> JumpPatch {
        self.emit_op(op);
        let offset = self.bytecode.len();
        self.emit_u32(0); // Placeholder
        JumpPatch { offset }
    }

    fn emit_switch_case_i8_jump(&mut self, val: i8) -> JumpPatch {
        self.emit_op(OpCode::SwitchCaseI8);
        self.emit_byte(val as u8);
        let offset = self.bytecode.len();
        self.emit_u32(0); // Placeholder
        JumpPatch { offset }
    }

    /// Patch a jump instruction to jump to the current position
    fn patch_jump(&mut self, patch: JumpPatch) {
        let target = self.bytecode.len() as i32;
        let jump_end = (patch.offset + 4) as i32;
        let offset = target - jump_end;

        self.bytecode[patch.offset] = (offset & 0xff) as u8;
        self.bytecode[patch.offset + 1] = ((offset >> 8) & 0xff) as u8;
        self.bytecode[patch.offset + 2] = ((offset >> 16) & 0xff) as u8;
        self.bytecode[patch.offset + 3] = ((offset >> 24) & 0xff) as u8;
    }

    /// Patch a jump instruction to jump to a specific bytecode offset.
    fn patch_jump_to_target(&mut self, patch: JumpPatch, target: usize) {
        let target = target as i32;
        let jump_end = (patch.offset + 4) as i32;
        let offset = target - jump_end;

        self.bytecode[patch.offset] = (offset & 0xff) as u8;
        self.bytecode[patch.offset + 1] = ((offset >> 8) & 0xff) as u8;
        self.bytecode[patch.offset + 2] = ((offset >> 16) & 0xff) as u8;
        self.bytecode[patch.offset + 3] = ((offset >> 24) & 0xff) as u8;
    }

    /// Patch a 32-bit value at the given offset
    fn patch_i32(&mut self, offset: usize, val: i32) {
        let bytes = val.to_le_bytes();
        self.bytecode[offset] = bytes[0];
        self.bytecode[offset + 1] = bytes[1];
        self.bytecode[offset + 2] = bytes[2];
        self.bytecode[offset + 3] = bytes[3];
    }

    /// Emit a loop back to a previous position
    fn emit_loop(&mut self, loop_start: usize) {
        self.emit_op(OpCode::Goto);
        let current = self.bytecode.len() + 4; // After the 4-byte offset
        let offset = (loop_start as i32) - (current as i32);
        self.emit_u32(offset as u32);
    }

    /// Add a constant to the pool and return its index
    fn add_constant(&mut self, value: Value) -> u16 {
        // Check for existing identical constant
        for (i, c) in self.constants.iter().enumerate() {
            if value.raw().0 == c.raw().0 {
                return i as u16;
            }
        }

        if self.constants.len() >= MAX_CONSTANTS {
            self.error("Too many constants");
            return 0;
        }

        let index = self.constants.len();
        self.constants.push(value);
        index as u16
    }

    /// Current bytecode offset
    fn current_offset(&self) -> usize {
        self.bytecode.len()
    }

    // =========================================================================
    // Variable handling
    // =========================================================================

    /// Declare a local variable
    fn declare_local(&mut self, name: &str) -> Result<usize, CompileError> {
        // Check for redeclaration in same scope
        for local in self.locals.iter().rev() {
            if local.depth < self.scope_depth {
                break;
            }
            if local.name == name {
                return Err(self.syntax_error(format!(
                    "Variable '{}' already declared in this scope",
                    name
                )));
            }
        }

        if self.locals.len() >= MAX_LOCALS {
            return Err(CompileError::TooManyLocals);
        }

        let index = self.locals.len();
        self.locals.push(Local {
            name: name.to_string(),
            depth: self.scope_depth,
        });

        // Track maximum locals for frame allocation
        if self.locals.len() > self.max_locals {
            self.max_locals = self.locals.len();
        }

        Ok(index)
    }

    /// Resolve a local variable, returning its index
    fn resolve_local(&self, name: &str) -> Option<usize> {
        for (i, local) in self.locals.iter().enumerate().rev() {
            if local.name == name {
                return Some(i);
            }
        }
        None
    }

    /// Resolve a captured variable, returning its capture index
    /// This also adds the capture if it doesn't exist yet
    fn resolve_capture(&mut self, name: &str) -> Option<usize> {
        // First check if we already have this capture
        for (i, capture) in self.captures.iter().enumerate() {
            if capture.name == name {
                return Some(i);
            }
        }

        // Try to resolve from outer function's locals
        if let Some(outer_locals) = self.outer_locals.as_mut() {
            for (i, local) in outer_locals.iter().enumerate().rev() {
                if local.name == name {
                    // Add a new capture
                    let capture_idx = self.captures.len();
                    self.captures.push(Capture {
                        name: name.to_string(),
                        outer_index: i,
                        is_local: true,
                    });
                    return Some(capture_idx);
                }
            }
        }

        // Try to resolve from outer function's captures (nested closure)
        let outer_capture_index = self.outer_captures.as_ref().and_then(|outer_captures| {
            outer_captures
                .iter()
                .enumerate()
                .find_map(|(i, capture)| (capture.name == name).then_some(i))
        });
        if let Some(i) = outer_capture_index {
            // Add a new capture that references outer's capture
            let capture_idx = self.captures.len();
            self.captures.push(Capture {
                name: name.to_string(),
                outer_index: i,
                is_local: false,
            });
            return Some(capture_idx);
        }

        None
    }

    /// Begin a new scope
    fn begin_scope(&mut self) {
        self.scope_depth += 1;
    }

    /// End the current scope
    fn end_scope(&mut self) {
        self.scope_depth -= 1;

        // Remove locals from ended scope from our tracking
        // Note: We don't emit Drop because locals are stored in fixed frame slots,
        // not on the value stack. The frame cleanup happens on function return.
        while let Some(local) = self.locals.last() {
            if local.depth <= self.scope_depth {
                break;
            }
            self.locals.pop();
        }
    }

    // =========================================================================
    // Statement parsing
    // =========================================================================

    /// Parse a statement
    fn statement(&mut self) -> Result<(), CompileError> {
        match &self.current_token {
            Token::Var => self.var_declaration(),
            Token::Let => self.let_declaration(),
            Token::Const => self.const_declaration(),
            Token::Function => self.function_declaration(),
            Token::If => self.if_statement(),
            Token::While => self.while_statement(),
            Token::Do => self.do_while_statement(),
            Token::For => self.for_statement(),
            Token::Switch => self.switch_statement(),
            Token::Break => self.break_statement(),
            Token::Continue => self.continue_statement(),
            Token::Return => self.return_statement(),
            Token::Print => self.print_statement(),
            Token::Try => self.try_statement(),
            Token::Throw => self.throw_statement(),
            Token::Debugger => self.debugger_statement(),
            Token::LBrace => self.block_statement(),
            _ => self.expression_statement(),
        }
    }

    /// Parse var declaration: var x = expr; or var x = 1, y = 2, z = 3;
    fn var_declaration(&mut self) -> Result<(), CompileError> {
        self.advance(); // consume 'var'

        loop {
            let name = match &self.current_token {
                Token::Ident(s) => s.clone(),
                _ => return Err(self.syntax_error("Expected variable name")),
            };
            self.advance();

            // Declare the variable
            let index = self.declare_local(&name)?;

            // Optional initializer
            if self.match_token(&Token::Eq) {
                self.expression()?;
            } else {
                self.emit_op(OpCode::Undefined);
            }

            self.emit_set_local(index);

            // Check for comma (more declarators)
            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        // Expect semicolon
        self.expect(Token::Semicolon)?;

        Ok(())
    }

    /// Parse let declaration
    fn let_declaration(&mut self) -> Result<(), CompileError> {
        self.var_declaration_impl("let")
    }

    /// Parse const declaration
    fn const_declaration(&mut self) -> Result<(), CompileError> {
        self.var_declaration_impl("const")
    }

    /// Common implementation for var/let/const (supports comma-separated declarators)
    fn var_declaration_impl(&mut self, keyword: &str) -> Result<(), CompileError> {
        self.advance(); // consume keyword

        loop {
            let name = match &self.current_token {
                Token::Ident(s) => s.clone(),
                _ => return Err(self.syntax_error("Expected variable name")),
            };
            self.advance();

            let index = self.declare_local(&name)?;

            if self.match_token(&Token::Eq) {
                self.expression()?;
            } else if keyword == "const" {
                return Err(self.syntax_error(format!(
                    "Missing initializer in const declaration '{}'",
                    name
                )));
            } else {
                self.emit_op(OpCode::Undefined);
            }

            self.emit_set_local(index);

            // Check for comma (more declarators)
            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        self.expect(Token::Semicolon)?;

        Ok(())
    }

    /// Parse function declaration: function name(args) { body }
    fn function_declaration(&mut self) -> Result<(), CompileError> {
        self.advance(); // consume 'function'

        // Get function name
        let name = match &self.current_token {
            Token::Ident(s) => s.clone(),
            _ => return Err(self.syntax_error("Expected function name")),
        };
        self.advance();

        // Declare the function as a local variable
        let func_index = self.declare_local(&name)?;

        // Parse parameter list
        self.expect(Token::LParen)?;
        let mut params: Vec<String> = Vec::new();

        if !self.check(&Token::RParen) {
            loop {
                if let Token::Ident(param_name) = &self.current_token {
                    params.push(param_name.clone());
                    self.advance();
                } else {
                    return Err(self.syntax_error("Expected parameter name"));
                }

                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }
        self.expect(Token::RParen)?;

        // Parse function body
        self.expect(Token::LBrace)?;

        // Compile the function body with a new compiler
        // Pass the function name so it can reference itself for recursion
        let body_bytecode = self.compile_function_body(Some(&name), &params)?;

        // Store the bytecode in functions list
        let bytecode_idx = self.functions.len();
        self.functions.push(body_bytecode);

        // Emit instruction to create function closure
        self.emit_op(OpCode::FClosure);
        self.emit_u16(bytecode_idx as u16);

        // At top-level scope (not inside a function), also register as a global
        // so that subsequent eval() calls can find this function via GetGlobal.
        if self.outer_locals.is_none() && self.scope_depth == 0 {
            let str_idx = self.string_constants.len() as u16;
            self.string_constants.push(name.clone());
            let const_idx = self.add_constant(Value::string(str_idx));
            self.emit_op(OpCode::SetGlobal);
            self.emit_u16(const_idx);
        }

        // Store to local
        self.emit_set_local(func_index);

        Ok(())
    }

    /// Compile a function body
    ///
    /// If `func_name` is provided, the function can reference itself for recursion.
    fn compile_function_body(
        &mut self,
        func_name: Option<&str>,
        params: &[String],
    ) -> Result<CompiledFunction, CompileError> {
        // Save current compiler state
        let saved_bytecode = core::mem::take(&mut self.bytecode);
        let saved_constants = core::mem::take(&mut self.constants);
        let saved_string_constants = core::mem::take(&mut self.string_constants);
        let saved_locals = core::mem::take(&mut self.locals);
        let saved_functions = core::mem::take(&mut self.functions);
        let saved_loop_stack = core::mem::take(&mut self.loop_stack);
        let saved_captures = core::mem::take(&mut self.captures);
        let saved_outer_locals = core::mem::take(&mut self.outer_locals);
        let saved_outer_captures = core::mem::take(&mut self.outer_captures);
        let saved_max_locals = self.max_locals;
        let saved_scope_depth = self.scope_depth;

        // Set outer locals and captures for closure resolution
        // The inner function can capture from our locals
        self.outer_locals = Some(saved_locals.clone());
        self.outer_captures = Some(saved_captures.clone());

        // Reset for function compilation
        self.bytecode = Vec::new();
        self.constants = Vec::new();
        self.string_constants = Vec::new();
        self.locals = Vec::new();
        self.functions = Vec::new();
        self.loop_stack = Vec::new();
        self.captures = Vec::new();
        self.max_locals = 0;
        self.scope_depth = 0;

        let arg_count = params.len();

        // Declare parameters as locals FIRST (they must be at slots 0..arg_count)
        for param in params {
            self.declare_local(param)?;
        }

        // If function has a name, declare it as a local for recursion
        // (after parameters so it doesn't affect argument slot positions)
        // and emit code to initialize it with ThisFunc at the start
        if let Some(name) = func_name {
            let func_slot = self.declare_local(name)?;
            self.emit_op(OpCode::ThisFunc);
            self.emit_set_local(func_slot);
        }

        // Parse function body statements
        while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
            self.statement()?;
        }
        self.expect(Token::RBrace)?;

        // Emit implicit return undefined
        self.emit_op(OpCode::ReturnUndef);

        // Convert captures to CaptureInfo
        let captures: Vec<CaptureInfo> = self
            .captures
            .iter()
            .map(|c| CaptureInfo {
                outer_index: c.outer_index,
                is_local: c.is_local,
            })
            .collect();

        // Create compiled function
        let result = CompiledFunction {
            bytecode: core::mem::take(&mut self.bytecode),
            constants: core::mem::take(&mut self.constants),
            string_constants: core::mem::take(&mut self.string_constants),
            local_count: self.max_locals,
            arg_count,
            functions: core::mem::take(&mut self.functions),
            captures,
        };

        // Restore compiler state
        self.bytecode = saved_bytecode;
        self.constants = saved_constants;
        self.string_constants = saved_string_constants;
        self.locals = saved_locals;
        self.functions = saved_functions;
        self.loop_stack = saved_loop_stack;
        self.captures = saved_captures;
        self.outer_locals = saved_outer_locals;
        self.outer_captures = saved_outer_captures;
        self.max_locals = saved_max_locals;
        self.scope_depth = saved_scope_depth;

        Ok(result)
    }

    /// Parse if statement
    fn if_statement(&mut self) -> Result<(), CompileError> {
        self.advance(); // consume 'if'
        self.expect(Token::LParen)?;
        self.expression()?;
        self.expect(Token::RParen)?;

        // Jump over then branch if condition is false
        let then_jump = self.emit_jump(OpCode::IfFalse);
        let body_start = self.current_offset();

        self.statement()?;

        // Parse else branch if present
        if self.match_token(&Token::Else) {
            // Jump over else branch
            let else_jump = self.emit_jump(OpCode::Goto);
            self.patch_jump(then_jump);
            self.statement()?;
            self.patch_jump(else_jump);
        } else {
            let _ = body_start;
            self.patch_jump(then_jump);
        }

        Ok(())
    }

    /// Parse while statement
    fn while_statement(&mut self) -> Result<(), CompileError> {
        self.advance(); // consume 'while'

        let loop_start = self.current_offset();

        // Push loop context for break/continue
        self.loop_stack.push(LoopContext {
            continue_target: loop_start,
            break_patches: Vec::new(),
            continue_patches: Vec::new(),
            scope_depth: self.scope_depth,
            is_switch: false,
        });

        self.expect(Token::LParen)?;
        self.expression()?;
        self.expect(Token::RParen)?;

        let exit_jump = self.emit_jump(OpCode::IfFalse);

        self.statement()?;

        self.emit_loop(loop_start);
        self.patch_jump(exit_jump);

        // Pop loop context and patch break jumps
        let loop_ctx = self.loop_stack.pop().unwrap();
        for patch in loop_ctx.break_patches {
            self.patch_jump(patch);
        }

        Ok(())
    }

    /// Parse do...while statement: do { body } while (cond);
    fn do_while_statement(&mut self) -> Result<(), CompileError> {
        self.advance(); // consume 'do'

        let loop_start = self.current_offset();

        // Push loop context; continue_target is deferred (set to sentinel)
        // because we don't know the condition offset yet
        self.loop_stack.push(LoopContext {
            continue_target: usize::MAX, // sentinel: will use continue_patches
            break_patches: Vec::new(),
            continue_patches: Vec::new(),
            scope_depth: self.scope_depth,
            is_switch: false,
        });

        // Compile body
        self.statement()?;

        // 'while' keyword
        self.expect(Token::While)?;

        // Patch any continue jumps to here (the condition)
        let condition_start = self.current_offset();
        let loop_ctx = self.loop_stack.last_mut().unwrap();
        let continue_patches: Vec<JumpPatch> = core::mem::take(&mut loop_ctx.continue_patches);
        for patch in continue_patches {
            self.patch_jump(patch);
        }

        // Compile condition
        self.expect(Token::LParen)?;
        self.expression()?;
        self.expect(Token::RParen)?;
        self.expect(Token::Semicolon)?;

        // If condition is true, jump back to loop_start
        let _ = condition_start; // used above for continue patches
        self.emit_op(OpCode::IfTrue);
        let current = self.bytecode.len() + 4;
        let offset = (loop_start as i32) - (current as i32);
        self.emit_u32(offset as u32);

        // Pop loop context and patch break jumps
        let loop_ctx = self.loop_stack.pop().unwrap();
        for patch in loop_ctx.break_patches {
            self.patch_jump(patch);
        }

        Ok(())
    }

    /// Parse for statement (C-style or for-in)
    fn for_statement(&mut self) -> Result<(), CompileError> {
        self.advance(); // consume 'for'
        self.expect(Token::LParen)?;

        self.begin_scope();

        // Check for for-in syntax: for (var/let x in obj)
        if self.check(&Token::Var) || self.check(&Token::Let) {
            self.advance(); // consume var/let

            if let Token::Ident(name) = self.current_token.clone() {
                // Look ahead to check if this is for-in
                // We need to save state and peek at next token
                let saved_name = name.clone();
                self.advance(); // consume identifier

                if self.match_token(&Token::In) {
                    // This is a for-in loop
                    return self.for_in_statement_rest(saved_name);
                }

                if self.match_token(&Token::Of) {
                    // This is a for-of loop
                    return self.for_of_statement_rest(saved_name);
                }

                // Not a for-in loop, restore and continue as C-style for
                // We already consumed var/let and identifier, need to handle initializer
                // Declare local first to get the index
                let index = self.declare_local(&saved_name)?;
                // Check for initializer
                if self.match_token(&Token::Eq) {
                    // Has initializer: var x = expr
                    self.expression()?;
                } else {
                    // No initializer, push undefined
                    self.emit_op(OpCode::Undefined);
                }
                // Store to local
                self.emit_set_local(index);
                self.expect(Token::Semicolon)?;
            } else {
                return Err(self.syntax_error("Expected identifier in for loop".to_string()));
            }
        } else if !self.match_token(&Token::Semicolon) {
            // C-style initializer expression
            self.expression_statement()?;
        }

        // Continue with C-style for loop
        self.for_c_style_rest()
    }

    /// Parse rest of for-in statement after "for (var/let name in"
    fn for_in_statement_rest(&mut self, var_name: String) -> Result<(), CompileError> {
        // Parse the object to iterate over
        self.expression()?;
        self.expect(Token::RParen)?;

        // Create iterator from object - ForInStart converts obj to iterator index
        self.emit_op(OpCode::ForInStart);

        // Declare a hidden local to store the iterator index
        // Use a name that can't conflict with user code
        let iter_slot = self.declare_local("\x00iter")?;
        self.emit_set_local(iter_slot);

        // Declare the loop variable
        self.emit_op(OpCode::Undefined);
        let var_slot = self.declare_local(&var_name)?;
        self.emit_set_local(var_slot);

        // Loop start - get next key
        let loop_start = self.current_offset();

        // Push loop context for break/continue
        self.loop_stack.push(LoopContext {
            continue_target: loop_start,
            break_patches: Vec::new(),
            continue_patches: Vec::new(),
            scope_depth: self.scope_depth,
            is_switch: false,
        });

        // Get iterator from hidden local
        self.emit_get_local(iter_slot);

        // ForInNext: iter -> key done
        self.emit_op(OpCode::ForInNext);

        // Check if done (pops done flag)
        let exit_jump = self.emit_jump(OpCode::IfTrue);

        // Store key to loop variable (PutLoc pops the key)
        self.emit_set_local(var_slot);

        // Body
        self.statement()?;

        // Loop back
        self.emit_loop(loop_start);

        // Exit point
        self.patch_jump(exit_jump);

        // Pop remaining key value (undefined from done iteration)
        self.emit_op(OpCode::Drop);

        // Pop loop context and patch break jumps
        let loop_ctx = self.loop_stack.pop().unwrap();
        for patch in loop_ctx.break_patches {
            self.patch_jump(patch);
        }

        self.end_scope();

        Ok(())
    }

    /// Parse rest of for-of statement after "for (var/let name of"
    fn for_of_statement_rest(&mut self, var_name: String) -> Result<(), CompileError> {
        // Parse the iterable to iterate over
        self.expression()?;
        self.expect(Token::RParen)?;

        // Create iterator from iterable - ForOfStart converts to iterator index
        self.emit_op(OpCode::ForOfStart);

        // Declare a hidden local to store the iterator index
        let iter_slot = self.declare_local("\x00iter")?;
        self.emit_set_local(iter_slot);

        // Declare the loop variable
        self.emit_op(OpCode::Undefined);
        let var_slot = self.declare_local(&var_name)?;
        self.emit_set_local(var_slot);

        // Loop start - get next value
        let loop_start = self.current_offset();

        // Push loop context for break/continue
        self.loop_stack.push(LoopContext {
            continue_target: loop_start,
            break_patches: Vec::new(),
            continue_patches: Vec::new(),
            scope_depth: self.scope_depth,
            is_switch: false,
        });

        // Get iterator from hidden local
        self.emit_get_local(iter_slot);

        // ForOfNext: iter -> value done
        self.emit_op(OpCode::ForOfNext);

        // Check if done (pops done flag)
        let exit_jump = self.emit_jump(OpCode::IfTrue);

        // Store value to loop variable (PutLoc pops the value)
        self.emit_set_local(var_slot);

        // Body
        self.statement()?;

        // Loop back
        self.emit_loop(loop_start);

        // Exit point
        self.patch_jump(exit_jump);

        // Pop remaining value (undefined from done iteration)
        self.emit_op(OpCode::Drop);

        // Pop loop context and patch break jumps
        let loop_ctx = self.loop_stack.pop().unwrap();
        for patch in loop_ctx.break_patches {
            self.patch_jump(patch);
        }

        self.end_scope();

        Ok(())
    }

    /// Parse rest of C-style for loop after initializer
    fn for_c_style_rest(&mut self) -> Result<(), CompileError> {
        let loop_start = self.current_offset();

        // Condition
        let exit_jump = if !self.match_token(&Token::Semicolon) {
            self.expression()?;
            self.expect(Token::Semicolon)?;
            let j = self.emit_jump(OpCode::IfFalse);
            Some(j)
        } else {
            None
        };

        // Increment bytecode is compiled now but appended after the body so
        // the initial iteration no longer needs an extra jump over it.
        let increment_code = if !self.check(&Token::RParen) {
            let start = self.bytecode.len();
            self.expression()?;
            if !self.try_rewrite_discarded_local_inc_one() {
                self.emit_op(OpCode::Drop); // Discard increment result
            }
            Some(self.bytecode.split_off(start))
        } else {
            None
        };

        self.expect(Token::RParen)?;

        // Push loop context for break/continue
        // Continue should jump to the increment section if present, but we
        // only know its final offset after the body has been emitted.
        let continue_target = if increment_code.is_some() {
            usize::MAX
        } else {
            loop_start
        };
        self.loop_stack.push(LoopContext {
            continue_target,
            break_patches: Vec::new(),
            continue_patches: Vec::new(),
            scope_depth: self.scope_depth,
            is_switch: false,
        });

        // Body
        self.statement()?;

        // Append increment section after the body and then jump back to the condition.
        if let Some(code) = increment_code {
            let increment_start = self.current_offset();
            let continue_patches = {
                let ctx = self.loop_stack.last_mut().unwrap();
                core::mem::take(&mut ctx.continue_patches)
            };
            for patch in continue_patches {
                self.patch_jump_to_target(patch, increment_start);
            }
            self.bytecode.extend_from_slice(&code);
            self.emit_loop(loop_start);
        } else {
            self.emit_loop(loop_start);
        }

        // Patch exit jump
        if let Some(j) = exit_jump {
            self.patch_jump(j);
        }

        // Pop loop context and patch break jumps
        let loop_ctx = self.loop_stack.pop().unwrap();
        for patch in loop_ctx.break_patches {
            self.patch_jump(patch);
        }

        self.end_scope();

        Ok(())
    }

    /// Parse break statement
    fn break_statement(&mut self) -> Result<(), CompileError> {
        self.advance(); // consume 'break'

        if self.loop_stack.is_empty() {
            return Err(self.syntax_error("'break' outside of loop or switch".to_string()));
        }

        // Emit jump (will be patched when loop ends)
        let patch = self.emit_jump(OpCode::Goto);

        // Register this jump to be patched
        if let Some(ctx) = self.loop_stack.last_mut() {
            ctx.break_patches.push(patch);
        }

        self.expect(Token::Semicolon)?;
        Ok(())
    }

    /// Parse continue statement
    fn continue_statement(&mut self) -> Result<(), CompileError> {
        self.advance(); // consume 'continue'

        // Find innermost loop context (skip switch contexts)
        let loop_idx = self.loop_stack.iter().rposition(|ctx| !ctx.is_switch);

        let Some(idx) = loop_idx else {
            return Err(self.syntax_error("'continue' outside of loop".to_string()));
        };

        let ctx = &self.loop_stack[idx];
        let continue_target = ctx.continue_target;
        let has_deferred_continue = continue_target == usize::MAX;
        let switch_depth = self.loop_stack[idx + 1..]
            .iter()
            .filter(|ctx| ctx.is_switch)
            .count();

        for _ in 0..switch_depth {
            self.emit_op(OpCode::Drop);
        }

        if has_deferred_continue {
            // do...while: continue target not yet known, emit a patch
            let patch = self.emit_jump(OpCode::Goto);
            self.loop_stack[idx].continue_patches.push(patch);
        } else {
            self.emit_loop(continue_target);
        }

        self.expect(Token::Semicolon)?;
        Ok(())
    }

    /// Parse return statement
    fn return_statement(&mut self) -> Result<(), CompileError> {
        self.advance(); // consume 'return'

        if self.match_token(&Token::Semicolon) {
            self.emit_op(OpCode::ReturnUndef);
        } else {
            self.expression()?;
            self.expect(Token::Semicolon)?;
            self.emit_op(OpCode::Return);
        }

        Ok(())
    }

    /// Parse print statement: print expr;
    fn print_statement(&mut self) -> Result<(), CompileError> {
        self.advance(); // consume 'print'

        self.expression()?;
        self.expect(Token::Semicolon)?;
        self.emit_op(OpCode::Print);

        Ok(())
    }

    /// Parse throw statement: throw expr;
    fn throw_statement(&mut self) -> Result<(), CompileError> {
        self.advance(); // consume 'throw'

        self.expression()?;
        self.expect(Token::Semicolon)?;
        self.emit_op(OpCode::Throw);

        Ok(())
    }

    /// Parse debugger statement (compiled as no-op)
    fn debugger_statement(&mut self) -> Result<(), CompileError> {
        self.advance(); // consume 'debugger'
        self.expect(Token::Semicolon)?;
        self.emit_op(OpCode::Nop);
        Ok(())
    }

    /// Parse switch statement: switch (expr) { case val: ...; default: ...; }
    ///
    /// Two-pass compilation with lexer save/restore for correct fall-through:
    ///   Pass 1: compile case expressions into a comparison chain (skip bodies)
    ///   Pass 2: restore lexer, skip expressions, compile bodies in order
    ///
    /// Bytecode layout:
    ///   <switch_expr>
    ///   Dup; <case1_expr>; StrictEq; IfTrue -> case1_body
    ///   Dup; <case2_expr>; StrictEq; IfTrue -> case2_body
    ///   Goto -> default_body | exit
    ///   case1_body: <stmts>   ← falls through to case2_body
    ///   case2_body: <stmts>
    ///   default_body: <stmts>
    ///   exit: Drop
    fn switch_statement(&mut self) -> Result<(), CompileError> {
        self.advance(); // consume 'switch'

        self.expect(Token::LParen)?;
        self.expression()?; // switch value on stack
        self.expect(Token::RParen)?;
        self.expect(Token::LBrace)?;

        // Push switch context for break
        self.loop_stack.push(LoopContext {
            continue_target: 0,
            break_patches: Vec::new(),
            continue_patches: Vec::new(),
            scope_depth: self.scope_depth,
            is_switch: true,
        });

        // Save lexer state for pass 2
        let saved_lexer = self.lexer.clone();
        let saved_token = self.current_token.clone();

        // Pass 1: Emit comparison chain, skip bodies
        let mut case_jumps: Vec<JumpPatch> = Vec::new();
        let mut has_default = false;

        loop {
            match &self.current_token {
                Token::Case => {
                    self.advance();
                    if let Token::Number(n) = self.current_token.clone() {
                        let int = n as i32;
                        if n == (int as f64) && int >= i8::MIN as i32 && int <= i8::MAX as i32 {
                            case_jumps.push(self.emit_switch_case_i8_jump(int as i8));
                            self.advance();
                        } else {
                            self.emit_op(OpCode::Dup);
                            self.expression()?;
                            self.emit_op(OpCode::StrictEq);
                            case_jumps.push(self.emit_jump(OpCode::IfTrue));
                        }
                    } else {
                        self.emit_op(OpCode::Dup);
                        self.expression()?;
                        self.emit_op(OpCode::StrictEq);
                        case_jumps.push(self.emit_jump(OpCode::IfTrue));
                    }
                    self.expect(Token::Colon)?;
                    self.skip_case_body();
                }
                Token::Default => {
                    self.advance();
                    self.expect(Token::Colon)?;
                    has_default = true;
                    case_jumps.push(self.emit_jump(OpCode::Goto));
                    self.skip_case_body();
                }
                Token::RBrace => break,
                Token::Eof => {
                    return Err(self.syntax_error("Unexpected EOF in switch".to_string()));
                }
                _ => {
                    return Err(self.syntax_error("Expected 'case', 'default', or '}'".to_string()));
                }
            }
        }

        // If no default clause, emit jump to exit
        let no_match_exit = if !has_default {
            Some(self.emit_jump(OpCode::Goto))
        } else {
            None
        };

        // Pass 2: Restore lexer, compile bodies
        self.lexer = saved_lexer;
        self.current_token = saved_token;

        let mut jump_idx = 0;
        loop {
            match &self.current_token {
                Token::Case => {
                    self.advance();
                    self.skip_to_colon();
                    self.expect(Token::Colon)?;
                    self.patch_jump(case_jumps[jump_idx]);
                    jump_idx += 1;
                    self.compile_case_body()?;
                }
                Token::Default => {
                    self.advance();
                    self.expect(Token::Colon)?;
                    self.patch_jump(case_jumps[jump_idx]);
                    jump_idx += 1;
                    self.compile_case_body()?;
                }
                Token::RBrace => break,
                Token::Eof => {
                    return Err(self.syntax_error("Unexpected EOF in switch".to_string()));
                }
                _ => {
                    return Err(self.syntax_error("Expected 'case', 'default', or '}'".to_string()));
                }
            }
        }

        self.expect(Token::RBrace)?;

        if let Some(exit) = no_match_exit {
            self.patch_jump(exit);
        }

        let switch_ctx = self.loop_stack.pop().unwrap();
        for patch in switch_ctx.break_patches {
            self.patch_jump(patch);
        }

        self.emit_op(OpCode::Drop); // drop switch value
        Ok(())
    }

    /// Skip tokens in a case body (pass 1), respecting brace nesting.
    fn skip_case_body(&mut self) {
        let mut depth = 0u32;
        loop {
            match &self.current_token {
                Token::LBrace => {
                    depth += 1;
                    self.advance();
                }
                Token::RBrace if depth > 0 => {
                    depth -= 1;
                    self.advance();
                }
                Token::RBrace | Token::Case | Token::Default if depth == 0 => break,
                Token::Eof => break,
                _ => {
                    self.advance();
                }
            }
        }
    }

    /// Skip tokens until a colon at depth 0 (used in pass 2 to skip case expressions).
    fn skip_to_colon(&mut self) {
        let mut depth = 0u32;
        let mut ternary_depth = 0u32;
        loop {
            match &self.current_token {
                Token::Colon if depth == 0 && ternary_depth == 0 => break,
                Token::Question if depth == 0 => {
                    ternary_depth += 1;
                    self.advance();
                }
                Token::Colon if depth == 0 && ternary_depth > 0 => {
                    ternary_depth -= 1;
                    self.advance();
                }
                Token::LParen | Token::LBracket => {
                    depth += 1;
                    self.advance();
                }
                Token::RParen | Token::RBracket if depth > 0 => {
                    depth -= 1;
                    self.advance();
                }
                Token::Eof => break,
                _ => {
                    self.advance();
                }
            }
        }
    }

    /// Compile case body statements until next case/default/} at top level.
    fn compile_case_body(&mut self) -> Result<(), CompileError> {
        while !self.check(&Token::Case)
            && !self.check(&Token::Default)
            && !self.check(&Token::RBrace)
            && !self.check(&Token::Eof)
        {
            self.statement()?;
        }
        Ok(())
    }

    fn parse_finally_clause(&mut self) -> Result<(), CompileError> {
        self.expect(Token::Finally)?;
        if !self.check(&Token::LBrace) {
            return Err(self.syntax_error("Expected '{' after finally"));
        }
        self.block_statement()
    }

    /// Parse try-catch-finally statement
    fn try_statement(&mut self) -> Result<(), CompileError> {
        self.advance(); // consume 'try'

        // Expect '{'
        if !self.check(&Token::LBrace) {
            return Err(self.syntax_error("Expected '{' after 'try'"));
        }

        // Emit Catch opcode with placeholder offset
        self.emit_op(OpCode::Catch);
        let catch_jump = self.bytecode.len();
        self.emit_i32(0); // placeholder

        // Parse try block
        self.block_statement()?;

        // If we get here without exception, remove the handler
        self.emit_op(OpCode::DropCatch);

        // Jump over catch block
        let end_jump = self.emit_jump(OpCode::Goto);

        // Patch catch jump target (points here - start of catch block)
        let catch_target = self.bytecode.len() as i32;
        let catch_offset = catch_target - (catch_jump as i32 + 4);
        self.patch_i32(catch_jump, catch_offset);

        // Check for catch clause
        let has_catch = self.check(&Token::Catch);
        if has_catch {
            self.advance(); // consume 'catch'

            // Optional (e) parameter
            if self.match_token(&Token::LParen) {
                let name = match &self.current_token {
                    Token::Ident(s) => s.clone(),
                    _ => {
                        return Err(self.syntax_error("Expected catch variable name"));
                    }
                };
                self.advance();
                self.expect(Token::RParen)?;

                // Begin a new scope for the catch variable
                self.begin_scope();

                // Declare the catch variable
                let index = self.declare_local(&name)?;

                // The exception value is on the stack from Catch opcode
                // Store it in the catch variable
                self.emit_set_local(index);

                // Parse catch body
                if !self.check(&Token::LBrace) {
                    return Err(self.syntax_error("Expected '{' after catch"));
                }
                self.block_statement()?;

                self.end_scope();
            } else {
                // No parameter - just discard the exception value
                self.emit_op(OpCode::Drop);

                // Parse catch body
                if !self.check(&Token::LBrace) {
                    return Err(self.syntax_error("Expected '{' after catch"));
                }
                self.block_statement()?;
            }
        } else {
            // No catch clause. If there is a finally block, duplicate it on the
            // exception path before re-throwing the original exception value.
        }

        let finally_state = if self.check(&Token::Finally) {
            Some((
                self.lexer.clone(),
                self.current_token.clone(),
                self.previous_token.clone(),
                self.current_pos,
            ))
        } else {
            None
        };

        if !has_catch {
            if finally_state.is_some() {
                self.parse_finally_clause()?;
            }
            self.emit_op(OpCode::Throw);
        }

        self.patch_jump(end_jump);

        if let Some((lexer, current_token, previous_token, current_pos)) = finally_state {
            self.lexer = lexer;
            self.current_token = current_token;
            self.previous_token = previous_token;
            self.current_pos = current_pos;
            self.parse_finally_clause()?;
        }

        Ok(())
    }

    /// Parse block statement
    fn block_statement(&mut self) -> Result<(), CompileError> {
        self.advance(); // consume '{'
        self.begin_scope();

        while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
            self.statement()?;
        }

        self.expect(Token::RBrace)?;
        self.end_scope();

        Ok(())
    }

    /// Parse expression statement
    fn expression_statement(&mut self) -> Result<(), CompileError> {
        self.expression()?;
        self.expect(Token::Semicolon)?;
        if !self.try_rewrite_discarded_local0_string_concat()
            && !self.try_rewrite_discarded_local_inc_one()
            && !self.try_rewrite_discarded_get_array_el()
            && !self.try_rewrite_discarded_put_array_false()
        {
            self.emit_op(OpCode::Drop); // Discard expression value
        }
        Ok(())
    }

    // =========================================================================
    // Expression parsing (precedence climbing)
    // =========================================================================

    /// Parse an expression
    fn expression(&mut self) -> Result<(), CompileError> {
        self.parse_precedence(Precedence::Assignment)
    }

    /// Parse expression with given minimum precedence
    fn parse_precedence(&mut self, min_prec: Precedence) -> Result<(), CompileError> {
        // Parse prefix expression
        self.prefix_expr()?;

        // Parse infix expressions at or above min precedence
        while let Some((prec, assoc)) = self.infix_precedence() {
            if prec < min_prec {
                break;
            }

            let left_const_string = self.last_expr_string_const.take();
            self.last_expr_bool_const = None;
            let left_concat_surround = self.last_expr_concat_surround.take();
            let op = self.current_token.clone();
            self.advance();

            // Handle assignment specially
            if prec == Precedence::Assignment {
                self.assignment_expr(&op)?;
                self.last_expr_string_const = None;
                self.last_expr_bool_const = None;
                self.last_expr_concat_surround = None;
                continue;
            }

            // Handle ternary operator
            if matches!(op, Token::Question) {
                self.ternary_expr()?;
                self.last_expr_string_const = None;
                self.last_expr_bool_const = None;
                self.last_expr_concat_surround = None;
                continue;
            }

            // Handle short-circuit operators
            if matches!(op, Token::AmpAmp | Token::PipePipe) {
                self.short_circuit_expr(&op)?;
                self.last_expr_string_const = None;
                self.last_expr_bool_const = None;
                self.last_expr_concat_surround = None;
                continue;
            }

            // Right-associative: use same precedence; left-associative: use next higher
            let next_prec = if assoc == Associativity::Right {
                prec
            } else {
                prec.next()
            };

            self.parse_precedence(next_prec)?;
            let right_const_string = self.last_expr_string_const.take();
            self.emit_binary_op(
                &op,
                left_const_string,
                left_concat_surround,
                right_const_string,
            )?;
        }

        Ok(())
    }

    /// Parse prefix expression (unary, literals, grouping)
    fn prefix_expr(&mut self) -> Result<(), CompileError> {
        // Clear last variable reference (set when we parse an identifier)
        self.last_var_ref = None;
        self.last_expr_string_const = None;
        self.last_expr_bool_const = None;
        match &self.current_token {
            // Literals
            Token::Number(n) => {
                let n = *n;
                self.advance();
                // Check if it's an integer that fits in short int range
                if (n - libm::trunc(n)) == 0.0
                    && n >= -(1i64 << 30) as f64
                    && n <= ((1i64 << 30) - 1) as f64
                {
                    self.emit_int(n as i32);
                } else {
                    // Emit as float constant
                    let idx = self.add_constant(Value::float(n as crate::value::Float));
                    self.emit_const(idx);
                }
            }
            Token::String(s) => {
                let s = s.clone();
                self.advance();
                let start = self.bytecode.len();
                if s.is_empty() {
                    self.emit_op(OpCode::PushEmptyString);
                    self.last_expr_string_const = Some((crate::value::STR_EMPTY, start));
                } else {
                    // Check for built-in strings used by typeof
                    use crate::value::{
                        STR_BOOLEAN, STR_FUNCTION, STR_NUMBER, STR_OBJECT, STR_STRING,
                        STR_UNDEFINED,
                    };
                    let builtin_idx = match s.as_str() {
                        "undefined" => Some(STR_UNDEFINED),
                        "object" => Some(STR_OBJECT),
                        "boolean" => Some(STR_BOOLEAN),
                        "number" => Some(STR_NUMBER),
                        "function" => Some(STR_FUNCTION),
                        "string" => Some(STR_STRING),
                        _ => None,
                    };

                    if let Some(idx) = builtin_idx {
                        // Use built-in string constant for typeof comparison
                        self.emit_op(OpCode::PushConst);
                        let const_idx = self.add_constant(Value::string(idx));
                        self.emit_u16(const_idx);
                        self.last_expr_string_const = Some((idx, start));
                    } else {
                        // Store string in string constant pool
                        let idx = self.string_constants.len() as u16;
                        self.string_constants.push(s);
                        // Emit PushConst with a string value
                        self.emit_op(OpCode::PushConst);
                        // We'll encode this as a Value::string(idx) in the constants
                        let const_idx = self.add_constant(Value::string(idx));
                        self.emit_u16(const_idx);
                        self.last_expr_string_const = Some((idx, start));
                    }
                }
            }
            Token::True => {
                let start = self.bytecode.len();
                self.advance();
                self.emit_op(OpCode::PushTrue);
                self.last_expr_bool_const = Some((true, start));
            }
            Token::False => {
                let start = self.bytecode.len();
                self.advance();
                self.emit_op(OpCode::PushFalse);
                self.last_expr_bool_const = Some((false, start));
            }
            Token::Null => {
                self.advance();
                self.emit_op(OpCode::Null);
            }
            Token::This => {
                self.advance();
                self.emit_op(OpCode::PushThis);
            }

            // Function expression: function(args) { body } or function name(args) { body }
            Token::Function => {
                self.advance(); // consume 'function'

                // Optional function name (for named function expressions / recursion)
                let func_name = if let Token::Ident(s) = &self.current_token {
                    let name = s.clone();
                    self.advance();
                    Some(name)
                } else {
                    None
                };

                // Parse parameter list
                self.expect(Token::LParen)?;
                let mut params: Vec<String> = Vec::new();

                if !self.check(&Token::RParen) {
                    loop {
                        if let Token::Ident(param_name) = &self.current_token {
                            params.push(param_name.clone());
                            self.advance();
                        } else {
                            return Err(self.syntax_error("Expected parameter name"));
                        }
                        if !self.match_token(&Token::Comma) {
                            break;
                        }
                    }
                }
                self.expect(Token::RParen)?;

                // Parse function body
                self.expect(Token::LBrace)?;
                let body_bytecode = self.compile_function_body(func_name.as_deref(), &params)?;

                let bytecode_idx = self.functions.len();
                self.functions.push(body_bytecode);

                // Emit instruction to create function closure (value stays on stack)
                self.emit_op(OpCode::FClosure);
                self.emit_u16(bytecode_idx as u16);
            }

            // Identifiers (variables)
            Token::Ident(name) => {
                let name = name.clone();
                self.advance();

                // Check for assignment
                if self.is_assignment_op() {
                    let op = self.current_token.clone();
                    self.advance();

                    // Resolve variable: local, capture, or error
                    let (is_local, idx) = if let Some(idx) = self.resolve_local(&name) {
                        (true, idx)
                    } else if let Some(idx) = self.resolve_capture(&name) {
                        (false, idx)
                    } else {
                        return Err(self.syntax_error(format!("Undefined variable '{}'", name)));
                    };

                    // For compound assignment (+=, -=, etc.), get the current value first
                    if !matches!(op, Token::Eq) {
                        if is_local {
                            self.emit_get_local(idx);
                        } else {
                            self.emit_get_capture(idx);
                        }
                    }

                    // Parse the right-hand side
                    self.parse_precedence(Precedence::Assignment)?;

                    // For compound assignment, apply the operation
                    match op {
                        Token::Eq => {}
                        Token::PlusEq => self.emit_op(OpCode::Add),
                        Token::MinusEq => self.emit_op(OpCode::Sub),
                        Token::StarEq => self.emit_op(OpCode::Mul),
                        Token::SlashEq => self.emit_op(OpCode::Div),
                        Token::PercentEq => self.emit_op(OpCode::Mod),
                        Token::AmpEq => self.emit_op(OpCode::And),
                        Token::PipeEq => self.emit_op(OpCode::Or),
                        Token::CaretEq => self.emit_op(OpCode::Xor),
                        Token::LtLtEq => self.emit_op(OpCode::Shl),
                        Token::GtGtEq => self.emit_op(OpCode::Sar),
                        Token::GtGtGtEq => self.emit_op(OpCode::Shr),
                        Token::StarStarEq => self.emit_op(OpCode::Pow),
                        _ => {}
                    }

                    // Duplicate value (for expression result) and store
                    self.emit_op(OpCode::Dup);
                    if is_local {
                        self.emit_set_local(idx);
                    } else {
                        self.emit_set_capture(idx);
                    }
                } else if let Some(idx) = self.resolve_local(&name) {
                    self.emit_get_local(idx);
                    self.last_var_ref = Some((true, idx));
                } else if let Some(idx) = self.resolve_capture(&name) {
                    self.emit_get_capture(idx);
                    self.last_var_ref = Some((false, idx));
                } else {
                    // Try as a global (builtin function)
                    self.emit_get_global(&name);
                    self.last_var_ref = None;
                }
            }

            // Grouping: (expr)
            Token::LParen => {
                self.advance();
                self.expression()?;
                self.expect(Token::RParen)?;
            }

            // Unary operators
            Token::Minus => {
                self.advance();
                self.parse_precedence(Precedence::Unary)?;
                self.emit_op(OpCode::Neg);
            }
            Token::Plus => {
                self.advance();
                self.parse_precedence(Precedence::Unary)?;
                self.emit_op(OpCode::Plus);
            }
            Token::Bang => {
                self.advance();
                self.parse_precedence(Precedence::Unary)?;
                self.emit_op(OpCode::LNot);
            }
            Token::Tilde => {
                self.advance();
                self.parse_precedence(Precedence::Unary)?;
                self.emit_op(OpCode::Not);
            }
            Token::TypeOf => {
                self.advance();
                // Special-case bare identifiers so `typeof missingVar` returns
                // "undefined" instead of throwing ReferenceError, matching JS.
                let bare_identifier = matches!(self.current_token, Token::Ident(_))
                    && !matches!(
                        self.peek_token(),
                        Token::LParen | Token::LBracket | Token::Dot
                    );

                if bare_identifier {
                    let Token::Ident(name) = &self.current_token else {
                        unreachable!()
                    };
                    let name = name.clone();
                    self.advance();

                    if let Some(idx) = self.resolve_local(&name) {
                        self.emit_get_local(idx);
                    } else if let Some(idx) = self.resolve_capture(&name) {
                        self.emit_get_capture(idx);
                    } else {
                        self.emit_op(OpCode::GetGlobalOrUndefined);
                        let str_idx = self.string_constants.len() as u16;
                        self.string_constants.push(name);
                        let const_idx = self.add_constant(Value::string(str_idx));
                        self.emit_u16(const_idx);
                    }
                } else {
                    self.parse_precedence(Precedence::Unary)?;
                }
                self.emit_op(OpCode::TypeOf);
            }

            // Void operator: void expr -> undefined
            Token::Void => {
                self.advance();
                self.parse_precedence(Precedence::Unary)?;
                self.emit_op(OpCode::Drop);
                self.emit_op(OpCode::Undefined);
            }

            // Delete operator: delete obj.prop or delete arr[idx]
            Token::Delete => {
                self.advance();
                // Parse the operand - we need to handle member access specially
                // to get both the object and the property key
                self.delete_expr()?;
            }

            // Pre-increment/decrement
            Token::PlusPlus => {
                self.advance();
                // Need to handle as lvalue modification
                if let Token::Ident(name) = &self.current_token {
                    let name = name.clone();
                    self.advance();
                    if let Some(idx) = self.resolve_local(&name) {
                        self.emit_get_local(idx);
                        self.emit_op(OpCode::Inc);
                        self.emit_op(OpCode::Dup);
                        self.emit_set_local(idx);
                    } else if let Some(idx) = self.resolve_capture(&name) {
                        self.emit_get_capture(idx);
                        self.emit_op(OpCode::Inc);
                        self.emit_op(OpCode::Dup);
                        self.emit_set_capture(idx);
                    } else {
                        return Err(self.syntax_error(format!("Undefined variable '{}'", name)));
                    }
                } else {
                    return Err(self.syntax_error("Invalid increment operand"));
                }
            }
            Token::MinusMinus => {
                self.advance();
                if let Token::Ident(name) = &self.current_token {
                    let name = name.clone();
                    self.advance();
                    if let Some(idx) = self.resolve_local(&name) {
                        self.emit_get_local(idx);
                        self.emit_op(OpCode::Dec);
                        self.emit_op(OpCode::Dup);
                        self.emit_set_local(idx);
                    } else if let Some(idx) = self.resolve_capture(&name) {
                        self.emit_get_capture(idx);
                        self.emit_op(OpCode::Dec);
                        self.emit_op(OpCode::Dup);
                        self.emit_set_capture(idx);
                    } else {
                        return Err(self.syntax_error(format!("Undefined variable '{}'", name)));
                    }
                } else {
                    return Err(self.syntax_error("Invalid decrement operand"));
                }
            }

            // Array literal: [1, 2, 3]
            Token::LBracket => {
                self.advance();
                let count = self.array_literal()?;
                // Emit ArrayFrom opcode with element count
                self.emit_op(OpCode::ArrayFrom);
                self.emit_u16(count);
            }

            // New expression: new Constructor() or new Constructor
            Token::New => {
                self.advance();
                // Parse the constructor expression (just the primary, not function call)
                // We handle the member access chain but not the call
                self.new_expr_target()?;
                // Check for argument list
                let arg_count = if self.check(&Token::LParen) {
                    self.advance();
                    self.argument_list()?
                } else {
                    0
                };
                // Emit CallConstructor opcode
                self.emit_op(OpCode::CallConstructor);
                self.emit_u16(arg_count);
            }

            // Object literal: { key: value, key2: value2 }
            // Also supports shorthand: { speed } => { speed: speed }
            Token::LBrace => {
                self.advance(); // consume '{'

                // Emit Object opcode to create empty object
                self.emit_op(OpCode::Object);
                self.emit_u16(0); // class id (unused)

                // Parse properties
                while !self.check(&Token::RBrace) && !self.check(&Token::Eof) {
                    // Get property key (identifier or string)
                    let key = match &self.current_token {
                        Token::Ident(s) => s.clone(),
                        Token::String(s) => s.clone(),
                        _ => return Err(self.syntax_error("Expected property name")),
                    };
                    self.advance();

                    let str_idx = self.string_constants.len() as u16;
                    self.string_constants.push(key.clone());

                    // Dup the object so PutField can consume it
                    self.emit_op(OpCode::Dup);

                    if self.match_token(&Token::Colon) {
                        // Normal property: key: value
                        self.expression()?;
                    } else {
                        // Shorthand property: { speed } => { speed: speed }
                        // Resolve the identifier as a variable
                        if let Some(idx) = self.resolve_local(&key) {
                            self.emit_get_local(idx);
                        } else if let Some(idx) = self.resolve_capture(&key) {
                            self.emit_get_capture(idx);
                        } else {
                            self.emit_get_global(&key);
                        }
                    }

                    // PutField pops obj+val, pushes val
                    self.emit_op(OpCode::PutField);
                    self.emit_u16(str_idx);
                    // Drop the val, keeping the original object on stack
                    self.emit_op(OpCode::Drop);

                    // Optional comma between properties
                    self.match_token(&Token::Comma);
                }

                self.expect(Token::RBrace)?;
            }

            _ => {
                return Err(
                    self.syntax_error(format!("Unexpected token: {:?}", self.current_token))
                );
            }
        }

        // Handle postfix operators and member access
        self.postfix_expr()
    }

    /// Parse postfix operators (++, --, call, member access)
    fn postfix_expr(&mut self) -> Result<(), CompileError> {
        loop {
            match &self.current_token {
                // Function call
                Token::LParen => {
                    self.last_expr_string_const = None;
                    self.last_expr_bool_const = None;
                    self.last_expr_concat_surround = None;
                    self.advance();
                    let arg_count = self.argument_list()?;
                    self.emit_op(OpCode::Call);
                    self.emit_u16(arg_count);
                }

                // Array access: a[b] or a[b] = c
                Token::LBracket => {
                    self.last_expr_string_const = None;
                    self.last_expr_bool_const = None;
                    self.last_expr_concat_surround = None;
                    self.advance();
                    self.expression()?;
                    self.expect(Token::RBracket)?;

                    // Check for assignment
                    if self.match_token(&Token::Eq) {
                        // arr[idx] = value
                        self.expression()?;
                        self.emit_op(OpCode::PutArrayEl);
                    } else {
                        self.emit_op(OpCode::GetArrayEl);
                    }
                }

                // Member access: a.b or a.b = c or a.b()
                Token::Dot => {
                    self.last_expr_string_const = None;
                    self.last_expr_bool_const = None;
                    self.last_expr_concat_surround = None;
                    self.advance();
                    if let Token::Ident(name) = &self.current_token {
                        let name = name.clone();
                        let is_length = bytecode_length_property_name(&name);
                        let is_push = name == "push";
                        let is_map = name == "map";
                        let is_filter = name == "filter";
                        let is_reduce = name == "reduce";
                        self.advance();

                        // Check for assignment
                        if self.match_token(&Token::Eq) {
                            let str_idx = self.string_constants.len() as u16;
                            self.string_constants.push(name);
                            // obj.prop = value
                            self.expression()?;
                            self.emit_op(OpCode::PutField);
                            self.emit_u16(str_idx);
                        } else if self.check(&Token::LParen) {
                            self.advance(); // consume LParen
                                            // Most method calls need the receiver duplicated before
                                            // argument evaluation so calls see
                                            // `[this, method, arg0, ...]`.
                            if is_push {
                                self.emit_op(OpCode::GetArrayPush2);
                            } else {
                                let str_idx = self.string_constants.len() as u16;
                                self.string_constants.push(name);
                                self.emit_op(OpCode::GetField2);
                                self.emit_u16(str_idx);
                            }
                            let arg_count = self.argument_list()?;
                            if is_push && arg_count == 1 {
                                self.last_expr_bool_const = None;
                                self.emit_op(OpCode::CallArrayPush1);
                            } else if is_map && arg_count == 1 {
                                self.emit_op(OpCode::CallArrayMap1);
                            } else if is_filter && arg_count == 1 {
                                self.emit_op(OpCode::CallArrayFilter1);
                            } else if is_reduce && arg_count == 2 {
                                self.emit_op(OpCode::CallArrayReduce2);
                            } else {
                                self.emit_op(OpCode::CallMethod);
                                self.emit_u16(arg_count);
                            }
                        } else if is_length {
                            self.emit_op(OpCode::GetLength);
                        } else {
                            let str_idx = self.string_constants.len() as u16;
                            self.string_constants.push(name);
                            // obj.prop
                            self.emit_op(OpCode::GetField);
                            self.emit_u16(str_idx);
                            let _ = self.try_rewrite_get_field_chain4();
                        }
                    } else {
                        return Err(self.syntax_error("Expected property name"));
                    }
                }

                // Post-increment: i++
                Token::PlusPlus => {
                    if let Some((is_local, idx)) = self.last_var_ref.take() {
                        self.last_expr_string_const = None;
                        self.last_expr_bool_const = None;
                        self.last_expr_concat_surround = None;
                        self.advance();
                        // Old value is on stack (expression result)
                        // Duplicate it, increment, store back
                        self.emit_op(OpCode::Dup);
                        self.emit_op(OpCode::Inc);
                        if is_local {
                            self.emit_set_local(idx);
                        } else {
                            self.emit_set_capture(idx);
                        }
                        // Old value remains on stack as expression result
                    } else {
                        break;
                    }
                }

                // Post-decrement: i--
                Token::MinusMinus => {
                    if let Some((is_local, idx)) = self.last_var_ref.take() {
                        self.last_expr_string_const = None;
                        self.last_expr_bool_const = None;
                        self.last_expr_concat_surround = None;
                        self.advance();
                        self.emit_op(OpCode::Dup);
                        self.emit_op(OpCode::Dec);
                        if is_local {
                            self.emit_set_local(idx);
                        } else {
                            self.emit_set_capture(idx);
                        }
                    } else {
                        break;
                    }
                }

                _ => break,
            }
        }
        Ok(())
    }

    /// Parse the target of a new expression (identifier or member access, but not function call)
    /// This is called for `new Foo()` or `new foo.Bar()` - we want to parse the constructor
    /// expression without consuming the `(` which we'll handle separately.
    fn new_expr_target(&mut self) -> Result<(), CompileError> {
        // Parse the primary expression (identifier, grouping, etc.)
        match &self.current_token {
            Token::Ident(name) => {
                let name = name.clone();
                self.advance();
                // Resolve as local, capture, or global
                if let Some(idx) = self.resolve_local(&name) {
                    self.emit_get_local(idx);
                } else if let Some(idx) = self.resolve_capture(&name) {
                    self.emit_get_capture(idx);
                } else {
                    // Try as a global (builtin function)
                    self.emit_get_global(&name);
                }
            }
            Token::LParen => {
                self.advance();
                self.expression()?;
                self.expect(Token::RParen)?;
            }
            _ => {
                return Err(self.syntax_error(format!(
                    "Expected constructor expression, got {:?}",
                    self.current_token
                )));
            }
        }

        // Handle member access chain (but NOT function calls)
        loop {
            match &self.current_token {
                Token::Dot => {
                    self.advance();
                    if let Token::Ident(name) = &self.current_token {
                        let name = name.clone();
                        self.advance();
                        let str_idx = self.string_constants.len() as u16;
                        self.string_constants.push(name);
                        self.emit_op(OpCode::GetField);
                        self.emit_u16(str_idx);
                        let _ = self.try_rewrite_get_field_chain4();
                    } else {
                        return Err(self.syntax_error("Expected property name"));
                    }
                }
                Token::LBracket => {
                    // Array access: foo[expr]
                    self.advance();
                    self.expression()?;
                    self.expect(Token::RBracket)?;
                    self.emit_op(OpCode::GetArrayEl);
                }
                _ => break,
            }
        }
        Ok(())
    }

    /// Parse the operand of a delete expression
    /// Handles: delete obj.prop, delete arr[idx], delete variable
    fn delete_expr(&mut self) -> Result<(), CompileError> {
        // Parse the base expression (identifier or grouped expression)
        match &self.current_token {
            Token::Ident(name) => {
                let name = name.clone();
                self.advance();

                // Resolve the variable as local, capture, or global
                if let Some(idx) = self.resolve_local(&name) {
                    self.emit_get_local(idx);
                } else if let Some(idx) = self.resolve_capture(&name) {
                    self.emit_get_capture(idx);
                } else {
                    // Try as a global (builtin function)
                    self.emit_get_global(&name);
                }
            }
            Token::LParen => {
                self.advance();
                self.expression()?;
                self.expect(Token::RParen)?;
            }
            _ => {
                return Err(self.syntax_error(format!(
                    "Expected expression after delete, got {:?}",
                    self.current_token
                )));
            }
        }

        // Now handle the property access (must have . or [])
        match &self.current_token {
            Token::Dot => {
                self.advance();
                if let Token::Ident(name) = &self.current_token {
                    let name = name.clone();
                    self.advance();
                    // Push property name as string constant
                    let str_idx = self.string_constants.len() as u16;
                    self.string_constants.push(name);
                    self.emit_op(OpCode::PushConst);
                    let const_idx = self.add_constant(Value::string(str_idx));
                    self.emit_u16(const_idx);
                    self.emit_op(OpCode::Delete);
                } else {
                    return Err(self.syntax_error("Expected property name after ."));
                }
            }
            Token::LBracket => {
                self.advance();
                self.expression()?;
                self.expect(Token::RBracket)?;
                self.emit_op(OpCode::Delete);
            }
            _ => {
                // delete on a simple variable - push undefined as key
                // This is non-standard but we'll return true
                self.emit_op(OpCode::Undefined);
                self.emit_op(OpCode::Delete);
            }
        }
        Ok(())
    }

    /// Parse function call arguments
    fn argument_list(&mut self) -> Result<u16, CompileError> {
        let mut count = 0;

        if !self.check(&Token::RParen) {
            loop {
                self.expression()?;
                count += 1;

                if count > 255 {
                    return Err(self.syntax_error("Too many arguments"));
                }

                if !self.match_token(&Token::Comma) {
                    break;
                }
            }
        }

        self.expect(Token::RParen)?;
        Ok(count)
    }

    /// Parse array literal elements: expr, expr, ... ]
    /// Called after the opening '[' has been consumed
    fn array_literal(&mut self) -> Result<u16, CompileError> {
        let mut count: u32 = 0;

        if !self.check(&Token::RBracket) {
            loop {
                self.expression()?;
                count += 1;

                if count > 65535 {
                    return Err(self.syntax_error("Too many array elements"));
                }

                if !self.match_token(&Token::Comma) {
                    break;
                }

                // Handle trailing comma: [1, 2, ]
                if self.check(&Token::RBracket) {
                    break;
                }
            }
        }

        self.expect(Token::RBracket)?;
        Ok(count as u16)
    }

    /// Get precedence and associativity of current infix operator
    fn infix_precedence(&self) -> Option<(Precedence, Associativity)> {
        use Associativity::*;
        use Precedence::*;

        match &self.current_token {
            // Assignment
            Token::Eq
            | Token::PlusEq
            | Token::MinusEq
            | Token::StarEq
            | Token::SlashEq
            | Token::PercentEq
            | Token::AmpEq
            | Token::PipeEq
            | Token::CaretEq
            | Token::LtLtEq
            | Token::GtGtEq
            | Token::GtGtGtEq
            | Token::StarStarEq => Some((Assignment, Right)),

            // Ternary
            Token::Question => Some((Ternary, Right)),

            // Logical OR
            Token::PipePipe => Some((LogicalOr, Left)),

            // Logical AND
            Token::AmpAmp => Some((LogicalAnd, Left)),

            // Bitwise OR
            Token::Pipe => Some((BitwiseOr, Left)),

            // Bitwise XOR
            Token::Caret => Some((BitwiseXor, Left)),

            // Bitwise AND
            Token::Amp => Some((BitwiseAnd, Left)),

            // Equality
            Token::EqEq | Token::BangEq | Token::EqEqEq | Token::BangEqEq => Some((Equality, Left)),

            // Relational
            Token::Lt | Token::LtEq | Token::Gt | Token::GtEq | Token::InstanceOf | Token::In => {
                Some((Relational, Left))
            }

            // Shift
            Token::LtLt | Token::GtGt | Token::GtGtGt => Some((Shift, Left)),

            // Additive
            Token::Plus | Token::Minus => Some((Additive, Left)),

            // Multiplicative
            Token::Star | Token::Slash | Token::Percent => Some((Multiplicative, Left)),

            // Exponentiation (right-associative)
            Token::StarStar => Some((Exponentiation, Right)),

            _ => None,
        }
    }

    /// Handle assignment expression
    fn assignment_expr(&mut self, _op: &Token) -> Result<(), CompileError> {
        // For now, only handle simple variable assignment
        // The left-hand side was already compiled; we need to undo that
        // This is a simplified implementation - a proper one would track lvalues
        Err(self.syntax_error("Assignment expressions not yet fully implemented"))
    }

    /// Handle ternary conditional: a ? b : c
    fn ternary_expr(&mut self) -> Result<(), CompileError> {
        // Condition already on stack
        let else_jump = self.emit_jump(OpCode::IfFalse);

        // Parse 'then' expression
        self.expression()?;
        let end_jump = self.emit_jump(OpCode::Goto);

        self.expect(Token::Colon)?;
        self.patch_jump(else_jump);

        // Parse 'else' expression
        self.parse_precedence(Precedence::Ternary)?;
        self.patch_jump(end_jump);

        Ok(())
    }

    /// Handle short-circuit logical operators
    fn short_circuit_expr(&mut self, op: &Token) -> Result<(), CompileError> {
        match op {
            Token::AmpAmp => {
                // Left is on stack; dup it so the conditional jump doesn't consume
                // the value we may want to keep as the result.
                self.emit_op(OpCode::Dup);
                let end_jump = self.emit_jump(OpCode::IfFalse);
                // Left was truthy — drop the duplicated left, evaluate right
                self.emit_op(OpCode::Drop);
                self.parse_precedence(Precedence::LogicalAnd.next())?;
                self.patch_jump(end_jump);
                // If left was falsy: IfFalse popped the dup, original left remains on stack.
            }
            Token::PipePipe => {
                // Left is on stack; dup it so the conditional jump doesn't consume
                // the value we may want to keep as the result.
                self.emit_op(OpCode::Dup);
                let end_jump = self.emit_jump(OpCode::IfTrue);
                // Left was falsy — drop the duplicated left, evaluate right
                self.emit_op(OpCode::Drop);
                self.parse_precedence(Precedence::LogicalOr.next())?;
                self.patch_jump(end_jump);
                // If left was truthy: IfTrue popped the dup, original left remains on stack.
            }
            _ => unreachable!(),
        }
        Ok(())
    }

    /// Emit binary operator
    fn emit_binary_op(
        &mut self,
        op: &Token,
        left_const_string: Option<(u16, usize)>,
        left_concat_surround: Option<(u16, u16, usize)>,
        right_const_string: Option<(u16, usize)>,
    ) -> Result<(), CompileError> {
        self.last_expr_string_const = None;
        self.last_expr_concat_surround = None;
        match op {
            Token::Plus => {
                if let (Some((left_idx, left_start)), Some((right_idx, _right_start))) =
                    (left_const_string, right_const_string)
                {
                    let Some(left) = self.string_const_content(left_idx).map(|s| s.to_string())
                    else {
                        self.emit_op(OpCode::Add);
                        return Ok(());
                    };
                    let Some(right) = self.string_const_content(right_idx).map(|s| s.to_string())
                    else {
                        self.emit_op(OpCode::Add);
                        return Ok(());
                    };
                    self.bytecode.truncate(left_start);
                    let combined_idx = self.plain_string_const_index(format!("{}{}", left, right));
                    self.emit_op(OpCode::PushConst);
                    let const_idx = self.add_constant(Value::string(combined_idx));
                    self.emit_u16(const_idx);
                    self.last_expr_string_const = Some((combined_idx, left_start));
                } else if let Some((str_idx, start)) = right_const_string {
                    // The right-hand expression was emitted most recently, so we
                    // can replace its constant push with a dedicated concat op.
                    self.bytecode.truncate(start);
                    if !self.try_merge_trailing_concat_with_right_const(str_idx) {
                        self.emit_op(OpCode::AddConstStringRight);
                        self.emit_u16(str_idx);
                    }
                } else if let Some((left_idx, right_idx, surround_start)) = left_concat_surround {
                    self.bytecode.drain(surround_start..surround_start + 5);
                    self.emit_op(OpCode::AddConstStringSurroundValue);
                    self.emit_u16(left_idx);
                    self.emit_u16(right_idx);
                } else if let Some((str_idx, _)) = left_const_string {
                    self.emit_op(OpCode::AddConstStringLeft);
                    self.emit_u16(str_idx);
                } else {
                    self.emit_op(OpCode::Add);
                }
            }
            Token::Minus => self.emit_op(OpCode::Sub),
            Token::Star => self.emit_op(OpCode::Mul),
            Token::Slash => self.emit_op(OpCode::Div),
            Token::Percent => self.emit_op(OpCode::Mod),
            Token::StarStar => self.emit_op(OpCode::Pow),
            Token::Amp => self.emit_op(OpCode::And),
            Token::Pipe => self.emit_op(OpCode::Or),
            Token::Caret => self.emit_op(OpCode::Xor),
            Token::LtLt => self.emit_op(OpCode::Shl),
            Token::GtGt => self.emit_op(OpCode::Sar),
            Token::GtGtGt => self.emit_op(OpCode::Shr),
            Token::Lt => self.emit_op(OpCode::Lt),
            Token::LtEq => self.emit_op(OpCode::Lte),
            Token::Gt => self.emit_op(OpCode::Gt),
            Token::GtEq => self.emit_op(OpCode::Gte),
            Token::EqEq => self.emit_op(OpCode::Eq),
            Token::BangEq => self.emit_op(OpCode::Neq),
            Token::EqEqEq => self.emit_op(OpCode::StrictEq),
            Token::BangEqEq => self.emit_op(OpCode::StrictNeq),
            Token::InstanceOf => self.emit_op(OpCode::InstanceOf),
            Token::In => self.emit_op(OpCode::In),
            _ => {
                return Err(self.syntax_error(format!("Unknown binary operator: {:?}", op)));
            }
        }
        let len = self.bytecode.len();
        if len >= 5 && self.bytecode[len - 5] == OpCode::AddConstStringSurround as u8 {
            let left_idx = u16::from_le_bytes([self.bytecode[len - 4], self.bytecode[len - 3]]);
            let right_idx = u16::from_le_bytes([self.bytecode[len - 2], self.bytecode[len - 1]]);
            self.last_expr_concat_surround = Some((left_idx, right_idx, len - 5));
        }
        Ok(())
    }
}

#[inline]
fn bytecode_length_property_name(name: &str) -> bool {
    name == "length"
}

/// Operator precedence levels (lowest to highest)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    Lowest,
    Assignment,     // = += -= etc.
    Ternary,        // ?:
    LogicalOr,      // ||
    LogicalAnd,     // &&
    BitwiseOr,      // |
    BitwiseXor,     // ^
    BitwiseAnd,     // &
    Equality,       // == != === !==
    Relational,     // < <= > >= instanceof in
    Shift,          // << >> >>>
    Additive,       // + -
    Multiplicative, // * / %
    Exponentiation, // **
    Unary,          // ! ~ - + typeof void delete
    Postfix,        // ++ -- (postfix)
    Call,           // () [] .
    Primary,        // literals, identifiers
}

impl Precedence {
    /// Get the next higher precedence
    fn next(self) -> Self {
        match self {
            Precedence::Lowest => Precedence::Assignment,
            Precedence::Assignment => Precedence::Ternary,
            Precedence::Ternary => Precedence::LogicalOr,
            Precedence::LogicalOr => Precedence::LogicalAnd,
            Precedence::LogicalAnd => Precedence::BitwiseOr,
            Precedence::BitwiseOr => Precedence::BitwiseXor,
            Precedence::BitwiseXor => Precedence::BitwiseAnd,
            Precedence::BitwiseAnd => Precedence::Equality,
            Precedence::Equality => Precedence::Relational,
            Precedence::Relational => Precedence::Shift,
            Precedence::Shift => Precedence::Additive,
            Precedence::Additive => Precedence::Multiplicative,
            Precedence::Multiplicative => Precedence::Exponentiation,
            Precedence::Exponentiation => Precedence::Unary,
            Precedence::Unary => Precedence::Postfix,
            Precedence::Postfix => Precedence::Call,
            Precedence::Call => Precedence::Primary,
            Precedence::Primary => Precedence::Primary,
        }
    }
}

/// Operator associativity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Associativity {
    Left,
    Right,
}

/// Compiled function
/// Capture info for closures (public for interpreter use)
#[derive(Debug, Clone)]
pub struct CaptureInfo {
    /// Index in the outer function's locals (or captures)
    pub outer_index: usize,
    /// Whether this captures from outer's locals (true) or outer's captures (false)
    pub is_local: bool,
}

pub struct CompiledFunction {
    /// Bytecode bytes
    pub bytecode: Vec<u8>,
    /// Constant pool
    pub constants: Vec<Value>,
    /// String constant pool
    pub string_constants: Vec<String>,
    /// Number of local variables
    pub local_count: usize,
    /// Number of arguments
    pub arg_count: usize,
    /// Inner functions defined within this function
    pub functions: Vec<CompiledFunction>,
    /// Capture information for closures
    pub captures: Vec<CaptureInfo>,
}

/// Compilation error
#[derive(Debug)]
pub enum CompileError {
    UnexpectedToken {
        expected: String,
        found: String,
        line: usize,
        column: usize,
    },
    SyntaxError {
        message: String,
        line: usize,
        column: usize,
    },
    TooManyConstants,
    TooManyLocals,
}

impl core::fmt::Display for CompileError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            CompileError::UnexpectedToken {
                expected,
                found,
                line,
                column,
            } => {
                write!(
                    f,
                    "Expected {}, found {} at line {}:{}",
                    expected, found, line, column
                )
            }
            CompileError::SyntaxError {
                message,
                line,
                column,
            } => write!(f, "Syntax error: {} at line {}:{}", message, line, column),
            CompileError::TooManyConstants => write!(f, "Too many constants"),
            CompileError::TooManyLocals => write!(f, "Too many local variables"),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CompileError {}

// Tests moved to tests/compiler_tests.rs
