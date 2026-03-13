# MQuickJS-RS JavaScript 特性规范

本文档描述 `mquickjs-rs` **当前代码与测试实际覆盖到的 JavaScript 能力**。

- 产品脚本约束以 `docs/LED_PROFILE.md` 为准
- 本文档面向“引擎实现现状”，不是 ECMAScript 兼容性承诺
- 除特别说明外，以下结论基于默认 `std` 构建与当前仓库测试结果

当前状态概览：

- `cargo test` 通过，当前共 **316** 个测试通过
- LED effect 最小闭环已完成，`blink / chase / rainbow / wave` 集成测试通过
- 引擎已支持源码执行、字节码编译与字节码反序列化执行
- 宿主产品 API 仍在补齐中，详见 `docs/PRODUCT_ROADMAP.md`

## 1. 总体定位

`mquickjs-rs` 是一个面向 MCU / ESP32 场景的轻量级 JavaScript 运行时。

它已经具备：

- 词法分析、单遍编译、字节码解释执行
- 面向 LED effect 的 `Uint8Array` / `ArrayBuffer` 数据通路
- 一组较完整的 ES5/受限 ES6 风格语言能力
- 基本内建对象与常用运行时函数

它**不追求完整 ECMAScript**，而是优先保证：

- 可实现
- 可测试
- 可在嵌入式场景集成

## 2. 实现基线

## 2.1 构建模式

- 默认启用 `std`
- 库本身支持 `no_std` 方向
- CLI `mqjs`、文件加载、部分计时/正则能力依赖 `std`

因此，本文档中的“已支持”默认指 **默认 `std` 构建** 下的行为。

### `no_std` / 裸板模式说明

当前库可在 `no_std + alloc` 模式下编译和使用；核心解释器、编译器、字节码执行路径可以脱离 `std`。

但在 `no_std` 下，以下能力会降级、失效或不应依赖：

- `mqjs` CLI 不可用
- `load(filename)` 不可用
- `clearTimeout()` 不可用
- `setTimeout()` 仍有符号，但调用会返回错误
- `Date.now()` / `performance.now()` 返回桩值（当前为 `0`）
- `console.log` / `console.error` / `console.warn` 为 no-op
- `print expr;` 不会输出到标准输出
- `RegExp.prototype.test` / `RegExp.prototype.exec` 不注册
- `String.prototype.match()` / `search()` 的正则路径退化为桩行为

对于 ESP32 裸板产品脚本，建议避免依赖以下能力：

- 正则表达式
- 文件加载
- 标准输出/标准错误输出
- 基于系统时间的逻辑
- 通用定时器 API

从宿主集成角度看，`no_std` 目标应优先使用：

- 预编译字节码加载
- `Context::execute()` 驱动执行
- `Uint8Array` 作为输出 buffer
- 原生宿主函数桥接硬件能力

更完整的嵌入式接入说明见 `docs/EMBEDDED_NO_STD.md`。

## 2.2 字节码工作流

已实现：

- 执行源码：`Context::eval()`
- 编译源码为 `FunctionBytecode`
- 序列化为 `.qbc`
- 从字节码反序列化并执行

未完成：

- 独立的产品级 effect 包格式
- Profile 静态校验
- 更严格的字节码版本治理

## 3. 当前数值与语义模型

这一部分是旧 SPEC 偏差最大的地方，当前实现应以这里为准。

## 3.1 Number 模型

已实现：

- 整数与浮点数并存
- 浮点内部使用 `f32`
- `NaN`、`Infinity`、`-Infinity` 已实现
- 算术结果会尽量归一化：如 `3.0` 会回落为整数值，非整值保留为浮点

这意味着：

- 当前引擎**不再是“纯 31 位整数模型”**
- 但它也**不是标准 JS 的双精度 `Number`**；行为更接近“`i32 + f32` 混合模型”

## 3.2 隐式类型转换

已实现：

- 算术运算对 `boolean / null / undefined / number / numeric string` 做 `ToNumber`
- 位运算按 `ToInt32` 处理 `boolean / null / undefined / NaN / Infinity`
- `+` 在任一操作数为字符串时支持字符串拼接
- `==` 支持部分抽象相等规则：
  - `null == undefined`
  - 布尔与数值之间的数值化比较
  - 整数与浮点的数值比较

目前已确认与标准行为一致的典型例子包括：

