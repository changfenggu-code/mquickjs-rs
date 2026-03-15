//! Integration tests for LED effect scripts from webfly.
//!
//! Each effect script defines `createEffect(config)` which returns a state machine
//! with start/tick/stop/pause/resume methods and a `leds` Uint8Array buffer.
//!
//! Key JS features exercised:
//! - Object literals with shorthand properties (`{ speed }`)
//! - Uint8Array constructor + .fill()
//! - Math.abs, Math.round, Math.floor
//! - Closures capturing mutable state
//! - Nested function calls, `||` defaults, `!= null`

use mquickjs::Context;

const BLINK_JS: &str = include_str!("../js/effects/blink/effect.js");
const CHASE_JS: &str = include_str!("../js/effects/chase/effect.js");
const RAINBOW_JS: &str = include_str!("../js/effects/rainbow/effect.js");
const WAVE_JS: &str = include_str!("../js/effects/wave/effect.js");

/// Helper: load an effect script, append driver code, eval and return result.
fn eval_effect(effect_js: &str, driver: &str) -> mquickjs::Value {
    let mut ctx = Context::new(256 * 1024);
    let source = format!("{}\n{}", effect_js, driver);
    match ctx.eval(&source) {
        Ok(val) => val,
        Err(e) => panic!("JS eval failed: {}", e),
    }
}

// ── createEffect + basic structure ──

