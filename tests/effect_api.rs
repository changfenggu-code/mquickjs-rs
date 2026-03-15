use mquickjs::{
    BlinkConfig, ChaseConfig, ColorConfig, ConfigValue, Context, EffectEngine, EffectManager,
    RainbowConfig, WaveConfig,
};

const BLINK_JS: &str = include_str!("../js/effects/blink/effect.js");
const CHASE_JS: &str = include_str!("../js/effects/chase/effect.js");
const RAINBOW_JS: &str = include_str!("../js/effects/rainbow/effect.js");

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

#[test]
fn effect_engine_supports_multiple_independent_instances() {
    let engine = EffectEngine::from_source(BLINK_JS).unwrap();

    let mut a = engine.instantiate("{ ledCount: 2 }").unwrap();
    let mut b = engine.instantiate("{ ledCount: 5 }").unwrap();

    a.start().unwrap();
    b.start().unwrap();

    a.tick().unwrap();
    let a_buf = a.led_buffer().unwrap().to_vec();
    let b_buf_before = b.led_buffer().unwrap().to_vec();

    assert_eq!(a.led_count().unwrap(), 2);
    assert_eq!(b.led_count().unwrap(), 5);
    assert_eq!(a_buf.len(), 6);
    assert_eq!(b_buf_before.len(), 15);

    // instance b should remain at its own initial state until ticked
    b.tick().unwrap();
    let b_buf_after = b.led_buffer().unwrap().to_vec();
    assert_ne!(b_buf_before, b_buf_after);
}

#[test]
fn effect_instance_pause_resume_stop_lifecycle() {
    let engine = EffectEngine::from_source(BLINK_JS).unwrap();
    let mut instance = engine.instantiate("{ ledCount: 3 }").unwrap();

    instance.start().unwrap();
    instance.tick().unwrap();
    let running = instance.led_buffer().unwrap().to_vec();
    assert!(running.iter().any(|b| *b != 0));

    instance.pause().unwrap();
    let paused_before = instance.led_buffer().unwrap().to_vec();
    instance.tick().unwrap();
    let paused_after = instance.led_buffer().unwrap().to_vec();
    assert_eq!(paused_before, paused_after);

    instance.resume().unwrap();
    instance.tick().unwrap();
    let resumed = instance.led_buffer().unwrap().to_vec();
    assert_ne!(paused_after, resumed);

    instance.stop().unwrap();
    let stopped = instance.led_buffer().unwrap().to_vec();
    assert!(stopped.iter().all(|b| *b == 0));
}

#[test]
fn effect_instance_set_config_changes_behavior() {
    let engine = EffectEngine::from_source(BLINK_JS).unwrap();
    let mut instance = engine.instantiate("{ ledCount: 2 }").unwrap();

    instance
        .set_config(
            "color",
            ConfigValue::Object(vec![
                ("mode".into(), ConfigValue::Str("rgb".into())),
                ("r".into(), ConfigValue::Int(255)),
                ("g".into(), ConfigValue::Int(0)),
                ("b".into(), ConfigValue::Int(0)),
            ]),
        )
        .unwrap();

    instance.set_config("speed", ConfigValue::Int(250)).unwrap();
    instance.start().unwrap();
    instance.tick().unwrap();

    let leds = instance.led_buffer().unwrap();
    assert_eq!(leds.len(), 6);
    assert_eq!(leds[0], 255);
    assert_eq!(leds[1], 0);
    assert_eq!(leds[2], 0);
}

#[test]
fn effect_engine_instantiate_config_supports_nested_objects() {
    let engine = EffectEngine::from_source(BLINK_JS).unwrap();
    let config = ConfigValue::Object(vec![
        ("ledCount".into(), ConfigValue::Int(2)),
        (
            "color".into(),
            ConfigValue::Object(vec![
                ("mode".into(), ConfigValue::Str("rgb".into())),
                ("r".into(), ConfigValue::Int(255)),
                ("g".into(), ConfigValue::Int(0)),
                ("b".into(), ConfigValue::Int(0)),
            ]),
        ),
    ]);

    let mut instance = engine.instantiate_config(config).unwrap();
    instance.start().unwrap();
    instance.tick().unwrap();

    let leds = instance.led_buffer().unwrap();
    assert_eq!(leds.len(), 6);
    assert_eq!(leds[0], 255);
    assert_eq!(leds[1], 0);
    assert_eq!(leds[2], 0);
}

