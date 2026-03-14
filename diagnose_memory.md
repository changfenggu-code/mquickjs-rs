# Memory Stats 诊断

## 当前实现分析

### Heap 结构
- buffer: Vec<u8> - 总内存缓冲
- heap_ptr: usize - 当前堆分配位置（从 0 开始增长）
- stack_ptr: usize - 当前栈指针（从 total_size 向下生长）

### 统计问题

#### 1. heap_used() 返回 heap_ptr
问题：返回的是分配边界位置，不是实际占用大小
- heap_ptr = 已分配的字节数（包括头部和对象数据）
- 但是释放时 heap_ptr 不会减少（GC 未实现）

#### 2. used 字段计算
当前：used = heap.heap_used() = heap_ptr
期望：used = 实际对象占用字节数（包括所有类型）

#### 3. 对象计数没有转换为字节数
当前只统计数量：
- runtime_strings: usize - 数量
- arrays: usize - 数量
- objects: usize - 数量
- closures: usize - 数量
- typed_arrays: usize - 数量

缺少：每个类型的实际字节占用

## 建议修复

### 短期：改进统计口径
- used 字段应该考虑对象实际大小
- 追加对象类型的字节占用统计

### 长期：实现完整 GC
- 自动回收未使用对象
- 释放时更新 heap_ptr
- 精确计算对象占用
