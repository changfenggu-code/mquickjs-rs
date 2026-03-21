# MQuickJS-RS 项目结构详解

本文档详细说明 MQuickJS-RS 项目的目录结构、文件组织及各模块的功能定位。

## 项目概览

MQuickJS-RS 是一个以嵌入式系统和 LED 特效场景为目标的 Rust JavaScript 运行时项目。项目采用模块化架构，将词法分析、解析编译、虚拟机、运行时等功能分离为独立模块。

## 目录结构

```
mquickjs-rs/
├── .claude/              # Claude Code 工具配置
├── .github/              # GitHub Actions CI/CD 配置
├── benches/              # 性能基准测试
├── docs/                 # 项目文档
├── led-runtime/          # LED / ESP32 产品层子项目（workspace 成员）
├── examples/             # Rust 示例程序
├── js/                   # JavaScript 测试脚本和示例
├── src/                  # 核心 Rust 源代码
├── tests/                # 集成测试
├── vendor/               # 第三方依赖（原始 C 实现）
└── target/               # 构建输出目录（不纳入版本控制）
```

## 文件归属分层

为了便于区分“哪些是 `mquickjs-rs` 引擎本体文件，哪些是为 LED / ESP32 产品方向准备的文件”，本仓库可按目标归属分为三层：

### A. 引擎本体层

这部分文件属于通用 JavaScript 引擎本体，即使脱离 LED 场景也成立：

- `src/`：核心 Rust 源码（编译器、VM、运行时、GC、builtins）
- `tests/eval_integration.rs`、`tests/error_messages.rs`：通用语义与错误处理测试
- `src/bin/mqjs.rs`：通用桌面 CLI
- `Cargo.toml` / `Cargo.lock`：包定义与依赖
- `vendor/`：原始 C 参考实现
- `README.md` / `README.zh.md`：仓库总说明
- `docs/HOW_IT_WORKS.md`、`docs/BENCHMARK_ANALYSIS.md`、`docs/PROJECT_STRUCTURE.md`：引擎实现与工程说明

### B. LED / ESP32 产品层

这部分文件明显面向 LED effect 产品脚本运行时，不应简单视为“纯引擎通用能力”：

- `led-runtime/`：产品层主开发位置（当前 workspace 子项目）
- `led-runtime/docs/LED_PROFILE.md`：产品脚本规范
- `led-runtime/docs/PRODUCT_ROADMAP.md`：产品化路线图
- `led-runtime/docs/EMBEDDED_NO_STD.md`：`no_std` / ESP32 裸板接入说明
- `led-runtime/js/effects/`：LED 效果脚本资源
- `led-runtime/tests/effects.rs`：LED 效果集成测试
- `led-runtime/examples/common/effects.rs`、`led-runtime/examples/effects_demo.rs`、`led-runtime/examples/effects_egui.rs`、`led-runtime/examples/effects_slint/`：LED 产品演示与采帧示例

### C. 混合层 / 边界说明层

这部分文件既在描述引擎现状，也在为产品脚本边界收口：

- `docs/JS_FEATURE_SPEC.md`：引擎实际能力说明，同时明确哪些能力属于产品脚本可依赖子集、哪些只是受限实现或非目标

理解这三层很重要：

- **A 层**回答“引擎本身实现了什么”
- **B 层**回答“为了 LED / ESP32 产品我们额外约束了什么”
- **C 层**回答“引擎现状与产品边界之间如何对齐”

当前补充说明：

- 根目录仓库仍处于**过渡期双轨状态**
- `led-runtime/` 已可单独编译和运行产品层测试/示例
- 根目录中仍保留部分产品层文件，用于迁移过渡和对照，后续会逐步去重

---

## 核心源代码 (src/)

### 入口与 API (src/lib.rs, src/context.rs)

**[src/lib.rs](../src/lib.rs)**
- **功能**：引擎库入口文件，定义核心引擎 API
- **导出类型**：`Context`, `Value`, `MemoryStats`, `FunctionBytecode`, `NativeFn`
- **特性**：支持 `no_std` 模式，条件编译 `std` 特性
- **模块组织**：
  - 核心模块：context, value
  - 垃圾回收：gc
  - 虚拟机：vm
  - 解析编译：parser
  - 内置对象：builtins
  - 运行时：runtime
  - 工具：util

**[src/context.rs](../src/context.rs)**
- **功能**：提供执行上下文，管理内存和解释器实例
- **主要 API**：
  - `Context::new(size)` - 创建指定内存大小的上下文
  - `ctx.eval(source)` - 执行 JavaScript 代码
  - `ctx.compile(source)` - 编译 JavaScript 源码为字节码对象
  - `ctx.execute(&bytecode)` - 执行预编译字节码
  - `ctx.memory_stats()` - 获取内存使用统计
  - `ctx.register_native(...)` - 注册宿主原生函数