- `1.0 === 1` → `true`
- `3.0 === 3` → `true`
- `Number.isInteger(3.0)` → `true`
- `Number.isInteger(NaN)` → `false`
- `Number.isInteger(Infinity)` → `false`
- `Number.isFinite(3.0)` → `true`
- `parseFloat("3.0") === 3` → `true`
- `"5" - 3` → `2`
- `"5" < 10` → `true`
- `isNaN('3')` → `false`
- `isFinite('3')` → `true`

未实现或不完整：

- 对象到原始值的完整标准转换链并不完整
- `==` 不是完整 ECMA-262 抽象相等实现

### 3.2.1 对象到原始值转换边界

本项目**不以完整实现标准 JS 的 `ToPrimitive` / `valueOf()` / 自定义 `toString()` 触发链为目标**。

当前的设计目标是：

- 保证数值、布尔、`null`、`undefined`、基础字符串数值化行为正确
- 保证 LED / 配置脚本常用的对象、数组、字符串基础能力稳定
- 不鼓励产品脚本依赖复杂对象在 `+ / - / < / String() / Number()` 中的隐式转换

因此，对象到原始值转换目前只承诺**基础、够用的子集**：

- 普通对象字面量与属性访问
- `Object.prototype.toString` / `Array.prototype.toString` / `Number.prototype.toString` 等已显式实现的方法
- `String(x)` / `Number(x)` / `Boolean(x)` 的基础转换入口

以下能力**不作为当前目标**：

- 自定义 `valueOf()` 驱动的标准 `ToPrimitive` 链
- 自定义 `toString()` 驱动的完整隐式转换语义
- 普通对象在数值运算、关系比较中的完整标准行为
- 为了兼容对象隐式转换而扩展到完整 ES 规范细节

## 3.3 与标准 JS 的已知差异

当前已知的非标准行为包括：

- 未声明变量会在**编译阶段**报错，而不是运行时再抛 `ReferenceError`
- 数组实现是稠密数组模型，不以稀疏数组为主要目标
- 对象到原始值转换仅支持基础子集，不保证完整标准 `ToPrimitive`
- 原型链 / 反射能力只实现了当前需要的子集

这些差异应视为**当前实现限制**，不是对外兼容性承诺。

## 4. 语言特性

## 4.1 已实现并有测试覆盖

### 基础语法

- `var` / `let` / `const`
- 数值、字符串、布尔、`null`、`undefined`
- 表达式语句、块语句
- `print expr;` 非标准打印语句
- `if / else`
- `while`
- `for`
- `for-in`
- `for-of`
- `break` / `continue`
- `return`
- `try / catch / finally`
- `throw`

### 函数与作用域

- `function` 声明
- 函数表达式
- 递归函数
- 闭包捕获
- 局部变量作用域
- `this`
- `new`
- 构造函数返回对象

### 表达式与运算符

- 算术运算：`+ - * / % **`
- 比较运算：`< <= > >= == != === !==`
- 逻辑运算：`&& || !`
- 位运算：`& | ^ ~ << >> >>>`
- 自增自减：`++ --`
- 三元表达式：`a ? b : c`
- 赋值与复合赋值：
  - `=`
  - `+= -= *= /= %=`
  - `&= |= ^=`
  - `<<= >>= >>>=`

### 对象与属性

- 对象字面量
- 对象简写属性：如 `{ speed }`
- 数组字面量
- 成员访问：`obj.prop` / `obj[key]`
- `delete`
- `in`
- `instanceof`
- `typeof`

### 其他

- 字符串比较（同为字符串时按字典序比较）
- 字符串拼接
- 正则对象构造与基础匹配
- 基本错误对象与异常传播

## 4.2 已实现，但属于受限/部分实现

### 定时与加载

- `setTimeout()`：存在，但更适合作为宿主能力而非通用 Web 定时器
- `clearTimeout()`：默认 `std` 下可用
- `load(filename)`：默认 `std` 下可用，用于加载文件执行

### RegExp

- `RegExp` 构造函数可用
- `RegExp.prototype.test`
- `RegExp.prototype.exec`
- `String.prototype.match`
- `String.prototype.search`

限制：

- 更偏“基础正则能力”，不是完整浏览器/Node.js 正则环境
- 默认构建下依赖 `regex` / `std`

### Array / Object 模型

- 当前实现优先稠密数组与常见对象操作
- 不以稀疏数组、完整属性描述符语义、复杂反射行为为目标
- 不以完整对象到原始值转换规则为目标
- 不鼓励依赖对象在数值运算与字符串拼接中的复杂隐式转换

进一步说明：

