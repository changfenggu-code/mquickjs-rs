#[path = "../examples/common/effects.rs"]
mod effects_legacy;

#[path = "../examples/common/effects_engine.rs"]
mod effects_engine;

#[test]
fn capture_effect_via_engine_returns_expected_shape() {
    let data = effects_engine::capture_effect_via_engine("Blink", effects_engine::BLINK_JS, 5);

    assert_eq!(data.name, "Blink");
    assert_eq!(data.led_count, 20);
    assert_eq!(data.frames.len(), 5);
    assert!(data.frames.iter().all(|frame| frame.len() == 20));
}

#[test]
fn capture_effect_via_engine_matches_legacy_helper_for_blink() {
    let legacy = effects_legacy::capture_effect("Blink", effects_legacy::BLINK_JS, 6);
    let engine = effects_engine::capture_effect_via_engine("Blink", effects_engine::BLINK_JS, 6);

    assert_eq!(engine.name, legacy.name);
    assert_eq!(engine.led_count, legacy.led_count);
    assert_eq!(engine.frames, legacy.frames);
}

#[test]
fn capture_all_via_engine_returns_all_effects() {
    let all = effects_engine::capture_all_via_engine(3);

    assert_eq!(all.len(), 4);
    assert_eq!(all[0].name, "Blink");
    assert_eq!(all[1].name, "Chase");
    assert_eq!(all[2].name, "Rainbow");
    assert_eq!(all[3].name, "Wave");
    assert!(all.iter().all(|d| d.frames.len() == 3));
}
