use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::{Context, FunctionBytecode};

const BYTECODE_MAGIC: &[u8] = b"MQJS";
const BYTECODE_VERSION: u8 = 1;
const EFFECT_GLOBAL: &str = "__mquickjs_effect";
const CONFIG_GLOBAL: &str = "__mquickjs_config";

/// 效果操作的统一返回类型
pub type EffectResult<T> = Result<T, String>;

/// Rust 侧配置值，可递归表示 JS 对象/数组/基本类型
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
    /// 将 ConfigValue 转换为 JS 字面量字符串（如 "{'r': 255, 'g': 0}"）
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

/// 效果引擎：持有编译后的字节码，可创建多个效果实例
pub struct EffectEngine {
    bytecode_bytes: Vec<u8>,
    memory_limit: usize,
}

/// 效果实例：一个正在运行的 JS 效果，内含 Context 和预编译的生命周期方法
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

/// 被 EffectManager 管理的效果实例（附带名称和所属引擎名）
struct ManagedEffectInstance {
    name: String,
    engine_name: String,
    instance: EffectInstance,
}

/// 效果管理器：管理多个引擎和实例，支持激活/切换/删除
pub struct EffectManager {
    engines: Vec<(String, EffectEngine)>,
    instances: Vec<ManagedEffectInstance>,
    active_instance: Option<usize>,
}

impl EffectEngine {
    /// 从 JS 源码编译为字节码，创建效果引擎（开发阶段用）
    pub fn from_source(source: &str) -> EffectResult<Self> {
        let ctx = Context::new(1024 * 1024);
        let bytecode = ctx.compile(source).map_err(|e| e.to_string())?;
        Ok(Self {
            bytecode_bytes: bytecode.serialize(),
            memory_limit: 256 * 1024,
        })
    }

    /// 从预编译字节码创建效果引擎（生产环境用，跳过编译）
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

    /// 设置实例的内存上限（链式调用）
    pub fn with_memory_limit(mut self, memory_limit: usize) -> Self {
        self.memory_limit = memory_limit;
        self
    }

    /// 用 JS 字符串配置创建效果实例（如 "{ ledCount: 20 }"）
    pub fn instantiate_from_expr(&self, config_expr: &str) -> EffectResult<EffectInstance> {
        let config_expr = if config_expr.trim().is_empty() {
            ConfigValue::Object(Vec::new()).to_js_literal()
        } else {
            format!("({})", config_expr)
        };
        self.instantiate_from_literal(&config_expr)
    }

    /// 用结构化 ConfigValue 创建效果实例（推荐方式）
    pub fn instantiate_config(&self, config: ConfigValue) -> EffectResult<EffectInstance> {
        self.instantiate_from_literal(&config.to_js_literal())
    }

