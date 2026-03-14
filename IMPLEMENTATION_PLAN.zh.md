# MQuickJS Rust 移植计划

## 项目概览

**目标**：完整功能的 Rust 移植版本 MQuickJS (Fabrice Bellard 的极简 JS 引擎)
**方案**：惯用 Rust 重写，性能匹配 C 版本
**API**：仅原生 Rust API

**源码统计**：~28K 行 C -> 预计 ~20-25K 行 Rust
**参考**：`/Users/qing/p/github/mquickjs-ref/`

---

## 实施阶段

### 阶段 1：基础设施
**目标**：核心类型和内存基础设施

- [x] 1.1 创建 Cargo 项目和工作区结构
- [x] 1.2 实现 `JSValue` 枚举（标记联合体，匹配 C 布局）
- [x] 1.3 实现 arena 分配器 (`gc/allocator.rs`)
- [x] 1.4 实现基础 GC 特性和收集器 (`gc/collector.rs`)
- [x] 1.5 实现 `JSContext` 结构体及内存布局
- [x] 1.6 添加 cutils 等价工具 (`util/mod.rs`)

**状态**：完成

---

### 阶段 2：对象系统
**目标**：JavaScript 对象表示和属性访问

- [x] 2.1 实现 `JSObject` 结构体及类系统
- [x] 2.2 实现 `JSString` 及 UTF-8 存储
- [x] 2.3 实现属性哈希表
- [x] 2.4 实现基本属性操作
- [x] 2.5 实现 `JSArray`（无空洞语义）
- [x] 2.6 实现 `JSFunction` 类型（闭包、C 函数）

**状态**：完成

---

### 阶段 3：字节码和 VM 核心
**目标**：执行字节码指令

- [x] 3.1 定义 opcode 枚举（移植 `mquickjs_opcode.h`）
- [x] 3.2 实现 `JSFunctionBytecode` 结构体
- [x] 3.3 实现值栈
- [x] 3.4 实现字节码解释器循环
- [x] 3.5 实现函数调用机制

**状态**：完成

---

### 阶段 4：解析器和编译器
**目标**：解析 JavaScript 源码为字节码

- [x] 4.1 实现 lexer/tokenizer
- [x] 4.2 实现解析器状态机
- [x] 4.3 实现表达式解析
- [x] 4.4 实现语句解析
- [x] 4.5 实现字节码生成
- [x] 4.6 实现作用域和变量解析（局部变量）

**状态**：完成（闭包待阶段 7）

---

### 阶段 5：核心内置对象
**目标**：核心 JavaScript 内置对象

- [x] 5.1 实现 `Object` 构造函数和原型（部分：Object.keys, Object.values, Object.entries）
- [x] 5.2 实现 `Function` 原型（call, apply, bind）
- [x] 5.3 实现 `Array` 构造函数和方法（push, pop, shift, unshift, indexOf, join, reverse, slice, length, Array.isArray, map, filter, forEach, reduce, find, findIndex, some, every, includes）
- [x] 5.4 实现 `String` 构造函数和方法（length, charAt, indexOf, slice, substring, toUpperCase, toLowerCase, trim, split）
- [x] 5.5 实现 `Number` 构造函数和方法（isInteger, isNaN, isFinite, MAX_VALUE, MIN_VALUE）
- [x] 5.6 实现 `Boolean` 构造函数（Boolean/Number/String 作为函数）
- [x] 5.7 实现全局函数（部分：parseInt, isNaN）

**状态**：进行中（原生函数基础设施已完成）

---

### 阶段 6：扩展内置对象
**目标**：完整的内置库

- [x] 6.1 实现 `Error` 层级（Error, TypeError, ReferenceError, SyntaxError, RangeError）
- [x] 6.2 实现 `Math` 对象（部分：abs, floor, ceil, round, sqrt, pow, max, min）
- [x] 6.3 实现 `JSON` 对象（stringify, parse）
- [x] 6.4 实现 `RegExp` 对象（构造函数, test, exec）
- [x] 6.5 实现 `TypedArray` 对象
- [x] 6.6 实现 `Date.now()`

**状态**：完成

---

### 阶段 7：高级特性
**目标**：完整语言特性

- [x] 7.1 实现 `for-in` 迭代
- [x] 7.2 实现 `for-of` 迭代
- [x] 7.3 实现 `try-catch-finally`
- [x] 7.4 实现闭包变量捕获
- [x] 7.5 实现数组字面量和操作
- [x] 7.6 实现 `new` 操作符和基本对象支持
- [x] 7.7 实现 `delete` 和 `in` 操作符
- [x] 7.8 实现 `instanceof`

**状态**：完成

---

### 阶段 8：REPL 和 CLI
**目标**：可用的命令行工具

