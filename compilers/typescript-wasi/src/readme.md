# TypeScript WASI 运行时 API 文档

## 概述

TypeScript WASI 运行时是 Rusty TypeScript 项目的重要组件，提供了在 WebAssembly 环境中执行 TypeScript 和 JavaScript 代码的能力。本文档详细介绍了 WASI 运行时的 API 接口，帮助开发者理解和使用运行时的各项功能。

## 核心组件

### WasiRuntime 结构体

`WasiRuntime` 是运行时的核心结构体，整合了文件系统、网络和内存管理功能：

```rust
pub struct WasiRuntime {
    pub fs: WasiFs,
    pub net: WasiNet,
    pub memory: WasiMemory,
    pub parser: TypeScriptParser,
    pub executor: JavaScriptExecutor,
}
```

## 编译和执行 API

### 编译 TypeScript 代码

| 函数名 | 描述 | 参数 | 返回类型 |
|-------|------|------|---------|
| `compile` | 编译 TypeScript 代码为 JavaScript | `code: *const u8, code_len: usize` | `*mut u8` (JSON 格式编译结果) |
| `get_compilation_errors` | 获取编译错误 | `code: *const u8, code_len: usize` | `*mut u8` (JSON 数组错误列表) |

### 执行 JavaScript 代码

| 函数名 | 描述 | 参数 | 返回类型 |
|-------|------|------|---------|
| `execute` | 执行 JavaScript 代码 | `code: *const u8, code_len: usize` | `*mut u8` (JSON 格式执行结果) |
| `evaluate_expression` | 求值单个表达式 | `expr: *const u8, expr_len: usize` | `*mut u8` (JSON 格式求值结果) |

## 文件系统 API

| 函数名 | 描述 | 参数 | 返回类型 |
|-------|------|------|---------|
| `fd_open` | 打开文件 | `path: *const u8, path_len: usize, mode: u32` | `i32` (文件描述符) |
| `fd_read` | 读取文件内容 | `fd: u32, buf: *mut u8, buf_len: usize` | `i32` (读取的字节数) |
| `fd_write` | 写入文件内容 | `fd: u32, buf: *const u8, buf_len: usize` | `i32` (写入的字节数) |
| `fd_close` | 关闭文件描述符 | `fd: u32` | `i32` (成功返回 0) |
| `fd_seek` | 定位文件指针 | `fd: u32, offset: i64, whence: u8` | `i64` (新的文件位置) |
| `fd_filestat_get` | 获取文件状态 | `fd: u32, buf: *mut u8` | `i32` (成功返回 0) |
| `fd_truncate` | 截断文件 | `fd: u32, size: u64` | `i32` (成功返回 0) |
| `mkdir` | 创建目录 | `path: *const u8, path_len: usize` | `i32` (成功返回 0) |
| `unlink` | 删除文件 | `path: *const u8, path_len: usize` | `i32` (成功返回 0) |
| `fd_sync_stdout` | 清空标准输出缓冲区 | 无 | `i32` (成功返回 0) |
| `fd_sync_stderr` | 清空标准错误缓冲区 | 无 | `i32` (成功返回 0) |

## 网络 API

| 函数名 | 描述 | 参数 | 返回类型 |
|-------|------|------|---------|
| `sock_open` | 创建套接字 | `socket_type: u32` | `i32` (套接字描述符) |
| `sock_bind` | 绑定套接字到地址 | `fd: u32, ip: *const u8, port: u16, is_ipv6: bool` | `i32` (成功返回 0) |
| `sock_listen` | 开始监听连接 | `fd: u32, backlog: usize` | `i32` (成功返回 0) |
| `sock_accept` | 接受新连接 | `fd: u32` | `i32` (新连接的套接字描述符) |
| `sock_connect` | 连接到远程地址 | `fd: u32, ip: *const u8, port: u16, is_ipv6: bool` | `i32` (成功返回 0) |
| `sock_send` | 发送数据 | `fd: u32, buf: *const u8, buf_len: usize` | `i32` (发送的字节数) |
| `sock_recv` | 接收数据 | `fd: u32, buf: *mut u8, buf_len: usize` | `i32` (接收的字节数) |
| `sock_close` | 关闭套接字 | `fd: u32` | `i32` (成功返回 0) |

## 系统 API

| 函数名 | 描述 | 参数 | 返回类型 |
|-------|------|------|---------|
| `get_current_time` | 获取当前时间 | 无 | `u64` (时间戳，毫秒) |
| `get_random` | 获取随机数 | 无 | `u32` (32位随机数) |
| `random_get` | 生成随机数据 | `buf: *mut u8, buf_len: usize` | `i32` (成功返回 0) |
| `clock_time_get` | 获取当前时间 | `timestamp: *mut u64` | `i32` (成功返回 0) |
| `get_environment_variable` | 获取环境变量 | `name: *const u8, name_len: usize` | `*mut u8` (环境变量值) |
| `set_environment_variable` | 设置环境变量 | `name: *const u8, name_len: usize, value: *const u8, value_len: usize` | `i32` (成功返回 0) |
| `environ_sizes_get` | 获取环境变量数量 | 无 | `i32` (环境变量数量) |
| `environ_get` | 获取环境变量 | `names_ptr: *mut *const u8, values_ptr: *mut *const u8` | `i32` (成功返回 0) |
| `get_system_info` | 获取系统信息 | 无 | `*mut u8` (系统信息 JSON) |
| `get_memory_usage` | 获取内存使用情况 | 无 | `*mut u8` (内存使用情况 JSON) |
| `run_gc` | 执行垃圾回收 | 无 | `i32` (成功返回 0) |
| `measure_execution_time` | 测量执行时间 | `callback: extern "C" fn()` | `u64` (执行时间，毫秒) |
| `register_callback` | 注册全局回调函数 | `name: *const u8, name_len: usize, callback: extern "C" fn()` | `i32` (成功返回 0) |
| `call_callback` | 调用已注册的回调函数 | `name: *const u8, name_len: usize` | `i32` (成功返回 0) |

