/// LED effects visual demo (EffectEngine API version)
///
/// Runs all 4 LED effects (blink, chase, rainbow, wave) through the
/// product-facing `EffectEngine` / `EffectInstance` API and renders the LED
/// strip in the terminal using ANSI 24-bit colors.
///
/// Usage:  cargo run --example effects_demo_engine
use mquickjs::EffectEngine;
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

    let engine = match EffectEngine::from_source(js) {
        Ok(engine) => engine,
        Err(e) => {
            eprintln!("  ERROR: {}", e);
            return;
        }
    };

    let mut instance = match engine.instantiate_from_expr("{ ledCount: 20 }") {
        Ok(instance) => instance,
        Err(e) => {
            eprintln!("  ERROR: {}", e);
            return;
        }
    };

    if let Err(e) = instance.start() {
        eprintln!("  ERROR: {}", e);
        return;
    }

    for frame in 0..FRAMES {
        if let Err(e) = instance.tick() {
            eprintln!("  ERROR: {}", e);
            return;
        }

        let led_count = match instance.led_count() {
            Ok(count) => count,
            Err(e) => {
                eprintln!("  ERROR: {}", e);
                return;
            }
        };

        let data = match instance.led_buffer() {
            Ok(data) => data,
            Err(e) => {
                eprintln!("  ERROR: {}", e);
                return;
            }
        };

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