#[test]
fn config_value_supports_arrays() {
    let array_literal = ConfigValue::Array(vec![
        ConfigValue::Int(1),
        ConfigValue::Float(2.5),
        ConfigValue::Str("x".into()),
    ]);

    let engine = EffectEngine::from_source(BLINK_JS).unwrap();
    let _ = engine.instantiate_config(ConfigValue::Object(vec![
        ("ledCount".into(), ConfigValue::Int(2)),
        ("extra".into(), array_literal),
    ]))
    .unwrap();
}

#[test]
fn typed_blink_config_can_instantiate_engine() {
    let engine = EffectEngine::from_source(BLINK_JS).unwrap();
    let config = BlinkConfig {
        led_count: Some(2),
        speed: Some(100),
        color: Some(ColorConfig::Rgb { r: 255, g: 0, b: 0 }),
    };

    let mut instance = engine.instantiate_config(config.into()).unwrap();
    instance.start().unwrap();
    instance.tick().unwrap();

    let leds = instance.led_buffer().unwrap();
    assert_eq!(leds.len(), 6);
    assert_eq!(leds[0], 255);
    assert_eq!(leds[1], 0);
    assert_eq!(leds[2], 0);
}

#[test]
fn typed_rainbow_config_can_instantiate_engine() {
    let engine = EffectEngine::from_source(RAINBOW_JS).unwrap();
    let config = RainbowConfig {
        led_count: Some(3),
        speed: Some(100),
        hue_step: Some(15),
        hue_spread: Some(60),
        saturation: Some(1.0),
        brightness: Some(1.0),
    };

    let mut instance = engine.instantiate_config(config.into()).unwrap();
    instance.start().unwrap();
    instance.tick().unwrap();

    let leds = instance.led_buffer().unwrap();
    assert_eq!(leds.len(), 9);
    assert!(leds.iter().any(|b| *b != 0));
}

#[test]
fn typed_chase_and_wave_configs_can_build_config_values() {
    let chase: ConfigValue = ChaseConfig {
        led_count: Some(5),
        speed: Some(80),
        color: Some(ColorConfig::Rgb {
            r: 251,
            g: 191,
            b: 36,
        }),
        chase_count: Some(2),
    }
    .into();

    let wave: ConfigValue = WaveConfig {
        led_count: Some(5),
        speed: Some(120),
        color: Some(ColorConfig::Hsv {
            h: 120.0,
            s: 1.0,
            v: 1.0,
        }),
        wave_width: Some(3),
    }
    .into();

    match chase {
        ConfigValue::Object(entries) => assert!(!entries.is_empty()),
        _ => panic!("expected object config"),
    }

    match wave {
        ConfigValue::Object(entries) => assert!(!entries.is_empty()),
        _ => panic!("expected object config"),
    }
}

#[test]
fn different_effect_engines_can_coexist() {
    let blink = EffectEngine::from_source(BLINK_JS).unwrap();
    let chase = EffectEngine::from_source(CHASE_JS).unwrap();

    let mut blink_instance = blink.instantiate("{ ledCount: 4 }").unwrap();
    let mut chase_instance = chase
        .instantiate("{ ledCount: 4, chaseCount: 1 }")
        .unwrap();

    blink_instance.start().unwrap();
    chase_instance.start().unwrap();

    blink_instance.tick().unwrap();
    chase_instance.tick().unwrap();

    let blink_buf = blink_instance.led_buffer().unwrap().to_vec();
    let chase_buf = chase_instance.led_buffer().unwrap().to_vec();

    assert_eq!(blink_buf.len(), 12);
    assert_eq!(chase_buf.len(), 12);
    assert_ne!(blink_buf, chase_buf);
}

