# 基准分析：Rust vs C 实现

本文档解释 MQuickJS-RS (Rust) 与原始 MQuickJS (C) 之间的性能差异。

## 运行基准测试

### 本地运行

```bash
# 方式1: 只测 Rust 版
./benches/compare.sh

# 方式2: 自动检测 C 版对比 (需要初始化 submodule)
git submodule update --init
./benches/compare.sh

# 方式3: 指定 C 二进制路径
./benches/compare.sh /path/to/mqjs
```

**注意**：首次运行会编译 Rust release 版本，需要一些时间。

### GitHub Actions

提交到 `main` 分支或提交 PR 时会自动触发 benchmark workflow：
- `.github/workflows/bench.yml`

结果会在 Actions 页面显示。

## 基准结果

**机器 1**：Apple M4 Max, 64 GB RAM, macOS (参考)

| 基准 | Rust (s) | C (s) | 比率 | 获胜者 |
|-----------|----------|-------|-------|--------|
| json | 0.021 | 0.024 | 0.88x | Rust 快 12% |
| string | 0.016 | 0.016 | 1.01x | 持平 |
| closure | 0.016 | 0.016 | 1.02x | 持平 |
| object | 0.019 | 0.017 | 1.12x | C 快 12% |
| array | 0.019 | 0.016 | 1.21x | C 快 21% |
| sieve | 0.039 | 0.021 | 1.84x | C 快 84% |
| fib | 0.132 | 0.059 | 2.25x | C 快 2.25 倍 |
| loop | 0.070 | 0.030 | 2.33x | C 快 2.33 倍 |

**机器 2**：AMD Ryzen 7 5800H, 32 GB RAM, Windows 11 (Rust only)

| 基准 | Rust (s) | 备注 |
|-----------|----------|------|
| fib | 0.36 | 最慢 - 递归函数调用 |
| loop | 0.19 | 循环性能 |
| sieve | 0.16 | 素数筛法 |
| json | 0.14 | JSON 解析 |
| array | 0.14 | 数组操作 |
| closure | 0.14 | 闭包 |
| object | 0.14 | 对象操作 |
| string | 0.14 | 字符串操作 |

**注意**：由于 Windows 环境下编译 C 版较为复杂，当前主要关注 Rust 版的绝对性能。有条件时可使用 `git submodule update --init` 初始化 C 版进行对比。

## 为什么 C 通常更快

### 1. 计算跳转 vs 匹配语句

**C 实现**使用计算跳转（GCC 扩展）：
```c
#define CASE(op)  op_label:
#define NEXT      goto *dispatch_table[*pc++]

static void *dispatch_table[] = {
    &&op_push_i32, &&op_push_const, ...
};

// 直接跳转到下一个操作码
CASE(OP_add):
    // ... 加法代码
    NEXT;
```

**Rust 实现**使用匹配语句：
```rust
loop {
    let opcode = bc[frame.pc];
    frame.pc += 1;

    match opcode {
        op if op == OpCode::Push0 as u8 => { ... }
        op if op == OpCode::Add as u8 => { ... }
        // 还有 80 多个分支
    }
}
```

**影响**：计算跳转消除了中央 switch/match 的分发开销。每个操作码处理器直接跳转到下一个处理器，而无需返回到分发循环。这为每个操作码节省约 2-3 条指令。

### 2. 内联缓存和短路路径

C 版本对属性查找使用激进的内存内联缓存：
```c
// C: 带缓存形状的快速路径
if (likely(prop_cache->shape == obj->shape)) {
    return obj->props[prop_cache->slot];
}
// 慢速路径
```

Rust 版本每次都进行完整属性查找：
```rust
// Rust: 每次都完整查找
self.objects[obj_idx].properties.get(&key)
```

### 3. 标记值表示

两者都使用标记值，但 C 版本有更优化的标记方式：

**C** - 32 位值，使用 NaN boxing 或指针标记：
```c
// 短整数：适合 31 位，无分配
// 短浮点数：适合 IEEE-754 静默 NaN payload
typedef uint32_t JSValue;
```

**Rust** - 64 位值，使用更直接的标记值编码：
```rust
// 所有值都是 64 位，包含 int / ptr / special / short-float 标签
struct RawValue(u64);
```

C 版本的紧凑表示提高了缓存效率；Rust 版本则用更清晰的 tagged-value 模型换取实现简单性与可维护性。

## 为什么 Rust 在 `json` 上更快 (12%)

### 高效的字符串处理

Rust 的 `serde_json`（概念上类似于我们的解析器方法）高效地处理 JSON 解析：

