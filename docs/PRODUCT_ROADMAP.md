# Productization Roadmap

本文档描述将 `mquickjs-rs` 演进为 ESP32 LED 特效产品运行时的优先级路线。

## 目标

- 设备端稳定执行受限 JavaScript 特效脚本
- 内存、执行时间、功能边界可控
- 文档、测试、生成器、运行时保持一致

## 当前主要差距

### P0 问题

- ~~文档仍以通用/学习型引擎表述为主~~ ✅ 已更新
- ~~LED Profile 尚未成为唯一规范源~~ ✅ 已建立
- ~~`effects` 集成测试尚未通过~~ ✅ 22/22 测试通过
- ~~`Uint8Array` 相关能力对 LED 场景仍不完整~~ ✅ 基本能力已满足
- **公开展开 API 仍偏通用 `eval`，宿主接口不稳定** ⚠️ 主要差距

### P1 问题

- **内存限制与真实运行时分配未统一计量**
- GC 设计与实际运行时对象管理尚未闭环
- **缺少执行预算、watchdog、中断机制**
- ~~缺少面向设备部署的字节码发布流程~~ ✅ CLI 支持编译/执行字节码

### P2 问题

- **缺少 Profile 一致性测试矩阵**
- 缺少 ESP32 端端到端性能与压力数据
- 缺少版本化兼容策略
- **缺少 CLI 文件的中文版本**（README.md 已有中文版）

## 路线分阶段

## ✅ Phase 1：规范冻结（已完成）

目标：先统一"允许什么、不允许什么"。

- ✅ 建立 `docs/LED_PROFILE.md` 作为唯一产品脚本规范
- ✅ README 与生成器说明统一改为"受限 ES6 风格 Profile"
- ✅ 将现有 `JS_FEATURE_SPEC` 与 Profile 建立引用关系
- ✅ 新增禁止特性测试，避免脚本能力漂移

验收标准：

- ✅ Profile 文档可单独指导脚本编写
- ✅ 代码生成器、测试、README 描述一致

---

## ✅ Phase 2：LED 最小闭环（已完成）

目标：让核心 effect 用例稳定跑通。

- ✅ 修复 `tests/effects.rs`（22 个测试全部通过）
- ✅ 补齐 `Uint8Array` 在 LED 场景必要方法
- ✅ 稳定对象字面量、闭包、配置对象路径
- ✅ 增加 effect 生命周期回归测试
- ✅ 实现 blink/chase/rainbow/wave 四类效果

验收标准：

- ✅ `cargo test --test effects` 全通过
- ✅ `blink/chase/rainbow/wave` 四类效果稳定执行

---

## 🔄 Phase 3：宿主接口产品化（进行中）

目标：把引擎改造成稳定的设备端组件，而不是仅供 `eval` 使用的库。

**当前状态**：
- ✅ CLI 支持基本 bytecode 编译和执行
- ✅ 基础 `Context::eval()` API 稳定
- ✅ 已有最小可用的 `EffectEngine` / `EffectInstance` API
- ✅ 已有最小可用的 `EffectManager` 调度层
- ✅ 多实例、生命周期、配置更新、重置等基础场景已有测试覆盖
- ✅ 多脚本 / 多效果 engine 基础共存场景已有测试覆盖
- ✅ 基础配置系统已支持对象 / 数组配置与最小领域配置层雏形
- ⚠️ 仍缺少更完整的产品级宿主接口收口（资源边界、更强调度策略、更多示例与最终文档收口）

**待实现**：

- **完善 effect 实例 API**
  - 继续收口“脚本加载/编译”和“实例创建”两个阶段
  - 继续增强实例边界、错误模型与资源约束

- **增强读取 LED buffer 的宿主接口**
  - 已提供 `led_buffer()` 方法返回 `&[u8]`
  - 后续继续完善零拷贝、错误处理与实例状态边界

- **增强加载 bytecode / 重置实例 / 更新配置能力**
  - 已提供 `from_bytecode()` / `from_source()` / `instantiate()` / `instantiate_config()` / `reset()` / `set_config()`
  - 已有基础对象/数组配置与领域配置雏形
  - 后续继续补齐更强类型配置体系与实例隔离语义