- **生命周期**：拥有 `Heap` (GC) 和 `Interpreter`，负责整体资源管理

**[led-runtime/src/lib.rs](../led-runtime/src/lib.rs)**
- **功能**：LED 产品层入口文件，导出 effect 宿主 API
- **导出类型**：`EffectEngine`, `EffectInstance`, `EffectManager`, `ConfigValue`

**[led-runtime/src/effect.rs](../led-runtime/src/effect.rs)**
- **功能**：最小产品级 effect 宿主 API 封装（产品层主实现位置）
- **主要类型**：
  - `EffectEngine` - 从源码或字节码创建可实例化 effect 模板
  - `EffectInstance` - 运行中的 effect 实例
  - `EffectManager` - 最小多实例 / 多脚本调度层
  - `ConfigValue` - 基础配置值类型
- **主要能力**：
  - `EffectEngine::from_source()`
  - `EffectEngine::from_bytecode()`
  - `engine.instantiate_from_expr(config)`
  - `manager.add_engine()` / `manager.instantiate_from_expr()` / `manager.activate()`
  - `manager.tick_active()` / `manager.active_led_buffer()`
  - `instance.start()` / `tick()` / `pause()` / `resume()` / `stop()`
  - `instance.led_buffer()` / `led_count()` / `set_config()` / `reset()`
- **定位**：当前是最小可用的产品 API 与调度层雏形，已在 `led-runtime/` 中形成独立产品层开发入口

---

### 值系统 (src/value.rs)

**[src/value.rs](../src/value.rs)**
- **功能**：定义 JavaScript 值的表示和操作
- **核心设计**：
  - 使用 `u64` 标签联合体（tagged union）
  - 31 位整数内联存储
  - 短浮点（`f32`）内联存储
  - 特殊值（null/undefined/bool）直接编码
  - 堆对象使用索引引用，而非原始指针
- **值类型**：
  - 数值：整数与短浮点
  - 特殊值：null, undefined, true, false
  - 堆对象：通过索引引用 interpreter 拥有的 Vec
    - 字符串
    - 对象
    - 数组
    - 闭包/函数
    - 错误对象
    - 正则表达式
    - 类型化数组（TypedArray）
    - 内置对象
- **API 示例**：
  ```rust
  Value::int(42)
  Value::bool(true)
  Value::null()
  value.to_i32()
  value.to_bool()
  value.to_f32()
  ```

---

### 虚拟机 (src/vm/)

#### **[src/vm/mod.rs](../src/vm/mod.rs)**
- VM 模块入口，导出解释器和相关类型

#### **[src/vm/interpreter.rs](../src/vm/interpreter.rs)**
- **功能**：核心解释器，实现字节码执行循环
- **主要组件**：
  - 字节码分发循环 (`run` 方法)
  - 算术运算符处理
  - 比较运算符处理
  - 堆对象管理（Vec 索引模式）
- **状态管理**：
  - 值栈
  - 调用栈
  - 所有堆分配对象的 Vec
- **特性**：
  - 完整的异常处理机制
  - 宿主桥接、全局变量与运行时对象管理

#### **[src/vm/opcode.rs](../src/vm/opcode.rs)**
- **功能**：定义解释器使用的操作码（opcode）
- **操作码分类**：
  - 常量与字面量
  - 栈操作与局部变量访问
  - 算术 / 比较 / 位运算
  - 对象、数组、类型化数组访问
  - 控制流与异常处理
  - 函数调用、闭包与宿主函数交互
  - `typeof` / `in` / `instanceof` / `delete` 等杂项操作

#### **[src/vm/natives.rs](../src/vm/natives.rs)**
- **功能**：实现内建对象与全局函数对应的原生方法
- **主要类别**：
  - Array 方法：`push`, `pop`, `shift`, `unshift`, `slice`, `splice`, `join`, `reverse`, `sort`, `indexOf`, `lastIndexOf`, `includes`
  - String 方法：`charCodeAt`, `charAt`, `indexOf`, `lastIndexOf`, `slice`, `substring`, `trim`, `toLowerCase`, `toUpperCase`, `split`
  - Math 函数：`abs`, `floor`, `ceil`, `round`, `min`, `max`, `random`, `sqrt`, `pow`, `sin`, `cos`, `tan`
  - JSON 方法：`parse`, `stringify`, `format_value`（内部）
  - RegExp 方法：`test`, `match`, `replace`, `split`, `exec`
  - Date 方法：基础日期时间操作
  - 工具函数：`isNaN`, `isFinite`, `parseInt`, `parseFloat`

