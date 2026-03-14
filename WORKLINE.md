# WORKLINE

本文档用于记录当前阶段已经完成的工作、仍待完成的工作，以及建议的下一步实现方向。

更新时间：2026-03-14

## 当前阶段概览

当前工作重点已经从”基础语义修复” → “宿主接口产品化” → “ESP32 no_std 验证”。

已经完成：

- 修复核心数值语义问题：
  - 字符串数值化（如 `"5" - 3`、`"5" < 10`）
  - 全局 `isNaN()` / `isFinite()` 的标准数值化
  - `typeof missingVar` 返回 `"undefined"`
- 审查并补充数值边界测试：
  - `Math.max/min`
  - `Number.prototype.toString/toFixed/toExponential/toPrecision`
  - `Array.prototype.sort`
- 完成 `Array.prototype.sort` 的受限产品语义收口：
  - 纯数值数组按数值升序排序
  - 纯字符串数组按字典序排序
  - comparator 显式报不支持
- 新增最小产品级宿主 API：
  - `EffectEngine`
  - `EffectInstance`
- 新增最小调度层：
  - `EffectManager`
- 升级基础配置系统：
  - `ConfigValue` 新增 `Array` / `Object`
  - 新增 `EffectEngine::instantiate_config(config)`
- 新增领域配置层雏形：
  - `ColorConfig`
  - `BlinkConfig`
  - `ChaseConfig`
  - `RainbowConfig`
  - `WaveConfig`
  - 并支持转换为 `ConfigValue`
- 继续收口 `EffectManager` 行为：
  - 命名冲突显式报错
  - `activate_by_name(...)`
  - `engine_names()` / `engine_count()`
  - `instance_names()` / `instance_count()`
  - `instances_for_engine(...)`
  - `remove_instance(...)` / `remove_instance_by_name(...)` / `remove_instances_by_engine(...)`
- 为新 API 补齐了测试覆盖：
  - 单实例运行
  - 从源码/字节码创建 engine
  - 配置更新与 reset
  - 多实例独立性
  - 多脚本共存
  - 最小调度层激活与切换
  - 按名字激活、列出实例、移除实例
  - 嵌套对象配置与数组配置
  - 命名冲突策略与按 engine 批量查询/移除
  - 领域配置对象到运行时配置的转换与实例化

## 关于“前者是不是已经实现”

你前面提到的这几个方向：

- 按名字激活
- 移除实例
- 列出实例
- 更强的配置系统
- 把 `instantiate(config_expr: &str)` 往更正式接口推进

当前状态如下：

### 已实现（部分）

- `EffectManager` 已实现，但能力仍然偏最小：
  - `add_engine(...)`
  - `instantiate(...)`
  - `activate(instance_idx)`
  - `activate_by_name(...)`
  - `active_name()`
  - `active_engine_name()`
  - `engine_names()`
  - `instance_names()`
  - `instance_count()`
  - `remove_instance(...)`
  - `remove_instance_by_name(...)`
  - `start_active()` / `tick_active()` / `pause_active()` / `resume_active()` / `stop_active()`
  - `active_led_buffer()` / `active_led_count()`
- `EffectEngine` 配置入口已增强：
  - `instantiate(config_expr: &str)` 保留为底层字符串表达式接口
  - `instantiate_config(config: ConfigValue)` 提供更正式的宿主侧配置入口

### 还没实现

- **更正式的实例化配置接口**
  - 当前 `instantiate_config(...)` 已可支持基础对象/数组配置
  - 已补最小领域配置层雏形，但仍未形成完整统一的产品配置体系
- **更完整的 manager 调度语义**
  - 当前已具备基础管理能力，但实例命名策略、切换策略、资源边界仍可继续细化

## 最新建议

### 建议方向：统一“通用基础配置层”（后置优化）

当前已经有：

- `ColorConfig`
- `BlinkConfig`
- `ChaseConfig`
- `RainbowConfig`
- `WaveConfig`

下一步最值得做的深层优化方向，是抽出一层**通用基础配置层**，例如：

- `BaseEffectConfig`
- 共享字段：`led_count` / `speed`
- 对需要颜色的 effect 统一使用 `ColorConfig`

### 做完以后能得到什么功能

1. **配置表达更统一**
   - 现在每种 effect 各有一套配置结构体雏形
   - 继续做下去会逐渐重复
   - 抽出基础配置层后，公共字段的表达方式会统一

2. **后续扩展新 effect 更容易**
   - 新增一个 effect 时，不必每次从零定义全部字段
   - 可以只定义该 effect 独有的部分，再组合基础配置

3. **宿主 API 更像产品 SDK**
   - 现在已经从 `instantiate(config_expr: &str)` 进化到 `instantiate_config(ConfigValue)`
   - 再往前一步，就是让宿主侧拿到真正稳定、统一的配置模型

4. **更适合对接 BLE / App / UI / 配置文件**
   - 公共字段统一后，外部系统映射配置会更简单
   - 例如所有 effect 的 `led_count` / `speed` 都能走同一套映射逻辑

5. **为后续 builder 或更强类型配置体系打基础**
   - 如果未来要做：
     - builder 风格构造器
     - 配置默认值系统
     - 配置校验
   - 都更容易建立在统一基础配置层之上

### 为什么现在先不做

- 当前目标先收敛为“做出一版可实际使用、可运行效果的产品 API”
- `EffectManager`、`EffectEngine`、`EffectInstance` 和基础配置系统已经足够支撑当前效果脚本运行
- 通用基础配置层属于**架构层优化**，不是当前阶段的功能阻塞项
- 因此这一项先记录为后置优化，等可用版本稳定后再继续推进

### 当前阶段优先方向

当前优先目标是：

- 继续围绕“能稳定运行 effect”补齐宿主 API
- 优先做可直接提升运行与调度能力的功能
- 先完成一版可用产品内核，再回头统一配置层设计

### 暂不优先的方向

当前先不优先做：

- 更多 manager 行为细节
- 更多文档扩张
- 更深的资源边界建模

这些都可以放在配置层统一之后再继续推进

## 建议的下一步任务

当前建议：

1. 先继续补齐“可实际运行效果”的宿主 API 功能
2. 优先做直接提升运行时能力和调度能力的工作
3. 通用基础配置层保留为后置优化项，待 API 形态更稳定后统一收口

## 备注

本文件后续应在每一轮阶段性工作结束后更新：

- 已完成什么
- 当前卡在哪
- 下一步要做什么

目的是避免功能推进和文档状态脱节。