- **明确单脚本/多脚本运行模型**
  - 单脚本多实例基础能力已具备并有测试覆盖
  - 多脚本共存基础能力已具备并有测试覆盖
  - 最小调度层（`EffectManager`）已具备 engine/instance 管理、激活、切换、移除能力
  - 后续继续完善效果切换策略、调度行为与资源边界模型
  - 实例生命周期管理（`start()`, `pause()`, `resume()`, `stop()`）已具备基础实现

建议 API 方向：

```rust
pub struct EffectEngine { /* effect 模板 */ }
pub struct EffectInstance { /* 运行中实例 */ }
pub struct EffectManager { /* 最小调度层 */ }

impl EffectEngine {
    pub fn from_bytecode(bytes: &[u8]) -> Result<Self>;
    pub fn from_source(source: &str) -> Result<Self>;
    pub fn instantiate(&self, config_expr: &str) -> Result<EffectInstance>;
    pub fn instantiate_config(&self, config: ConfigValue) -> Result<EffectInstance>;
}

impl EffectInstance {
    pub fn tick(&mut self) -> Result<()>;
    pub fn led_buffer(&mut self) -> Result<&[u8]>;
    pub fn led_count(&mut self) -> Result<usize>;
    pub fn set_config(&mut self, key: &str, value: ConfigValue) -> Result<()>;
    pub fn start(&mut self) -> Result<()>;
    pub fn pause(&mut self) -> Result<()>;
    pub fn resume(&mut self) -> Result<()>;
    pub fn stop(&mut self) -> Result<()>;
    pub fn reset(&mut self) -> Result<()>;
}
```

验收标准：

- ✅ 宿主侧已可无需直接依赖通用 `eval` 即可驱动 effect 基本生命周期
- ✅ 宿主集成测试已有基础覆盖：多实例 / 多脚本 / 配置 / 重置 / manager 基础行为已补齐
- ⚠️ API 已有最小实现，但资源边界语义、调度策略和最终文档收口仍未完成

### 多脚本 / 多实例模型完成后可支持的产品功能

当 Phase 3 进一步补齐“多脚本共存 + 多实例调度”后，宿主侧将能够直接支持以下产品能力：

- **多效果预加载**
  - 设备启动时同时加载多份 effect 模板（如 Blink / Rainbow / Wave / Chase）
  - 切换效果时无需重新走整套脚本载入流程

- **单效果多实例独立运行**
  - 同一个 `EffectEngine` 可按不同配置创建多个 `EffectInstance`
  - 例如同一份脚本同时运行“短灯带版”和“长灯带版”

- **宿主层效果切换与轮播**
  - BLE / App / 按钮 / 本地配置系统可以切换当前激活效果
  - 支持 stop 当前实例、start 下一个实例、按顺序轮播多个效果

- **分区 / 多通道灯效控制**
  - 不同灯带、不同分区或不同输出通道可由不同实例独立驱动
  - 为左右分区、主灯带 + 氛围灯等场景提供基础能力

- **后台预热与前台切换**
  - 在当前效果运行时，宿主侧可提前准备另一个效果实例
  - 切换时减少初始化开销，提升切换体验

- **更清晰的宿主调度模型**
  - 宿主不再关心 `Context` / `eval` / `createEffect()` 等底层细节
  - 只需要管理“哪些 engine 已加载、哪些 instance 正在运行、当前激活哪个实例”

一句话说，完成这一步后，仓库将不再只是“能运行一个 effect 的 JS 引擎”，而是“能管理多个灯效脚本与实例的产品运行时内核”。

---

## ⏳ Phase 4：资源模型重构（待启动）

目标：建立真实可靠的 MCU 资源边界。

- 统一对象、数组、闭包、TypedArray 的内存计量
- 决定保留并补齐 GC，或改为更简单的句柄表/arena 模型
- 增加最大对象数、最大数组长度、最大 TypedArray bytes 限制
- 修复 `memory_stats` 口径与真实分配不一致的问题

验收标准：

- 任意脚本都不能绕过内存上限
- 内存统计可用于线上观测与问题定位

