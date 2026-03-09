//! Virtual machine module
//!
//! The VM executes JavaScript bytecode using a stack-based architecture.

pub mod interpreter;
mod natives;
pub mod opcode;
mod ops;
mod property;
pub mod stack;
pub mod types;

pub use interpreter::{
    CallFrame, Interpreter, InterpreterError, InterpreterResult, InterpreterStats,
};
pub use opcode::OpCode;
pub use stack::Stack;
