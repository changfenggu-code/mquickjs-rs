//! LED 效果帧捕获逻辑
//!
//! 预计算动画帧：运行 effect JS 脚本，将 RGB 数据返回给 GUI 播放。

use core::cell::RefCell;
use mquickjs::{Context, NativeFn, Value};

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

thread_local! {
    static CAPTURED: RefCell<Vec<Vec<[u8; 3]>>> = RefCell::new(Vec::new());
}

/// native_capture: JS 端调用的原生函数
///
/// 当 JS 脚本调用 `__capture(leds, ledCount)` 时，此函数被触发，
/// 将当前帧的 RGB 数据保存到 CAPTURED 中。
///
/// # 参数
/// - `interp`: JS 解释器
/// - `_this`: this 上下文（未使用）
/// - `args[0]`: leds (Uint8Array，RGB 数据)
/// - `args[1]`: ledCount (LED 数量，可选，默认 20)
///
/// # 返回
/// undefined
fn native_capture(
    interp: &mut mquickjs::vm::Interpreter,
    _this: Value,
    args: &[Value],
) -> Result<Value, String> {
    // 获取第一个参数：leds (Uint8Array)
    let leds_val = args.first().copied().unwrap_or_default();
    // 获取第二个参数：ledCount（LED 数量），默认为 20
    let led_count = args.get(1).and_then(|v| v.to_i32()).unwrap_or(20) as usize;
    // 读取 Uint8Array 的数据
    let data = interp.read_typed_array(leds_val).unwrap_or(&[]);

    // 将 RGB 数据转换为 Vec<[u8; 3]> 格式
    let mut frame = Vec::with_capacity(led_count);
    for i in 0..led_count {
        let o = i * 3;
        if o + 2 < data.len() {
            // RGB 三字节
            frame.push([data[o], data[o + 1], data[o + 2]]);
        } else {
            // 数据不足时填充黑色
            frame.push([0, 0, 0]);
        }
    }
    // 保存到线程本地的捕获队列
    CAPTURED.with(|c| c.borrow_mut().push(frame));
    Ok(Value::undefined())
}

/// capture_effect: 捕获单个效果的动画帧
///
/// 1. 创建 JS 上下文
/// 2. 注册 __capture 原生函数
/// 3. 执行 JS：创建效果 → start() → 循环 tick() 并捕获每一帧
/// 4. 返回捕获的帧数据
///
/// # 参数
/// - `name`: 效果名称
/// - `js`: 效果的 JS 源码
/// - `num_frames`: 需要捕获的帧数
///
/// # 返回
/// EffectData 包含效果名称、LED 数量、所有帧的 RGB 数据
pub fn capture_effect(name: &'static str, js: &str, num_frames: usize) -> EffectData {
    // 清空之前捕获的帧
    CAPTURED.with(|c| c.borrow_mut().clear());

    // 创建 JS 上下文（256KB 堆内存）
    let mut ctx = Context::new(256 * 1024);
    // 注册 __capture 函数到 JS 全局
    ctx.register_native("__capture", native_capture as NativeFn, 2);

    // 构造执行脚本：
    // 1. 加载效果脚本
    // 2. 创建效果实例
    // 3. 启动效果
    // 4. 循环 num_frames 次：tick() 更新帧 → __capture() 捕获数据
    let program = format!(
        "{js}\nvar __m = createEffect();\n__m.start();\n\
         for (var __i = 0; __i < {n}; __i++) {{ __m.tick(); __capture(__m.leds, __m.ledCount); }}",
        js = js,
        n = num_frames
    );
    // 执行 JS，可能失败会 panic
    ctx.eval(&program).expect("effect eval failed");

    // 取出所有捕获的帧
    let frames = CAPTURED.with(|c| c.borrow_mut().drain(..).collect::<Vec<_>>());
    // 从第一帧获取 LED 数量
    let led_count = frames.first().map(|f| f.len()).unwrap_or(20);
    EffectData {
        name,
        led_count,
        frames,
    }
}

/// capture_all: 捕获所有内置效果的动画帧
///
/// 遍历所有效果脚本（Blink、Chase、Rainbow、Wave），
/// 分别为每个效果捕获指定帧数的动画数据。
///
/// # 参数
/// - `num_frames`: 每个效果捕获的帧数
///
/// # 返回
/// Vec<EffectData> 包含所有效果的捕获数据
pub fn capture_all(num_frames: usize) -> Vec<EffectData> {
    vec![
        capture_effect("Blink", BLINK_JS, num_frames),
        capture_effect("Chase", CHASE_JS, num_frames),
        capture_effect("Rainbow", RAINBOW_JS, num_frames),
        capture_effect("Wave", WAVE_JS, num_frames),
    ]
}

