#![no_std]

extern crate alloc;

pub mod effect;

pub use effect::{
    ConfigValue, EffectEngine, EffectError, EffectInstance, EffectManager, EffectResult,
};
