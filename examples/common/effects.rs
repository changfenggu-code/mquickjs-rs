/// Shared frame-capture logic for LED effects demos.
///
/// Pre-captures animation frames by running effect JS through mquickjs,
/// then returns the raw RGB data for GUI playback.
use core::cell::RefCell;
use mquickjs::{Context, NativeFn, Value};

pub const BLINK_JS: &str = include_str!("../../js/effects/blink/effect.js");
pub const CHASE_JS: &str = include_str!("../../js/effects/chase/effect.js");
pub const RAINBOW_JS: &str = include_str!("../../js/effects/rainbow/effect.js");
pub const WAVE_JS: &str = include_str!("../../js/effects/wave/effect.js");

pub struct EffectData {
    pub name: &'static str,
    pub led_count: usize,
    pub frames: Vec<Vec<[u8; 3]>>,
}

thread_local! {
    static CAPTURED: RefCell<Vec<Vec<[u8; 3]>>> = RefCell::new(Vec::new());
}

fn native_capture(
    interp: &mut mquickjs::vm::Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    let leds_val = args.first().copied().unwrap_or_default();
    let led_count = args.get(1).and_then(|v| v.to_i32()).unwrap_or(20) as usize;
    let data = interp.read_typed_array(leds_val).unwrap_or(&[]);

    let mut frame = Vec::with_capacity(led_count);
    for i in 0..led_count {
        let o = i * 3;
        if o + 2 < data.len() {
            frame.push([data[o], data[o + 1], data[o + 2]]);
        } else {
            frame.push([0, 0, 0]);
        }
    }
    CAPTURED.with(|c| c.borrow_mut().push(frame));
    Ok(Value::undefined())
}

pub fn capture_effect(name: &'static str, js: &str, num_frames: usize) -> EffectData {
    CAPTURED.with(|c| c.borrow_mut().clear());

    let mut ctx = Context::new(256 * 1024);
    ctx.register_native("__capture", native_capture as NativeFn, 2);

    let program = format!(
        "{js}\nvar __m = createEffect();\n__m.start();\n\
         for (var __i = 0; __i < {n}; __i++) {{ __m.tick(); __capture(__m.leds, __m.ledCount); }}",
        js = js,
        n = num_frames
    );
    ctx.eval(&program).expect("effect eval failed");

    let frames = CAPTURED.with(|c| c.borrow_mut().drain(..).collect::<Vec<_>>());
    let led_count = frames.first().map(|f| f.len()).unwrap_or(20);
    EffectData {
        name,
        led_count,
        frames,
    }
}

pub fn capture_all(num_frames: usize) -> Vec<EffectData> {
    vec![
        capture_effect("Blink", BLINK_JS, num_frames),
        capture_effect("Chase", CHASE_JS, num_frames),
        capture_effect("Rainbow", RAINBOW_JS, num_frames),
        capture_effect("Wave", WAVE_JS, num_frames),
    ]
}
