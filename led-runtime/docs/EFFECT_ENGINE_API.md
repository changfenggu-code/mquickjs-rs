# EffectEngine API 使用说明

本文档说明当前仓库中新增的最小产品级 effect 宿主 API：

- `EffectEngine`
- `EffectInstance`
- `EffectManager`
- `ConfigValue`

这套 API 的目标是：

- 让宿主侧不再依赖手工拼接 driver JS
- 把 effect 的“加载 / 实例化 / 逐帧执行 / 读取 LED buffer / 更新配置 / 重置”收口成稳定 Rust 接口

相关源码：

- `led-runtime/src/effect.rs`
- `led-runtime/tests/effect_api.rs`
- `LED_PROFILE.md`

> 说明：当前 workspace 拆分已经开始，产品层 API 的主开发位置应优先视为 `led-runtime/`。根目录仍保留部分过渡期文件，用于逐步迁移与对照。

## 1. 它解决了什么问题

在此前的示例工作流中，宿主层通常需要：

1. 把 effect 脚本内嵌成字符串
2. 创建 `Context`
3. 注册 native 函数
4. 手工拼接 driver JS
5. 执行 `createEffect()` / `tick()`
6. 手工读取 `leds: Uint8Array`

这种方式适合示例与演示，但不适合作为正式宿主 API。

`EffectEngine` 的作用就是把这条链路收口起来。

### 手动方式 vs 新 API 方式

下面这张表可以直接回答：

- 宿主层如果手动要自己做什么
- 现在分别由哪个抽象封装起来了

| 宿主手动要做的事 | 现在由谁封装 | 当前推荐用法 |
|---|---|---|
| 把 effect 脚本内嵌成字符串后再手动处理 | `EffectEngine` | `EffectEngine::from_source(js)` |
| 手工创建 `Context` | `EffectEngine` / `EffectInstance` 内部 | `engine.instantiate_from_expr(...)` / `instantiate_config(...)` |
| 注册 native 函数把 `leds` 传回 Rust | `EffectInstance` | `instance.led_buffer()` |
| 手工拼接 driver JS | `EffectEngine` / `EffectInstance` 内部 | 不再推荐宿主手拼 |
| 手工调用 `createEffect()` | `EffectEngine` | `instantiate_from_expr(...)` / `instantiate_config(...)` |
| 手工调用 `tick()` / `start()` / `stop()` | `EffectInstance` | `instance.tick()` / `start()` / `stop()` |
| 手工读取 `Uint8Array leds` | `EffectInstance` / `EffectManager` | `led_buffer()` / `active_led_buffer()` |
| 手工拼配置字符串 | `ConfigValue` | `instantiate_config(config)` |
| 管理多个 effect / 多实例切换 | `EffectManager` | `add_engine()` / `instantiate_*()` / `activate_*()` |

一句话说：

- **旧方式**：宿主自己驱动 JS 引擎细节
- **新方式**：宿主优先面向 `EffectManager` 编程；`EffectEngine` / `EffectInstance` 作为底层构件保留

## 2. 核心类型

### `EffectEngine`

表示：

- 一份 effect 脚本模板
- 或一份可实例化的 effect 字节码

主要入口：

- `EffectEngine::from_source(source)` — JS 源码编译为字节码存入内存，开发阶段使用
- `EffectEngine::from_bytecode(bytes)` — 直接加载预编译字节码，生产环境秒开
- `engine.instantiate_from_expr(config_expr)` — 底层接口：直接传 JS 配置表达式字符串
- `engine.instantiate_config(config)` — 更正式的宿主接口：传 `ConfigValue`

### `EffectInstance`

表示：

- 一个真正运行中的 effect 实例

主要能力：

- `start()`
- `tick()`
- `pause()`
- `resume()`
- `stop()`
- `led_buffer()`
- `led_count()`
- `set_config()`
- `reset()`

### `ConfigValue`

用于把 Rust 侧配置值传回 effect 的 `setConfig(key, value)` 接口。

当前支持：

- `Undefined`
- `Null`
- `Bool(bool)`
- `Int(i32)`
- `Float(f32)`
- `Str(String)`
- `Array(Vec<ConfigValue>)`
- `Object(Vec<(String, ConfigValue)>)`

### 配置风格

当前核心库统一采用：

- `ConfigValue`

作为正式配置入口。

也就是说：

- 复杂对象配置使用 `ConfigValue::Object(...)`
- 数组配置使用 `ConfigValue::Array(...)`
- 不再要求为每个 effect 预先提供专门的强类型配置结构体

### `EffectManager`

表示：

- 一个最小的宿主侧调度层
- 用于管理多个 effect engine、多个实例，以及当前激活实例

主要能力：