#### **[src/vm/property.rs](../src/vm/property.rs)**
- **功能**：属性访问的分发逻辑
- **方法**：
  - `get_array_property` - 数组属性访问
  - `get_string_property` - 字符串属性访问（length, 方法）
  - `get_builtin_property` - 内置对象属性访问
  - `get_number_property` - 数字属性访问
  - `get_error_property` - 错误对象属性访问
  - `get_regexp_property` - 正则表达式属性访问
  - `get_typed_array_property` - 类型化数组属性访问

#### **[src/vm/stack.rs](../src/vm/stack.rs)**
- **功能**：值栈和调用帧管理
- **主要结构**：
  - `ValueStack` - 值栈，支持 push/pop/peek
  - `CallFrame` - 调用帧，记录函数调用上下文

#### **[src/vm/ops.rs](../src/vm/ops.rs)**
- **功能**：运算符实现
- **运算类别**：
  - 算术运算：整数运算，处理溢出和类型检查
  - 比较运算：严格和宽松比较
  - 位运算：完整支持
  - 逻辑运算：短路求值

#### **[src/vm/types.rs](../src/vm/types.rs)**
- **功能**：VM 相关类型定义
- **主要类型**：
  - `NativeFn` - 原生函数类型别名
  - `ErrorInfo` - 错误信息结构

---

### 解析与编译 (src/parser/)

#### **[src/parser/mod.rs](../src/parser/mod.rs)**
- 解析器模块入口

#### **[src/parser/lexer.rs](../src/parser/lexer.rs)**
- **功能**：词法分析器，将源码转换为 Token 流
- **支持的 Token**：
  - 关键字：var, let, const, function, if, else, for, while, return, break, continue, try, catch, finally, throw, new, typeof, instanceof, in, delete, true, false, null, undefined
  - 字面量：整数、字符串、布尔值
  - 运算符：所有标准 JS 运算符
  - 标点符号：括号、大括号、方括号、逗号、分号等
- **特性**：
  - UTF-8 支持
  - 正则表达式字面量解析
  - 错误位置跟踪

#### **[src/parser/compiler.rs](../src/parser/compiler.rs)**
- **功能**：语法分析和字节码生成
- **主要组件**：
  - 递归下降解析器
  - 优先级爬升表达式解析
  - 字节码发射器
- **支持的语法**：
  - 变量声明：var, let, const
  - 函数：声明和表达式，支持递归
  - 控制流：if/else, while, for, for-in, for-of, break, continue
  - 异常处理：try/catch/finally, throw
  - 表达式：完整支持，包括三元运算符、逻辑短路
  - 对象和数组字面量
  - 成员访问：点表示法和括号表示法
  - 函数调用：普通调用和方法调用（this 绑定）
- **输出**：`FunctionBytecode` 结构体

---

### 运行时类型 (src/runtime/)

#### **[src/runtime/mod.rs](../src/runtime/mod.rs)**
- 运行时模块入口

#### **[src/runtime/object.rs](../src/runtime/object.rs)**
- **功能**：JavaScript 对象表示
- **主要结构**：
  - `JsObject` - JS 对象，包含属性表
  - `PropertyTable` - 属性表实现
- **特性**：
  - 属性存储优化
  - 原型链支持（简化版）

#### **[src/runtime/string.rs](../src/runtime/string.rs)**
- **功能**：JavaScript 字符串表示
- **主要结构**：
  - `JsString` - JS 字符串，UTF-8 编码
- **特性**：
  - 长度缓存
  - 哈希缓存

#### **[src/runtime/array.rs](../src/runtime/array.rs)**
- **功能**：JavaScript 数组表示
- **主要结构**：
  - `JsArray` - JS 数组，稀疏数组支持
- **特性**：
  - 密集数组优化
  - 稀疏数组处理

#### **[src/runtime/function.rs](../src/runtime/function.rs)**
- **功能**：JavaScript 函数和闭包
- **主要结构**：
  - `JsFunction` - JS 函数
  - `Closure` - 闭包，包含环境和字节码
  - `NativeFunction` - 原生函数包装
- **特性**：
  - 闭包捕获
  - 环境链
  - this 绑定

#### **[src/runtime/property.rs](../src/runtime/property.rs)**
- **功能**：属性表示和访问
- **主要结构**：
  - `Property` - 属性值
  - `PropertyKey` - 属性键（字符串或整数）

#### **[src/runtime/call.rs](../src/runtime/call.rs)**
- **功能**：函数调用机制
- **主要功能**：
  - 调用参数处理
  - this 绑定
  - 返回值处理

---

### 内置对象 (src/builtins/)

#### **[src/builtins/mod.rs](../src/builtins/mod.rs)**
- 内置对象模块入口

