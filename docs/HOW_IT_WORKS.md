# JavaScript 引擎工作原理

深入了解 MQuickJS-RS 的内部实现，适用于对编程语言实现感兴趣的学习者。

## 目录

1. [概述](#概述)
2. [执行流水线](#执行流水线)
3. [词法分析器](#词法分析器tokenizer)
4. [解析器与编译器](#解析器与编译器)
5. [字节码](#字节码)
6. [虚拟机](#虚拟机)
7. [值表示](#值表示)
8. [垃圾回收](#垃圾回收)
9. [内置对象](#内置对象)
10. [闭包](#闭包)
11. [异常处理](#异常处理)

---

## 概述

JavaScript 引擎将人类可读的 JavaScript 代码转换为计算机可以执行的形式。这个过程涉及多个阶段：

```
源代码 → 词法分析器 → Token → 解析/编译器 → 字节码 → 虚拟机 → 结果
```

MQuickJS 通过将解析和编译合并为单次遍历（不显式构建 AST）来简化这一过程，从而减少内存使用——这对嵌入式系统至关重要。

---

## 执行流水线

### 执行 `1 + 2 * 3` 时会发生什么？

```javascript
// 输入
1 + 2 * 3

// 第 1 步：词法分析器生成 Token
[NUMBER(1), PLUS, NUMBER(2), STAR, NUMBER(3)]

// 第 2 步：解析器/编译器生成字节码
push_i8 1      // 将 1 压入栈
push_i8 2      // 将 2 压入栈
push_i8 3      // 将 3 压入栈
mul            // 弹出 2,3 → 压入 6
add            // 弹出 1,6 → 压入 7
return         // 返回栈顶值 (7)

// 第 3 步：虚拟机执行字节码
栈: [] → [1] → [1,2] → [1,2,3] → [1,6] → [7]
结果: 7
```

---

## 词法分析器 (Tokenizer)

词法分析器（`src/parser/lexer.rs`）将源代码分解为 token —— 最小的有意义单元。

### Token 类型

```rust
pub enum Token {
    // 字面量
    Number(f64),           // 42, 3.14
    String(String),        // "hello"
    Identifier(String),    // foo, myVar

    // 关键字
    Var, Let, Const,       // 变量声明
    Function, Return,      // 函数
    If, Else, While, For,  // 控制流
    True, False, Null,     // 字面量

    // 运算符
    Plus, Minus, Star, Slash,  // + - * /
    Equal, EqualEqual,         // = ==
    Less, Greater,             // < >

    // 标点符号
    LeftParen, RightParen,     // ( )
    LeftBrace, RightBrace,     // { }
    LeftBracket, RightBracket, // [ ]
    Semicolon, Comma, Dot,     // ; , .
}
```

### 词法分析如何工作

```rust
// 简化的词法分析逻辑
fn next_token(&mut self) -> Token {
    self.skip_whitespace();

    match self.current_char() {
        '0'..='9' => self.read_number(),
        'a'..='z' | 'A'..='Z' | '_' => self.read_identifier(),
        '"' | '\'' => self.read_string(),
        '+' => Token::Plus,
        '-' => Token::Minus,
        // ... 等等
    }
}
```

### 示例

```javascript
var x = 10 + 20;
```

生成：
```
VAR, IDENTIFIER("x"), EQUAL, NUMBER(10), PLUS, NUMBER(20), SEMICOLON
```

---

## 解析器与编译器

解析器（`src/parser/compiler.rs`）读取 token 并生成字节码。MQuickJS 使用**单次遍历编译器** —— 它同时进行解析和字节码生成，而不构建 AST。

### 运算符优先级

对于像 `1 + 2 * 3` 这样的表达式，我们需要处理运算符优先级（乘法先于加法）。

MQuickJS 使用 **Pratt 解析**（优先级爬升）：

```rust
// 优先级级别（越高 = 结合越紧密）
fn precedence(op: &Token) -> u8 {
    match op {
        Token::Or => 1,              // ||
        Token::And => 2,             // &&
        Token::EqualEqual => 3,      // ==
        Token::Less | Token::Greater => 4,  // < >
        Token::Plus | Token::Minus => 5,    // + -
        Token::Star | Token::Slash => 6,    // * /
        Token::StarStar => 7,        // ** (右结合)
        _ => 0,
    }
}

// 解析具有最小优先级的表达式
fn parse_expression(&mut self, min_prec: u8) {
    // 解析左操作数（原子：数字、变量、括号表达式）
    self.parse_atom();

    // 当下一个运算符具有更高优先级时，继续
    while precedence(self.current()) >= min_prec {
        let op = self.advance();
        // 用更高优先级解析右侧
        self.parse_expression(precedence(&op) + 1);
        // 发出运算符字节码
        self.emit_binary_op(op);
    }
}
```

### 解析语句

```rust
fn parse_statement(&mut self) {
    match self.current() {
        Token::Var => self.parse_var_declaration(),
        Token::If => self.parse_if_statement(),
        Token::While => self.parse_while_loop(),
        Token::For => self.parse_for_loop(),
        Token::Function => self.parse_function_declaration(),
        Token::Return => self.parse_return(),
        Token::LeftBrace => self.parse_block(),
        _ => self.parse_expression_statement(),
    }
}
```

### 控制流编译

对于 `if/else`，我们需要**跳转指令**：

```javascript
if (x > 0) {
    print("positive");
} else {
    print("non-positive");
}
```

编译为：
```
get_local x           // 压入 x
push_i8 0             // 压入 0
greater               // x > 0 → true/false
if_false [ELSE_ADDR]  // 如果为 false 跳转到 else
push_string "positive"
call print
goto [END_ADDR]       // 跳过 else 块
[ELSE_ADDR]:
push_string "non-positive"
call print
[END_ADDR]:
```

编译器使用**回填**：发出占位符跳转地址，然后在知道目标时填入。

---

## 字节码

字节码是程序的紧凑、高效表示。每条指令为 1-3 字节。

### 指令格式

```
[opcode: 1 字节] [operand: 0-2 字节]
```

### 核心操作码（`src/vm/opcode.rs`）

```rust
pub enum OpCode {
    // 栈操作
    Push0, Push1, Push2, ..., Push7,  // 压入小整数（0 字节）
    PushI8(i8),                        // 压入有符号字节（1 字节）
    PushI16(i16),                      // 压入有符号短整（2 字节）
    PushConst(u16),                    // 从常量池压入
    PushUndefined, PushNull, PushTrue, PushFalse,

    Pop,                               // 丢弃栈顶
    Dup,                               // 复制栈顶
    Swap,                              // 交换栈顶两个

    // 算术运算
    Add, Sub, Mul, Div, Mod,
    Neg,                               // 一元负号

    // 比较运算
    Lt, Le, Gt, Ge, Eq, Ne, StrictEq, StrictNe,

    // 逻辑运算
    Not,                               // !x

    // 位运算
    BitAnd, BitOr, BitXor, BitNot,
    Shl, Sar, Shr,                     // 移位

    // 变量
    GetLocal(u8),                      // 获取局部变量
    SetLocal(u8),                      // 设置局部变量
    GetGlobal(u16),                    // 按名称获取全局变量

    // 控制流
    Goto(i16),                         // 无条件跳转
    IfFalse(i16),                      // 如果栈顶为假值则跳转
    IfTrue(i16),                       // 如果栈顶为真值则跳转

    // 函数
    Call(u8),                          // 调用 N 个参数
    Return,                            // 从函数返回

    // 对象
    GetField(u16),                     // obj.property
    PutField(u16),                     // obj.property = value
    GetArrayEl,                        // arr[index]
    PutArrayEl,                        // arr[index] = value
}
```

### 字节码示例

```javascript
function add(a, b) {
    return a + b;
}
add(3, 4);
```

```
# 函数 'add' 字节码：
00: get_local 0      # 压入参数 'a'
02: get_local 1      # 压入参数 'b'
04: add              # a + b
05: return           # 返回结果

# 主代码：
00: push_i8 3        # 压入参数 3
02: push_i8 4        # 压入参数 4
04: call 2           # 调用 2 个参数
06: return
```

---

## 虚拟机

虚拟机（`src/vm/interpreter.rs`）使用**基于栈的架构**执行字节码。

### 栈机基础

与寄存器机（如 x86）不同，栈机使用操作数栈：

```
操作: 3 + 4 * 2

push 3       栈: [3]
push 4       栈: [3, 4]
push 2       栈: [3, 4, 2]
mul          栈: [3, 8]      (4 * 2 = 8)
add          栈: [11]        (3 + 8 = 11)
```

### 解释器循环

```rust
fn execute(&mut self, bytecode: &[u8]) -> Result<Value, Error> {
    let mut pc = 0;  // 程序计数器

    loop {
        let opcode = bytecode[pc];
        pc += 1;

        match opcode {
            OP_PUSH_I8 => {
                let value = bytecode[pc] as i8;
                pc += 1;
                self.stack.push(Value::int(value as i32));
            }

            OP_ADD => {
                let b = self.stack.pop();
                let a = self.stack.pop();
                self.stack.push(a + b);
            }

            OP_GET_LOCAL => {
                let index = bytecode[pc];
                pc += 1;
                let value = self.get_local(index);
                self.stack.push(value);
            }

            OP_IF_FALSE => {
                let offset = read_i16(&bytecode[pc..]);
                pc += 2;
                if self.stack.pop().is_falsy() {
                    pc = (pc as i32 + offset as i32) as usize;
                }
            }

            OP_CALL => {
                let argc = bytecode[pc];
                pc += 1;
                let func = self.stack.pop();
                let args = self.stack.pop_n(argc);
                self.call_function(func, args)?;
            }

            OP_RETURN => {
                let result = self.stack.pop();
                return Ok(result);
            }

            // ... 还有 ~80 个操作码
        }
    }
}
```

### 调用栈

对于函数调用，我们维护一个**调用栈**帧：

```rust
struct CallFrame {
    return_pc: usize,      // 返回到哪里
    base_pointer: usize,   // 栈上局部变量的起始位置
    function: FunctionRef, // 当前函数
}
```

```
# 调用 add(3, 4):

主栈:  [... | 3 | 4]
                   ↑ base_pointer

调用 'add':
- 保存返回地址
- 设置 base_pointer 指向参数
- 执行 'add' 字节码
- 弹出帧，恢复状态
```

---

## 值表示

JavaScript 具有动态类型。每个值必须在运行时携带其类型信息。

### 标记值（`src/value.rs`）

MQuickJS 使用**标记值（tagged value）**来编码类型信息：在一个 64 位字中同时保存类型标签与数据。

```rust
// 64 位值表示（简化版）
// 最低位为 0      -> 31 位整数（内联）
// 低 3 位为 001   -> 堆对象引用
// 低 2/3 位为 011 -> 特殊值（null / undefined / bool / exception ...）
// 低 3 位为 101   -> 内联短浮点（f32）

pub struct RawValue(u64);
pub struct Value(RawValue);
```

当前实现不是“纯整数模型”，而是：

1. **小整数内联** —— 常见整数无需分配
2. **特殊值打标** —— `null` / `undefined` / `bool` 等直接编码在值里
3. **对象走堆引用** —— 字符串、数组、对象等通过标记引用表示
4. **短浮点内联** —— 非整数数值可用 `f32` 形式直接编码

这让运行时既保留了紧凑表示，又支持 `NaN`、`Infinity` 和非整数计算。

### 为什么使用标记值？

1. **小整数是免费的** —— 无内存分配
2. **单字大小** —— 适合寄存器，缓存友好
3. **类型检查快速** —— 只需检查位

### 堆对象

较大的值（字符串、数组、对象）位于堆上：

```rust
// 堆分配的字符串
struct JSString {
    header: GcHeader,  // 用于垃圾回收器
    length: u32,
    data: [u8],        // UTF-8 字节
}

// 堆分配的对象
struct JSObject {
    header: GcHeader,
    properties: HashMap<String, Value>,
}
```

---

## 垃圾回收

JavaScript 自动管理内存。MQuickJS 使用**标记-压缩**收集。

### 为什么使用标记-压缩？

| 方法 | 优点 | 缺点 |
|------|------|------|
| 引用计数 | 立即清理 | 循环泄漏，每次写入有开销 |
| 标记-清除 | 处理循环 | 碎片化 |
| **标记-压缩** | 无碎片化，处理循环 | 暂停时间 |

### 工作原理

```
1. 标记阶段：找到所有可达对象
   - 从"根"开始（栈、全局变量）
   - 递归标记所有可达内容

2. 压缩阶段：将存活对象移到一起
   - 滑动对象以消除间隙
   - 更新所有指针
```

### 示例

```
GC 前：
[A][垃圾][B][垃圾][C][垃圾]

标记后：A, B, C 存活

压缩后：
[A][B][C][空闲空间...]
```

### 算法（`src/gc/collector.rs`）

```rust
fn collect(heap: &mut Heap) {
    // 标记阶段
    for root in get_roots() {
        mark(root);
    }

    // 压缩阶段
    let mut write_ptr = heap.start;
    for obj in heap.objects() {
        if obj.is_marked() {
            // 移动对象到 write_ptr
            if write_ptr != obj.address() {
                copy(obj, write_ptr);
                update_references(obj.address(), write_ptr);
            }
            write_ptr += obj.size();
            obj.unmark();
        }
    }
    heap.free_ptr = write_ptr;
}

fn mark(obj: &Object) {
    if obj.is_marked() { return; }  // 已访问
    obj.set_marked();

    // 递归标记子对象
    for child in obj.references() {
        mark(child);
    }
}
```

---

## 内置对象

JavaScript 有许多内置对象。MQuickJS 将它们实现为原生 Rust 函数。

### 原生函数接口

```rust
// 原生函数签名
type NativeFn = fn(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value]
) -> Result<Value, String>;

// 示例：Array.prototype.push
fn array_push(
    interp: &mut Interpreter,
    this: Value,
    args: &[Value]
) -> Result<Value, String> {
    let array = this.as_array()?;
    for arg in args {
        array.push(*arg);
    }
    Ok(Value::int(array.len() as i32))
}

// 注册
interp.register_native("Array.prototype.push", array_push);
```

### 方法分发

当你调用 `arr.push(x)` 时：

```
1. GetField2 "push"     // 获取方法，保留 arr 在栈上
2. // 栈: [arr, push_function]
3. CallMethod 1         // 调用 1 个参数，this=arr
```

```rust
fn get_field(&self, obj: Value, name: &str) -> Value {
    if obj.is_array() {
        match name {
            "length" => Value::int(obj.len()),
            "push" => self.get_native("Array.prototype.push"),
            "pop" => self.get_native("Array.prototype.pop"),
            // ...
        }
    } else if obj.is_string() {
        // 字符串方法...
    }
    // 等等
}
```

---

## 闭包

闭包是从其封闭作用域捕获变量的函数。

### 挑战

```javascript
function makeCounter() {
    var count = 0;
    return function() {
        count = count + 1;  // 访问外部 'count'
        return count;
    };
}

var counter = makeCounter();
counter();  // 1
counter();  // 2
```

当 `makeCounter` 返回时，其局部变量 `count` 应该消失了……但内部函数仍然需要它！

### 解决方案：捕获变量

```rust
struct Closure {
    function: FunctionBytecode,
    captured: Vec<Value>,  // 捕获的变量值
}
```

编译器跟踪哪些变量被捕获：

```rust
struct CaptureInfo {
    outer_index: usize,  // 在外部函数中的索引
    is_local: bool,      // 来自局部变量还是外部捕获
}
```

### 编译

```javascript
function outer() {
    var x = 10;
    return function inner() {
        return x;  // 捕获 x
    };
}
```

编译为：

```
# outer:
push_i8 10
set_local 0          # x = 10
fclosure [inner], [CaptureInfo { index: 0, is_local: true }]
return

# inner:
get_var_ref 0        # 获取捕获的 x
return
```

### 运行时

```rust
fn execute_fclosure(&mut self, func_idx: usize, captures: &[CaptureInfo]) {
    let mut captured_values = Vec::new();

    for cap in captures {
        let value = if cap.is_local {
            self.get_local(cap.outer_index)
        } else {
            // 已是捕获值 - 从当前闭包获取
            self.current_closure().captured[cap.outer_index]
        };
        captured_values.push(value);
    }

    let closure = Closure {
        function: self.get_function(func_idx),
        captured: captured_values,
    };
    self.stack.push(Value::closure(closure));
}
```

---

## 异常处理

JavaScript 使用 `try/catch/finally` 进行错误处理。

### 机制

```javascript
try {
    throw new Error("oops");
} catch (e) {
    print(e.message);
} finally {
    print("cleanup");
}
```

### 异常处理器

```rust
struct ExceptionHandler {
    catch_pc: usize,       // 异常时跳转到哪里
    stack_depth: usize,    // 要恢复的栈深度
    frame_depth: usize,    // 调用帧深度
}

// 处理器栈
exception_handlers: Vec<ExceptionHandler>
```

### 字节码

```
00: catch [20]         # 在 PC 20 注册处理器
02: # try 块
    push_string "oops"
    new Error 1
    throw              # 跳转到 catch
10: drop_catch         # 移除处理器（正常退出）
12: goto [30]          # 跳过 catch 块
20: # catch 块
    set_local 0        # e = 异常值
    get_local 0
    get_field "message"
    call print 1
30: # finally
    push_string "cleanup"
    call print 1
```

### Throw 实现

```rust
fn do_throw(&mut self, exception: Value) -> Result<(), Error> {
    // 查找匹配的处理器
    while let Some(handler) = self.exception_handlers.pop() {
        // 展开调用栈到处理器的帧
        while self.call_stack.len() > handler.frame_depth {
            self.call_stack.pop();
        }

        // 恢复栈深度
        self.stack.truncate(handler.stack_depth);

        // 压入异常给 catch 块
        self.stack.push(exception);

        // 跳转到 catch
        self.pc = handler.catch_pc;
        return Ok(());
    }

    // 未找到处理器 - 传播错误
    Err(Error::UncaughtException(exception))
}
```

---

## 延伸阅读

### 书籍
- *Crafting Interpreters* by Robert Nystrom - 优秀的免费在线书籍
- *Engineering a Compiler* by Cooper & Torczon
- *Modern Compiler Implementation* by Andrew Appel

### 论文
- "A No-Frills Introduction to Lua 5.1 VM Instructions" - 优秀的字节码解释
- "Efficient Implementation of Smalltalk-80 System" - 开创性的 VM 论文

### 源代码
- [MQuickJS (C)](https://bellard.org/quickjs/) - 原始实现
- [QuickJS (C)](https://bellard.org/quickjs/) - 功能完善的前身
- [LuaJIT](https://luajit.org/) - 极度优化的 Lua VM
- [V8](https://v8.dev/) - Google 的 JavaScript 引擎

---

## 学习者练习

1. **添加新运算符**：实现 `**`（幂运算）运算符
   - 在词法分析器中添加 token
   - 添加优先级（右结合！）
   - 发出字节码
   - 在虚拟机中实现

2. **添加内置函数**：实现 `Math.sin()`
   - 注册原生函数
   - 在 `get_builtin_property` 中处理
   - 添加测试

3. **跟踪执行**：添加调试模式，打印每个操作码的执行

4. **优化**：找到频繁执行的字节码序列并添加专用操作码

5. **添加类型**：实现简单的 `Set` 对象
   - 创建 `SetObject` 结构体
   - 为值编码添加标记位
   - 实现 `add`、`has`、`delete` 方法

---

*本文档是 MQuickJS-RS 的一部分，完全由 Claude 编写的 JavaScript 引擎。*
