# Workspace 拆分方案（引擎层 / LED 产品层）

本文档描述如何把当前仓库逐步整理为一个更清晰的 Rust workspace：

- `mquickjs-rs`：通用 JavaScript 引擎库
- `led-runtime`：面向 LED / ESP32 的产品运行时项目

目标不是马上物理拆分现有仓库，而是先给出一份**可执行的拆分方案**，让后续迁移有明确方向。

## 1. 为什么值得拆分

当前仓库已经同时承载了两类职责：

### A. 引擎层

- JavaScript 解析 / 编译 / 执行
- VM / runtime / builtins
- 数值语义
- no_std 能力
- 通用测试与性能分析

### B. LED 产品层

- `EffectEngine` / `EffectInstance` / `EffectManager`
- LED effect 脚本
- `LED_PROFILE`
- 产品路线图
- 宿主 API
- 多效果调度、配置、实例管理

如果长期继续混在一个 crate / 一个目录认知里，会逐渐出现：

- 难以区分哪些改动是在增强引擎，哪些是在做产品逻辑
- 文档越来越混：既写引擎又写产品
- 测试边界模糊：引擎语义测试和产品测试混在一起
- 后续如果真要把引擎单独复用，会越来越难拆

因此，建议后续按 workspace 两层模型来组织。

## 2. 推荐的最终结构

推荐先拆成一个 workspace，包含两个成员：

```text
workspace/
├── Cargo.toml                # workspace 根配置
├── mquickjs-rs/              # 通用引擎库
└── led-runtime/              # LED / ESP32 产品项目
```

### workspace 根目录 Cargo.toml

```toml
[workspace]
members = [
  "mquickjs-rs",
  "led-runtime",
]
resolver = "2"
```

## 3. `mquickjs-rs` 应保留什么

`mquickjs-rs` 作为引擎库，应只保留通用引擎能力。

### 建议保留文件

#### 核心源码

- `src/lib.rs`
- `src/context.rs`
- `src/value.rs`
- `src/parser/`
- `src/vm/`
- `src/runtime/`
- `src/gc/`
- `src/builtins/`
- `src/util/`
- `src/bin/mqjs.rs`

#### 测试

- `tests/eval_integration.rs`
- `tests/error_messages.rs`

#### 文档

- `README.md`
- `README.zh.md`
- `docs/HOW_IT_WORKS.md`
- `docs/BENCHMARK_ANALYSIS.md`
- `docs/JS_FEATURE_SPEC.md`
- `docs/PROJECT_STRUCTURE.md`
- `docs/EMBEDDED_NO_STD.md`
- `docs/NUMERIC_AUDIT_TODO.md`

#### 其它

- `vendor/`
- `benches/`

### 这个项目负责什么

- 解析 / 编译 / 字节码 / VM
- 通用 builtins
- 通用 API：`Context`, `Value`, `FunctionBytecode`
- 作为可复用库对外发布

## 4. `led-runtime` 应承接什么

`led-runtime` 作为产品项目，负责 effect 宿主运行时与业务层。

### 建议迁移过去的文件

#### 产品 API / 宿主层

- `src/effect.rs`

拆分后可考虑细化为：

```text
led-runtime/src/
├── lib.rs
├── effect_engine.rs
├── effect_instance.rs
├── effect_manager.rs
├── config.rs
└── host/
```

#### effect 脚本

- `js/effects/`

#### 产品测试

- `tests/effects.rs`
- `tests/effect_api.rs`

#### 示例

- `examples/common/effects.rs`
- `examples/effects_demo.rs`
- `examples/effects_demo_engine.rs`
- `examples/effects_demo_manager.rs`
- `examples/effects_egui.rs`
- `examples/effects_slint/`
- `examples/docs/EFFECT_WORKFLOW.md`

#### 产品文档

- `docs/LED_PROFILE.md`
- `docs/PRODUCT_ROADMAP.md`
- `docs/EFFECT_ENGINE_API.md`
- `WORKLINE.md`

### 这个项目负责什么

- `EffectEngine` / `EffectInstance` / `EffectManager`
- effect 配置和调度
- LED 产品脚本规范
- BLE / UI / ESP32 宿主集成（后续）
- 示例、产品测试和效果演示

## 5. 为什么 `src/effect.rs` 更适合放到 `led-runtime`

这是一个关键判断。

当前 `src/effect.rs` 已经包含：

- `EffectEngine`
- `EffectInstance`
- `EffectManager`
- effect 配置接口
- 宿主调度能力

这些明显不是“通用 JS 引擎本体”的职责，而是：

- 基于引擎构建出来的产品运行时层

所以如果未来真的拆分，`src/effect.rs` 优先应该迁移到 `led-runtime`。

## 6. 当前最推荐的迁移顺序

不建议一次性暴力拆分，推荐分阶段进行。

### Phase A：先认知分层（现在就可以）

即使暂时不改目录，也要明确：

- 引擎层：`Context` / VM / parser / runtime
- 产品层：`effect.rs` / `LED_PROFILE` / effect 脚本 / product tests

### Phase B：建立 workspace 外壳

先创建：

- workspace 根目录
- `mquickjs-rs/`
- `led-runtime/`

但先不大规模迁移文件，只保证两个成员 crate 能正常编译。

### Phase C：迁移产品层文件

优先迁移：

- `src/effect.rs`
- `tests/effect_api.rs`
- `tests/effects.rs`
- `js/effects/`
- `docs/LED_PROFILE.md`
- `docs/PRODUCT_ROADMAP.md`
- `docs/EFFECT_ENGINE_API.md`

### Phase D：迁移示例

最后迁移：

- `examples/effects_demo*.rs`
- `examples/common/`
- GUI 示例

因为示例经常会引用宿主 API，属于产品层后置迁移对象。

## 7. `led-runtime` 如何依赖 `mquickjs-rs`

在 `led-runtime/Cargo.toml` 中，建议先用 path 依赖：

```toml
[dependencies]
mquickjs = { package = "mquickjs-rs", path = "../mquickjs-rs", default-features = false }
```

如果你在桌面环境调试，也可以按需要启用默认 feature。

## 8. 拆分后怎么开发

### 改引擎时

在 `mquickjs-rs` 中工作：

- 改 parser / vm / runtime / context / builtins
- 跑引擎测试：

```bash
cargo test -p mquickjs-rs
```

### 改产品层时

在 `led-runtime` 中工作：

- 改 `EffectEngine` / `EffectManager` / 产品配置 / effect 脚本
- 跑产品测试：

```bash
cargo test -p led-runtime
```

### 联调时

在 workspace 根目录：

```bash
cargo test --workspace
```

## 9. 拆分后能得到什么好处

### 1. 引擎边界更清楚

你会明确知道：

- 哪些文件属于 JS 引擎本身
- 哪些属于 LED 产品逻辑

### 2. 产品层可以更自由演进

例如：

- BLE
- UI
- 配置系统
- 调度层
- 设备宿主集成

都不会继续污染引擎本体。

### 3. 更适合后续独立发布

以后如果要：

- 单独发布 `mquickjs-rs`
- 或让其它项目复用引擎库

工作量会小很多。

## 10. 当前建议

如果你现在还在快速迭代功能，建议：

- **先不马上物理拆仓库**
- 但从现在开始按“引擎层 / 产品层”思路维护文件和接口

等到：

- `EffectManager` / 配置系统 / 宿主 API 再稳定一些

再开始做 workspace 迁移，会更稳。

一句话总结：

> 当前最推荐的方向是：先把仓库按职责分层思考和维护，
> 后续再把 `mquickjs-rs` 和 `led-runtime` 迁移成 workspace 双项目结构。