#### **[src/builtins/object.rs](../src/builtins/object.rs)**
- **功能**：Object 对象方法
- **实现方法**：
  - `Object.keys()`
  - `Object.values()`
  - `Object.entries()`
  - `Object.assign()`
  - `Object.create()`
  - `Object.defineProperty()`
  - `Object.defineProperties()`
  - `Object.getOwnPropertyDescriptor()`
  - `Object.getOwnPropertyNames()`

#### **[src/builtins/array.rs](../src/builtins/array.rs)**
- **功能**：Array 对象方法
- **实现方法**：
  - `Array()`, `Array.of()`, `Array.from()`
  - `push()`, `pop()`, `shift()`, `unshift()`
  - `slice()`, `splice()`, `concat()`
  - `join()`, `reverse()`, `sort()`
  - `indexOf()`, `lastIndexOf()`, `includes()`
  - `forEach()`, `map()`, `filter()`, `reduce()`, `find()`
  - `every()`, `some()`, `findIndex()`

#### **[src/builtins/string.rs](../src/builtins/string.rs)**
- **功能**：String 对象方法
- **实现方法**：
  - `charAt()`, `charCodeAt()`
  - `indexOf()`, `lastIndexOf()`
  - `slice()`, `substring()`, `substr()`
  - `trim()`, `trimLeft()`, `trimRight()`
  - `toLowerCase()`, `toUpperCase()`
  - `split()`, `replace()`, `match()`
  - `startsWith()`, `endsWith()`, `includes()`

#### **[src/builtins/math.rs](../src/builtins/math.rs)**
- **功能**：Math 对象和函数
- **实现方法**：
  - 常量：`E`, `PI`, `LN2`, `LN10`, `LOG2E`, `LOG10E`, `SQRT2`, `SQRT1_2`
  - 函数：`abs()`, `ceil()`, `floor()`, `round()`, `max()`, `min()`, `random()`, `sqrt()`, `pow()`, `exp()`, `log()`, `sin()`, `cos()`, `tan()`, `asin()`, `acos()`, `atan()`, `atan2()`

#### **[src/builtins/json.rs](../src/builtins/json.rs)**
- **功能**：JSON 解析和序列化
- **实现方法**：
  - `JSON.parse()`
  - `JSON.stringify()`
- **特性**：
  - 支持嵌套对象和数组
  - 支持转义字符
  - 处理循环引用

#### **[src/builtins/regexp.rs](../src/builtins/regexp.rs)**
- **功能**：RegExp 对象实现
- **主要结构**：
  - `RegExpObject` - 正则表达式对象
- **支持**：
  - 字面量语法：`/pattern/flags`
  - 构造函数：`new RegExp(pattern, flags)`
  - 方法：`test()`, `match()`, `replace()`, `split()`, `exec()`
  - 标志：`g` (global), `i` (ignore case), `m` (multiline)

#### **[src/builtins/error.rs](../src/builtins/error.rs)**
- **功能**：错误对象和异常
- **实现**：
  - `Error` 构造函数
  - `name`, `message` 属性
  - 标准错误类型：`Error`, `TypeError`, `ReferenceError`, `SyntaxError`, `RangeError`

#### **[src/builtins/typed_array.rs](../src/builtins/typed_array.rs)**
- **功能**：类型化数组（TypedArray）实现
- **主要结构**：
  - `TypedArrayObject` - 类型化数组对象
- **支持类型**：
  - `Uint8Array` - 主要支持，用于 LED 缓冲区
  - `Int8Array`
  - `Uint16Array`, `Int16Array`
  - `Uint32Array`, `Int32Array`
- **方法**：
  - 构造函数：`new TypedArray(length)`, `new TypedArray(array)`
  - 属性：`length`, `buffer`, `byteOffset`, `byteLength`
  - 方法：`fill()`, `slice()`, `set()`, `subarray()`

#### **[src/builtins/date.rs](../src/builtins/date.rs)**
- **功能**：Date 对象实现
- **实现方法**：
  - `Date()` 构造函数
  - `getTime()`, `setTime()`
  - `getFullYear()`, `getMonth()`, `getDate()`
  - `getHours()`, `getMinutes()`, `getSeconds()`, `getMilliseconds()`
  - `getTimezoneOffset()`
  - `toDateString()`, `toTimeString()`, `toISOString()`

#### **[src/builtins/number.rs](../src/builtins/number.rs)**
- **功能**：Number 对象和常量
- **实现**：
  - 常量：`MAX_VALUE`, `MIN_VALUE`, `NaN`, `Infinity`
  - 方法：`isFinite()`, `isNaN()`, `parseInt()`, `parseFloat()`, `toString()`