- `EffectManager::new()`
- `add_engine(name, engine)`
- `instantiate_from_expr(engine_name, instance_name, config_expr)`
- `instantiate_config(engine_name, instance_name, config)`
- `activate(instance_idx)`
- `activate_by_name(instance_name)`
- `active_name()` / `active_engine_name()`
- `engine_names()` / `engine_count()`
- `instance_names()` / `instance_count()`
- `instances_for_engine(engine_name)`
- `remove_instance(index)` / `remove_instance_by_name(name)`
- `remove_instances_by_engine(engine_name)`
- `start_active()` / `tick_active()` / `pause_active()` / `resume_active()` / `stop_active()`
- `set_active_config(key, value)`
- `reset_active()`
- `memory_stats_active()`
- `active_led_buffer()` / `active_led_count()`

## 3. 基本用法

### 从源码创建 effect 引擎

```rust
use led_runtime::EffectEngine;

let js = include_str!("../js/effects/blink/effect.js");
let engine = EffectEngine::from_source(js)?;
```

### 创建实例（字符串配置）

```rust
let mut instance = engine.instantiate_from_expr("{ ledCount: 4, speed: 100 }")?;
```

这里的 `config_expr` 当前是一个 **JS 对象字面量字符串**。

例如：

- `"{ ledCount: 4 }"`
- `"{ ledCount: 60, speed: 120 }"`

当前设计保留了这条底层接口，用于最大兼容现有脚本和调试场景。

### 创建实例（结构化配置）

```rust
use led_runtime::{ConfigValue, EffectEngine};

let engine = EffectEngine::from_source(js)?;
let config = ConfigValue::Object(vec![
    ("ledCount".into(), ConfigValue::Int(4)),
    ("speed".into(), ConfigValue::Int(100)),
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

let mut instance = engine.instantiate_config(config)?;
```

这是当前更推荐的宿主侧写法。

### 驱动生命周期

```rust
instance.start()?;
instance.tick()?;
```

如果脚本实现了这些方法，也可以调用：

```rust
instance.pause()?;
instance.resume()?;
instance.stop()?;
```

### 读取 LED buffer

```rust
let leds = instance.led_buffer()?;
let led_count = instance.led_count()?;
```

当前 `led_buffer()` 返回：

- `&[u8]`

这更适合设备侧直接交给 LED 驱动层。

### 动态更新配置

```rust
use led_runtime::ConfigValue;

instance.set_config("speed", ConfigValue::Int(500))?;
instance.set_config("label", ConfigValue::Str("demo".into()))?;
```

如果配置值本身是对象，也可以直接传：

```rust
instance.set_config(
    "color",
    ConfigValue::Object(vec![
        ("mode".into(), ConfigValue::Str("rgb".into())),
        ("r".into(), ConfigValue::Int(255)),
        ("g".into(), ConfigValue::Int(0)),
        ("b".into(), ConfigValue::Int(0)),
    ]),
)?;
```

### 重置实例

```rust
instance.reset()?;
```

`reset()` 的含义是：

- 使用当前保存的配置重新调用 `createEffect(config)`
- 重新生成 effect 实例状态

## 4. 从字节码创建引擎

### 方式 A：直接用 `FunctionBytecode::serialize()` 的结果

```rust
use mquickjs::Context;
use led_runtime::EffectEngine;

let ctx = Context::new(64 * 1024);
let bytecode = ctx.compile(js_source)?;
let bytes = bytecode.serialize();

let engine = EffectEngine::from_bytecode(&bytes)?;
```

### 方式 B：使用 `.qbc` 文件内容

如果是 `.qbc` 文件，支持带头部的格式：

- magic: `MQJS`
- version: 1 字节
- payload: `FunctionBytecode::serialize()` 的内容

`EffectEngine::from_bytecode()` 会自动识别并剥离这个头部。

## 5. 当前 API 的实现方式

这套 API 不是重新发明一套 VM，而是建立在现有 `Context` 能力之上：

- `from_source()`：编译源码并保存字节码序列化结果
- `from_bytecode()`：保存字节码 payload
- `instantiate_from_expr()` / `instantiate_config()`：
  - 创建 `Context`
  - 加载 effect 字节码
  - 创建配置对象并放进全局变量
  - 调用 `createEffect(config)` 生成实例
  - 再预编译 `start/tick/pause/resume/stop/leds/ledCount` 访问脚本

这意味着当前版本的 `EffectEngine`：

- 已经把宿主使用方式收口成 Rust API
- 但内部仍然是建立在现有 `Context` / `FunctionBytecode` 之上的最小封装

## 6. 当前 API 的边界

这是一版**最小可用雏形**，不是最终定稿。

当前已具备：

- 从源码/字节码创建引擎
- 实例化 effect
- 通过 `instantiate_config(...)` 使用结构化配置实例化 effect
- 生命周期方法调用
- 读取 LED buffer
- 更新配置
- 重置实例
- 最小调度层（`EffectManager`）

当前仍未完全产品化的部分：

- 更统一的通用基础配置层（当前仍以通用 `ConfigValue` 为正式配置入口）
- 多实例 / 多脚本运行模型的进一步抽象（当前已有最小 `EffectManager` 雏形）
- 更正式的错误类型
- 更完整的文档与示例矩阵
- 更严格的内存与执行预算集成

