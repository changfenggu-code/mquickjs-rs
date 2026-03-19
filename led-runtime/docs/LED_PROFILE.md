# MQuickJS LED Profile v1

`MQuickJS LED Profile` 是面向 ESP32 / MCU LED 特效脚本的受限 JavaScript 规范。

它的目标不是完整实现 ES6，而是在以下约束下提供稳定、可验证、可维护的脚本能力：

- 可预测的执行时间
- 可控的内存占用
- 稳定的宿主接口
- 便于离线编译与线上发布

本 Profile 是**产品脚本规范**。特效脚本、代码生成器、测试用例和运行时实现都应以本文件为准。

## 设计原则

- **优先确定性**：避免需要复杂调度、隐式异步或不可控分配的特性
- **优先宿主集成**：优先支持 LED 特效需要的语言能力，而不是通用 Web JavaScript
- **优先减法**：不需要的标准特性明确禁用，不做模糊承诺
- **优先可测试**：每条支持/限制都应能映射到自动化测试

## 目标使用场景

- LED 特效脚本的离线编译与设备端执行
- 基于 `createEffect(config)` 的状态机式动画
- 以 `Uint8Array` 作为 RGB 帧缓冲区
- 每帧由宿主主动调用 `tick()`，不依赖事件循环

## 脚本模型

产品脚本应导出或定义以下约定接口：

- `createEffect(config)`：创建效果实例
- 返回对象至少包含：
  - `status` — 当前状态字符串："idle" / "running" / "paused" / "stopped"
  - `speed` — 动画节奏（毫秒），控制效果内部状态变化的时间间隔，**不是帧率**。
    例如 blink 的 `speed=500` 表示每 500ms 切换一次亮灭，与宿主调用 `tick()` 的频率无关。
    宿主以固定帧率（如 30fps / 33ms）调用 `tick()`，效果内部根据 `speed` 和帧间隔自行计数。
  - `frameMs` — 宿主调用 `tick()` 的时间间隔（毫秒），默认 33（约 30fps）。
    由宿主在创建时通过 config 传入，效果内部据此计算 `speed` 对应的 tick 次数。
  - `ledCount` — LED 灯珠数量（即 leds.length / 3）
  - `leds: Uint8Array` — LED 颜色数据数组（长度 = ledCount * 3，格式 [R,G,B, R,G,B, ...]）
  - `tick()` — 帧更新函数，宿主以固定帧率调用。效果内部自行决定是否推进动画状态
  - `start()` — 开始播放效果（idle → running）
  - `pause()` — 暂停效果（running → paused）
  - `resume()` — 恢复播放（paused → running）
  - `stop()` — 停止效果（running/paused → idle）
  - `setConfig(key, value)` — 动态设置配置参数

推荐状态流转：`idle -> running -> paused -> running -> idle`

## 支持的语言能力

以下能力属于 v1 范围。

### 语法

- `var` / `let` / `const`
- `function` 声明与函数表达式
- 闭包捕获
- 对象字面量
- 对象简写属性，如 `{ speed }`
- 数组字面量
- `if / else`
- `while`
- `for`
- `for-in`
- `for-of`
- `return`
- `new`
- 三元表达式 `a ? b : c`
- 逻辑运算 `&&` / `||`
- 成员访问 `obj.prop` / `obj[key]`
- 基本赋值与复合赋值

### 内建对象与函数

- `Math` 的 LED 场景必要能力
  - `abs`
  - `floor`
  - `ceil`
  - `round`
  - `min`
  - `max`
- `Uint8Array`
- `Array`
- `Object`
- `String`
- `Number`

### 产品脚本常见模式

- `config || {}` 默认配置
- `x != null` 空值判断
- RGB / HSV 配置对象
- 闭包保存 effect 内部状态
- 使用 `Uint8Array` 作为可复用缓冲区

## 受限语义

以下不是完整 ES6 语义，而是 Profile 允许的裁剪版本。

### 数值模型

- 以整数为主，优先满足 LED 动画计算需求
- 浮点相关行为不承诺完全兼容标准 JavaScript
- 文档、测试与实现必须对齐，不允许“看起来像 JS，结果像别的语言”的灰区

### 对象模型

- 允许普通对象字面量与属性访问
- 不要求完整原型链兼容性
- 不鼓励依赖复杂反射能力

### TypedArray

- `Uint8Array` 是一等能力
- 必须保证以下行为稳定：
  - `new Uint8Array(length)`
  - `length`
  - 索引读写
  - 必要的原地方法，如 `fill`
- 超出 LED 特效需要的 TypedArray API 不保证支持

## 明确不支持的特性

以下特性不属于 v1 范围，建议在编译阶段直接报错。

- `class`
- `import` / `export`
- `Promise`
- `async` / `await`
- generator / `yield`
- `Proxy` / `Reflect`
- `Map` / `Set` / `WeakMap` / `WeakSet`
- `Symbol`
- `BigInt`
- 模板字符串
- 解构赋值
- 展开运算符 `...`
- 可选链 `?.`
- 空值合并 `??`
- `eval`
- `with`
- 动态模块加载

## 宿主运行时要求

运行时实现必须满足以下产品要求：

- 支持离线编译后的脚本执行
- 支持 effect 实例生命周期管理
- 支持读取 `Uint8Array leds` 作为输出帧
- 支持配置更新与状态复位
- 支持执行预算、递归深度限制、对象数量限制
- 支持真实的内存统计与配额控制

## 安全与资源约束

产品实现必须具备：

- 指令步数预算或可中断执行机制
- 最大调用深度限制
- 最大脚本大小限制
- 最大字节码大小限制
- 最大对象/数组/TypedArray 数量限制
- 最大 LED buffer 大小限制

## 推荐工程边界

为了降低设备端复杂度，推荐采用以下分层：

- **离线工具链**：源码检查、Profile 校验、编译字节码
- **设备运行时**：仅加载已验证字节码
- **宿主适配层**：对接 LED 驱动、定时器、配置系统

## 测试要求

至少维护以下测试集：

- Profile 语法支持测试
- Profile 禁止特性测试
- `Uint8Array` 行为测试
- effect 生命周期测试
- 内存配额测试
- 步数预算 / 超时保护测试
- ESP32 集成测试

## 版本策略

- `v1`：聚焦 LED 特效必要能力
- 任何新增语法/内建都必须先更新 Profile，再进入实现
- 对已发布设备，Profile 变更应视为兼容性事件管理