#### **[src/builtins/function.rs](../src/builtins/function.rs)**
- **功能**：Function 对象和方法
- **实现**：
  - `Function()` 构造函数
  - `call()`, `apply()`, `bind()`
  - `length`, `name` 属性

---

### 垃圾回收

#### 活跃 GC：Plan B（src/vm/gc.rs）

**[src/vm/gc.rs](../src/vm/gc.rs)**
- **功能**：代际式标记-清除（generation-based mark-sweep）垃圾回收器
- **设计**：
  - 所有堆对象存储在 Vec 中，通过索引引用
  - 使用 `gen[]` 代际数组标记活跃/空闲槽位
  - 迭代式标记（堆分配工作队列，无栈溢出）
  - 空闲槽位复用（free-list allocation）
  - 自适应触发阈值调整
- **GC 托管容器**：
  - closures, var_cells, arrays, objects
  - for_in_iterators, for_of_iterators
  - error_objects, regex_objects
  - typed_arrays, array_buffers, timers
- **根集合**：
  - 值栈 / 活跃调用帧
  - 全局变量
  - 通过 var_cells 捕获的闭包
  - timers.callback
- **特性**：
  - 支持循环引用回收
  - `no_std` 兼容（仅使用 `alloc`）
  - 详见 `docs/GC_PROGRESS.md`

#### Plan C 预备（src/gc/）

**[src/gc/mod.rs](../src/gc/mod.rs)**
- GC 模块入口

**[src/gc/allocator.rs](../src/gc/allocator.rs)**
- **功能**：竞技场分配器（Arena allocator）
- **设计**：
  - 基于内存块的连续分配
  - 快速分配和释放
  - 适用于嵌入式环境
- **用途**：被 `Context` 用于内存统计，同时是 Plan C arena 的预备代码

**[src/gc/collector.rs](../src/gc/collector.rs)**
- **状态**：占位符 stub — `collect()` 调用保守的 `mark_all()`
- **目的**：Plan C（完整标记-压缩 GC）的起点代码
- **未来工作**：
  - 完成 `mark_object()` 对所有类型的支持
  - 实现压缩阶段的指针更新逻辑
  - `Value` 从索引编码改为指针编码（影响约 20 个文件、1850 行）

---

### 工具函数 (src/util/)

#### **[src/util/mod.rs](../src/util/mod.rs)**
- 工具模块入口

#### **[src/util/dtoa.rs](../src/util/dtoa.rs)**
- **功能**：数字到字符串转换
- **实现**：
  - 高精度浮点数格式化
  - 优化算法，减少内存分配

#### **[src/util/unicode.rs](../src/util/unicode.rs)**
- **功能**：Unicode 处理工具
- **实现**：
  - UTF-8 编码/解码
  - Unicode 分类（字母、数字、空白等）

---

### 命令行工具 (src/bin/)

#### **[src/bin/mqjs.rs](../src/bin/mqjs.rs)**
- **功能**：JavaScript REPL 和 CLI 工具
- **特性**：
  - 交互式 REPL（使用 rustyline）
  - 执行脚本文件
  - 执行表达式（`-e` 参数）
  - 编译字节码（`-c` 参数）
  - 调试信息（`-d` 参数）
  - 内存限制（`--memory-limit` 参数）
  - 包含文件（`-I` 参数）
- **命令行参数**：
  ```
  -h, --help              显示帮助
  -e, --eval EXPR         计算表达式
  -i, --interactive       执行后进入 REPL
  -I, --include FILE      在主脚本前包含文件
  -d, --dump             转储内存使用统计
  -c, --compile          编译为字节码（.qbc 文件）
  --memory-limit N        限制内存（支持 k/K, m/M 后缀）
  ```

---

## JavaScript 脚本 (js/)

### 效果脚本 (led-runtime/js/effects/)

#### LED 特效脚本，演示产品使用场景

**[led-runtime/js/effects/blink/effect.js](../led-runtime/js/effects/blink/effect.js)**
- **效果**：闪烁灯效
- **功能**：所有 LED 同时闪烁
- **配置参数**：
  - `speed` - 闪烁速度（毫秒）
  - `color` - 颜色配置（RGB 或 HSV）
  - `ledCount` - LED 数量

**[led-runtime/js/effects/chase/effect.js](../led-runtime/js/effects/chase/effect.js)**
- **效果**：追逐灯效
- **功能**：颜色在 LED 条上移动
- **配置参数**：
  - `speed` - 移动速度
  - `chaseCount` - 追逐点数量
  - `color` - 颜色配置
  - `ledCount` - LED 数量

**[led-runtime/js/effects/rainbow/effect.js](../led-runtime/js/effects/rainbow/effect.js)**
- **效果**：彩虹渐变
- **功能**：LED 显示平滑的颜色渐变
- **配置参数**：
  - `speed` - 渐变速度
  - `hueSpread` - 色相分布
  - `ledCount` - LED 数量

