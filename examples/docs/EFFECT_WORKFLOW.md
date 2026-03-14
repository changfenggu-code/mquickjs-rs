# Effect 示例工作流说明

本文档解释 `examples/common/effects.rs` 是如何把一份 LED effect 脚本交给 `mquickjs-rs` 执行，并把结果转成 Rust 可用帧数据的。

适用对象：

- 第一次接触本项目的人
- 想理解 effect.js 到 Rust 帧数据完整链路的人
- 想基于当前示例继续做产品级宿主 API 的人

相关文件：

- 示例辅助：`examples/common/effects.rs`
- 终端演示：`examples/effects_demo.rs`
- GUI 演示：`examples/effects_egui.rs`
- Slint 演示：`examples/effects_slint/main.rs`
- 效果脚本：`js/effects/*/effect.js`
- 产品脚本规范：`docs/LED_PROFILE.md`

## 1. 整体思路

当前示例的目标不是提供一个正式的产品级 `EffectEngine` API，而是：

1. 把一份 effect 脚本作为字符串送进 `mquickjs-rs`
2. 在 JS 引擎里创建 effect 实例
3. 连续调用 `tick()` 推进动画
4. 每一帧把 `leds: Uint8Array` 读回 Rust
5. 把结果整理成 Rust 的帧数据，供 demo / GUI / 测试使用

所以，当前示例更像是：

- **示例级封装**
- **离线采帧工具**
- **未来产品宿主 API 的雏形**

而不是：

- 正式稳定的产品运行时接口

## 2. effect 脚本和实际执行脚本不是一回事

这一点非常重要。

### effect 脚本

effect 脚本负责**定义效果**，例如：

- `createEffect(config)`
- `tick()`
- `start()`
- `leds: Uint8Array`

它本身更像“效果模板”或“工厂函数定义”，并不会自己自动运行。

### 实际执行脚本

真正交给引擎执行的是：

- effect 脚本源码
- 再加上一段 Rust 拼出来的 driver JS

在 `examples/common/effects.rs` 里，这段 driver JS 是：

```rust
let program = format!(
    "{js}\nvar __m = createEffect();\n__m.start();\n\
     for (var __i = 0; __i < {n}; __i++) {{ __m.tick(); __capture(__m.leds, __m.ledCount); }}",
    js = js,
    n = num_frames
);
```

这意味着真正执行的是下面这整个程序：

```js
// 第一部分：effect 脚本
function createEffect(config) {
  ...
}

// 第二部分：driver
var __m = createEffect();
__m.start();
for (var __i = 0; __i < n; __i++) {
  __m.tick();
  __capture(__m.leds, __m.ledCount);
}
```

## 3. 逐步解释整个执行过程

下面按 `examples/common/effects.rs` 的真实逻辑一步一步解释。

### 步骤 1：内嵌 effect 脚本

在文件顶部，有这些常量：

```rust
pub const BLINK_JS: &str = include_str!("../../js/effects/blink/effect.js");
pub const CHASE_JS: &str = include_str!("../../js/effects/chase/effect.js");
pub const RAINBOW_JS: &str = include_str!("../../js/effects/rainbow/effect.js");
pub const WAVE_JS: &str = include_str!("../../js/effects/wave/effect.js");
```

“内嵌”的意思是：

- 在编译 Rust 程序时
- 把对应 `.js` 文件内容直接编译进二进制里
- 最终在 Rust 里得到一个 `&str`

你可以把它理解成：

- Rust 程序启动时不再去磁盘读取 effect.js
- 而是直接已经拿到脚本文本

### 步骤 2：创建 `Context`

在 `capture_effect()` 里：

```rust
let mut ctx = Context::new(256 * 1024);
```

`Context` 可以理解成：

- 一个 JavaScript 运行环境
- 一个脚本执行容器
- 一个小型 JS 虚拟机实例

它负责：

- 保存解释器状态
- 管理内存
- 执行 JS 源码/字节码

### 步骤 3：注册 native 函数

接着示例做了这件事：

```rust
ctx.register_native("__capture", native_capture as NativeFn, 2);
```

这表示：

- 往 JS 全局环境里注册一个名为 `__capture` 的函数
- 但这个函数真正的实现写在 Rust 里

之后 JS 里可以直接写：

```js
__capture(__m.leds, __m.ledCount)
```

它会回调到 Rust 的 `native_capture()`。

这就是 JS 和 Rust 通信的桥梁。

### 步骤 4：拼 driver JS

effect.js 只是定义了 `createEffect()`，不会自己跑。

所以 Rust 要补一段“驱动代码”：

1. 创建 effect 实例
2. 调用 `start()`
3. 循环调用 `tick()`
4. 每一帧调用 `__capture()` 把 `leds` 数据传回 Rust

