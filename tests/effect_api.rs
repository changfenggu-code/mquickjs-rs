use mquickjs::{ConfigValue, Context, EffectEngine};

const BLINK_JS: &str = include_str!("../js/effects/blink/effect.js");

#[test]
fn effect_engine_from_source_runs_blink() {
    let engine = EffectEngine::from_source(BLINK_JS).unwrap();
    let mut instance = engine.instantiate("{ ledCount: 4 }").unwrap();

    instance.start().unwrap();
    instance.tick().unwrap();

    assert_eq!(instance.led_count().unwrap(), 4);
    let leds = instance.led_buffer().unwrap();
    assert_eq!(leds.len(), 12);
    assert!(leds.iter().any(|b| *b != 0));
}

#[test]
fn effect_engine_from_bytecode_runs_blink() {
    let ctx = Context::new(64 * 1024);
    let bytecode = ctx.compile(BLINK_JS).unwrap();
    let bytes = bytecode.serialize();

    let engine = EffectEngine::from_bytecode(&bytes).unwrap();
    let mut instance = engine.instantiate("{ ledCount: 3 }").unwrap();

    instance.start().unwrap();
    instance.tick().unwrap();

    assert_eq!(instance.led_count().unwrap(), 3);
    let leds = instance.led_buffer().unwrap();
    assert_eq!(leds.len(), 9);
}

#[test]
fn effect_instance_set_config_and_reset() {
    let engine = EffectEngine::from_source(BLINK_JS).unwrap();
    let mut instance = engine.instantiate("{ ledCount: 2, speed: 100 }").unwrap();

    instance.set_config("speed", ConfigValue::Int(500)).unwrap();
    instance.start().unwrap();
    instance.tick().unwrap();
    let first = instance.led_buffer().unwrap().to_vec();

    instance.reset().unwrap();
    instance.start().unwrap();
    instance.tick().unwrap();
    let second = instance.led_buffer().unwrap().to_vec();

    assert_eq!(first.len(), second.len());
    assert_eq!(first, second);
}