**[led-runtime/js/effects/wave/effect.js](../led-runtime/js/effects/wave/effect.js)**
- **效果**：波浪效果
- **功能**：颜色波浪在 LED 条上传播
- **配置参数**：
  - `speed` - 波浪速度
  - `waveWidth` - 波浪宽度
  - `color` - 颜色配置
  - `ledCount` - LED 数量

### 示例脚本 (js/examples/)

展示引擎各项功能的示例代码

**[js/examples/hello.js](../js/examples/hello.js)**
- 最简单的 Hello World 示例

**[js/examples/arrays.js](../js/examples/arrays.js)**
- 数组操作示例
- 包含：创建、访问、迭代、方法使用

**[js/examples/strings.js](../js/examples/strings.js)**
- 字符串操作示例
- 包含：拼接、查找、替换、分割

**[js/examples/objects.js](../js/examples/objects.js)**
- 对象操作示例
- 包含：创建、属性访问、方法定义

**[js/examples/math.js](../js/examples/math.js)**
- 数学运算示例
- 包含：Math 对象、三角函数、随机数

**[js/examples/closures.js](../js/examples/closures.js)**
- 闭包示例
- 展示：变量捕获、作用域链

**[js/examples/functional.js](../js/examples/functional.js)**
- 函数式编程示例
- 包含：高阶函数、map/filter/reduce

**[js/examples/factorial.js](../js/examples/factorial.js)**
- 递归函数示例
- 计算 n! 阶乘

**[js/examples/fibonacci.js](../js/examples/fibonacci.js)**
- 斐波那契数列示例
- 递归实现

**[js/examples/json.js](../js/examples/json.js)**
- JSON 操作示例
- 包含：解析、序列化

**[js/examples/regex.js](../js/examples/regex.js)**
- 正则表达式示例
- 包含：匹配、替换、分割

**[js/examples/typed_arrays.js](../js/examples/typed_arrays.js)**
- 类型化数组示例
- 包含：创建、操作、LED 缓冲区场景

**[js/examples/exceptions.js](../js/examples/exceptions.js)**
- 异常处理示例
- 包含：try/catch/finally, throw

### 测试脚本 (js/tests/)

错误处理测试用例

**[js/tests/compile_errors.js](../js/tests/compile_errors.js)**
- 编译时错误测试
- 验证语法错误、未声明变量等

**[js/tests/runtime_errors.js](../js/tests/runtime_errors.js)**
- 运行时错误测试
- 验证类型错误、引用错误等

**[js/tests/test_errors.js](../js/tests/test_errors.js)**
- 综合错误测试

---

## 示例程序 (examples/)

### 基础示例

**[led-runtime/examples/effects_demo.rs](../led-runtime/examples/effects_demo.rs)**
- **功能**：终端 LED 效果演示
- **特性**：
  - 运行所有 4 个 LED 效果
  - 使用 ANSI 24 位颜色渲染
  - 原生函数：`__renderFrame()` 用于渲染
- **运行**：`cargo run -p led-runtime --example effects_demo`

**[led-runtime/examples/common/effects.rs](../led-runtime/examples/common/effects.rs)**
- **功能**：效果引擎公共代码
- **内容**：
  - 效果加载和管理
  - 配置处理
  - 生命周期管理

### GUI 示例

**[led-runtime/examples/effects_egui.rs](../led-runtime/examples/effects_egui.rs)**
- **功能**：EGUI 图形界面 LED 效果演示
- **特性**：
  - 使用 eframe/egui 框架
  - 可视化 LED 条
  - 实时效果切换
- **运行**：`cargo run -p led-runtime --example effects_egui`

**[led-runtime/examples/effects_slint/main.rs](../led-runtime/examples/effects_slint/main.rs)**
- **功能**：Slint GUI 框架 LED 效果演示
- **特性**：
  - 使用 Slint 框架
  - 跨平台支持
  - 现代 UI 设计
- **运行**：`cargo run -p led-runtime --example effects_slint`

---

## 测试 (tests/)

### 集成测试

**[led-runtime/tests/effects.rs](../led-runtime/tests/effects.rs)**
- LED 效果集成测试
- **测试覆盖**（22 个测试）：
  - createEffect 基础功能
  - 状态机转换（idle/running/paused）
  - tick 行为
  - 配置更新（setConfig）
  - 颜色转换（HSV → RGB）
  - 生命周期管理（start/pause/resume/stop）
- **运行**：`cargo test -p led-runtime --test effects`

**[tests/eval_integration.rs](../tests/eval_integration.rs)**
- eval 功能集成测试
- **测试内容**：
  - 基本表达式求值
  - 作用域和闭包
  - 异常处理
  - 类型转换