    /// 内部方法：创建 JS Context，加载字节码，执行 createEffect()，预编译生命周期方法
    fn instantiate_from_literal(&self, config_literal: &str) -> EffectResult<EffectInstance> {
        let mut ctx = Context::new(self.memory_limit);
        // 反序列化字节码并加载到 Context
        let (bytecode, _) = FunctionBytecode::deserialize(&self.bytecode_bytes)
            .map_err(|e| e.to_string())?;
        ctx.load_bytecode(bytecode).map_err(|e| e.to_string())?;

        // 解析配置对象并存入全局变量
        let config_val = ctx
            .eval(&format!("return {};", config_literal))
            .map_err(|e| e.to_string())?;
        ctx.set_global(CONFIG_GLOBAL, config_val);

        // 调用 createEffect(config) 生成效果实例
        let create_bc = ctx
            .compile(&format!("return createEffect({CONFIG_GLOBAL});"))
            .map_err(|e| e.to_string())?;
        let effect_val = ctx.execute(&create_bc).map_err(|e| e.to_string())?;
        ctx.set_global(EFFECT_GLOBAL, effect_val);

        // 预编译各生命周期方法的调用脚本，后续 tick/start/stop 直接执行
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
    /// 创建空的效果管理器
    pub fn new() -> Self {
        Self {
            engines: Vec::new(),
            instances: Vec::new(),
            active_instance: None,
        }
    }

    /// 注册一个效果引擎（返回引擎索引）
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

    /// 用 JS 字符串配置创建实例并加入管理列表（返回实例索引）
    pub fn instantiate_from_expr(
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

        let instance = engine.1.instantiate_from_expr(config_expr)?;
        self.instances.push(ManagedEffectInstance {
            name: instance_name,
            engine_name: engine_name.to_string(),
            instance,
        });
        Ok(self.instances.len() - 1)
    }

    /// 用结构化 ConfigValue 创建实例并加入管理列表（推荐方式）
    pub fn instantiate_config(
        &mut self,
        engine_name: &str,
        instance_name: impl Into<String>,
        config: ConfigValue,
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

        let instance = engine.1.instantiate_config(config)?;
        self.instances.push(ManagedEffectInstance {
            name: instance_name,
            engine_name: engine_name.to_string(),
            instance,
        });
        Ok(self.instances.len() - 1)
    }

    /// 按索引激活某个实例（后续 tick/start 等操作将作用于该实例）
    pub fn activate(&mut self, instance_idx: usize) -> EffectResult<()> {
        if instance_idx >= self.instances.len() {
            return Err(format!("invalid effect instance index: {}", instance_idx));
        }
        self.active_instance = Some(instance_idx);
        Ok(())
    }

    /// 按名称激活某个实例
    pub fn activate_by_name(&mut self, instance_name: &str) -> EffectResult<()> {
        let idx = self
            .instances
            .iter()
            .position(|entry| entry.name == instance_name)
            .ok_or_else(|| format!("unknown effect instance: {}", instance_name))?;
        self.active_instance = Some(idx);
        Ok(())
    }

    /// 返回所有已注册引擎的名称列表
    pub fn engine_names(&self) -> Vec<&str> {
        self.engines.iter().map(|(name, _)| name.as_str()).collect()
    }

    /// 返回已注册引擎数量
    pub fn engine_count(&self) -> usize {
        self.engines.len()
    }

    /// 返回所有实例的名称列表
    pub fn instance_names(&self) -> Vec<&str> {
        self.instances.iter().map(|entry| entry.name.as_str()).collect()
    }

    /// 返回实例数量
    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }

    /// 按索引删除实例（自动修正激活索引）
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

    /// 按名称删除实例
    pub fn remove_instance_by_name(&mut self, instance_name: &str) -> EffectResult<()> {
        let idx = self
            .instances
            .iter()
            .position(|entry| entry.name == instance_name)
            .ok_or_else(|| format!("unknown effect instance: {}", instance_name))?;
        self.remove_instance(idx)
    }

    /// 返回属于指定引擎的所有实例名称
    pub fn instances_for_engine(&self, engine_name: &str) -> Vec<&str> {
        self.instances
            .iter()
            .filter(|entry| entry.engine_name == engine_name)
            .map(|entry| entry.name.as_str())
            .collect()
    }

    /// 删除属于指定引擎的所有实例（返回删除数量）
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

    /// 返回当前激活实例的名称
    pub fn active_name(&self) -> Option<&str> {
        self.active_instance
            .and_then(|idx| self.instances.get(idx))
            .map(|entry| entry.name.as_str())
    }

    /// 返回当前激活实例所属引擎的名称
    pub fn active_engine_name(&self) -> Option<&str> {
        self.active_instance
            .and_then(|idx| self.instances.get(idx))
            .map(|entry| entry.engine_name.as_str())
    }

    /// 获取当前激活实例的可变引用（内部方法）
    fn active_instance_mut(&mut self) -> EffectResult<&mut EffectInstance> {
        let idx = self
            .active_instance
            .ok_or_else(|| "no active effect instance".to_string())?;
        self.instances
            .get_mut(idx)
            .map(|entry| &mut entry.instance)
            .ok_or_else(|| format!("invalid active effect instance index: {}", idx))
    }