#[test]
fn interleaved_multi_script_instances_keep_independent_state() {
    let blink = EffectEngine::from_source(BLINK_JS).unwrap();
    let rainbow = EffectEngine::from_source(RAINBOW_JS).unwrap();

    let mut blink_instance = blink.instantiate("{ ledCount: 3 }").unwrap();
    let mut rainbow_instance = rainbow
        .instantiate("{ ledCount: 3, hueSpread: 60 }")
        .unwrap();

    blink_instance.start().unwrap();
    rainbow_instance.start().unwrap();

    blink_instance.tick().unwrap();
    let blink_first = blink_instance.led_buffer().unwrap().to_vec();

    rainbow_instance.tick().unwrap();
    let rainbow_first = rainbow_instance.led_buffer().unwrap().to_vec();

    blink_instance.tick().unwrap();
    let blink_second = blink_instance.led_buffer().unwrap().to_vec();

    rainbow_instance.tick().unwrap();
    let rainbow_second = rainbow_instance.led_buffer().unwrap().to_vec();

    assert_ne!(blink_first, blink_second);
    assert_ne!(rainbow_first, rainbow_second);
    assert_ne!(blink_second, rainbow_second);
}

#[test]
fn effect_manager_can_activate_and_tick_instances() {
    let mut manager = EffectManager::new();
    manager
        .add_engine("blink", EffectEngine::from_source(BLINK_JS).unwrap())
        .unwrap();
    manager
        .add_engine("rainbow", EffectEngine::from_source(RAINBOW_JS).unwrap())
        .unwrap();

    let blink_idx = manager
        .instantiate("blink", "blink-a", "{ ledCount: 3 }")
        .unwrap();
    let rainbow_idx = manager
        .instantiate("rainbow", "rainbow-a", "{ ledCount: 3, hueSpread: 60 }")
        .unwrap();

    manager.activate(blink_idx).unwrap();
    assert_eq!(manager.active_name(), Some("blink-a"));
    assert_eq!(manager.active_engine_name(), Some("blink"));
    manager.start_active().unwrap();
    manager.tick_active().unwrap();
    let blink_buf = manager.active_led_buffer().unwrap().to_vec();

    manager.activate(rainbow_idx).unwrap();
    assert_eq!(manager.active_name(), Some("rainbow-a"));
    assert_eq!(manager.active_engine_name(), Some("rainbow"));
    manager.start_active().unwrap();
    manager.tick_active().unwrap();
    let rainbow_buf = manager.active_led_buffer().unwrap().to_vec();

    assert_eq!(blink_buf.len(), 9);
    assert_eq!(rainbow_buf.len(), 9);
    assert_ne!(blink_buf, rainbow_buf);
}

#[test]
fn effect_manager_switching_preserves_instance_state() {
    let mut manager = EffectManager::new();
    manager
        .add_engine("blink", EffectEngine::from_source(BLINK_JS).unwrap())
        .unwrap();
    manager
        .add_engine("chase", EffectEngine::from_source(CHASE_JS).unwrap())
        .unwrap();

    let blink_idx = manager
        .instantiate("blink", "blink-a", "{ ledCount: 2 }")
        .unwrap();
    let chase_idx = manager
        .instantiate("chase", "chase-a", "{ ledCount: 2, chaseCount: 1 }")
        .unwrap();

    manager.activate(blink_idx).unwrap();
    manager.start_active().unwrap();
    manager.tick_active().unwrap();
    let blink_first = manager.active_led_buffer().unwrap().to_vec();

    manager.activate(chase_idx).unwrap();
    manager.start_active().unwrap();
    manager.tick_active().unwrap();
    let chase_first = manager.active_led_buffer().unwrap().to_vec();

    manager.activate(blink_idx).unwrap();
    manager.tick_active().unwrap();
    let blink_second = manager.active_led_buffer().unwrap().to_vec();

    assert_ne!(blink_first, blink_second);
    assert_ne!(blink_second, chase_first);
}