**[tests/error_messages.rs](../tests/error_messages.rs)**
- 错误消息测试
- **测试内容**：
  - 编译错误消息格式
  - 运行时错误消息格式
  - 错误位置信息

---

## 性能基准测试 (benches/)

**[benches/js_benchmarks.rs](../benches/js_benchmarks.rs)**
- **功能**：使用 criterion 的性能基准测试
- **基准场景**：
  - 数组操作
  - 函数调用
  - 字符串处理
  - JSON 操作
  - 数学运算
- **运行**：`cargo bench`

**[benches/scripts/](../benches/scripts/)**
- **[fib.js](../benches/scripts/fib.js)** - 斐波那契数列基准
- **[sieve.js](../benches/scripts/sieve.js)** - 质数筛法基准
- **[array.js](../benches/scripts/array.js)** - 数组操作基准
- **[string.js](../benches/scripts/string.js)** - 字符串操作基准
- **[object.js](../benches/scripts/object.js)** - 对象操作基准
- **[closure.js](../benches/scripts/closure.js)** - 闭包基准
- **[loop.js](../benches/scripts/loop.js)** - 循环基准
- **[json.js](../benches/scripts/json.js)** - JSON 操作基准

**[benches/compare.sh](../benches/compare.sh)**
- 与原始 C 实现的性能对比脚本

---

## 文档 (docs/)

### 产品文档

**[led-runtime/docs/LED_PROFILE.md](../led-runtime/docs/LED_PROFILE.md)**
- **内容**：LED 特效脚本的产品规范
- **定义**：
  - 脚本模型（createEffect, tick, leds 等）
  - 支持的语言能力（受限 ES6 风格）
  - 受限语义（整数优先、简化对象模型）
  - 不支持的特性（class, async/await 等）
  - 宿主运行时要求
  - 安全与资源约束

**[led-runtime/docs/PRODUCT_ROADMAP.md](../led-runtime/docs/PRODUCT_ROADMAP.md)**
- **内容**：产品化路线图
- **阶段划分**：
  - Phase 1：规范冻结（已完成）
  - Phase 2：LED 最小闭环（已完成）
  - Phase 3：宿主接口产品化（进行中）
  - Phase 4-7：资源重构、执行安全、离线工具链、ESP32 集成

### 技术文档

**[docs/JS_FEATURE_SPEC.md](../docs/JS_FEATURE_SPEC.md)**
- **内容**：JavaScript 特性规范
- **定义**：
  - 当前数值与语义模型（`i32 + f32` 混合模型）
  - 支持的特性（语法、内置对象、受限实现边界）
  - 不支持或不应依赖的特性（如 `class`、`async/await`、完整 `ToPrimitive` 等）

**[led-runtime/docs/EFFECT_ENGINE_API.md](../led-runtime/docs/EFFECT_ENGINE_API.md)**
- **内容**：最小产品级 effect 宿主 API 说明
- **定义**：
  - `EffectEngine` / `EffectInstance` 的定位与职责
  - 从源码 / 字节码创建引擎
  - effect 实例生命周期与 `led_buffer()` 的使用方式

**[docs/HOW_IT_WORKS.md](../docs/HOW_IT_WORKS.md)**
- **内容**：引擎工作原理说明
- **章节**：
  - 执行流水线概述
  - 值表示
  - 字节码设计
  - 对象模型
  - 函数与闭包
  - 垃圾回收

**[docs/BENCHMARK_ANALYSIS.md](../docs/BENCHMARK_ANALYSIS.md)**
- **内容**：性能基准分析
- **包含**：
  - 与其他引擎的对比
  - 各场景性能数据
  - 优化建议

### 项目文档

**[IMPLEMENTATION_PLAN.md](../IMPLEMENTATION_PLAN.md)**
- **内容**：实现计划和技术决策记录
- **用途**：开发过程参考

**[CLAUDE.md](../CLAUDE.md)**
- **内容**：Claude Code 工作指导
- **包含**：
  - 项目概述
  - 命令参考
  - 架构说明
  - 约定规范

**[CLAUDE.zh.md](../CLAUDE.zh.md)**
- **内容**：CLAUDE.md 的中文版本

**[README.md](../README.md)**
- **内容**：项目首页文档
- **包含**：
  - 项目介绍
  - 特性列表
  - 安装说明
  - 使用指南
  - CLI 参数

**[README.zh.md](../README.zh.md)**
- **内容**：README.md 的中文版本

---

## CI/CD 配置 (.github/workflows/)

**[.github/workflows/ci.yml](../.github/workflows/ci.yml)**
- **功能**：持续集成测试
- **检查项**：
  - 格式检查（cargo fmt）
  - Clippy lint
  - 单元测试
  - 集成测试
  - 跨平台测试

