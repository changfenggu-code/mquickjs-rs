# MQuickJS-RS

[English](README.md) | 中文

Fabrice Bellard 的 [MQuickJS](https://github.com/bellard/mquickjs) 的 Rust 移植版，当前已定位为面向 ESP32 等 MCU 设备的 LED 特效脚本运行时。

本仓库的产品目标不是完整实现 ECMAScript，而是提供一个**受限的 ES6 风格脚本子集**，重点保证可预测执行、资源可控和宿主集成稳定性。

相关文档：

- `docs/LED_PROFILE.md`：产品脚本规范
- `docs/PRODUCT_ROADMAP.md`：产品化路线图
- `docs/JS_FEATURE_SPEC.md`：当前引擎实现说明
- `docs/EMBEDDED_NO_STD.md`：`no_std` / ESP32 裸板集成说明
- `docs/EFFECT_ENGINE_API.md`：最小产品级 effect 宿主 API 说明

## 特性

- **面向嵌入式**：聚焦 MCU 上的 LED 特效脚本执行
- **受限 ES6 风格子集**：以文档定义的 Profile 为准，而不是完整 ES6
- **基于栈的虚拟机**：保持较小核心与较清晰的执行路径
- **TypedArray 数据通路**：以 `Uint8Array` 作为 LED 帧缓冲的核心载体
- **适合离线编译流程**：便于源码校验、编译与设备端加载

## 安装

```bash
# 克隆仓库
git clone https://github.com/user/mquickjs-rs.git
cd mquickjs-rs

# 构建
cargo build --release

# 运行测试
cargo test
```

## 使用方法

### 命令行

```bash
# 执行 JavaScript 文件
mqjs script.js

# 直接求值表达式
mqjs -e "1 + 2 * 3"

# 交互式 REPL
mqjs

# 编译为字节码
mqjs -c script.js    # 生成 script.qbc

# 执行字节码
mqjs script.qbc

# 显示内存用量
mqjs -d script.js

# 设置内存上限
mqjs --memory-limit 512k script.js
```

### 命令行选项

```
-h, --help         显示帮助
-e, --eval EXPR    直接求值表达式
-i, --interactive  执行脚本后进入 REPL
-I, --include FILE 在主脚本前加载指定文件
-d, --dump         输出内存使用统计
-c, --compile      编译为字节码（生成 .qbc 文件）
--memory-limit N   限制内存用量（支持 k/K、m/M 后缀）
```

### 库 API

```rust
use mquickjs::{Context, Value};

fn main() {
    // 创建上下文，分配 64KB 内存
    let mut ctx = Context::new(64 * 1024);

    // 执行 JavaScript
    let result = ctx.eval("1 + 2").unwrap();
    assert_eq!(result.to_i32(), Some(3));

    // 执行更复杂的代码
    let result = ctx.eval(r#"
        function factorial(n) {
            if (n <= 1) return 1;
            return n * factorial(n - 1);
        }
        factorial(5)
    "#).unwrap();
    assert_eq!(result.to_i32(), Some(120));
}
```

## 脚本 Profile

产品脚本应以 `docs/LED_PROFILE.md` 为准。

本项目有意采用受限脚本规范，而不是追求完整标准兼容。凡未在 Profile 中明确列出的特性，均不应视为产品可依赖能力。

## 引擎能力

### 语言核心

- 变量：`var`、`let`、`const`
- 函数：声明式、表达式、闭包、构造函数
- 控制流：`if/else`、`while`、`for`、`for-in`、`for-of`
- 运算符：算术、比较、逻辑、位运算、三元
- 异常处理：`try/catch/finally`、`throw`
- 对象字面量与属性访问
- 数组字面量与操作
- `new` 运算符（对象构造）
- `typeof`、`instanceof`、`in`、`delete` 运算符

### 内置对象

| 对象 | 方法 / 属性 |
|------|------------|
| **Object** | `keys`、`values`、`entries`、`create`、`defineProperty`、`getPrototypeOf`、`setPrototypeOf`、`hasOwnProperty`、`toString` |
| **Array** | `push`、`pop`、`shift`、`unshift`、`slice`、`splice`、`indexOf`、`lastIndexOf`、`join`、`reverse`、`concat`、`map`、`filter`、`forEach`、`reduce`、`reduceRight`、`find`、`findIndex`、`some`、`every`、`includes`、`sort`、`flat`、`fill`、`isArray`、`toString` |
| **String** | `length`、`charAt`、`charCodeAt`、`codePointAt`、`indexOf`、`lastIndexOf`、`slice`、`substring`、`toUpperCase`、`toLowerCase`、`trim`、`trimStart`、`trimEnd`、`split`、`concat`、`repeat`、`startsWith`、`endsWith`、`includes`、`padStart`、`padEnd`、`replace`、`replaceAll`、`match`、`search`、`fromCharCode`、`fromCodePoint` |
| **Number** | `isInteger`、`isNaN`、`isFinite`、`parseInt`、`MAX_VALUE`、`MIN_VALUE`、`MAX_SAFE_INTEGER`、`MIN_SAFE_INTEGER`、`toString`、`toFixed`、`toExponential`、`toPrecision` |
| **Math** | `abs`、`floor`、`ceil`、`round`、`sqrt`、`pow`、`max`、`min`、`sign`、`sin`、`cos`、`tan`、`asin`、`acos`、`atan`、`atan2`、`exp`、`log`、`log2`、`log10`、`random`、`imul`、`clz32`、`fround`、`trunc`，以及 `PI`、`E`、`LN2`、`LN10`、`LOG2E`、`LOG10E`、`SQRT2`、`SQRT1_2` 常量 |
| **JSON** | `parse`、`stringify` |
| **RegExp** | `test`、`exec`、`source`、`flags`、`lastIndex` |
| **Error** | `Error`、`TypeError`、`ReferenceError`、`SyntaxError`、`RangeError`、`EvalError`、`URIError`、`InternalError`（含 `name`、`message`、`stack`、`toString`） |
| **TypedArray** | `Int8Array`、`Uint8Array`、`Uint8ClampedArray`、`Int16Array`、`Uint16Array`、`Int32Array`、`Uint32Array`、`Float32Array`、`Float64Array`（含 `length`、`byteLength`、`BYTES_PER_ELEMENT`、`subarray`） |
| **ArrayBuffer** | `byteLength` |
| **Date** | `now` |
| **Function** | `call`、`apply`、`bind`、`toString` |

### 全局函数

- `parseInt`、`parseFloat`
- `isNaN`、`isFinite`
- `Boolean`、`Number`、`String`（类型转换）
- `print`、`console.log`、`console.error`、`console.warn`
- `gc`（触发垃圾回收）
- `load`（加载并执行 JavaScript 文件）
- `setTimeout`、`clearTimeout`
- `performance.now`
- `globalThis`

## 架构

```
src/
├── lib.rs           # 库入口
├── value.rs         # 标记联合体值表示
├── context.rs       # JavaScript 上下文与求值
├── gc/
│   ├── allocator.rs # 区域分配器
│   └── collector.rs # 标记-压缩 GC
├── vm/
│   ├── opcode.rs    # 字节码操作码（约 80 条）
│   ├── interpreter.rs # 字节码解释器
│   └── stack.rs     # 值栈
├── parser/
│   ├── lexer.rs     # 词法分析器
│   └── compiler.rs  # 解析器与字节码生成器
├── runtime/
│   ├── object.rs    # 对象表示
│   ├── string.rs    # 字符串处理
│   ├── array.rs     # 无空洞语义的数组
│   ├── function.rs  # 函数与闭包类型
│   └── property.rs  # 属性哈希表
├── util/
│   ├── dtoa.rs      # 数字转字符串
│   └── unicode.rs   # UTF-8/UTF-16 处理
└── bin/
    └── mqjs.rs      # CLI/REPL 应用
```

## 字节码格式

MQuickJS-RS 支持将 JavaScript 编译为字节码以加快加载速度：

```bash
# 编译
mqjs -c app.js        # 生成 app.qbc

# 执行编译后的字节码
mqjs app.qbc
```

字节码文件使用 `.qbc` 扩展名，格式如下：
- 魔术字节：`MQJS`
- 版本：1 字节
- 序列化的函数字节码

## 内存模型

值使用标记联合体表示，放入单个机器字：

- **数值**：31 位有符号整数与内联短浮点（基于 `f32`）
- **特殊值**：`null`、`undefined`、`true`、`false`
- **对象**：指向 GC 管理堆对象的指针
- **字符串**：UTF-8 编码，已内化（interned）

垃圾回收器采用标记-压缩算法：
- 对象头比引用计数更小
- 消除内存碎片
- 自动处理循环引用

## 测试

```bash
# 运行全部测试
cargo test

# 显示测试输出
cargo test -- --nocapture
```

目前共有 **312 个测试**，覆盖当前已实现的主要特性。

## MQuickJS 与 QuickJS 的区别

[MQuickJS](https://github.com/bellard/mquickjs) 是 Fabrice Bellard 的**极简** JavaScript 引擎，与他的全功能 [QuickJS](https://bellard.org/quickjs/) 是两个独立项目。MQuickJS-RS 是 MQuickJS 的 Rust 移植版。

| 特性 | QuickJS | MQuickJS / MQuickJS-RS |
|------|---------|------------------------|
| 语言范围 | ES2020+ | 受限 ES5/ES6 风格 Profile |
| 内存模型 | 引用计数 | 追踪式 GC（标记-压缩） |
| Generator | 支持 | 不支持 |
| Async/Await | 支持 | 不支持 |
| ES 模块 | 支持 | 不支持 |
| BigInt | 支持 | 不支持 |
| Proxy | 支持 | 不支持 |
| 目标规模 | ~200KB 二进制 | 面向低内存嵌入式目标 |
| 适用场景 | 通用 | 嵌入式系统 |

## 学习资源

- **[工作原理](docs/HOW_IT_WORKS.md)**（英文）—— 深入讲解 JavaScript 引擎内部机制：词法分析、解析、字节码、虚拟机、垃圾回收、闭包等
- **[性能分析](docs/BENCHMARK_ANALYSIS.md)**（英文）—— 与 C 版本的性能对比及原因分析

## 性能基准

MQuickJS-RS（Rust）与原版 MQuickJS（C）的性能对比。

**测试环境**：Apple M4 Max，64 GB RAM，macOS

| 基准测试 | Rust (s) | C (s) | 比率 | 备注 |
|----------|----------|-------|------|------|
| json | 0.021 | 0.024 | **0.88x** | Rust 快 12% |
| string | 0.016 | 0.016 | 1.01x | 基本持平 |
| closure | 0.016 | 0.016 | 1.02x | 基本持平 |
| object | 0.019 | 0.017 | 1.12x | C 快 12% |
| array | 0.019 | 0.016 | 1.21x | C 快 21% |
| sieve | 0.039 | 0.021 | 1.84x | C 快 84% |
| fib | 0.132 | 0.059 | 2.25x | C 快 2.25 倍 |
| loop | 0.070 | 0.030 | 2.33x | C 快 2.33 倍 |

**小结**：C 版本整体更快，主要得益于手工优化的解释器循环（计算跳转表）和内联缓存。Rust 版本优先保证安全性（边界检查、热路径无 unsafe）和正确性。在 JSON 解析上 Rust 更快（得益于高效的字符串处理），字符串和闭包操作上两者相当。详见[性能分析](docs/BENCHMARK_ANALYSIS.md)。

### 运行基准测试

```bash
# 构建原版 C 实现
git submodule update --init
make -C vendor/mquickjs

# 运行对比测试
./benches/compare.sh

# 运行详细 Rust 基准测试（Criterion）
cargo bench
```

## 许可证

MIT License

## 致谢

- [Fabrice Bellard](https://bellard.org/) —— 原版 MQuickJS C 实现
- **整个 Rust 移植版由 [Claude](https://claude.ai)（Anthropic AI 助手）编写**，使用 [Claude Code](https://claude.ai/claude-code) 基于原版 C 参考实现，自主完成了本仓库当前测试集和 Rust 代码实现。
