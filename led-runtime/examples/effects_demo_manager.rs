/// LED effects visual demo (EffectManager + structured ConfigValue version)
///
/// This example demonstrates the newest product-facing API stack:
/// - `EffectEngine`
/// - `EffectInstance`
/// - `EffectManager`
/// - structured config values (`ConfigValue::Object` / `Array`)
///
/// Compared to `effects_demo.rs` and `effects_demo_engine.rs`, this version:
/// - preloads multiple engines up front
/// - instantiates named effect instances with structured config values
/// - switches the active instance through `EffectManager`
/// - drives rendering via `tick_active()` / `active_led_buffer()`
///
/// Usage: cargo run --example effects_demo_manager
use led_runtime::{ConfigValue, EffectEngine, EffectManager};
use std::io::Write;
use std::{thread, time::Duration};

const BLINK_JS: &str = include_str!("../js/effects/blink/effect.js");
const CHASE_JS: &str = include_str!("../js/effects/chase/effect.js");
const RAINBOW_JS: &str = include_str!("../js/effects/rainbow/effect.js");
const WAVE_JS: &str = include_str!("../js/effects/wave/effect.js");

/// 自定义火焰效果 JavaScript 代码
/// 这是一个完整的 LED 效果实现，展示了如何自定义效果
const FLAME_JS: &str = r#"
function createEffect(config) {
    var cfg = config || {};
    var ledCount = cfg.ledCount || 20;
    var frame = 0;
    var status = 'idle';
    var leds = new Uint8Array(ledCount * 3);
    return {
        status: status,
        ledCount: ledCount,
        leds: leds,
        tick: function() {
            frame++;
            for (var i = 0; i < ledCount; i++) {
                leds[i * 3] = 255;
                leds[i * 3 + 1] = frame % 256;
                leds[i * 3 + 2] = i * 10;
            }
        },
        start: function() { status = 'running'; },
        stop: function() { status = 'idle'; }
    };
}
"#;

const FRAMES: usize = 40;
const DELAY_MS: u64 = 80;

fn main() {
    #[cfg(windows)]
    enable_ansi_windows();

    print!("\x1b[?25l");

    println!();
    println!("  mquickjs LED Effects Demo (EffectManager + ConfigValue)");
    println!("  ====================================================");
    println!();

    let mut manager = build_manager();

    for (instance_name, label) in &[
        ("blink-main", "Blink  "),
        ("chase-main", "Chase  "),
        ("rainbow-main", "Rainbow"),
        ("wave-main", "Wave   "),
        ("flame-main", "Flame  "),
    ] {
        run_effect(&mut manager, instance_name, label);
        println!();
    }

    print!("\x1b[?25h");
    println!("  Done.");
    println!();
}

