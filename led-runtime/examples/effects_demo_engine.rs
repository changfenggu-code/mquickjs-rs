/// LED effects visual demo (EffectEngine API version)
///
/// Runs all 4 LED effects (blink, chase, rainbow, wave) through the
/// product-facing `EffectEngine` / `EffectInstance` API and renders the LED
/// strip in the terminal using ANSI 24-bit colors.
///
/// Usage:  cargo run --example effects_demo_engine
use led_runtime::EffectEngine;
use std::io::Write;
use std::{thread, time::Duration};

const BLINK_JS: &str = include_str!("../js/effects/blink/effect.js");
const CHASE_JS: &str = include_str!("../js/effects/chase/effect.js");
const RAINBOW_JS: &str = include_str!("../js/effects/rainbow/effect.js");
const WAVE_JS: &str = include_str!("../js/effects/wave/effect.js");

const FRAMES: usize = 40;
const DELAY_MS: u64 = 80;

fn main() {
    #[cfg(windows)]
    enable_ansi_windows();

    print!("\x1b[?25l");

    println!();
    println!("  mquickjs LED Effects Demo (EffectEngine)");
    println!("  ======================================");
    println!();

    for (name, js) in &[
        ("Blink  ", BLINK_JS),
        ("Chase  ", CHASE_JS),
        ("Rainbow", RAINBOW_JS),
        ("Wave   ", WAVE_JS),
    ] {
        run_effect(name, js);
        println!();
    }

    print!("\x1b[?25h");
    println!("  Done.");
    println!();
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

fn run_effect(name: &str, js: &str) {
    print!("  {name} :");
    println!();

    // 步骤 1：编译 JS 源码为字节码，创建引擎
    // from_source 内部会创建一个临时 Context(48KB) 完成编译，编译完即释放
    let engine = match EffectEngine::from_source(js) {
        Ok(engine) => engine,
        Err(e) => {
            eprintln!("  ERROR: {}", e);
            return;
        }
    };

    // 步骤 2：实例化效果，注入配置（ledCount: 20 表示 20 颗灯）
    // 内部创建运行时 Context(32KB)，加载字节码，执行 createEffect(config)
    // 并预编译 tick/start/stop 等调用脚本
    let mut instance = match engine.instantiate_from_expr("{ ledCount: 20, frameMs: 80 }") {
        Ok(instance) => instance,
        Err(e) => {
            eprintln!("  ERROR: {}", e);
            return;
        }
    };

    // 步骤 3：启动效果（调用 JS 的 start()，执行初始化逻辑）
    if let Err(e) = instance.start() {
        eprintln!("  ERROR: {}", e);
        return;
    }

    // 步骤 4：主循环，每帧驱动一次动画
    for frame in 0..FRAMES {
        // 4a：执行一帧（调用 JS 的 tick()，更新内部 leds 数组）
        if let Err(e) = instance.tick() {
            eprintln!("  ERROR: {}", e);
            return;
        }

        // 4b：读取 LED 数量（已缓存，无 JS 调用开销）
        let led_count = instance.led_count();

        // 4c：读取 LED 颜色缓冲区（从 JS 的 leds Uint8Array，格式 [R,G,B,R,G,B,...]）
        // 在嵌入式场景中，这里拿到的 &[u8] 直接传给硬件驱动（如 WS2812）
        let data = match instance.led_buffer() {
            Ok(data) => data,
            Err(e) => {
                eprintln!("  ERROR: {}", e);
                return;
            }
        };

        // 4d：渲染到终端（demo 用，嵌入式替换为 drive_leds(data)）
        render_frame(data, led_count, frame, FRAMES);
    }
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
