# NAPI 模块加载指南

本文档介绍 rusty-typescript 如何加载 NAPI 扩展模块（.node 文件）。

## 概述

rusty-typescript 支持加载 Node.js NAPI 扩展模块，使 TypeScript 代码能够调用原生模块的功能。这是通过以下组件实现的：

1. **跨平台动态库加载器** (`platform/dylib.rs`)
2. **NAPI 模块加载器** (`ffi/napi.rs`)
3. **FFI 管理器扩展** (`ffi/mod.rs`)
4. **VM 模块系统支持** (`vm/mod.rs`)

## 架构

### 动态库加载

动态库加载器提供跨平台的动态库加载功能：

- **Windows**: 使用 `LoadLibraryA`/`GetProcAddress`/`FreeLibrary`
- **Unix** (Linux/macOS): 使用 `dlopen`/`dlsym`/`dlclose`

```rust
use typescript::platform::dylib::DynamicLibrary;

// 加载动态库
let lib = DynamicLibrary::open("/path/to/module.node")?;

// 获取符号
let symbol: Symbol<extern "C" fn()> = lib.get("function_name")?;
```

### NAPI 模块加载器

NAPI 模块加载器负责加载 .node 文件并解析其导出：

```rust
use typescript::ffi::NapiModuleLoader;

let mut loader = NapiModuleLoader::new();
let module = loader.load_module("myModule", "/path/to/myModule.node")?;

// 获取导出的函数
if let Some(func) = module.get_function("myFunction") {
    let result = func.call(&[TsValue::Number(42.0)])?;
}
```

### FFI 管理器

FFI 管理器集成了 NAPI 模块加载功能：

```rust
use typescript::ffi::FfiManager;

let mut manager = FfiManager::new();

// 加载 NAPI 模块
manager.load_napi_module("myModule", "/path/to/myModule.node")?;

// 检查模块是否已加载
if manager.has_napi_module("myModule") {
    let module = manager.get_napi_module("myModule").unwrap();
    // 使用模块...
}
```

## 类型转换

### TsValue 到 NAPI 值

```rust
use typescript::ffi::{ts_value_to_napi, NapiEnv};

let env = NapiEnv(std::ptr::null_mut());
let ts_value = TsValue::Number(42.0);
let napi_value = ts_value_to_napi(env, &ts_value)?;
```

### NAPI 值到 TsValue

```rust
use typescript::ffi::{napi_to_ts_value, NapiEnv, NapiValue};

let env = NapiEnv(std::ptr::null_mut());
let napi_value = NapiValue(std::ptr::null_mut());
let ts_value = napi_to_ts_value(env, napi_value)?;
```

## 在 TypeScript 中使用

在 TypeScript 代码中，可以通过 `import` 语句加载 NAPI 模块：

```typescript
// 加载 NAPI 模块
import myModule from './myModule.node';

// 调用导出的函数
const result = myModule.myFunction(42);
```

VM 会自动检测 `.node` 文件扩展名，并通过 FFI 管理器加载模块。

## 支持的类型

### 基本类型

- `undefined` → `TsValue::Undefined`
- `null` → `TsValue::Null`
- `boolean` → `TsValue::Boolean`
- `number` → `TsValue::Number`
- `string` → `TsValue::String`
- `bigint` → `TsValue::BigInt`
- `symbol` → `TsValue::Symbol`

### 复杂类型

- `object` → `TsValue::Object`
- `array` → `TsValue::Array`
- `function` → `TsValue::Function`
- `Date` → `TsValue::Date`
- `RegExp` → `TsValue::RegExp`
- `Map` → `TsValue::Map`
- `Set` → `TsValue::Set`
- `Promise` → `TsValue::Promise`

## 错误处理

NAPI 模块加载可能产生以下错误：

- `DylibError::OpenError`: 打开动态库失败
- `DylibError::SymbolError`: 查找符号失败
- `TsError::ReferenceError`: 模块未找到
- `TsError::Other`: 其他错误

```rust
match loader.load_module("myModule", "/path/to/myModule.node") {
    Ok(module) => {
        // 使用模块
    }
    Err(TsError::Other(msg)) => {
        eprintln!("Failed to load module: {}", msg);
    }
    Err(e) => {
        eprintln!("Error: {:?}", e);
    }
}
```

## 跨平台支持

NAPI 模块加载支持以下平台：

- **Windows**: `.dll` 文件（通过 `.node` 扩展名）
- **Linux**: `.so` 文件（通过 `.node` 扩展名）
- **macOS**: `.dylib` 文件（通过 `.node` 扩展名）

动态库加载器会自动处理平台差异。

## 性能考虑

1. **模块缓存**: 已加载的模块会被缓存，避免重复加载
2. **对象池**: 使用对象池减少内存分配
3. **延迟加载**: 符号解析在首次使用时进行

## 安全注意事项

1. 只加载可信来源的 NAPI 模块
2. NAPI 模块以原生代码运行，可能访问系统资源
3. 注意内存管理，避免内存泄漏

## 示例

### 创建测试用的 NAPI 模块

```c
// test_module.c
#include <node_api.h>

napi_value Hello(napi_env env, napi_callback_info info) {
    napi_value greeting;
    napi_create_string_utf8(env, "Hello from NAPI!", NAPI_AUTO_LENGTH, &greeting);
    return greeting;
}

napi_value Init(napi_env env, napi_value exports) {
    napi_property_descriptor desc = {
        "hello", 0, Hello, 0, 0, 0, napi_default, 0
    };
    napi_define_properties(env, exports, 1, &desc);
    return exports;
}

NAPI_MODULE(NODE_GYP_MODULE_NAME, Init)
```

### 在 rusty-typescript 中加载

```rust
use typescript::TypeScript;

let mut runtime = TypeScript::new();

// 执行 TypeScript 代码加载 NAPI 模块
let script = r#"
    import testModule from './test_module.node';
    console.log(testModule.hello());
"#;

runtime.execute_script(script)?;
```

## 限制

当前实现为简化版本，以下功能尚未完全实现：

1. 完整的 NAPI 环境初始化
2. 完整的 NAPI 对象属性解析
3. 异步 NAPI 函数调用
4. NAPI 回调函数支持

这些功能将在后续版本中完善。