#[test]
fn effect_manager_can_activate_by_name_and_list_instances() {
    let mut manager = EffectManager::new();
    manager
        .add_engine("blink", EffectEngine::from_source(BLINK_JS).unwrap())
        .unwrap();
    manager
        .add_engine("rainbow", EffectEngine::from_source(RAINBOW_JS).unwrap())
        .unwrap();

    assert_eq!(manager.engine_count(), 2);

    manager
        .instantiate("blink", "blink-a", "{ ledCount: 2 }")
        .unwrap();
    manager
        .instantiate("rainbow", "rainbow-a", "{ ledCount: 2 }")
        .unwrap();

    assert_eq!(manager.engine_names(), vec!["blink", "rainbow"]);
    assert_eq!(manager.instance_names(), vec!["blink-a", "rainbow-a"]);
    assert_eq!(manager.instance_count(), 2);

    manager.activate_by_name("rainbow-a").unwrap();
    assert_eq!(manager.active_name(), Some("rainbow-a"));
    assert_eq!(manager.active_engine_name(), Some("rainbow"));
}

#[test]
fn effect_manager_can_remove_instances() {
    let mut manager = EffectManager::new();
    manager
        .add_engine("blink", EffectEngine::from_source(BLINK_JS).unwrap())
        .unwrap();

    let a = manager
        .instantiate("blink", "blink-a", "{ ledCount: 2 }")
        .unwrap();
    let _b = manager
        .instantiate("blink", "blink-b", "{ ledCount: 2 }")
        .unwrap();

    manager.activate(a).unwrap();
    assert_eq!(manager.active_name(), Some("blink-a"));

    manager.remove_instance_by_name("blink-a").unwrap();
    assert_eq!(manager.instance_count(), 1);
    assert_eq!(manager.instance_names(), vec!["blink-b"]);
    assert_eq!(manager.active_name(), None);

    manager.activate_by_name("blink-b").unwrap();
    assert_eq!(manager.active_name(), Some("blink-b"));
}

#[test]
fn effect_manager_rejects_duplicate_names() {
    let mut manager = EffectManager::new();

    manager
        .add_engine("blink", EffectEngine::from_source(BLINK_JS).unwrap())
        .unwrap();
    assert!(manager
        .add_engine("blink", EffectEngine::from_source(BLINK_JS).unwrap())
        .is_err());

    manager
        .instantiate("blink", "blink-a", "{ ledCount: 2 }")
        .unwrap();
    assert!(manager
        .instantiate("blink", "blink-a", "{ ledCount: 2 }")
        .is_err());
}

#[test]
fn effect_manager_can_query_and_remove_by_engine() {
    let mut manager = EffectManager::new();
    manager
        .add_engine("blink", EffectEngine::from_source(BLINK_JS).unwrap())
        .unwrap();
    manager
        .add_engine("rainbow", EffectEngine::from_source(RAINBOW_JS).unwrap())
        .unwrap();

    manager
        .instantiate("blink", "blink-a", "{ ledCount: 2 }")
        .unwrap();
    manager
        .instantiate("blink", "blink-b", "{ ledCount: 3 }")
        .unwrap();
    manager
        .instantiate("rainbow", "rainbow-a", "{ ledCount: 2 }")
        .unwrap();

    assert_eq!(manager.instances_for_engine("blink"), vec!["blink-a", "blink-b"]);
    assert_eq!(manager.instances_for_engine("rainbow"), vec!["rainbow-a"]);

    let removed = manager.remove_instances_by_engine("blink");
    assert_eq!(removed, 2);
    assert_eq!(manager.instance_names(), vec!["rainbow-a"]);
}

#[test]
fn effect_manager_can_instantiate_with_typed_config() {
    let mut manager = EffectManager::new();
    manager
        .add_engine("blink", EffectEngine::from_source(BLINK_JS).unwrap())
        .unwrap();

    manager
        .instantiate_config(
            "blink",
            "blink-a",
            BlinkConfig {
                led_count: Some(2),
                speed: Some(100),
                color: Some(ColorConfig::Rgb { r: 255, g: 0, b: 0 }),
            }
            .into(),
        )
        .unwrap();

    manager.activate_by_name("blink-a").unwrap();
    manager.start_active().unwrap();
    manager.tick_active().unwrap();
    let leds = manager.active_led_buffer().unwrap().to_vec();
    assert_eq!(leds.len(), 6);
    assert_eq!(leds[0], 255);
}
