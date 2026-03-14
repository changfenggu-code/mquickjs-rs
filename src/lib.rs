//! MQuickJS - A Rust port of Fabrice Bellard's MicroQuickJS JavaScript engine
//!
//! MQuickJS is a lightweight JavaScript engine targeting embedded systems.
//! This crate focuses on constrained script execution for MCU-hosted effect runtimes.
//!
//! # Features
//! - Constrained ES5/ES6-style JavaScript profile
//! - Tracing and compacting garbage collector
//! - Stack-based bytecode VM
//! - Inline integers and short-float (`f32`) number model
//! - UTF-8 string storage
//!
//! # Example
//! ```ignore
//! use mquickjs::{Context, Value};
//!
//! let mut ctx = Context::new(64 * 1024); // 64KB memory
//! let result = ctx.eval("1 + 2").unwrap();
//! assert_eq!(result.to_i32(), Some(3));
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(dead_code)] // During development

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

// Core modules
pub mod context;
pub mod value;

// Garbage collector
pub mod gc;

// Virtual machine
pub mod vm;

// Parser and compiler
pub mod parser;

// Built-in objects
pub mod builtins;

// Runtime support
pub mod runtime;

// Utilities
pub mod util;
pub mod effect;

// Re-export main types
pub use context::{Context, MemoryStats};
pub use effect::{
    BlinkConfig, ChaseConfig, ColorConfig, ConfigValue, EffectEngine, EffectInstance,
    EffectManager, EffectResult, RainbowConfig, WaveConfig,
};
pub use runtime::FunctionBytecode;
pub use value::Value;
pub use vm::types::NativeFn;