- `Object.defineProperty` 入口已实现，但**不保证完整标准 property descriptor 语义**
- `Array.prototype.sort` 已实现基础路径，但**不保证完整自定义 comparator 语义**
- `gc()` 可调用，但**不应视为稳定产品 API 或完整 GC 控制接口**
- `toString()` 仅对当前已显式实现的类型提供基础支持；**`valueOf()` 与完整 `ToPrimitive` 链不在当前目标范围**

### Error 模型

- `Error` 及常见错误类型可创建和抛出
- `name` / `message` / `toString()` 已覆盖

限制：

- 与浏览器/Node.js 的 `stack`、格式化细节不保证完全一致

## 4.3 未实现或不应依赖

以下能力当前没有实现，或至少不应视为可依赖的稳定能力：

- 箭头函数 `=>`
- `class`
- `switch`
- `do...while`
- generator / `yield`
- `async` / `await`
- `Promise`
- ES module：`import` / `export`
- 解构赋值
- 展开运算符 `...`
- 模板字符串
- 可选链 `?.`
- 空值合并 `??`
- `with`
- `Symbol`
- `BigInt`
- `Map` / `Set` / `WeakMap` / `WeakSet`
- `Proxy` / `Reflect`
- `DataView`
- 完整 `arguments` 对象语义
- getter / setter 语法
- 计算属性名 `{ [expr]: value }`
- 完整标准 `eval()` 语义
- 完整标准 `ToPrimitive` / `valueOf()` / 自定义 `toString()` 触发链

说明：

- 某些关键字在 lexer 中可能已被识别，但编译器未实现对应语义
- 是否“词法可识别”不代表“语义已支持”
- 某些 API 名称已存在，不代表其具备完整标准行为；如未在 Profile 或本文档中明确承诺，应视为受限实现

## 5. 内建对象与函数

以下为当前默认构建下**已实现且可用**的主要能力。

## 5.1 全局函数

已实现：

- `parseInt`
- `parseFloat`
- `isNaN`
- `isFinite`
- `gc`
- `setTimeout`
- `clearTimeout`（`std`）
- `load`（`std`）

## 5.2 Object

已实现：

- `Object.keys`
- `Object.values`
- `Object.entries`
- `Object.create`
- `Object.defineProperty`
- `Object.getPrototypeOf`
- `Object.setPrototypeOf`
- `Object.prototype.hasOwnProperty`
- `Object.prototype.toString`

限制：

- `Object.defineProperty` 不保证完整标准 descriptor 行为
- 不保证完整标准原型链与反射语义

## 5.3 Array

已实现：

- `push` / `pop`
- `shift` / `unshift`
- `slice` / `splice`
- `indexOf` / `lastIndexOf`
- `join`
- `reverse`
- `concat`
- `map`
- `filter`
- `forEach`
- `reduce`
- `reduceRight`
- `find`
- `findIndex`
- `some`
- `every`
- `includes`
- `sort`
- `flat`
- `fill`
- `toString`
- `Array.isArray`

限制：

- 稀疏数组语义不是主要目标
- `sort` 不保证完整自定义 comparator 语义

## 5.4 String

已实现：

- `length`
- `charAt`
- `charCodeAt`
- `codePointAt`
- `indexOf`
- `lastIndexOf`
- `slice`
- `substring`
- `split`
- `concat`
- `repeat`
- `startsWith`
- `endsWith`
- `includes`
- `padStart`
- `padEnd`
- `replace`
- `replaceAll`
- `match`
- `search`
- `trim`
- `trimStart`
- `trimEnd`
- `toUpperCase`
- `toLowerCase`
- `String.fromCharCode`
- `String.fromCodePoint`

## 5.5 Number

已实现：

- `Number.isInteger`
- `Number.isNaN`
- `Number.isFinite`
- `Number.prototype.toString`
- `Number.prototype.toFixed`
- `Number.prototype.toExponential`
- `Number.prototype.toPrecision`
- 常量：`NaN` / `Infinity`

## 5.6 Math

已实现：

- `abs`
- `floor`
- `ceil`
- `round`
- `sqrt`
- `pow`
- `max`
- `min`
- `sign`
- `sin`
- `cos`
- `tan`
- `asin`
- `acos`
- `atan`
- `atan2`
- `exp`
- `log`
- `log2`
- `log10`
- `random`
- `imul`
- `clz32`
- `fround`
- `trunc`

说明：

- 结果受当前 `f32` 数值模型影响，不承诺与标准 JS 完全一致