```rust
// Rust: 尽可能零拷贝字符串解析
let s: &str = ...;  // 借用切片，无分配

// 高效的字符串构建
let mut s = String::with_capacity(estimated_len);
```

C 版本必须手动管理字符串内存，可能有更多的分配。

## 为什么 C 在 `loop` (2.3x) 和 `fib` (2.25x) 上快得多

### Loop 基准

`loop` 基准运行 100 万次简单算术运算：
```javascript
for (var i = 0; i < 1000000; i = i + 1) {
    sum = (sum + i) % mod;
}
```

**C 更快的原因：**
1. **更紧凑的分发循环**：计算跳转消除了匹配开销
2. **更好的分支预测**：直接跳转有可预测的模式
3. **更小的代码**：C 操作码处理器更紧凑，I-cache 使用更好

### Fib 基准

`fib` 基准进行递归函数调用：
```javascript
function fib(n) {
    if (n <= 1) return n;
    return fib(n-1) + fib(n-2);
}
fib(30);
```

**C 更快的原因：**
1. **优化的调用/返回**：C 版本有手工调优的函数调用代码路径
2. **更小的调用帧**：C 使用紧凑的 8 字节调用帧
3. **寄存器分配**：C 编译器可以在寄存器中保持更多值

**注意**：Rust 版本使用无栈解释器设计（堆分配的调用帧），这可以处理深度递归而不会栈溢出。这以一些性能换取了深度嵌套调用的正确性。

## 为什么 C 在 `sieve` (1.84x) 和 `array` (1.21x) 上更快

### 数组访问模式

两个基准都是数组密集型：

**C** - 直接指针算术：
```c
// 无边界检查，直接内存访问
val = arr->values[i];
arr->values[i] = val;
```

**Rust** - 带边界检查的安全访问：
```rust
// 每次访问都有边界检查
let val = self.arrays[arr_idx].get(i)?;
self.arrays[arr_idx].set(i, val)?;
```

即使在热路径中使用 `unsafe` 优化，Rust 仍然有更多的间接性：
```rust
// Rust 优化路径仍涉及更多步骤：
// 1. 从解释器获取数组引用
// 2. 检查数组类型
// 3. 访问底层 Vec
// 4. 获取/设置元素
```

### 方法调用开销

JavaScript 中的每次 `array.push()` 调用需要：
1. 查找 "push" 属性
2. 函数调用设置
3. 原生函数分发

C 版本使用专用操作码优化常见的数组方法，而 Rust 使用通用属性查找。

## 总结

| 类别 | 获胜者 | 原因 |
|----------|--------|--------|
| **JSON 解析** | Rust | 高效的字符串处理 |
| **字符串/闭包操作** | 持平 | 类似的实现策略 |
| **对象访问** | C | 内联缓存，更小的对象 |
| **数组操作** | C | 直接指针算术，无边界检查 |
| **循环** | C | 计算跳转，更紧凑的分发 |
| **递归** | C | 优化的调用/返回路径 |

## 设计权衡

Rust 实现优先考虑：
- **安全性**：边界检查，无未定义行为
- **正确性**：处理边缘情况（深度递归、大值）
- **可维护性**：清晰、地道的 Rust 代码
- **工程可读性**：文档完善，便于调试、扩展与宿主集成

C 实现优先考虑：
- **性能**：嵌入式系统中每个周期都很重要
- **内存效率**：受限设备的最小内存占用
- **兼容性**：已在许多平台上验证

## 潜在的进一步优化

1. **计算跳转等效方案**：使用 `#[cold]` 和配置文件引导优化
2. **内联缓存**：添加基于形状的属性缓存
3. **基于寄存器的 VM**：从基于栈转换为基于寄存器的字节码
4. **Unsafe 热路径**：在解释器循环中更激进地使用 unsafe
5. **配置文件引导优化**：使用 PGO 优化分发模式

## Stage 9: 优化计划

### 当前优先级

| 优化项 | 预期收益 | 复杂度 | 状态 |
|--------|----------|--------|------|
| 减少 match 分支开销 | 中 | 低 | 待评估 |
| 函数调用优化 (fib/loop) | 高 | 中 | 待评估 |
| 属性访问优化 | 中 | 中 | 待评估 |
| GC 优化 | 低 | 高 | 待评估 |

### 快速优化方向

1. **热点代码内联**：使用 `#[inline]` 标记频繁调用的函数
2. **减少堆分配**：在栈上缓存临时值
3. **优化 dispatch**：使用函数指针数组代替 match

### 评估方法

```bash
# 运行基准测试
./benches/compare.sh

# 运行 Criterion 微基准
cargo bench

# 查看生成的代码
cargo show --lib -Z timings
```
