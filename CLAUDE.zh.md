# CLAUDE.md

本文件为 Claude Code (claude.ai/code) 在本项目中工作时提供指导。

> **注意**：本文件有中文版本 [CLAUDE.zh.md](CLAUDE.zh.md)

## 项目概述

MQuickJS-RS 是一个**研究项目**—— 是 Fabrice Bellard 的 [MQuickJS](https://github.com/bellard/mquickjs)  minimalist JavaScript 引擎的纯安全 Rust 移植版。它实现了 ES5 子集，包含追踪式标记压缩 GC、基于栈的字节码虚拟机，且没有使用 `unsafe` 代码。
**ESP32/裸机开发的关键约束**：本项目设计为在 ESP32 裸机上以 no_std 模式运行。添加功能或修改代码时，必须始终验证 no_std 兼容性。

## 命令

```bash
# 构建
cargo build
cargo build --release

# 为 ESP32 裸机构建 (no_std)
cargo build --release --no-default-features

# 运行测试 (458 个测试)
cargo test

# 按名称运行单个测试
cargo test test_name

# 显示 stdout 运行测试
cargo test -- --nocapture

# Lint 检查 (CI 强制零警告)
cargo clippy -- -D warnings

# 格式检查
cargo fmt --check
cargo fmt

# 运行 JS REPL/CLI
cargo run --bin mqjs
cargo run --bin mqjs -- script.js
cargo run --bin mqjs -- -e "1 + 2"

# 基准测试 (需要 C 子模块进行对比)
cargo bench
git submodule update --init
./benches/compare.sh

# 带调试特性构建
cargo build --features dump        # 启用字节码/GC 转储
cargo build --features debug-gc    # GC 每次分配都移动对象
```

## 架构

执行流水线：**JS 源码 → 词法分析器 → 解析器/编译器 → 字节码 → 解释器 (VM)**

### 核心模块

- **[src/context.rs](src/context.rs)** — `Context` 是公共 API 入口。拥有 `Heap` (GC) 和 `Interpreter`。调用 `ctx.eval(source)` 运行 JavaScript。

- **[src/value.rs](src/value.rs)** — `Value` 标签联合体：整数（31 位内联）、特殊值（null/undefined/bool），以及字符串、对象、数组、闭包、错误、正则、类型化数组和内置对象的指针索引。所有堆引用都是解释器拥有的 `Vec` 的索引，而不是原始指针。

- **[src/parser/lexer.rs](src/parser/lexer.rs)** — 词法分析器。

- **[src/parser/compiler.rs](src/parser/compiler.rs)** — 递归下降解析器 + 字节码发射器（表达式优先级爬升）。生成 `FunctionBytecode`。

- **[src/vm/opcode.rs](src/vm/opcode.rs)** — 约 80 个操作码。

- **[src/vm/interpreter.rs](src/vm/interpreter.rs)** — `Interpreter` 结构体、字节码分发循环 (`run`)、算术和比较运算符。拥有所有堆分配的 JS 对象作为按 `Value` 标签索引的 `Vec`。

- **[src/vm/property.rs](src/vm/property.rs)** — `get_*_property()` 方法，分发数组、字符串、数字、错误、正则、类型化数组和内置对象的属性访问。

- **[src/vm/natives.rs](src/vm/natives.rs)** — 约 100 个原生函数实现（Array/String/Math/JSON/RegExp/Date 方法，`format_value`，JSON 解析器等）。

- **[src/vm/stack.rs](src/vm/stack.rs)** — 包含调用帧的值栈。

- **[src/gc/](src/gc/)** — 竞技场分配器 (`allocator.rs`) 和标记压缩收集器 (`collector.rs`)。GC 基本上是个存根；实际对象生命周期通过解释器中的 `Vec`-对象模式管理。

- **[src/runtime/](src/runtime/)** — 解释器使用的对象、字符串、数组、函数和属性表类型。

- **[src/builtins/](src/builtins/)** — 内置方法实现（Array, String, Math, JSON, RegExp, Error, TypedArray 等）。

- **[src/bin/mqjs.rs](src/bin/mqjs.rs)** — 使用 `rustyline` 的 CLI/REPL。支持 `-e`, `-i`, `-I`, `-d`, `-c`, `--memory-limit`。

### 值编码

堆对象类型使用 `Value` u64 中的标记位编码：
- `BUILTIN_OBJECT_MARKER` → 内置全局对象（Math, JSON, console 等）按 `BUILTIN_*` 索引
- `ERROR_OBJECT_MARKER` (第 20 位) → `ErrorObject` 按解释器 `error_objects` 索引
- `REGEXP_OBJECT_MARKER` (第 19 位) → `RegExpObject` 按索引
- `TYPED_ARRAY_MARKER` (第 18 位) → `TypedArrayObject` 按索引

### 方法分发

对象属性/方法访问通过解释器中的 `get_*_property()` 辅助函数（如 `get_array_property`, `get_string_property`, `get_builtin_property`）分发。`GetField2` + `CallMethod` 操作码对用于方法调用语法 (`obj.method()`)，将 `this` 保留在栈上。

### 测试

测试内联在每个源文件（`#[cfg(test)]` 模块）中。458 个测试覆盖完整的语言特性和所有内置方法。

## 项目结构

这是一个 Cargo workspace，包含两个成员：

- **mquickjs-rs**（根目录）— 核心 JavaScript 引擎 (`src/`)
- **led-runtime** (`led-runtime/`) — 基于 mquickjs-rs 构建的 LED 效果运行时

## 约定

- **CI 必须通过** — 在认为工作完成前，运行 `cargo clippy -- -D warnings` 和 `cargo fmt --check`。
- **JS 脚本** 放在 `js/examples/`（功能演示）和 `js/tests/`（错误处理测试）。按 Cargo 惯例，顶层 `examples/` 保留给 Rust 示例。
- 新操作码添加到 [src/vm/opcode.rs](src/vm/opcode.rs)，处理程序添加到解释器主 `match` 中（见 [src/vm/interpreter.rs](src/vm/interpreter.rs)）。
- 新内置方法添加到 `src/builtins/<object>.rs`，通过解释器中的 `get_*_property()` 接入。
- **ESP32 需要 `no_std`** — 项目必须能够在没有 `std` 的情况下编译。添加依赖时，验证它们是否有 `no_std` 支持，或在 `Cargo.toml` 中添加 `default-features = ["std"]` 并使用 `#[cfg(feature = "std")]` 条件编译。
- **内存受限设计** — ESP32 的 RAM 有限（通常 320-520KB）。优先使用内联分配，在热路径中避免动态分配，使用标记值来减少堆使用。
- **交叉编译目标** — 对于 ESP32，使用目标：`riscv32imac-unknown-none-elf`。如果尚未安装，运行 `rustup target add riscv32imac-unknown-none-elf`。
