//! LED 效果帧捕获逻辑（基于 EffectEngine API 的对照实现）
//!
//! 这个文件与 `examples/common/effects.rs` 的目标相同：
//! 预计算动画帧并返回给 demo / GUI / 测试使用。
//!
//! 主要区别在于：
//! - `effects.rs`：直接使用 `Context` + 拼接 driver JS
//! - `effects_engine.rs`：使用最小产品级 `EffectEngine` / `EffectInstance` API

use mquickjs::EffectEngine;

/// Blink 效果脚本
pub const BLINK_JS: &str = include_str!("../../js/effects/blink/effect.js");
/// Chase（跑马灯）效果脚本
pub const CHASE_JS: &str = include_str!("../../js/effects/chase/effect.js");
/// Rainbow（彩虹）效果脚本
pub const RAINBOW_JS: &str = include_str!("../../js/effects/rainbow/effect.js");
/// Wave（波浪）效果脚本
pub const WAVE_JS: &str = include_str!("../../js/effects/wave/effect.js");

/// 捕获的效果数据结构
pub struct EffectData {
    /// 效果名称（如 "Blink"、"Rainbow"）
    pub name: &'static str,
    /// LED 灯珠数量
    pub led_count: usize,
    /// 每一帧的 RGB 数据，frames[i][j] = [R, G, B]
    pub frames: Vec<Vec<[u8; 3]>>,
}

/// capture_effect_via_engine: 使用 EffectEngine API 捕获单个效果的动画帧
///
/// 与 `capture_effect()` 不同，这里不再手工：
/// - 创建 `Context`
/// - 注册 native 函数
/// - 拼接 driver JS
///
/// 而是：
/// - `EffectEngine::from_source()`
/// - `engine.instantiate_expr(config)`
/// - `instance.start()`
/// - `instance.tick()`
/// - `instance.led_buffer()`
pub fn capture_effect_via_engine(
    name: &'static str,
    js: &str,
    num_frames: usize,
) -> EffectData {
    let engine = EffectEngine::from_source(js).expect("effect compile failed");

    let mut instance = engine
        .instantiate_expr("{ ledCount: 20 }")
        .expect("effect instantiate failed");

    instance.start().expect("effect start failed");

    let led_count = instance.led_count().expect("invalid ledCount");
    let mut frames = Vec::with_capacity(num_frames);

    for _ in 0..num_frames {
        instance.tick().expect("effect tick failed");

        let data = instance.led_buffer().expect("effect leds buffer missing");
        let mut frame = Vec::with_capacity(led_count);

        for i in 0..led_count {
            let offset = i * 3;
            if offset + 2 < data.len() {
                frame.push([data[offset], data[offset + 1], data[offset + 2]]);
            } else {
                frame.push([0, 0, 0]);
            }
        }

        frames.push(frame);
    }

    EffectData {
        name,
        led_count,
        frames,
    }
}

/// capture_all_via_engine: 使用 EffectEngine API 捕获所有内置效果
pub fn capture_all_via_engine(num_frames: usize) -> Vec<EffectData> {
    vec![
        capture_effect_via_engine("Blink", BLINK_JS, num_frames),
        capture_effect_via_engine("Chase", CHASE_JS, num_frames),
        capture_effect_via_engine("Rainbow", RAINBOW_JS, num_frames),
        capture_effect_via_engine("Wave", WAVE_JS, num_frames),
    ]
}