    /// 启动当前激活实例
    pub fn start_active(&mut self) -> EffectResult<()> {
        self.active_instance_mut()?.start()
    }

    /// 驱动当前激活实例执行一帧
    pub fn tick_active(&mut self) -> EffectResult<()> {
        self.active_instance_mut()?.tick()
    }

    /// 暂停当前激活实例
    pub fn pause_active(&mut self) -> EffectResult<()> {
        self.active_instance_mut()?.pause()
    }

    /// 恢复当前激活实例
    pub fn resume_active(&mut self) -> EffectResult<()> {
        self.active_instance_mut()?.resume()
    }

    /// 停止当前激活实例
    pub fn stop_active(&mut self) -> EffectResult<()> {
        self.active_instance_mut()?.stop()
    }

    /// 更新当前激活实例的单项配置
    pub fn set_active_config(&mut self, key: &str, value: ConfigValue) -> EffectResult<()> {
        self.active_instance_mut()?.set_config(key, value)
    }

    /// 重置当前激活实例
    pub fn reset_active(&mut self) -> EffectResult<()> {
        self.active_instance_mut()?.reset()
    }

    /// 读取当前激活实例的 LED 颜色数据（&[u8]，格式 [R,G,B,R,G,B,...]）
    pub fn active_led_buffer(&mut self) -> EffectResult<&[u8]> {
        self.active_instance_mut()?.led_buffer()
    }

    /// 返回当前激活实例的 LED 灯珠数量
    pub fn active_led_count(&mut self) -> EffectResult<usize> {
        self.active_instance_mut()?.led_count()
    }

    /// 返回当前激活实例的内存统计信息
    pub fn memory_stats_active(&mut self) -> EffectResult<crate::MemoryStats> {
        Ok(self.active_instance_mut()?.memory_stats())
    }
}

impl EffectInstance {
    /// 启动效果（调用 JS 的 start()）
    pub fn start(&mut self) -> EffectResult<()> {
        self.ctx.execute(&self.start_bc).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 执行一帧动画（调用 JS 的 tick()）
    pub fn tick(&mut self) -> EffectResult<()> {
        self.ctx.execute(&self.tick_bc).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 暂停效果（调用 JS 的 pause()）
    pub fn pause(&mut self) -> EffectResult<()> {
        self.ctx.execute(&self.pause_bc).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 恢复效果（调用 JS 的 resume()）
    pub fn resume(&mut self) -> EffectResult<()> {
        self.ctx.execute(&self.resume_bc).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 停止效果（调用 JS 的 stop()）
    pub fn stop(&mut self) -> EffectResult<()> {
        self.ctx.execute(&self.stop_bc).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 读取 LED 颜色数据（从 JS 的 leds Uint8Array 读取 &[u8]）
    pub fn led_buffer(&mut self) -> EffectResult<&[u8]> {
        let leds_val = self.ctx.execute(&self.leds_bc).map_err(|e| e.to_string())?;
        self.ctx
            .read_typed_array(leds_val)
            .ok_or_else(|| "effect leds is not a TypedArray".to_string())
    }

    /// 返回 LED 灯珠数量（从 JS 的 ledCount 属性读取）
    pub fn led_count(&mut self) -> EffectResult<usize> {
        let val = self.ctx.execute(&self.led_count_bc).map_err(|e| e.to_string())?;
        val.to_i32()
            .map(|v| v as usize)
            .ok_or_else(|| "effect ledCount is not an integer".to_string())
    }

    /// 动态更新配置（调用 JS 的 setConfig(key, value)）
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

    /// 重置效果实例（重新执行 createEffect(config)）
    pub fn reset(&mut self) -> EffectResult<()> {
        let effect_val = self.ctx.execute(&self.create_bc).map_err(|e| e.to_string())?;
        self.ctx.set_global(EFFECT_GLOBAL, effect_val);
        Ok(())
    }

    /// 返回当前实例的内存统计
    pub fn memory_stats(&self) -> crate::MemoryStats {
        self.ctx.memory_stats()
    }
}

