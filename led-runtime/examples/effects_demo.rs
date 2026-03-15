/// LED effects visual demo
///
/// Runs all 4 LED effects (blink, chase, rainbow, wave) through the mquickjs
/// JS engine and renders the LED strip in the terminal using ANSI 24-bit colors.
///
/// Usage:  cargo run --example effects_demo
use mquickjs::{Context, NativeFn, Value};
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

    print!("\x1b[?25l"); // hide cursor

    println!();
    println!("  mquickjs LED Effects Demo");
    println!("  ========================");
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

    print!("\x1b[?25h"); // show cursor
    println!("  Done.");
    println!();
}

/// Native: __renderFrame(leds, ledCount, frameNum, totalFrames)
/// Reads Uint8Array, prints ANSI colored LED strip on one line.
fn native_render(
    interp: &mut mquickjs::vm::Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let leds_val = args.first().copied().unwrap_or_default();
    let led_count = args.get(1).and_then(|v| v.to_i32()).unwrap_or(20) as usize;
    let frame = args.get(2).and_then(|v| v.to_i32()).unwrap_or(0);
    let total = args.get(3).and_then(|v| v.to_i32()).unwrap_or(1);

    let data = interp.read_typed_array(leds_val).unwrap_or(&[]);
    let out = std::io::stdout();
    let mut out = out.lock();

    // Overwrite previous line (except first frame)
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
    Ok(Value::undefined())
}

fn run_effect(name: &str, js: &str) {
    let mut ctx = Context::new(256 * 1024);
    ctx.register_native("__render", native_render as NativeFn, 4);

    // Single eval: effect script + driver loop
    let program = format!(
        r#"{js}
var __m = createEffect();
__m.start();
for (var __i = 0; __i < {frames}; __i++) {{
    __m.tick();
    __render(__m.leds, __m.ledCount, __i, {frames});
}}
"#,
        js = js,
        frames = FRAMES,
    );

    print!("  {name} :");
    println!();

    match ctx.eval(&program) {
        Ok(_) => {}
        Err(e) => eprintln!("  ERROR: {}", e),
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
