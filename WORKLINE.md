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

## 最新建议

### 建议方向：继续把 `EffectManager` 收口成宿主主入口

当前已经有：

- `add_engine(...)`
- `instantiate_from_expr(...)` / `instantiate_config(...)`
- `activate(...)` / `activate_by_name(...)`
- `start_active()` / `tick_active()` / `pause_active()` / `resume_active()` / `stop_active()`
- `active_led_buffer()` / `active_led_count()`
- `set_active_config(...)`
- `reset_active()`
- `memory_stats_active()`
- 实例/引擎查询与删除能力

当前状态：

- `EffectManager` 已足够承担宿主主入口角色
- `EffectEngine` / `EffectInstance` 继续保留为底层构件层
- 配置接口已统一收口到通用 `ConfigValue`
- 更深层的通用基础配置层优化继续后置

当前主入口建议：

- 宿主侧优先通过 `EffectManager` 完成：
  - engine 注册
  - 实例创建
  - 激活/切换
  - 当前效果配置更新
  - 当前效果 reset
  - 当前效果资源观测

### 这一方向做完以后能得到什么功能

1. **宿主层几乎只需要面向 manager 编程**
   - 不必频繁掉到底层 `EffectInstance`
   - 当前激活效果的配置、重置、运行、观测都能通过 manager 完成

2. **更适合做产品控制逻辑**
   - BLE / App / UI 可以直接针对“当前激活效果”发命令
   - 例如改 speed、改 color、reset 当前效果、查看当前资源占用

3. **更容易把 manager 明确为宿主主入口**
   - `EffectEngine` / `EffectInstance` 保留为底层能力
   - `EffectManager` 承担主要使用路径

4. **后续再做配置体系优化时更从容**
   - 当前先保证“可用、可管理、可切换、可观测”
   - 更深层的配置体系统一可以继续后置

5. **为下一步的调度策略收口打基础**
   - 比如 active 删除策略
   - 默认实例策略
   - 是否自动 start / 自动切换

### 为什么现在推荐做这个

- 当前目标是先做出一版“宿主真正能拿来用”的产品 API
- manager 现在已经具备基础调度能力，继续收口它的主入口地位收益最高
- 相比继续做更深层配置体系，这一步更直接提升运行时可用性

### 当前阶段优先方向

当前优先目标是：

- 继续围绕“能稳定运行 effect”补齐宿主 API
- 优先做可直接提升运行与调度能力的功能
- 先让 `EffectManager` 成为真正明确的主入口，再回头统一配置层设计

### 暂不优先的方向

当前先不优先做：

- 更多文档扩张
- 更深的资源边界建模
- 更深的通用基础配置层优化

这些都可以放在配置层统一之后再继续推进

## 建议的下一步任务

当前建议：

1. 继续把 `EffectManager` 作为宿主主入口来收口
2. 优先补齐和主入口定位直接相关的行为与语义
3. 通用基础配置层继续保留为后置优化项

## 备注

本文件后续应在每一轮阶段性工作结束后更新：

- 已完成什么
- 当前卡在哪
- 下一步要做什么

目的是避免功能推进和文档状态脱节。

