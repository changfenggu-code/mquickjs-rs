use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::{Context, FunctionBytecode};

const BYTECODE_MAGIC: &[u8] = b"MQJS";
const BYTECODE_VERSION: u8 = 1;
const EFFECT_GLOBAL: &str = "__mquickjs_effect";
const CONFIG_GLOBAL: &str = "__mquickjs_config";

pub type EffectResult<T> = Result<T, String>;

pub enum ConfigValue {
    Undefined,
    Null,
    Bool(bool),
    Int(i32),
    Float(f32),
    Str(String),
}

impl ConfigValue {
    fn to_js_literal(&self) -> String {
        match self {
            ConfigValue::Undefined => "undefined".to_string(),
            ConfigValue::Null => "null".to_string(),
            ConfigValue::Bool(v) => v.to_string(),
            ConfigValue::Int(v) => v.to_string(),
            ConfigValue::Float(v) => crate::value::format_float(*v),
            ConfigValue::Str(v) => format!(
                "'{}'",
                v.replace('\\', "\\\\")
                    .replace('\'', "\\'")
                    .replace('\n', "\\n")
                    .replace('\r', "\\r")
            ),
        }
    }
}

pub struct EffectEngine {
    bytecode_bytes: Vec<u8>,
    memory_limit: usize,
}

pub struct EffectInstance {
    ctx: Context,
    create_bc: FunctionBytecode,
    start_bc: FunctionBytecode,
    tick_bc: FunctionBytecode,
    pause_bc: FunctionBytecode,
    resume_bc: FunctionBytecode,
    stop_bc: FunctionBytecode,
    leds_bc: FunctionBytecode,
    led_count_bc: FunctionBytecode,
}

impl EffectEngine {
    pub fn from_source(source: &str) -> EffectResult<Self> {
        let ctx = Context::new(1024 * 1024);
        let bytecode = ctx.compile(source).map_err(|e| e.to_string())?;
        Ok(Self {
            bytecode_bytes: bytecode.serialize(),
            memory_limit: 256 * 1024,
        })
    }

    pub fn from_bytecode(bytes: &[u8]) -> EffectResult<Self> {
        let payload = if bytes.len() >= 5 && &bytes[0..4] == BYTECODE_MAGIC {
            if bytes[4] != BYTECODE_VERSION {
                return Err(format!(
                    "Unsupported bytecode version: {} (expected {})",
                    bytes[4], BYTECODE_VERSION
                ));
            }
            &bytes[5..]
        } else {
            bytes
        };

        // validate once
        let _ = FunctionBytecode::deserialize(payload).map_err(|e| e.to_string())?;

        Ok(Self {
            bytecode_bytes: payload.to_vec(),
            memory_limit: 256 * 1024,
        })
    }

    pub fn with_memory_limit(mut self, memory_limit: usize) -> Self {
        self.memory_limit = memory_limit;
        self
    }

    pub fn instantiate(&self, config_expr: &str) -> EffectResult<EffectInstance> {
        let mut ctx = Context::new(self.memory_limit);
        let (bytecode, _) = FunctionBytecode::deserialize(&self.bytecode_bytes)
            .map_err(|e| e.to_string())?;
        ctx.load_bytecode(bytecode).map_err(|e| e.to_string())?;

        let config_expr = if config_expr.trim().is_empty() {
            "({})".to_string()
        } else {
            format!("({})", config_expr)
        };

        let config_val = ctx
            .eval(&format!("return {};", config_expr))
            .map_err(|e| e.to_string())?;
        ctx.set_global(CONFIG_GLOBAL, config_val);

        let create_bc = ctx
            .compile(&format!("return createEffect({CONFIG_GLOBAL});"))
            .map_err(|e| e.to_string())?;
        let effect_val = ctx.execute(&create_bc).map_err(|e| e.to_string())?;
        ctx.set_global(EFFECT_GLOBAL, effect_val);

        let start_bc = ctx.compile(&format!("return {EFFECT_GLOBAL}.start();")).map_err(|e| e.to_string())?;
        let tick_bc = ctx.compile(&format!("return {EFFECT_GLOBAL}.tick();")).map_err(|e| e.to_string())?;
        let pause_bc = ctx.compile(&format!("return {EFFECT_GLOBAL}.pause();")).map_err(|e| e.to_string())?;
        let resume_bc = ctx.compile(&format!("return {EFFECT_GLOBAL}.resume();")).map_err(|e| e.to_string())?;
        let stop_bc = ctx.compile(&format!("return {EFFECT_GLOBAL}.stop();")).map_err(|e| e.to_string())?;
        let leds_bc = ctx.compile(&format!("return {EFFECT_GLOBAL}.leds;")).map_err(|e| e.to_string())?;
        let led_count_bc = ctx.compile(&format!("return {EFFECT_GLOBAL}.ledCount;")).map_err(|e| e.to_string())?;

        Ok(EffectInstance {
            ctx,
            create_bc,
            start_bc,
            tick_bc,
            pause_bc,
            resume_bc,
            stop_bc,
            leds_bc,
            led_count_bc,
        })
    }
}

impl EffectInstance {
    pub fn start(&mut self) -> EffectResult<()> {
        self.ctx.execute(&self.start_bc).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn tick(&mut self) -> EffectResult<()> {
        self.ctx.execute(&self.tick_bc).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn pause(&mut self) -> EffectResult<()> {
        self.ctx.execute(&self.pause_bc).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn resume(&mut self) -> EffectResult<()> {
        self.ctx.execute(&self.resume_bc).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn stop(&mut self) -> EffectResult<()> {
        self.ctx.execute(&self.stop_bc).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn led_buffer(&mut self) -> EffectResult<&[u8]> {
        let leds_val = self.ctx.execute(&self.leds_bc).map_err(|e| e.to_string())?;
        self.ctx
            .read_typed_array(leds_val)
            .ok_or_else(|| "effect leds is not a TypedArray".to_string())
    }

    pub fn led_count(&mut self) -> EffectResult<usize> {
        let val = self.ctx.execute(&self.led_count_bc).map_err(|e| e.to_string())?;
        val.to_i32()
            .map(|v| v as usize)
            .ok_or_else(|| "effect ledCount is not an integer".to_string())
    }

    pub fn set_config(&mut self, key: &str, value: ConfigValue) -> EffectResult<()> {
        let script = format!(
            "{cfg}.{key} = {value}; return {effect}.setConfig('{key}', {value});",
            cfg = CONFIG_GLOBAL,
            effect = EFFECT_GLOBAL,
            key = key,
            value = value.to_js_literal()
        );
        self.ctx.eval(&script).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn reset(&mut self) -> EffectResult<()> {
        let effect_val = self.ctx.execute(&self.create_bc).map_err(|e| e.to_string())?;
        self.ctx.set_global(EFFECT_GLOBAL, effect_val);
        Ok(())
    }

    pub fn memory_stats(&self) -> crate::MemoryStats {
        self.ctx.memory_stats()
    }
}