## 6.1 `EffectManager` 的作用

当宿主侧需要：

- 预加载多个 effect
- 管理多个实例
- 在不同实例间切换当前激活效果

时，可以使用 `EffectManager`。

一个最小示例：

```rust
use led_runtime::{EffectEngine, EffectManager};

let mut manager = EffectManager::new();
manager.add_engine("blink", EffectEngine::from_source(BLINK_JS)?)?;
manager.add_engine("rainbow", EffectEngine::from_source(RAINBOW_JS)?)?;

let blink_idx = manager.instantiate_from_expr("blink", "blink-a", "{ ledCount: 20 }")?;
let rainbow_idx = manager.instantiate_from_expr("rainbow", "rainbow-a", "{ ledCount: 20 }")?;

manager.activate(blink_idx)?;
manager.start_active()?;
manager.tick_active()?;
let blink_leds = manager.active_led_buffer()?;

manager.activate(rainbow_idx)?;
manager.start_active()?;
manager.tick_active()?;
let rainbow_leds = manager.active_led_buffer()?;
```

这说明当前仓库已经不只具备“单 effect 实例 API”，还具备“最小调度层雏形”。

如果你希望完全采用结构化配置，也可以这样使用：

```rust
use led_runtime::{ConfigValue, EffectEngine, EffectManager};

let mut manager = EffectManager::new();
manager.add_engine("blink", EffectEngine::from_source(BLINK_JS)?)?;

manager.instantiate_config(
    "blink",
    "blink-a",
    ConfigValue::Object(vec![
        ("ledCount".into(), ConfigValue::Int(20)),
        ("speed".into(), ConfigValue::Int(100)),
    ]),
)?;

manager.activate_by_name("blink-a")?;
manager.start_active()?;
manager.tick_active()?;
let leds = manager.active_led_buffer()?;
```

### 当前 `EffectManager` 能实现什么

当前这层已经可以支持：

- 预加载多个不同 effect engine
- 同一个 engine 创建多个实例
- 通过索引或实例名激活当前实例
- 查询有哪些 engine / instance 已加载
- 按实例删除、按 engine 批量删除实例
- 直接修改当前激活实例配置
- 直接重置当前激活实例
- 查询当前激活实例的内存统计信息
- 读取当前激活实例的 LED buffer 并驱动宿主输出

### 如果以后主要以 `EffectManager` 作为宿主主入口

现在的接口分层可以理解成：

- `EffectEngine` / `EffectInstance`：底层构件层
- `EffectManager`：宿主主入口层

推荐宿主主流程优先使用：

- `add_engine(...)`
- `instantiate_config(...)`（优先） / `instantiate_from_expr(...)`（低层兼容）
- `activate_by_name(...)`
- `set_active_config(...)`
- `start_active()` / `tick_active()`
- `active_led_buffer()` / `active_led_count()`
- `reset_active()`
- `memory_stats_active()`

这样宿主层就可以尽量少直接操作底层 `EffectInstance`。

### 新旧示例对照

| 文件/接口 | 定位 |
|---|---|
| `led-runtime/examples/effects_demo.rs` | 旧方式：`Context + driver JS` 终端演示 |
| `led-runtime/examples/common/effects.rs` | 旧方式 helper：离线采帧工具 |
| `led-runtime/examples/effects_demo_engine.rs` | 新方式：`EffectEngine` / `EffectInstance` 终端演示 |
| `led-runtime/examples/effects_demo_manager.rs` | 新方式：`EffectManager + ConfigValue` 调度演示 |

## 7. 与 `led-runtime/examples/common/effects.rs` 的区别

`led-runtime/examples/common/effects.rs` 仍然是：

- 示例级离线采帧 helper
- 用于 demo / GUI / 测试播放

而 `EffectEngine` / `EffectInstance` 则是：

- 面向宿主侧调用的最小产品 API

一句话区别：

- `led-runtime/examples/common/effects.rs` 产出的是 **一批预采样帧**
- `EffectInstance` 表达的是 **一个正在运行的 effect 实例**

## 8. 一个完整示例

```rust
use led_runtime::{EffectEngine, ConfigValue};

fn main() -> Result<(), String> {
    let js = include_str!("../js/effects/blink/effect.js");

    let engine = EffectEngine::from_source(js)?;
    let mut instance = engine.instantiate_from_expr("{ ledCount: 4, speed: 100 }")?;

    instance.start()?;
    instance.tick()?;

    let leds = instance.led_buffer()?;
    let led_count = instance.led_count()?;

    println!("led_count = {}", led_count);
    println!("buffer_len = {}", leds.len());

    instance.set_config("speed", ConfigValue::Int(500))?;
    instance.reset()?;

    Ok(())
}
```

## 9. 对产品路线图的意义

这套 API 说明：

- 仓库已经不再只有 `Context::eval()` 这一层通用引擎接口
- 已经开始形成面向产品宿主层的 effect API 雏形

后续仍需继续完善，但这已经是从“示例驱动”走向“产品化宿主接口”的关键一步。