#[test]
fn test_blink_create_effect() {
    let result = eval_effect(
        BLINK_JS,
        r#"
        var m = createEffect();
        return m.status === "idle";
        "#,
    );
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_blink_default_led_count() {
    let result = eval_effect(
        BLINK_JS,
        r#"
        var m = createEffect();
        return m.ledCount;
        "#,
    );
    assert_eq!(result.to_i32(), Some(20));
}

#[test]
fn test_blink_default_speed() {
    let result = eval_effect(
        BLINK_JS,
        r#"
        var m = createEffect();
        return m.speed;
        "#,
    );
    assert_eq!(result.to_i32(), Some(200));
}

#[test]
fn test_blink_leds_is_uint8array() {
    let result = eval_effect(
        BLINK_JS,
        r#"
        var m = createEffect();
        return m.leds.length;
        "#,
    );
    // 20 LEDs * 3 bytes (RGB) = 60
    assert_eq!(result.to_i32(), Some(60));
}

// ── State machine transitions ──

#[test]
fn test_blink_start_sets_running() {
    let result = eval_effect(
        BLINK_JS,
        r#"
        var m = createEffect();
        m.start();
        return m.status === "running";
        "#,
    );
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_blink_pause_resume() {
    let result = eval_effect(
        BLINK_JS,
        r#"
        var m = createEffect();
        m.start();
        m.pause();
        var paused = m.status === "paused";
        m.resume();
        var running = m.status === "running";
        return paused && running;
        "#,
    );
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_blink_stop_resets() {
    let result = eval_effect(
        BLINK_JS,
        r#"
        var m = createEffect();
        m.start();
        m.stop();
        return m.status === "idle";
        "#,
    );
    assert_eq!(result.to_bool(), Some(true));
}

// ── Tick behavior ──

#[test]
fn test_blink_tick_when_idle_does_nothing() {
    // tick should be a no-op when not running
    let result = eval_effect(
        BLINK_JS,
        r#"
        var m = createEffect();
        m.tick();
        var allZero = true;
        for (var i = 0; i < m.leds.length; i++) {
            if (m.leds[i] !== 0) allZero = false;
        }
        return allZero;
        "#,
    );
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_blink_tick_toggles_leds() {
    // After start + 1 tick, LEDs should be on (non-zero)
    let result = eval_effect(
        BLINK_JS,
        r#"
        var m = createEffect();
        m.start();
        m.tick();
        var hasColor = m.leds[0] > 0 || m.leds[1] > 0 || m.leds[2] > 0;
        return hasColor;
        "#,
    );
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_blink_tick_second_toggles_off() {
    // After 2 ticks, LEDs should be off again
    let result = eval_effect(
        BLINK_JS,
        r#"
        var m = createEffect();
        m.start();
        m.tick();
        m.tick();
        var allZero = true;
        for (var i = 0; i < m.leds.length; i++) {
            if (m.leds[i] !== 0) allZero = false;
        }
        return allZero;
        "#,
    );
    assert_eq!(result.to_bool(), Some(true));
}

// ── Chase effect ──

#[test]
fn test_chase_create_and_tick() {
    let result = eval_effect(
        CHASE_JS,
        r#"
        var m = createEffect();
        m.start();
        m.tick();
        // First LED should have color (position 0)
        var hasColor = m.leds[0] > 0 || m.leds[1] > 0 || m.leds[2] > 0;
        return hasColor;
        "#,
    );
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_chase_moves_position() {
    let result = eval_effect(
        CHASE_JS,
        r#"
        var m = createEffect({ ledCount: 10, chaseCount: 1 });
        m.start();
        m.tick();
        // After first tick, position advances to 1, so LED 1 should be lit
        m.tick();
        var led1_has_color = m.leds[3] > 0 || m.leds[4] > 0 || m.leds[5] > 0;
        return led1_has_color;
        "#,
    );
    assert_eq!(result.to_bool(), Some(true));
}

// ── Rainbow effect ──

#[test]
fn test_rainbow_create_and_tick() {
    let result = eval_effect(
        RAINBOW_JS,
        r#"
        var m = createEffect();
        m.start();
        m.tick();
        // All LEDs should have color since rainbow fills every LED
        var hasColor = m.leds[0] > 0 || m.leds[1] > 0 || m.leds[2] > 0;
        return hasColor;
        "#,
    );
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_rainbow_different_colors_per_led() {
    // Adjacent LEDs should have different colors in rainbow
    let result = eval_effect(
        RAINBOW_JS,
        r#"
        var m = createEffect({ ledCount: 5, hueSpread: 72 });
        m.start();
        m.tick();
        // Compare LED 0 and LED 1 - they should differ
        var r0 = m.leds[0], g0 = m.leds[1], b0 = m.leds[2];
        var r1 = m.leds[3], g1 = m.leds[4], b1 = m.leds[5];
        var different = (r0 !== r1) || (g0 !== g1) || (b0 !== b1);
        return different;
        "#,
    );
    assert_eq!(result.to_bool(), Some(true));
}

// ── Wave effect ──

#[test]
fn test_wave_create_and_tick() {
    let result = eval_effect(
        WAVE_JS,
        r#"
        var m = createEffect({ ledCount: 10, waveWidth: 3 });
        m.start();
        m.tick();
        // First 3 LEDs should be lit (position 0, width 3)
        var led0 = m.leds[0] > 0 || m.leds[1] > 0 || m.leds[2] > 0;
        return led0;
        "#,
    );
    assert_eq!(result.to_bool(), Some(true));
}

#[test]
fn test_wave_moves() {
    let result = eval_effect(
        WAVE_JS,
        r#"
        var m = createEffect({ ledCount: 10, waveWidth: 1 });
        m.start();
        m.tick();
        // position was 0, wave at LED 0, then position advances to 1
        m.tick();
        // Now wave at LED 1, LED 0 should be dark
        var led0_dark = m.leds[0] === 0 && m.leds[1] === 0 && m.leds[2] === 0;
        var led1_lit = m.leds[3] > 0 || m.leds[4] > 0 || m.leds[5] > 0;
        return led0_dark && led1_lit;
        "#,
    );
    assert_eq!(result.to_bool(), Some(true));
}

// ── setConfig ──

#[test]
fn test_blink_set_speed() {
    let result = eval_effect(
        BLINK_JS,
        r#"
        var m = createEffect();
        m.setConfig("speed", 500);
        return m.speed;
        "#,
    );
    assert_eq!(result.to_i32(), Some(500));
}

#[test]
fn test_blink_set_color() {
    let result = eval_effect(
        BLINK_JS,
        r#"
        var m = createEffect();
        m.setConfig("color", { mode: "rgb", r: 255, g: 0, b: 0 });
        m.start();
        m.tick();
        // LED 0 should be pure red
        return m.leds[0];
        "#,
    );
    assert_eq!(result.to_i32(), Some(255));
}

// ── HSV color conversion ──

#[test]
fn test_rainbow_hsv_conversion() {
    // Rainbow uses hsvToRgb internally - verify it produces valid RGB
    let result = eval_effect(
        RAINBOW_JS,
        r#"
        var rgb = hsvToRgb(0, 1, 1);
        return rgb[0];
        "#,
    );
    // HSV(0, 1, 1) = pure red = RGB(255, 0, 0)
    assert_eq!(result.to_i32(), Some(255));
}

// ── Custom config ──

#[test]
fn test_create_with_custom_config() {
    let result = eval_effect(
        BLINK_JS,
        r#"
        var m = createEffect({ ledCount: 5, speed: 100 });
        return m.ledCount * 1000 + m.speed;
        "#,
    );
    // 5 * 1000 + 100 = 5100
    assert_eq!(result.to_i32(), Some(5100));
}

#[test]
fn test_custom_config_leds_buffer_size() {
    let result = eval_effect(
        BLINK_JS,
        r#"
        var m = createEffect({ ledCount: 5 });
        return m.leds.length;
        "#,
    );
    // 5 LEDs * 3 = 15
    assert_eq!(result.to_i32(), Some(15));
}

// ── Stop resets leds buffer ──

#[test]
fn test_stop_clears_leds() {
    let result = eval_effect(
        BLINK_JS,
        r#"
        var m = createEffect();
        m.start();
        m.tick();
        m.stop();
        var allZero = true;
        for (var i = 0; i < m.leds.length; i++) {
            if (m.leds[i] !== 0) allZero = false;
        }
        return allZero;
        "#,
    );
    assert_eq!(result.to_bool(), Some(true));
}