- [x] 8.1 实现 CLI 框架
- [x] 8.2 实现参数解析 (-h, -e, -i, -I, -d, -c, --memory-limit)
- [x] 8.3 实现行编辑（rustyline 历史记录）
- [x] 8.4 实现字节码序列化（.qbc 文件）
- [x] 8.5 实现内存统计（dump_memory_stats, MemoryStats 结构体）

**状态**：完成

---

### 阶段 9：优化和完善
**目标**：性能与 C 版本相当

- [ ] 9.1 分析和优化热点路径
- [ ] 9.2 优化 GC 性能
- [ ] 9.3 减少内存使用
- [x] 9.4 添加基准测试 ⚠️ 部分完成（已有 fib/array/json 等基准）
- [ ] 9.5 文档完善 ⚠️ 部分完成（docs/ 目录有文档）

**状态**：进行中
- ✅ 2026-03-14：验证 `no_std` 编译兼容性
- ✅ 2026-03-14：验证 `no_std` 测试通过（109/109）

---

## 当前进度

**最后更新**：阶段 8 完成（带内存统计的 CLI）

**创建/更新的文件**：
- `src/lib.rs` - 主库入口
- `src/value.rs` - JSValue 标记联合体，支持 string, closure, array
- `src/context.rs` - JSContext，包含闭包、try-catch、数组测试
- `src/gc/mod.rs`, `allocator.rs`, `collector.rs` - GC 系统
- `src/vm/mod.rs`, `opcode.rs`, `interpreter.rs`, `stack.rs` - VM，支持闭包、异常、数组
- `src/parser/mod.rs`, `lexer.rs`, `compiler.rs` - 解析器，支持闭包捕获、try-catch-finally、数组
- `src/builtins/` - 内置对象存根
- `src/runtime/mod.rs` - 运行时模块
- `src/runtime/object.rs` - JSObject, ClassId, Property 类型
- `src/runtime/string.rs` - JSString, StringTable
- `src/runtime/property.rs` - PropertyTable 哈希表
- `src/runtime/array.rs` - JSArray（无空洞语义）
- `src/runtime/function.rs` - CFunction, Closure, FunctionBytecode 及 CaptureInfo
- `src/util/mod.rs`, `dtoa.rs`, `unicode.rs` - 工具函数
- `src/bin/mqjs.rs` - REPL 二进制
- `src/effect.rs` - EffectEngine API（LED 效果引擎）

**测试数量**：328 个通过

**额外 mquickjs 特性（阶段 8 后）**：
- String.charCodeAt, String.lastIndexOf
- String.fromCharCode, String.fromCodePoint
- Array.lastIndexOf
- performance.now
- Object.getPrototypeOf, Object.setPrototypeOf, Object.create, Object.defineProperty
- Object.prototype.toString
- Math.sign, Math.sin, Math.cos, Math.tan, Math.exp, Math.log, Math.random, Math.atan2
- Math.asin, Math.acos, Math.atan
- Math.pow, Math.sqrt
- Math 常量：PI, E, LN2, LN10, LOG2E, LOG10E, SQRT2, SQRT1_2
- parseFloat, isFinite 全局函数
- Number.prototype.toString, toFixed, toExponential, toPrecision
- ArrayBuffer 构造函数及 byteLength 属性
- TypedArray.prototype.subarray
- Uint8ClampedArray, Float32Array, Float64Array TypedArray 类型
- EvalError, URIError, InternalError 错误类型
- Error.prototype.stack, Error.prototype.toString
- Array.prototype.toString, Array.prototype.reduceRight
- Function.prototype.toString
- gc() - 触发垃圾回收（占位符）
- load(filename) - 加载并执行 JavaScript 文件
- setTimeout(callback, delay) - 调度回调（返回 timer ID）
- clearTimeout(id) - 取消已调度的超时
- switch/case/do-while/void/debugger 语句

**阶段 8 CLI 特性**：
- 完整参数解析 (-h, -e, -i, -I, -d, -c, --memory-limit)
- 内存限制支持 k/K, m/M 后缀（如 --memory-limit 512k）
- 内存统计显示（堆大小、已用、运行时字符串、数组、对象、闭包等）
- Include 文件支持 (-I file)
- 交互模式 (-i) 在脚本执行后
- 带 rustyline 的 REPL（行编辑、历史记录、Ctrl+C/D 支持）
- 命令历史保存到 ~/.mqjs_history
- 字节码编译（-c 标志，输出 .qbc 文件）
- 字节码执行（自动加载 .qbc 文件）