---

## ⏳ Phase 5：执行安全（待启动）

目标：避免脚本卡死主循环。

- 增加执行步数预算
- 增加递归深度预算
- 增加长循环中断能力
- 增加超时/看门狗联动策略

验收标准：

- 恶意或错误脚本不会长期阻塞 LED 主循环

---

## ⏳ Phase 6：离线工具链（待启动）

目标：降低设备端复杂度并提升发布可靠性。

**当前状态**：
- ✅ CLI 支持源码编译为字节码
- ⚠️ 缺少 Profile 校验
- ⚠️ 缺少版本管理
- ⚠️ 缺少打包格式

**待实现**：

- 构建源码到 bytecode 的离线编译流程
- 加入 Profile 校验（禁止特性检测）
- 加入 bytecode 版本号与兼容性检查
- 建立 effect 发布包格式（包含 bytecode、元数据、依赖）

验收标准：

- 设备端仅加载已验证字节码
- 字节码版本不兼容时可明确报错

---

## ⏳ Phase 7：ESP32 集成验收（待启动）

目标：验证产品化指标，而不是只看桌面测试。

- 启动时间测试
- 单帧执行延迟测试
- 内存峰值测试
- 长时间稳定性测试
- 异常脚本恢复测试
- 实际 BLE 集成测试

验收标准：

- 在目标芯片与目标 LED 数量上满足产品 KPI

---

## 近期建议优先级

### 立即执行（当前）

- ~~冻结 LED Profile~~ ✅ 已完成
- ~~更新 README 与相关文档~~ ✅ 已完成
- ~~修复 `effects` 测试~~ ✅ 已完成

### 紧接执行

**Phase 3：宿主接口产品化**

- 完善 `EffectManager` 调度策略与资源边界
- 收口配置系统（是否继续扩展统一基础配置层）
- 最终整理并收口 API 文档和示例

### 之后执行

- Phase 4：资源模型重构
- Phase 5：执行安全
- Phase 6：离线工具链增强
- Phase 7：ESP32 压测

---

## 里程碑时间线（建议）

- **Week 1-2**：完成 Phase 3 宿主接口产品化
- **Week 3-4**：完成 Phase 4 资源模型重构
- **Week 5-6**：完成 Phase 5 执行安全
- **Week 7-8**：完成 Phase 6 离线工具链
- **Week 9-10**：完成 Phase 7 ESP32 集成验收

---

## 技术债务记录

### 已完成

- ✅ LED Profile 规范建立
- ✅ JS 特性文档与产品规范对齐
- ✅ 效果集成测试覆盖
- ✅ 中英文文档同步（CLAUDE.md/README.md）
- ✅ `EffectEngine` / `EffectInstance` 最小产品 API
- ✅ `EffectManager` 最小调度层
- ✅ 多实例 / 多脚本 / 配置 / 重置 / manager 基础测试覆盖

### 进行中

- ⚠️ 宿主接口产品化收口（资源边界、调度策略、最终文档）

### 待处理

- ❌ 内存统计准确性
- ❌ Profile 一致性自动化测试
- ❌ 字节码版本管理
- ❌ ESP32 交叉编译支持
- ❌ 执行预算和中断机制

---

## 核心库语义修复优先级

以下优先级仅针对**核心库语义**，不包含 `mqjs` CLI、REPL、文件系统、`std` 宿主能力与桌面输出层。

### P0：已完成

- ✅ 字符串参与数值运算时的 `ToNumber` 语义
- ✅ 全局 `isNaN()` / `isFinite()` 的标准数值化
- ✅ 相关语义回归测试补齐

### P1：建议尽快修复

- **对象到原始值转换边界梳理**
  - 明确哪些行为保证兼容，哪些行为继续受限

### P2：可延后处理

- `mqjs` CLI 输出层（例如字符串直接返回时显示不正确）
- `print` / `console.*`
- `Date.now()` / `performance.now()` 等宿主能力

### P3：当前不作为核心目标

- `RegExp`
- 完整 ES6+ 高级语法与运行时能力（如 `class`、`Promise`、`async/await`、模块系统、`Map/Set` 等）