fn build_manager() -> EffectManager {
    let mut manager = EffectManager::new();

    // 添加自定义火焰效果引擎
    manager
        .add_engine(
            "flame",
            EffectEngine::from_source(FLAME_JS).expect("compile flame"),
        )
        .expect("add flame engine");

    manager
        .add_engine(
            "blink",
            EffectEngine::from_source(BLINK_JS).expect("compile blink"),
        )
        .expect("add blink engine");
    manager
        .add_engine(
            "chase",
            EffectEngine::from_source(CHASE_JS).expect("compile chase"),
        )
        .expect("add chase engine");
    manager
        .add_engine(
            "rainbow",
            EffectEngine::from_source(RAINBOW_JS).expect("compile rainbow"),
        )
        .expect("add rainbow engine");
    manager
        .add_engine(
            "wave",
            EffectEngine::from_source(WAVE_JS).expect("compile wave"),
        )
        .expect("add wave engine");

    manager
        .instantiate_from_expr("blink", "blink-main", "{ ledCount: 20, speed: 200 }")
        .expect("instantiate blink-main");

    manager
        .instantiate_config(
            "chase",
            "chase-main",
            ConfigValue::Object(vec![
                ("ledCount".into(), ConfigValue::Int(20)),
                ("speed".into(), ConfigValue::Int(80)),
                (
                    "color".into(),
                    ConfigValue::Object(vec![
                        ("mode".into(), ConfigValue::Str("rgb".into())),
                        ("r".into(), ConfigValue::Int(251)),
                        ("g".into(), ConfigValue::Int(191)),
                        ("b".into(), ConfigValue::Int(36)),
                    ]),
                ),
                ("chaseCount".into(), ConfigValue::Int(2)),
            ]),
        )
        .expect("instantiate chase-main");

    manager
        .instantiate_config(
            "rainbow",
            "rainbow-main",
            ConfigValue::Object(vec![
                ("ledCount".into(), ConfigValue::Int(20)),
                ("speed".into(), ConfigValue::Int(100)),
                ("hueStep".into(), ConfigValue::Int(10)),
                ("hueSpread".into(), ConfigValue::Int(18)),
                ("saturation".into(), ConfigValue::Float(1.0)),
                ("brightness".into(), ConfigValue::Float(1.0)),
            ]),
        )
        .expect("instantiate rainbow-main");

    manager
        .instantiate_config(
            "wave",
            "wave-main",
            ConfigValue::Object(vec![
                ("ledCount".into(), ConfigValue::Int(20)),
                ("speed".into(), ConfigValue::Int(100)),
                (
                    "color".into(),
                    ConfigValue::Object(vec![
                        ("mode".into(), ConfigValue::Str("rgb".into())),
                        ("r".into(), ConfigValue::Int(52)),
                        ("g".into(), ConfigValue::Int(211)),
                        ("b".into(), ConfigValue::Int(153)),
                    ]),
                ),
                ("waveWidth".into(), ConfigValue::Int(5)),
            ]),
        )
        .expect("instantiate wave-main");

    // 使用 ConfigValue 添加自定义火焰效果实例
    manager
        .instantiate_config(
            "flame",
            "flame-main",
            ConfigValue::Object(vec![("ledCount".into(), ConfigValue::Int(20))]),
        )
        .expect("instantiate flame-main");

    manager
}

fn run_effect(manager: &mut EffectManager, instance_name: &str, label: &str) {
    print!("  {label} :");
    println!();

    manager
        .activate_by_name(instance_name)
        .unwrap_or_else(|e| panic!("activate {} failed: {}", instance_name, e));
    manager.start_active().expect("start active effect failed");

    for frame in 0..FRAMES {
        manager.tick_active().expect("tick active effect failed");

        let led_count = manager.active_led_count().expect("missing ledCount");
        let data = manager.active_led_buffer().expect("missing leds buffer");

        render_frame(data, led_count, frame, FRAMES);
    }

    manager.stop_active().expect("stop active effect failed");
}

fn render_frame(data: &[u8], led_count: usize, frame: usize, total: usize) {
    let out = std::io::stdout();
    let mut out = out.lock();

    if frame > 0 {
        write!(out, "\x1b[1A\r").ok();
    }

    write!(out, "  ").ok();
    for i in 0..led_count {
        let o = i * 3;
        let (r, g, b) = if o + 2 < data.len() {
            (data[o], data[o + 1], data[o + 2])
        } else {
            (0, 0, 0)
        };
        if r == 0 && g == 0 && b == 0 {
            write!(out, "\x1b[48;2;20;20;30m  \x1b[0m").ok();
        } else {
            write!(out, "\x1b[48;2;{r};{g};{b}m  \x1b[0m").ok();
        }
    }
    write!(out, "  {}/{}", frame + 1, total).ok();
    writeln!(out).ok();
    out.flush().ok();

    thread::sleep(Duration::from_millis(DELAY_MS));
}

#[cfg(windows)]
fn enable_ansi_windows() {
    use std::os::windows::io::AsRawHandle;
    unsafe {
        let handle = std::io::stdout().as_raw_handle();
        let mut mode: u32 = 0;
        extern "system" {
            fn GetConsoleMode(handle: *mut core::ffi::c_void, mode: *mut u32) -> i32;
            fn SetConsoleMode(handle: *mut core::ffi::c_void, mode: u32) -> i32;
        }
        GetConsoleMode(handle as *mut _, &mut mode);
        SetConsoleMode(handle as *mut _, mode | 0x0004);
    }
}