## 5.7 JSON

已实现：

- `JSON.parse`
- `JSON.stringify`

## 5.8 Error

已实现：

- `Error`
- `TypeError`
- `ReferenceError`
- `SyntaxError`
- `RangeError`
- `EvalError`
- `URIError`
- `InternalError`
- `Error.prototype.toString`

## 5.9 Date / performance

已实现：

- `Date.now`
- `performance.now`

限制：

- 当前不是完整 `Date` API
- `no_std` 下时间能力会退化为桩实现或受限行为

## 5.10 RegExp

已实现：

- `new RegExp(pattern, flags)`
- `RegExp.prototype.test`
- `RegExp.prototype.exec`
- 属性：`source` / `flags` / `lastIndex`

## 5.11 Function

已实现：

- `Function.prototype.call`
- `Function.prototype.apply`
- `Function.prototype.bind`
- `Function.prototype.toString`

限制：

- 不保证函数对象的完整标准反射语义
- 不保证 `valueOf()` / 自定义 `toString()` 参与隐式转换的完整标准行为

## 5.12 ArrayBuffer / TypedArray

已实现：

- `ArrayBuffer`
- `ArrayBuffer.byteLength`
- `Int8Array`
- `Uint8Array`
- `Uint8ClampedArray`
- `Int16Array`
- `Uint16Array`
- `Int32Array`
- `Uint32Array`
- `Float32Array`
- `Float64Array`

已覆盖能力：

- `new TypedArray(length)`
- `new TypedArray(array)`
- `length`
- `byteLength`
- 索引读写
- `fill`
- `subarray`

限制：

- TypedArray API 目前是“够用子集”，不是完整标准实现
- `Uint8Array` 是 LED 场景的一等能力，其他类型以当前测试覆盖为准

## 5.13 console

已实现：

- `console.log`
- `console.error`
- `console.warn`

## 5.14 非目标 / 不应依赖的半实现能力

以下内容在当前代码中可能存在入口、token、占位实现或基础版本，但**不应视为产品脚本可稳定依赖的能力**：

- 完整标准 `ToPrimitive` / `valueOf()` / 自定义 `toString()` 触发链
- 完整标准 property descriptor 语义
- `Array.prototype.sort` 的完整 comparator 语义
- `gc()` 作为完整 GC 控制接口的行为语义
- lexer 已识别但编译器未完整支持的语法（如 `switch` / `do...while` / `debugger` / `void` 等）
- 仅在 `std` 下存在或更完整的能力（如 `RegExp` 路径、文件加载、宿主定时器）

## 6. LED / 产品脚本相关实现状态

对 `docs/LED_PROFILE.md` 最关键的部分，当前已实现：

- `createEffect(config)` 风格脚本可运行
- `Uint8Array leds` 输出缓冲区可用
- `tick()` 驱动逐帧更新的模型可运行
- `blink / chase / rainbow / wave` 四类 effect 已通过集成测试
- 宿主可通过 native 函数读取 TypedArray 原始字节

当前尚未完全产品化的部分：

- 通用化的 `EffectEngine / EffectInstance` 宿主 API
- 更稳定的 `led_buffer()` 只读宿主接口
- 执行预算 / watchdog / 可中断执行
- 更严格的内存计量与资源配额
- ESP32 端端到端验收与压测

## 7. 建议的依赖边界

如果你在写产品脚本：

- 以 `docs/LED_PROFILE.md` 为准
- 只依赖其中明确列出的语法与运行时约定

如果你在写宿主侧集成：

- 可依赖 `Context::eval()`、`compile()`、`execute()`、字节码序列化/反序列化
- 可依赖 `register_native()` 做宿主桥接
- 不要把当前实现中的非标准细节当成长期稳定 ABI

## 8. 本次修订结论

相较于旧版 SPEC，以下内容已经明显过时，现已修正：

- 不再是“纯 31 位整数、无浮点”模型
- `NaN` / `Infinity` / `parseFloat` / 浮点 TypedArray 已实现
- 抽象相等与基础隐式类型转换已经实现一部分
- `effects`、`Uint8Array`、`ArrayBuffer`、字节码工作流都已落地
- 箭头函数等若未在编译器中落地，不应继续标记为“已支持”

后续如实现新增语法或宿主 API，应优先同步：

- `docs/LED_PROFILE.md`
- `docs/PRODUCT_ROADMAP.md`
- 本文件 `docs/JS_FEATURE_SPEC.md`
