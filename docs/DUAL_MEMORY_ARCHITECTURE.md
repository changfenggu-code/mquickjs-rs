# 双内存系统架构

本文档说明 MQuickJS-RS 中的双内存系统架构：Vec 索引存储（活跃）与 Arena 分配器（预留）。

## 架构概述

MQuickJS-RS 使用了独特的双内存架构来平衡性能和内存管理：

### 1. Vec 索引存储（主要系统）

**位置**: `src/vm/` - 所有解释器容器
**状态**: **活跃系统**
**用途**: JavaScript 对象的实际存储

**核心组件**:
- `Interpreter` 结构体包含 11 个主要 Vec 容器
- 每个容器都有对应的 generation 数组进行 GC 管理
- 使用索引而非指针进行引用（内存安全）

**容器列表**:
```rust
closures: Vec<Closure>           // 闭包
var_cells: Vec<VarCell>           // 变量单元格
arrays: Vec<Array>                // 数组
objects: Vec<Object>              // 对象
for_in_iterators: Vec<ForInIterator>  // for-in 迭代器
for_of_iterators: Vec<ForOfIterator>  // for-of 迭代器
error_objects: Vec<ErrorObject>    // 错误对象
regex_objects: Vec<RegExpObject>  // 正则表达式对象
typed_arrays: Vec<TypedArrayObject>  // 类型化数组
array_buffers: Vec<ArrayBuffer>    // 数组缓冲区
timers: Vec<Timer>                // 定时器
```

**GC 集成**:
- 每个容器都有对应的 `gen_*: Vec<u32>` 数组
- `gen[i] == gc_phase` 表示槽位 i 存活
- `gen[i] == u32::MAX` 表示槽位 i 已空闲

### 2. Arena 分配器（预留系统）

**位置**: `src/gc/allocator.rs`
**状态**: **预留系统 - 当前仅用于统计**
**用途**: 为 Plan C（标记压缩 GC）做准备

**核心组件**:
- `Heap` 结构体实现 arena 分配器
- `BlockHeader` 管理内存块
- `MemoryTag` 标记内存用途
- 目前仅用于内存统计，不参与实际分配

## 双系统关系

### 当前状态（Plan B 活跃）

```
JavaScript 对象
    ↓
Vec 索引存储（实际存储）
    ↓
Plan B GC（generation-based mark-sweep）
    ↓
内存回收和重用
```

### 未来计划（Plan C 预留）

```
JavaScript 对象
    ↓
Arena 分配器（标记压缩）
    ↓
Plan C GC（mark-compact）
    ↓
内存整理和压缩
```

## 设计优势

### 1. 内存安全
- 所有引用都是索引，不是原始指针
- 避免了悬垂指针和内存安全问题
- 适合 no_std 嵌入式环境

### 2. 性能优化
- Vec 提供连续内存访问
- 索引访问比指针解引用更高效
- 缓存友好的数据布局

### 3. GC 灵活性
- Plan B 已实现并活跃运行
- Plan C 预留 arena 空间用于未来优化
- 可以平滑迁移到标记压缩算法

### 4. 嵌入式友好
- no_std 兼容，适合 ESP32
- 固定的内存访问模式
- 可预测的内存使用

## 迁移路径

### Plan B → Plan C（未来计划）

1. **当前状态**: Vec 索引 + generation 标记
2. **Plan C 过渡**: 添加 arena 分配器接口
3. **最终状态**: arena 存储 + 标记压缩

### 兼容性保证

- 所有 JavaScript API 保持不变
- 解释器接口保持稳定
- 内存统计功能继续工作
- 测试套件继续通过

## 使用示例

### 获取内存统计

```rust
let ctx = Context::new();
let stats = ctx.memory_stats();
// 显示 Vec 存储和 arena 统计
```

### 手动触发 GC

```rust
let ctx = Context::new();
ctx.eval("let arr = [1, 2, 3]; arr = null;")?;
ctx.gc();  // 触发 Plan B GC
```

## 注意事项

1. **不要删除 `src/gc/`** - 这是 Plan C 的基础设施
2. **Vec 索引存储是核心** - 所有对象都通过索引访问
3. **Arena 预留空间** - 为未来的内存压缩做准备
4. **保持兼容性** - 公共 API 不应改变

## 总结

双内存系统架构让 MQuickJS-RS 能够：
- 使用高效的 Vec 索引存储（Plan B）
- 为未来的标记压缩预留空间（Plan C）
- 保持内存安全和嵌入式兼容性
- 平衡当前性能和未来优化