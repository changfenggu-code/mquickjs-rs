# MQuickJS-RS `no_std` / ESP32 裸板集成说明

本文档说明如何将 `mquickjs-rs` 作为库嵌入到 `no_std` 的 ESP32 裸板工程中。

## 1. 适用范围

本文档面向：

- `no_std` + `alloc` 的嵌入式 Rust 工程
- ESP32 裸板 / 无文件系统 / 无标准输入输出场景
- 以 LED effect 脚本执行为目标的宿主集成

不适用于：

- 直接运行仓库自带的 `mqjs` CLI
- 依赖 `std::fs`、`std::time`、`std::env` 的桌面运行方式

## 2. 这个库在 `no_std` 下是什么意思

这里的意思不是“直接运行整个仓库”，而是：

- 你的 ESP32 固件工程依赖 `mquickjs-rs`
- 关闭默认 `std` feature
- 只把库核心编进你的固件

也就是说，使用方式是：

- 你自己的 `esp32-firmware` 是主工程
- `mquickjs-rs` 是其中一个依赖库
- 由你的宿主工程提供内存分配、panic、时间、日志、硬件访问能力

## 3. 最小依赖配置

在你的 ESP32 工程 `Cargo.toml` 里加入：

```toml
[dependencies]
mquickjs = { package = "mquickjs-rs", path = "../mquickjs-rs", default-features = false }
```

关键点：

- `package = "mquickjs-rs"`：源包名
- `mquickjs`：你在代码里使用的依赖名
- `default-features = false`：关闭默认 `std`

如果你不是 path 依赖，也可以换成 git / registry 形式，但核心点仍然是：

- **必须关闭默认 feature**

## 4. 宿主工程需要自己提供什么

`mquickjs-rs` 在 `no_std` 下依赖 `alloc`，因此你的裸板工程需要提供：

- 全局分配器 `#[global_allocator]`
- `alloc` 可用环境
- `#[panic_handler]`
- 板级启动代码 / HAL / runtime

对于 ESP32 裸板，这通常意味着你已经有自己的：

- heap 初始化
- panic 输出策略
- 时钟初始化
- 外设初始化

## 5. `no_std` 下建议使用的能力

对于 LED effect / ESP32 裸板场景，建议优先使用以下路径：

- 宿主侧离线生成字节码
- 固件内嵌或加载字节码
- 使用 `Context::execute()` 执行字节码
- 通过 `Uint8Array` 获取 LED buffer
- 通过 native 函数把硬件能力暴露给脚本

推荐 API 组合：

- `Context::new(mem_size)`
- `Context::execute(&bytecode)`
- `Context::register_native(...)`
- 宿主读取 TypedArray buffer

## 6. `no_std` 下不要依赖什么

以下能力在裸板模式下应避免依赖：

- `mqjs` CLI
- `load(filename)`
- `clearTimeout()`
- `setTimeout()`
- `Date.now()`
- `performance.now()`
- `console.log/error/warn`
- `print expr;`
- `RegExp.prototype.test/exec`
- `String.prototype.match/search` 的正则路径

原因不是“语法不存在”，而是这些能力依赖：

- 文件系统
- 系统时间
- 标准输出
- `regex` / `std`

## 7. 最小宿主集成思路

### 方案 A：设备端只执行预编译字节码

这是最推荐的方式。

流程：

1. 在 PC 侧把 JS 编译为 `.qbc`
2. 将字节码作为资源嵌入固件或写入 Flash
3. 启动时反序列化字节码
4. 在主循环中驱动执行
5. 从 `Uint8Array` 读取 LED 输出帧

优点：

- 设备端逻辑更简单
- 避免设备端源码编译成本
- 更容易做发布与版本控制

### 方案 B：设备端直接 `eval()` 源码

理论上可行，但不推荐作为产品路径。

原因：

- 占用更多设备资源
- 更难做脚本发布治理
- 对产品化一致性不如字节码方式稳定

## 8. 最小代码示意

```rust
#![no_std]
extern crate alloc;

use mquickjs::Context;

fn run() {
    let mut ctx = Context::new(64 * 1024);
    let _ = ctx.eval("1 + 2");
}
```

如果走产品化路径，更建议替换成：

```rust
#![no_std]
extern crate alloc;

use mquickjs::{Context, FunctionBytecode};

fn run_precompiled(bytecode: &FunctionBytecode) {
    let mut ctx = Context::new(64 * 1024);
    let _ = ctx.execute(bytecode);
}
```

## 9. ESP32 裸板场景的实际建议

如果目标是 ESP32 LED effect 运行时，建议你们的边界这样划分：

- PC / 工具链：源码校验、Profile 约束、编译字节码
- ESP32 固件：加载字节码、驱动脚本、输出 LED buffer
- 宿主适配层：时间、配置、LED 驱动、日志、重启恢复

也就是说，把 `mquickjs-rs` 当成：

- **脚本执行内核**

而不是：

- 完整的设备应用框架

## 10. 当前已知限制

### ✅ 已验证（2026-03-14）

- **no_std 编译验证通过**：`cargo build --release --no-default-features` 成功
- **测试全部通过**：`cargo test --no-default-features --lib` 109/109 测试通过
- **库大小**：`libmquickjs.rlib` ~2.5MB（release 模式）
- **API 完整性**：EffectEngine、EffectManager、ConfigValue 已支持结构化配置

### ✅ 交叉编译验证（2026-03-14）

- **x86_64（Windows）**：编译成功，库大小 2.5MB
- **riscv32imac（RISC-V）**：编译成功，库大小 2.4MB
- **编译时间**：x86_64 用时 3.66s，RISC-V 用时 3.86s

### ⚠️ ESP32（Xtensa）交叉编译限制

- **标准 stable Rust 工具链不支持 `xtensa-esp32-none-elf` 目标**
- 需要使用 ESP-IDF / esp-rs 工具链
- 参考：
  - https://github.com/esp-rs/esp
  - https://docs.esp-rs.com/book/
- **注意**：当前验证仅确认 `no_std` 兼容性，ESP32 实际编译需要特殊工具链

### 待完善的产品化功能

若你们要做真正的 ESP32 裸板产品，还需要关注：

- 执行预算 / watchdog
- 更稳定的宿主 effect API（EffectManager 调度语义完善）
- 更严格的内存计量
- 更完整的资源上限控制

这些是产品化层面的工作，不是 `no_std` 编译本身的阻塞。