**[.github/workflows/bench.yml](../.github/workflows/bench.yml)**
- **功能**：性能基准测试
- **用途**：监控性能回归

---

## Claude Code 配置 (.claude/)

**[.claude/commands/add-builtin.md](../.claude/commands/add-builtin.md)**
- 添加内置方法的命令指南

**[.claude/commands/check.md](../.claude/commands/check.md)**
- 运行完整 CI 检查的命令

**[.claude/commands/run.md](../.claude/commands/run.md)**
- 运行 JavaScript 文件或表达式的命令

**[.claude/settings.json](../.claude/settings.json)**
- Claude Code 全局设置

**[.claude/settings.local.json](../.claude/settings.local.json)**
- Claude Code 本地设置（不纳入版本控制）

---

## 项目配置文件

**[Cargo.toml](../Cargo.toml)**
- **包信息**：名称、版本、描述、许可证
- **依赖**：regex, rustyline, libm
- **特性**：
  - `std` - 标准库支持（默认）
  - `dump` - 调试转储
  - `debug-gc` - 调试 GC
- **编译配置**：LTO，opt-level 3
- **二进制和示例**：mqjs CLI, effects_egui, effects_slint
- **基准测试**：js_benchmarks

**[Cargo.lock](../Cargo.lock)**
- 依赖版本锁定文件

**[.gitignore](../.gitignore)**
- Git 忽略规则
- 忽略：target/, .cargo/, IDE 文件等

**[.gitmodules](../.gitmodules)**
- Git 子模块配置
- 包含：vendor/mquickjs（原始 C 实现）

---

## 第三方依赖 (vendor/)

**[vendor/mquickjs/](../vendor/mquickjs/)**
- **来源**：Fabrice Bellard 的原始 MQuickJS C 实现
- **用途**：参考实现，用于对比和验证

---

## 构建与测试命令

### 构建

```bash
cargo build              # Debug 构建
cargo build --release    # Release 构建
```

### 测试

```bash
cargo test                       # 运行所有测试
cargo test -p led-runtime --test effects         # 运行效果集成测试
cargo test --test eval_integration # 运行 eval 集成测试
cargo test -- --nocapture        # 显示测试输出
```

### Lint 和格式

```bash
cargo fmt --check               # 检查格式
cargo fmt                      # 格式化代码
cargo clippy -- -D warnings     # Clint 检查（零警告）
```

### 运行

```bash
cargo run --bin mqjs                          # 启动 REPL
cargo run --bin mqjs -- script.js             # 运行脚本
cargo run --bin mqjs -- -e "1 + 2"           # 计算表达式
cargo run -p led-runtime --example effects_demo              # 运行终端演示
cargo run -p led-runtime --example effects_egui              # 运行 EGUI 演示
cargo run -p led-runtime --example effects_slint             # 运行 Slint 演示
```

### 基准测试

```bash
cargo bench                                   # 运行基准测试
```

---

## 代码组织原则

### 模块职责清晰
- **parser/**：仅负责词法分析和语法分析
- **vm/**：仅负责字节码执行
- **runtime/**：仅负责运行时数据结构
- **builtins/**：仅负责内置对象方法

### 最小化依赖
- 核心功能无外部依赖
- 仅 CLI 需要 rustyline（REPL）
- 仅正则表达式需要 regex

### 嵌入式友好
- 支持 `no_std` 模式
- 内存可控
- 无动态分配（核心路径）

### 测试覆盖
- 单元测试内联在源文件中（`#[cfg(test)]`）
- 集成测试在 `tests/` 目录
- 性能基准在 `benches/` 目录

---

## 开发约定

### 代码风格
- 使用 `cargo fmt` 统一格式
- CI 强制零 Clippy 警告
- 模块按功能组织，而非按类型

### 提交规范
- 提交前必须通过 CI
- 包含相应的测试用例
- 更新相关文档

### 文档更新
- 新增功能时更新 JS_FEATURE_SPEC.md
- 产品化变更时更新 PRODUCT_ROADMAP.md
- 架构变更时更新 HOW_IT_WORKS.md

---

## 相关资源

- **主文档**：[README.md](../README.md)
- **产品规范**：[led-runtime/docs/LED_PROFILE.md](../led-runtime/docs/LED_PROFILE.md)
- **特性规范**：[docs/JS_FEATURE_SPEC.md](../docs/JS_FEATURE_SPEC.md)
- **路线图**：[led-runtime/docs/PRODUCT_ROADMAP.md](../led-runtime/docs/PRODUCT_ROADMAP.md)
- **Claude 指导**：[CLAUDE.md](../CLAUDE.md)



