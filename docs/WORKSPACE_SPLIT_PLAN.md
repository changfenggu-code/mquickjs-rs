# Workspace 拆分方案（引擎层 / LED 产品层）

本文档描述如何把当前仓库逐步整理为一个更清晰的 Rust workspace：

- `mquickjs-rs`：通用 JavaScript 引擎库
- `led-runtime`：面向 LED / ESP32 的产品运行时项目

目标不是一次性暴力拆分现有仓库，而是先给出一份**可执行的拆分方案**，让后续迁移有明确方向。

## 当前进展（已开始执行）

当前 workspace 拆分已经开始，仓库中已经存在：

- `led-runtime/` 子项目
- `led-runtime/src/effect.rs`
- `led-runtime/tests/effect_api.rs`
- `led-runtime/tests/effects.rs`
- `led-runtime/examples/` 下的产品层示例
- 根目录中原先的主要产品层文件已完成迁移并开始去重

这意味着本方案已经进入“渐进迁移”阶段，而不是纯规划状态。

当前仍然保留根目录里的部分产品层文件，主要目的是：

- 保持迁移过程平滑
- 方便对照
- 降低一次性大迁移的风险

当前最新状态：

- 根目录的主要产品层代码/测试/示例已经迁到 `led-runtime/`
- 根目录逐步收回纯引擎层角色
- 后续剩余工作将更多集中在文档彻底收口与最终去重

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
- `led-runtime/docs/EMBEDDED_NO_STD.md`
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

### 已迁移的文件

#### 产品 API / 宿主层

- `led-runtime/src/effect.rs`

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

- `led-runtime/js/effects/`

#### 产品测试

- `led-runtime/tests/effects.rs`
- `led-runtime/tests/effect_api.rs`

#### 示例

- `led-runtime/examples/common/effects.rs`
- `led-runtime/examples/effects_demo.rs`
- `led-runtime/examples/effects_demo_engine.rs`
- `led-runtime/examples/effects_demo_manager.rs`
- `led-runtime/examples/effects_egui.rs`
- `led-runtime/examples/effects_slint/`
- `led-runtime/examples/docs/EFFECT_WORKFLOW.md`

#### 产品文档

- `led-runtime/docs/LED_PROFILE.md`
- `led-runtime/docs/PRODUCT_ROADMAP.md`
- `led-runtime/docs/EFFECT_ENGINE_API.md`
- `led-runtime/WORKLINE.md`

### 这个项目负责什么

- `EffectEngine` / `EffectInstance` / `EffectManager`
- effect 配置和调度
- LED 产品脚本规范
- BLE / UI / ESP32 宿主集成（后续）
- 示例、产品测试和效果演示

## 5. 为什么 `led-runtime/src/effect.rs` 更适合放在产品层

这是一个关键判断。

当前 `led-runtime/src/effect.rs` 已经包含：

- `EffectEngine`
- `EffectInstance`
- `EffectManager`
- effect 配置接口
- 宿主调度能力

这些明显不是“通用 JS 引擎本体”的职责，而是：

- 基于引擎构建出来的产品运行时层

这也正是它现在已经迁移到 `led-runtime` 的原因。

## 6. 已执行的迁移顺序

拆分采用了分阶段推进，而不是一次性暴力迁移。

### Phase A：先认知分层（已完成）

首先明确了：

- 引擎层：`Context` / VM / parser / runtime
- 产品层：`led-runtime/src/effect.rs` / `led-runtime/docs/LED_PROFILE.md` / `led-runtime/js/effects/` / product tests

### Phase B：建立 workspace 外壳（已完成）

当前实际结构是：

- workspace 根目录（当前仓库）
- 根 crate：`mquickjs-rs`
- 子 crate：`led-runtime/`

并先保证两个成员 crate 都能独立编译和测试。

### Phase C：迁移产品层文件（已完成）

已迁移：

- `led-runtime/src/effect.rs`
- `led-runtime/tests/effect_api.rs`
- `led-runtime/tests/effects.rs`
- `led-runtime/js/effects/`
- `led-runtime/docs/LED_PROFILE.md`
- `led-runtime/docs/PRODUCT_ROADMAP.md`
- `led-runtime/docs/EFFECT_ENGINE_API.md`

### Phase D：迁移示例（已完成）

已迁移：

- `led-runtime/examples/effects_demo*.rs`
- `led-runtime/examples/common/`
- `led-runtime/examples/effects_egui.rs`
- `led-runtime/examples/effects_slint/`

示例经常会引用宿主 API，因此在前面几步稳定后再迁移是更稳妥的做法。

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

