# Numeric Audit TODO

本文档记录 `mquickjs-rs` 当前围绕 `i32` / `f32` 混合数值模型做的一轮审查结果。

目标：

- 先把问题审出来
- 区分“确实有坑”和“看起来用了 `to_i32()` 但其实合理”
- 为后续修复提供明确清单

说明：

- 本文档只关注**核心库语义**
- 不包含 `mqjs` CLI、REPL、文件系统或 `std` 宿主能力
- 不追求完整标准 JS，只关注“基础数值计算正确、产品脚本够用”

## 审查结论总览

### 已确认需要修复 / 重点关注

1. 当前无新的 P0 数值语义阻塞项
   - 本轮已完成 `Array.prototype.sort` 的默认排序语义修复
   - 后续重点转为 `sort(compareFn)` 的策略收口与数值边界测试完善

### 已审查，当前实现基本合理

1. `Math.max` / `Math.min`
   - 已使用 `to_number_f32()`
   - 对空参数、`NaN` 路径也有处理
   - 位置：`src/vm/natives.rs:1069`, `src/vm/natives.rs:1094`

2. `Math.sqrt` / `Math.pow` / `Math.sign` / `Math.exp` / `Math.log*` / 三角函数
   - 主要走 `to_number_f32()`
   - 与当前 `f32` 数值模型一致
   - 位置：`src/vm/natives.rs:1135`, `src/vm/natives.rs:1149`, `src/vm/natives.rs:1245` 及其后续段落

3. `Number.prototype.toFixed` / `toExponential` / `toPrecision`
   - 已使用 `to_number_f32()`
   - 基本符合当前数值模型预期
   - 位置：`src/vm/natives.rs:922`, `src/vm/natives.rs:938`, `src/vm/natives.rs:952`

4. `Number.prototype.toString`
   - 已同时处理 `i32` 和 `f32`
   - 不是当前最优先问题
   - 位置：`src/vm/natives.rs:899`

### 看起来是 `i32`，但目前不建议改

以下大多属于“参数本来就应该是整数”的 API：

1. 字符串索引 / 长度 / 次数 / 位置参数
   - 如 `slice` / `substring` / `charCodeAt` / `codePointAt` / `repeat` / `padStart` / `padEnd`
   - 这些参数语义本来就偏整数，不必因为引入 `f32` 就强行改成浮点路径

2. TypedArray 索引 / 长度相关参数
   - 如 `fill` / `subarray` 等
   - 本质上是索引，不是 number 语义问题

3. 位运算 / 32 位整数语义 API
   - 如 `Math.imul` / `Math.clz32`
   - 本来就偏 `i32` 路径，继续保留合理

4. radix / 精度 / 深度 / 计数参数
   - 如 `toString(radix)` / `toFixed(digits)` / `flat(depth)`
   - 本来就应该是整数参数

## 重点问题详情

## 1. `Array.prototype.sort`（已修复）

### 修复前问题

旧实现的 `sort()` 默认行为：

- 原地排序
- 直接把元素做 `to_i32().unwrap_or(0)`
- 再做整数升序比较

代码位置：

- `src/vm/natives.rs:683`
- `src/vm/natives.rs:685`
- `src/vm/natives.rs:686`
- `src/vm/natives.rs:691`
- `src/vm/natives.rs:692`
- `src/vm/natives.rs:693`

### 为什么当时是问题

在当前 `i32 + f32` 数值模型下，这会导致：

- `f32` 不能被正确排序
- `NaN` / `Infinity` / `-Infinity` 行为不明确
- 非整数数值会被粗暴降级
- 传 comparator 时当前也不会真正走 comparator 逻辑

### 当前状态

当前已收敛为以下受限产品语义：

- 纯数值数组按数值升序排序（支持 `int` / `float`）
- 纯字符串数组按字典序排序
- 混合类型数组报错
- 传 comparator 时显式报不支持

相关实现与验证：

- 实现：`src/vm/natives.rs:663`
- 测试：`tests/eval_integration.rs:4407` 及新增 sort 测试

## 2. `sort(compareFn)` comparator 语义

### 当前状态

- 当前已改为：传 comparator 时显式报错
- 不再保留“看起来支持、实际上忽略参数”的行为

历史位置：

- `src/vm/natives.rs:673`
- `src/vm/natives.rs:680`
- `src/vm/natives.rs:681`
- `src/vm/natives.rs:682`

### 后续建议

如果短期不打算实现 comparator，当前策略已经足够清晰：

- 保持显式报错即可
- 不建议为了兼容性引入半实现 comparator 逻辑

这样比“看起来支持、实际上忽略参数”更不容易埋坑。

## 3. `Math` 系列函数审查结果

本轮审查确认：

- `Math.max` / `Math.min` 已经走 `to_number_f32()`，无需优先修
- `Math.sqrt` / `Math.pow` / `Math.sign` / `Math.exp` / `Math.log` / 三角函数主要也已走 `to_number_f32()`
- `Math.abs` / `floor` / `ceil` / `round` / `trunc` 虽有 `to_i32()` 快路径，但 float 分支已存在，当前不构成明显错误

### 后续建议

若后续继续审数值细节，可重点补一轮行为测试，而不是急着改实现。

## 4. `Number.prototype.*` 审查结果

本轮审查确认：

- `Number.prototype.toString` 同时支持整数与浮点路径
- `toFixed` / `toExponential` / `toPrecision` 已走 `to_number_f32()`

### 当前判断

这块不是当前主要风险点。

## 不建议作为本轮修复目标的项目

以下虽然也存在大量 `to_i32()`，但它们本质上不是“数值模型遗留问题”：

- 字符串 index / length / repeat 参数
- TypedArray 索引 / 长度参数
- bitwise / `imul` / `clz32`
- radix / precision / depth / count 一类控制参数

## 推荐后续修复顺序

### P0

- `sort(compareFn)` 策略保持清晰（当前为显式 unsupported）

### P1

- 对 `Math` 系列补充一轮 `f32` / `NaN` / `Infinity` 行为测试（已补）
- 对 `Number.prototype.*` 补充格式化边界测试（已补）

### P2

- 仅在需要时继续深入其它 number API 细节

## 当前结论

本轮最重要的数值语义缺口 `Array.prototype.sort` 已修复。

其余大部分 `to_i32()` 使用点，当前要么已经有 `f32` 分支，要么本身就应该是整数参数，不必因为引入 `f32` 而全面改造。