这段 driver JS 是当前示例真正的调度层。

### 步骤 5：调用 `createEffect()`

driver 里的这句：

```js
var __m = createEffect();
```

会从 effect 脚本里创建出一个 effect 实例对象。

这个实例一般包含：

- `leds`
- `ledCount`
- `start()`
- `tick()`
- 其它生命周期方法

所以 `createEffect()` 更像一个“工厂函数”。

### 步骤 6：调用 `tick()`

driver 里的核心循环是：

```js
for (var __i = 0; __i < n; __i++) {
  __m.tick();
  __capture(__m.leds, __m.ledCount);
}
```

这里的 `tick()` 可以理解成：

- 推进动画一帧
- 更新内部状态
- 刷新 `leds` 里的 RGB 数据

每调用一次 `tick()`：

- effect 的当前帧就向前走一步

### 步骤 7：把 `leds` 抓回 Rust

每次 `tick()` 后，JS 会调用：

```js
__capture(__m.leds, __m.ledCount)
```

Rust 侧的 `native_capture()` 会：

1. 拿到 `leds` 这个 `Uint8Array`
2. 用 `interp.read_typed_array(...)` 读取原始字节
3. 按 `RGB` 三个字节一组拆成 `[u8; 3]`
4. 存进线程本地 `CAPTURED`

关键逻辑是：

```rust
let data = interp.read_typed_array(leds_val).unwrap_or(&[]);
```

然后再把它转成：

```rust
Vec<[u8; 3]>
```

也就是：

- 一帧里的所有灯珠颜色

### 步骤 8：整理成 `EffectData`

当所有帧都跑完以后，Rust 会把捕获到的数据整理成：

```rust
pub struct EffectData {
    pub name: &'static str,
    pub led_count: usize,
    pub frames: Vec<Vec<[u8; 3]>>,
}
```

这个结构的含义是：

- `name`：效果名
- `led_count`：灯珠数量
- `frames`：所有已捕获帧

注意：

这不是“正在运行的 effect 实例”，而是“**离线采样结果**”。

## 4. `EffectData` 到底是什么

这个结构很容易被误解。

它不是：

- 产品运行时中的 effect 实例

它更像：

- demo/测试用的预渲染帧数据包

所以更准确地说，它表达的是：

- “某个 effect 运行若干帧后得到的结果集合”

而不是：

- “一个还在运行中的 effect 对象”

## 5. 当前示例层封装和未来产品 API 的区别

当前 `examples/common/effects.rs` 已经算一层封装，但它仍然属于：

- **示例辅助封装**
- **离线采帧工具**

还不是：

- 正式产品级 `EffectEngine` / `EffectInstance` API

### 当前示例的特点

- 通过拼 JS 字符串驱动运行
- 一次性采集很多帧
- 返回 `frames: Vec<Vec<[u8; 3]>>`
- 更适合 GUI / demo / 测试

### 产品级 API 理想形态

产品级 API 更应该像：

```rust
let engine = EffectEngine::from_source(js)?;
let mut instance = engine.instantiate(config)?;

instance.start()?;
instance.tick()?;
let leds = instance.led_buffer();
```

区别在于：

- Rust 不再拼 driver JS
- Rust 不再直接操心 `createEffect()` / `tick()` 的脚本调度细节
- 生命周期和 buffer 读取都变成正式 Rust 方法

## 6. 现有工作流总结图

可以把当前示例工作流理解成：

```text
effect.js
  ↓
Rust 内嵌脚本文本（include_str!）
  ↓
创建 Context
  ↓
注册 __capture 原生函数
  ↓
拼接 driver JS
  ↓
ctx.eval( effect脚本 + driver脚本 )
  ↓
JS 中 createEffect() 创建实例
  ↓
循环调用 tick()
  ↓
每帧通过 __capture(leds, ledCount) 回传数据
  ↓
Rust 读取 Uint8Array
  ↓
整理为 frames: Vec<Vec<[u8; 3]>>
  ↓
供 demo / GUI / 测试使用
```

## 7. 这份示例最适合拿来学什么

如果你是第一次接触本项目，这个文件最适合帮助你理解：

1. 如何把 JS effect 脚本送进引擎
2. 如何用 Rust 调用 JS 中的 `createEffect()` / `tick()`
3. 如何从 JS 的 `Uint8Array` 读取 RGB 数据
4. 为什么现在还需要 driver JS
5. 为什么说当前只是“示例封装”，还不是“正式产品 API”

## 8. 一句话总结

`examples/common/effects.rs` 做的事情可以概括成：

> 把 effect 脚本和一段 driver JS 拼接后交给 `mquickjs-rs` 执行，
> 再通过 native 函数把每一帧 `Uint8Array leds` 抓回 Rust，整理成可供 demo 和 GUI 播放的帧数据。