**阶段 4 编译器特性**：
- 优先级爬升表达式解析器
- 所有二元操作符 (+, -, *, /, %, **, &, |, ^, <<, >>, >>>)
- 比较操作符 (<, <=, >, >=, ==, !=, ===, !==)
- 一元操作符 (-, +, !, ~, typeof, ++, --)
- 三元操作符 (?:)
- 短路逻辑操作符 (&&, ||)
- 赋值表达式 (=, +=, -=, *=, /=, %=, &=, |=, ^=, <<=, >>=, >>>=)
- 语句解析 (var/let/const, if/else, while, for, return, block)
- 局部变量跟踪和 max_locals 用于正确的帧分配
- 优化的整数发射 (Push0-7, PushI8, PushI16)
- 控制流的跳转补丁
- Context.eval() 用于端到端 JavaScript 执行
- 带参数的函数声明
- 带参数传递的函数调用
- 递归函数（通过 ThisFunc opcode）
- 循环中的 break 和 continue 语句
- typeof 操作符（返回正确的字符串值）
- 带连接支持的字符串字面量
- print 语句用于输出

**阶段 7.4 闭包特性**：
- 闭包变量捕获（值捕获语义）
- CaptureInfo 结构体用于跟踪捕获的变量
- GetVarRef/PutVarRef opcodes 用于访问捕获的变量
- ClosureData 结构体在解释器中存储捕获的值
- FClosure opcode 创建带捕获变量值的闭包
- Call opcode 处理闭包调用及正确的帧设置
- 从外部函数局部变量或捕获的嵌套闭包
- typeof closure 返回 "function"

**阶段 7.3 Try-Catch-Finally 特性**：
- throw 语句用于抛出异常
- try-catch 语句用于捕获异常
- 带 finally 块的 try-catch-finally 语句
- Catch opcode 设置异常处理器
- DropCatch opcode 在 try 正常完成时移除异常处理器
- Throw opcode 触发异常展开到最近的处理器
- ExceptionHandler 结构体跟踪帧深度、catch PC 和栈深度
- 异常值作为参数传递给 catch 块
- 嵌套 try-catch 及正确的处理器链式调用
- 异常通过函数调用传播

**阶段 7.5 数组特性**：
- 使用特殊标记编码的数组值类型
- 解释器中的数组存储 (Vec<Vec<Value>>)
- ArrayFrom opcode 从栈元素创建数组
- GetArrayEl/GetArrayEl2 opcodes 用于元素访问
- PutArrayEl opcode 用于元素赋值（自动扩展）
- 数组字面量解析：[expr, expr, ...]
- 数组访问解析：arr[idx] 和 arr[idx] = value
- 越界访问返回 undefined
- 数组字面量中的尾随逗号支持

**阶段 7.6 对象和 New 操作符特性**：
- 使用特殊标记编码的对象值类型（第 25 位标记）
- 解释器中的对象存储（Vec<(String, Value)> 用于属性）
- GetField/PutField opcodes 用于属性访问（obj.prop 和 obj.prop = val）
- new_expr_target() 解析构造函数但不消费调用
- CallConstructor opcode 创建对象并调用构造函数（this=object）
- typeof 对象返回 "object"
- typeof 比较的内置字符串常量

**阶段 7.8 InstanceOf 特性**：
- ObjectInstance 结构体在通过 `new` 创建时存储构造函数引用
- InstanceOf opcode 比较存储的构造函数与右操作数
- 同一构造函数的多个实例被正确识别
- 支持闭包和普通函数

**阶段 7.1 For-In 特性**：
- ForInIterator 结构体存储键和迭代位置
- 迭代器索引存储在隐藏局部变量中
- ForInStart opcode 从对象/数组创建迭代器
- ForInNext opcode 返回下一个键和 done 标志
- 迭代对象属性名或数组索引
- 支持循环中的 break 和 continue

**阶段 7.2 For-Of 特性**：
- ForOfIterator 结构体存储值和迭代位置
- 迭代器索引存储在隐藏局部变量中（类似 for-in）
- ForOfStart opcode 从对象/数组创建迭代器
- ForOfNext opcode 返回下一个值和 done 标志
- 迭代数组元素或对象属性值
- 支持循环中的 break 和 continue
- Token::Of 关键字添加到 lexer

**构造函数返回修复**：
- 向 CallFrame 添加 is_constructor 标志
- CallConstructor 现在使用 new_constructor/new_closure_constructor 帧创建器
- do_return 在构造函数不返回对象时自动返回 'this'
- 启用标准 JavaScript 构造函数行为（隐式 this 返回）

**阶段 5.7 原生函数特性**：
- 原生函数类型 (`NativeFn`) 和注册表 (`NativeFunction` 结构体)
- `native_functions: Vec<NativeFunction>` 解释器中的注册表
- `register_native()` 方法用于添加原生函数
- `get_native_func()` 方法用于按名称查找函数
- `call_native_func()` 方法用于调用原生函数
- Call opcode 处理器中的原生函数支持

**下一步**：阶段 8 CLI 改进或阶段 9 优化

---

**注意**：本计划会随着开发进度持续更新，请参考英文版 IMPLEMENTATION_PLAN.md 获取最新详细信息。