## 工具 API

| 函数名 | 描述 | 参数 | 返回类型 |
|-------|------|------|---------|
| `compute_hash` | 计算哈希值 | `data: *const u8, data_len: usize, algorithm: u32` | `*mut u8` (哈希值字符串) |
| `base64_encode_decode` | 编码/解码 Base64 | `data: *const u8, data_len: usize, encode: u32` | `*mut u8` (编码/解码结果) |
| `get_version` | 获取运行时版本 | 无 | `*mut u8` (版本字符串) |
| `get_performance_metrics` | 获取性能指标 | 无 | `*mut u8` (性能指标 JSON) |
| `free_memory` | 释放内存 | `ptr: *mut u8` | 无 |

## 数据结构

### 编译结果 (CompileResult)

```rust
pub struct CompileResult {
    pub success: bool,
    pub output: String,
    pub errors: Vec<CompileError>,
    pub warnings: Vec<String>,
}
```

### 执行结果 (ExecutionResult)

```rust
pub struct ExecutionResult {
    pub success: bool,
    pub result: String,
    pub stdout: String,
    pub stderr: String,
    pub error: Option<RuntimeError>,
}
```

### 求值结果 (EvaluationResult)

```rust
pub struct EvaluationResult {
    pub success: bool,
    pub value: String,
    pub type_name: String,
    pub error: Option<String>,
}
```

### 编译错误 (CompileError)

```rust
pub enum CompileError {
    SyntaxError { message: String, line: usize, column: usize },
    TypeError { message: String },
    UndefinedVariable { name: String },
    UndefinedFunction { name: String },
    InvalidCode { message: String },
    InternalError { message: String },
}
```

### 运行时错误 (RuntimeError)

```rust
pub enum RuntimeError {
    TypeError { message: String },
    DivisionByZero,
    NullReference { message: String },
    IndexOutOfBounds { index: usize, length: usize },
    StackOverflow,
    OutOfMemory,
    NotImplemented { operation: String },
    Timeout,
    InternalError { message: String },
}
```

## 示例用法

### 编译和执行 TypeScript 代码

```javascript
// 编译 TypeScript 代码
const code = `
function add(a: number, b: number): number {
    return a + b;
}

console.log(add(1, 2));
`;

const compileResult = compile(code);
if (compileResult.success) {
    // 执行编译后的 JavaScript 代码
    const executeResult = execute(compileResult.output);
    console.log(executeResult.stdout); // 输出: 3
}
```

### 使用文件系统 API

```javascript
// 打开文件
const fd = fd_open("test.txt", 0); // 0 表示只读模式
if (fd >= 0) {
    // 读取文件内容
    const buffer = new Uint8Array(1024);
    const bytesRead = fd_read(fd, buffer.buffer, buffer.length);
    if (bytesRead > 0) {
        const content = new TextDecoder().decode(buffer.subarray(0, bytesRead));
        console.log(content);
    }
    // 关闭文件
    fd_close(fd);
}
```

### 使用系统 API

```javascript
// 获取当前时间
const timestamp = get_current_time();
console.log(`Current time: ${timestamp}ms`);

// 获取系统信息
const systemInfo = JSON.parse(get_system_info());
console.log(`OS: ${systemInfo.os}`);
console.log(`Arch: ${systemInfo.arch}`);

// 获取内存使用情况
const memoryUsage = JSON.parse(get_memory_usage());
console.log(`Used memory: ${memoryUsage.used} bytes`);
```

## 性能优化

- **内存管理**: 及时释放不再使用的内存，避免内存泄漏
- **代码优化**: 减少不必要的计算和内存分配
- **批量操作**: 对于文件和网络操作，尽量使用批量处理
- **缓存**: 对于频繁使用的数据，可以使用缓存来提高性能

## 错误处理

运行时提供了详细的错误类型，开发者应该适当处理这些错误：

```javascript
try {
    const result = execute("throw new Error('Test error');");
    if (!result.success) {
        console.error(`Runtime error: ${result.error}`);
    }
} catch (e) {
    console.error(`Exception: ${e}`);
}
```

## 安全注意事项

- **内存安全**: 注意内存分配和释放，避免缓冲区溢出
- **文件访问**: 限制文件系统访问权限，避免恶意操作
- **网络安全**: 注意网络连接的安全性，避免恶意请求
- **代码执行**: 限制执行时间，避免无限循环

## 总结

TypeScript WASI 运行时 API 提供了丰富的功能，支持在 WebAssembly 环境中执行 TypeScript 和 JavaScript 代码。通过这些 API，开发者可以：

1. 编译和执行 TypeScript 代码
2. 操作文件系统
3. 进行网络通信
4. 访问系统资源
5. 执行各种工具函数

这些 API 为 Rusty TypeScript 项目提供了完整的运行时支持，使得 TypeScript 代码可以在 WebAssembly 环境中高效运行。