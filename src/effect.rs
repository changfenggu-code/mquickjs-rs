use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::{Context, FunctionBytecode};

const BYTECODE_MAGIC: &[u8] = b"MQJS";
const BYTECODE_VERSION: u8 = 1;
const EFFECT_GLOBAL: &str = "__mquickjs_effect";
const CONFIG_GLOBAL: &str = "__mquickjs_config";

pub type EffectResult<T> = Result<T, String>;

#[derive(Clone, Debug)]
pub enum ConfigValue {
    Undefined,
    Null,
    Bool(bool),
    Int(i32),
    Float(f32),
    Str(String),
    Array(Vec<ConfigValue>),
    Object(Vec<(String, ConfigValue)>),
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
            ConfigValue::Array(values) => {
                let parts: Vec<String> = values.iter().map(|v| v.to_js_literal()).collect();
                format!("[{}]", parts.join(", "))
            }
            ConfigValue::Object(entries) => {
                let parts: Vec<String> = entries
                    .iter()
                    .map(|(key, value)| {
                        let key = key
                            .replace('\\', "\\\\")
                            .replace('\'', "\\'")
                            .replace('\n', "\\n")
                            .replace('\r', "\\r");
                        format!("'{}': {}", key, value.to_js_literal())
                    })
                    .collect();
                format!("{{{}}}", parts.join(", "))
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum ColorConfig {
    Rgb { r: u8, g: u8, b: u8 },
    Hsv { h: f32, s: f32, v: f32 },
}

impl From<ColorConfig> for ConfigValue {
    fn from(value: ColorConfig) -> Self {
        match value {
            ColorConfig::Rgb { r, g, b } => {
                let entries = vec![
                    ("mode".into(), ConfigValue::Str("rgb".into())),
                    ("r".into(), ConfigValue::Int(r as i32)),
                    ("g".into(), ConfigValue::Int(g as i32)),
                    ("b".into(), ConfigValue::Int(b as i32)),
                ];
                ConfigValue::Object(entries)
            }
            ColorConfig::Hsv { h, s, v } => {
                let entries = vec![
                    ("mode".into(), ConfigValue::Str("hsv".into())),
                    ("h".into(), ConfigValue::Float(h)),
                    ("s".into(), ConfigValue::Float(s)),
                    ("v".into(), ConfigValue::Float(v)),
                ];
                ConfigValue::Object(entries)
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct BlinkConfig {
    pub led_count: Option<usize>,
    pub speed: Option<i32>,
    pub color: Option<ColorConfig>,
}

#[derive(Clone, Debug, Default)]
pub struct ChaseConfig {
    pub led_count: Option<usize>,
    pub speed: Option<i32>,
    pub color: Option<ColorConfig>,
    pub chase_count: Option<i32>,
}

#[derive(Clone, Debug, Default)]
pub struct RainbowConfig {
    pub led_count: Option<usize>,
    pub speed: Option<i32>,
    pub hue_step: Option<i32>,
    pub hue_spread: Option<i32>,
    pub saturation: Option<f32>,
    pub brightness: Option<f32>,
}

#[derive(Clone, Debug, Default)]
pub struct WaveConfig {
    pub led_count: Option<usize>,
    pub speed: Option<i32>,
    pub color: Option<ColorConfig>,
    pub wave_width: Option<i32>,
}

fn push_opt_int(entries: &mut Vec<(String, ConfigValue)>, key: &str, value: Option<i32>) {
    if let Some(value) = value {
        entries.push((key.into(), ConfigValue::Int(value)));
    }
}

fn push_opt_float(entries: &mut Vec<(String, ConfigValue)>, key: &str, value: Option<f32>) {
    if let Some(value) = value {
        entries.push((key.into(), ConfigValue::Float(value)));
    }
}

fn push_opt_usize(entries: &mut Vec<(String, ConfigValue)>, key: &str, value: Option<usize>) {
    if let Some(value) = value {
        entries.push((key.into(), ConfigValue::Int(value as i32)));
    }
}

fn push_opt_color(entries: &mut Vec<(String, ConfigValue)>, key: &str, value: Option<ColorConfig>) {
    if let Some(value) = value {
        entries.push((key.into(), value.into()));
    }
}

impl From<BlinkConfig> for ConfigValue {
    fn from(value: BlinkConfig) -> Self {
        let mut entries = Vec::new();
        push_opt_usize(&mut entries, "ledCount", value.led_count);
        push_opt_int(&mut entries, "speed", value.speed);
        push_opt_color(&mut entries, "color", value.color);
        ConfigValue::Object(entries)
    }
}

impl From<ChaseConfig> for ConfigValue {
    fn from(value: ChaseConfig) -> Self {
        let mut entries = Vec::new();
        push_opt_usize(&mut entries, "ledCount", value.led_count);
        push_opt_int(&mut entries, "speed", value.speed);
        push_opt_color(&mut entries, "color", value.color);
        push_opt_int(&mut entries, "chaseCount", value.chase_count);
        ConfigValue::Object(entries)
    }
}

impl From<RainbowConfig> for ConfigValue {
    fn from(value: RainbowConfig) -> Self {
        let mut entries = Vec::new();
        push_opt_usize(&mut entries, "ledCount", value.led_count);
        push_opt_int(&mut entries, "speed", value.speed);
        push_opt_int(&mut entries, "hueStep", value.hue_step);
        push_opt_int(&mut entries, "hueSpread", value.hue_spread);
        push_opt_float(&mut entries, "saturation", value.saturation);
        push_opt_float(&mut entries, "brightness", value.brightness);
        ConfigValue::Object(entries)
    }
}

impl From<WaveConfig> for ConfigValue {
    fn from(value: WaveConfig) -> Self {
        let mut entries = Vec::new();
        push_opt_usize(&mut entries, "ledCount", value.led_count);
        push_opt_int(&mut entries, "speed", value.speed);
        push_opt_color(&mut entries, "color", value.color);
        push_opt_int(&mut entries, "waveWidth", value.wave_width);
        ConfigValue::Object(entries)
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

struct ManagedEffectInstance {
    name: String,
    engine_name: String,
    instance: EffectInstance,
}

pub struct EffectManager {
    engines: Vec<(String, EffectEngine)>,
    instances: Vec<ManagedEffectInstance>,
    active_instance: Option<usize>,
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
        let config_expr = if config_expr.trim().is_empty() {
            ConfigValue::Object(Vec::new()).to_js_literal()
        } else {
            format!("({})", config_expr)
        };
        self.instantiate_from_literal(&config_expr)
    }

    pub fn instantiate_config(&self, config: ConfigValue) -> EffectResult<EffectInstance> {
        self.instantiate_from_literal(&config.to_js_literal())
    }

    fn instantiate_from_literal(&self, config_literal: &str) -> EffectResult<EffectInstance> {
        let mut ctx = Context::new(self.memory_limit);
        let (bytecode, _) = FunctionBytecode::deserialize(&self.bytecode_bytes)
            .map_err(|e| e.to_string())?;
        ctx.load_bytecode(bytecode).map_err(|e| e.to_string())?;

        let config_val = ctx
            .eval(&format!("return {};", config_literal))
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

impl EffectManager {
    pub fn new() -> Self {
        Self {
            engines: Vec::new(),
            instances: Vec::new(),
            active_instance: None,
        }
    }

    pub fn add_engine(
        &mut self,
        name: impl Into<String>,
        engine: EffectEngine,
    ) -> EffectResult<usize> {
        let name = name.into();
        if self.engines.iter().any(|(existing, _)| existing == &name) {
            return Err(format!("duplicate effect engine name: {}", name));
        }

        self.engines.push((name, engine));
        Ok(self.engines.len() - 1)
    }

    pub fn instantiate(
        &mut self,
        engine_name: &str,
        instance_name: impl Into<String>,
        config_expr: &str,
    ) -> EffectResult<usize> {
        let instance_name = instance_name.into();
        if self.instances.iter().any(|entry| entry.name == instance_name) {
            return Err(format!("duplicate effect instance name: {}", instance_name));
        }

        let engine = self
            .engines
            .iter()
            .find(|(name, _)| name == engine_name)
            .ok_or_else(|| format!("unknown effect engine: {}", engine_name))?;

        let instance = engine.1.instantiate(config_expr)?;
        self.instances.push(ManagedEffectInstance {
            name: instance_name,
            engine_name: engine_name.to_string(),
            instance,
        });
        Ok(self.instances.len() - 1)
    }

    pub fn activate(&mut self, instance_idx: usize) -> EffectResult<()> {
        if instance_idx >= self.instances.len() {
            return Err(format!("invalid effect instance index: {}", instance_idx));
        }
        self.active_instance = Some(instance_idx);
        Ok(())
    }

    pub fn activate_by_name(&mut self, instance_name: &str) -> EffectResult<()> {
        let idx = self
            .instances
            .iter()
            .position(|entry| entry.name == instance_name)
            .ok_or_else(|| format!("unknown effect instance: {}", instance_name))?;
        self.active_instance = Some(idx);
        Ok(())
    }

    pub fn engine_names(&self) -> Vec<&str> {
        self.engines.iter().map(|(name, _)| name.as_str()).collect()
    }

    pub fn engine_count(&self) -> usize {
        self.engines.len()
    }

    pub fn instance_names(&self) -> Vec<&str> {
        self.instances.iter().map(|entry| entry.name.as_str()).collect()
    }

    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }

    pub fn remove_instance(&mut self, instance_idx: usize) -> EffectResult<()> {
        if instance_idx >= self.instances.len() {
            return Err(format!("invalid effect instance index: {}", instance_idx));
        }
        self.instances.remove(instance_idx);

        self.active_instance = match self.active_instance {
            Some(active) if active == instance_idx => None,
            Some(active) if active > instance_idx => Some(active - 1),
            other => other,
        };

        Ok(())
    }

    pub fn remove_instance_by_name(&mut self, instance_name: &str) -> EffectResult<()> {
        let idx = self
            .instances
            .iter()
            .position(|entry| entry.name == instance_name)
            .ok_or_else(|| format!("unknown effect instance: {}", instance_name))?;
        self.remove_instance(idx)
    }

    pub fn instances_for_engine(&self, engine_name: &str) -> Vec<&str> {
        self.instances
            .iter()
            .filter(|entry| entry.engine_name == engine_name)
            .map(|entry| entry.name.as_str())
            .collect()
    }

    pub fn remove_instances_by_engine(&mut self, engine_name: &str) -> usize {
        let mut removed = 0usize;
        let mut idx = 0usize;

        while idx < self.instances.len() {
            if self.instances[idx].engine_name == engine_name {
                self.instances.remove(idx);
                removed += 1;

                self.active_instance = match self.active_instance {
                    Some(active) if active == idx => None,
                    Some(active) if active > idx => Some(active - 1),
                    other => other,
                };
            } else {
                idx += 1;
            }
        }

        removed
    }

    pub fn active_name(&self) -> Option<&str> {
        self.active_instance
            .and_then(|idx| self.instances.get(idx))
            .map(|entry| entry.name.as_str())
    }

    pub fn active_engine_name(&self) -> Option<&str> {
        self.active_instance
            .and_then(|idx| self.instances.get(idx))
            .map(|entry| entry.engine_name.as_str())
    }

    fn active_instance_mut(&mut self) -> EffectResult<&mut EffectInstance> {
        let idx = self
            .active_instance
            .ok_or_else(|| "no active effect instance".to_string())?;
        self.instances
            .get_mut(idx)
            .map(|entry| &mut entry.instance)
            .ok_or_else(|| format!("invalid active effect instance index: {}", idx))
    }

    pub fn start_active(&mut self) -> EffectResult<()> {
        self.active_instance_mut()?.start()
    }

    pub fn tick_active(&mut self) -> EffectResult<()> {
        self.active_instance_mut()?.tick()
    }

    pub fn pause_active(&mut self) -> EffectResult<()> {
        self.active_instance_mut()?.pause()
    }

    pub fn resume_active(&mut self) -> EffectResult<()> {
        self.active_instance_mut()?.resume()
    }

    pub fn stop_active(&mut self) -> EffectResult<()> {
        self.active_instance_mut()?.stop()
    }

    pub fn active_led_buffer(&mut self) -> EffectResult<&[u8]> {
        self.active_instance_mut()?.led_buffer()
    }

    pub fn active_led_count(&mut self) -> EffectResult<usize> {
        self.active_instance_mut()?.led_count()
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
