# Rusty TypeScript 用户指南

本文档提供了 Rusty TypeScript 的完整用户指南，包括工具链、宏系统的使用方法和最佳实践。

## 目录

- [1. 项目简介](#1-项目简介)
- [2. 安装指南](#2-安装指南)
  - [2.1 从源码构建](#21-从源码构建)
  - [2.2 环境要求](#22-环境要求)
- [3. 工具链使用指南](#3-工具链使用指南)
  - [3.1 tsc 命令](#31-tsc-命令)
    - [3.1.1 基本用法](#311-基本用法)
    - [3.1.2 命令行选项](#312-命令行选项)
    - [3.1.3 配置文件](#313-配置文件)
    - [3.1.4 使用示例](#314-使用示例)
  - [3.2 tsup 命令](#32-tsup-命令)
    - [3.2.1 基本用法](#321-基本用法)
    - [3.2.2 常用选项](#322-常用选项)
    - [3.2.3 使用示例](#323-使用示例)
- [4. 宏系统使用指南](#4-宏系统使用指南)
  - [4.1 安装](#41-安装)
  - [4.2 可用宏列表](#42-可用宏列表)
  - [4.3 宏使用详解](#43-宏使用详解)
    - [4.3.1 TypescriptClass 宏](#431-typescriptclass-宏)
    - [4.3.2 TypescriptInterface 宏](#432-typescriptinterface-宏)
    - [4.3.3 TypescriptEnum 宏](#433-typescriptenum-宏)
    - [4.3.4 TypescriptType 宏](#434-typescripttype-宏)
    - [4.3.5 TypescriptUnion 宏](#435-typescriptunion-宏)
    - [4.3.6 TypescriptGuard 宏](#436-typescriptguard-宏)
    - [4.3.7 typescript_function 宏](#437-typescript_function-宏)
    - [4.3.8 typescript_function_declaration 宏](#438-typescript_function_declaration-宏)
    - [4.3.9 typescript_namespace 宏](#439-typescript_namespace-宏)
  - [4.4 字段属性](#44-字段属性)
  - [4.5 类型映射规则](#45-类型映射规则)
- [5. 示例代码](#5-示例代码)
  - [5.1 工具链示例](#51-工具链示例)
  - [5.2 宏系统示例](#52-宏系统示例)
- [6. 常见问题](#6-常见问题)
- [7. 最佳实践](#7-最佳实践)
- [8. 贡献指南](#8-贡献指南)
- [9. 许可证](#9-许可证)

## 1. 项目简介

Rusty TypeScript 是一个用 Rust 实现的 TypeScript 编译器项目，包含以下核心模块：

- **核心编译器**：负责 TypeScript 代码的解析、编译和执行
- **中间表示**：提供 TypeScript 代码的中间表示形式，用于优化和代码生成
- **语言服务器**：提供代码补全、定义查找等 IDE 功能
- **工具链**：包括 tsc 编译器和 tsup 打包工具
- **宏系统**：用于从 Rust 代码生成 TypeScript 类型定义
- **类型系统**：提供 TypeScript 类型的表示和操作
- **WebAssembly 支持**：支持在 Web 环境中运行 TypeScript 代码
- **前端界面**：提供交互式 TypeScript 体验

## 2. 安装指南

### 2.1 从源码构建

1. 克隆仓库：

```bash
git clone https://github.com/nyar-vm/rusty-typescript.git
cd rusty-typescript
```

2. 构建项目：

```bash
cargo build --release
```

3. 将可执行文件添加到 PATH：

```bash
# Linux/macOS
export PATH="$PATH:$(pwd)/target/release"

# Windows
set PATH=%PATH%;%CD%\target\release
```

### 2.2 环境要求

- Rust 1.60 或更高版本
- Cargo 包管理器
- Node.js（可选，用于前端开发）

## 3. 工具链使用指南

### 3.1 tsc 命令

`tsc` 命令是 Rusty TypeScript 的编译器命令，用于编译 TypeScript 文件。

#### 3.1.1 基本用法

```bash
tsc [options] [files]
```

#### 3.1.2 命令行选项

| 选项 | 描述 | 示例 |
|------|------|------|
| `--target` | 指定编译目标 ECMAScript 版本 | `tsc --target es2020 file.ts` |
| `--module` | 指定模块系统 | `tsc --module esm file.ts` |
| `--outDir` | 指定输出目录 | `tsc --outDir dist file.ts` |
| `--sourceMap` | 生成源映射文件 | `tsc --sourceMap file.ts` |
| `--execute` | 直接执行 TypeScript 脚本 | `tsc --execute script.ts` |
| `--strict` | 启用严格模式 | `tsc --strict file.ts` |
| `--noEmit` | 不生成输出文件 | `tsc --noEmit file.ts` |
| `--format` | 格式化代码 | `tsc --format file.ts` |
| `--json` | 输出 JSON 格式 | `tsc --json file.ts` |
| `--quiet` | 安静模式，仅显示错误 | `tsc --quiet file.ts` |
| `--watch` | 监视模式 | `tsc --watch file.ts` |
| `--declaration` | 生成声明文件 | `tsc --declaration file.ts` |
| `--noEmitOnError` | 仅类型检查，有错误时不生成输出 | `tsc --noEmitOnError file.ts` |
| `--skipLibCheck` | 跳过库检查 | `tsc --skipLibCheck file.ts` |
| `--removeComments` | 移除注释 | `tsc --removeComments file.ts` |
| `--minify` | 压缩输出 | `tsc --minify file.ts` |
| `--project` | 配置文件路径 | `tsc --project tsconfig.json` |
| `--webidl` | WebIDL 文件路径 | `tsc --webidl interface.idl` |

#### 3.1.3 配置文件

`tsc` 命令支持使用 `tsconfig.json` 配置文件，示例：

```json
{
  "compilerOptions": {
    "target": "es2020",
    "module": "esm",
    "outDir": "dist",
    "strict": true,
    "sourceMap": true,
    "declaration": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}
```

#### 3.1.4 使用示例

1. 编译单个文件：

```bash
tsc file.ts
```

2. 编译多个文件：

```bash
tsc file1.ts file2.ts
```

3. 执行 TypeScript 脚本：

```bash
tsc --execute script.ts
```

4. 使用配置文件：

```bash
tsc --project tsconfig.json
```

5. 仅进行类型检查：

```bash
tsc --noEmit file.ts
```

6. 生成声明文件和 source map：

```bash
tsc --declaration --sourceMap file.ts
```

### 3.2 tsup 命令

`tsup` 命令是 Rusty TypeScript 的打包工具，用于将 TypeScript 代码打包为单个文件。

#### 3.2.1 基本用法

```bash
tsup [options] [files]
```

#### 3.2.2 常用选项

| 选项 | 描述 | 示例 |
|------|------|------|
| `--outDir` | 指定输出目录 | `tsup --outDir dist file.ts` |
| `--format` | 指定输出格式（esm、cjs） | `tsup --format esm file.ts` |
| `--minify` | 压缩输出代码 | `tsup --minify file.ts` |
| `--sourceMap` | 生成源映射文件 | `tsup --sourceMap file.ts` |
| `--watch` | 启用监视模式 | `tsup --watch file.ts` |

#### 3.2.3 使用示例

1. 打包单个文件：

```bash
tsup file.ts
```

2. 打包多个文件：

```bash
tsup file1.ts file2.ts
```

3. 打包并压缩：

```bash
tsup --minify file.ts
```

## 4. 宏系统使用指南

### 4.1 安装

在 `Cargo.toml` 文件中添加依赖：

```toml
[dependencies]
typescript-macros = { path = "compilers/typescript-macros" }
```

### 4.2 可用宏列表

| 宏名称 | 用途 | 适用类型 |
|--------|------|----------|
| `TypescriptClass` | 生成 TypeScript 类定义 | 结构体 |
| `TypescriptInterface` | 生成 TypeScript 接口定义 | 结构体 |
| `TypescriptEnum` | 生成 TypeScript 枚举定义 | 枚举 |
| `TypescriptType` | 生成 TypeScript 类型别名 | 结构体 |
| `TypescriptUnion` | 生成 TypeScript 联合类型 | 枚举 |
| `TypescriptGuard` | 生成 TypeScript 类型守卫 | 结构体 |
| `typescript_function` | 生成 TypeScript 函数类型定义 | 函数 |
| `typescript_function_declaration` | 生成 TypeScript 函数声明 | 函数 |
| `typescript_namespace` | 生成 TypeScript 命名空间 | 模块 |

### 4.3 宏使用详解

#### 4.3.1 TypescriptClass 宏

`TypescriptClass` 宏用于从 Rust 结构体生成 TypeScript 类定义。

**用法**：

```rust
use typescript_macros::TypescriptClass;

#[derive(TypescriptClass)]
struct User {
    id: u32,
    name: String,
    active: bool,
}
```

**生成的 TypeScript 代码**：

```typescript
class User {
    id: number;
    name: string;
    active: boolean;
    
    constructor(id: number, name: string, active: boolean) {
        this.id = id;
        this.name = name;
        this.active = active;
    }
}
```

**获取生成的 TypeScript 代码**：

```rust
let ts_class = User::ts_class_definition();
println!("{}", ts_class);
```

#### 4.3.2 TypescriptInterface 宏

`TypescriptInterface` 宏用于从 Rust 结构体生成 TypeScript 接口定义。

**用法**：

```rust
use typescript_macros::TypescriptInterface;

#[derive(TypescriptInterface)]
struct User {
    id: u32,
    name: String,
    active: bool,
}
```

**生成的 TypeScript 代码**：

```typescript
interface User {
    id: number;
    name: string;
    active: boolean;
}
```

**获取生成的 TypeScript 代码**：

```rust
let ts_interface = User::ts_interface_definition();
println!("{}", ts_interface);
```

#### 4.3.3 TypescriptEnum 宏

`TypescriptEnum` 宏用于从 Rust 枚举生成 TypeScript 枚举定义。

**用法**：

```rust
use typescript_macros::TypescriptEnum;

#[derive(TypescriptEnum)]
enum Color {
    Red,
    Green,
    Blue,
}

#[derive(TypescriptEnum)]
enum Direction {
    Up = "UP",
    Down = "DOWN",
    Left = "LEFT",
    Right = "RIGHT",
}
```

**生成的 TypeScript 代码**：

```typescript
enum Color {
    Red = 0,
    Green = 1,
    Blue = 2,
}

enum Direction {
    Up = "UP",
    Down = "DOWN",
    Left = "LEFT",
    Right = "RIGHT",
}
```

**获取生成的 TypeScript 代码**：

```rust
let ts_color_enum = Color::ts_enum_definition();
println!("{}", ts_color_enum);

let ts_direction_enum = Direction::ts_enum_definition();
println!("{}", ts_direction_enum);
```

#### 4.3.4 TypescriptType 宏

`TypescriptType` 宏用于从 Rust 结构体生成 TypeScript 类型别名。

**用法**：

```rust
use typescript_macros::TypescriptType;

#[derive(TypescriptType)]
struct User {
    id: u32,
    name: String,
    active: bool,
}
```

**生成的 TypeScript 代码**：

```typescript
type User = {
    id: number;
    name: string;
    active: boolean;
};
```

**获取生成的 TypeScript 代码**：

```rust
let ts_type = User::ts_type_definition();
println!("{}", ts_type);
```

#### 4.3.5 TypescriptUnion 宏

`TypescriptUnion` 宏用于从 Rust 枚举生成 TypeScript 联合类型。

**用法**：

```rust
use typescript_macros::TypescriptUnion;

#[derive(TypescriptUnion)]
enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
    Triangle { base: f64, height: f64 },
}
```

**生成的 TypeScript 代码**：

```typescript
type Shape = 
    | { type: "Circle"; radius: number }
    | { type: "Rectangle"; width: number; height: number }
    | { type: "Triangle"; base: number; height: number };
```

**获取生成的 TypeScript 代码**：

```rust
let ts_union = Shape::ts_union_definition();
println!("{}", ts_union);
```

#### 4.3.6 TypescriptGuard 宏

`TypescriptGuard` 宏用于从 Rust 结构体生成 TypeScript 类型守卫。

**用法**：

```rust
use typescript_macros::TypescriptGuard;

#[derive(TypescriptGuard)]
struct User {
    id: u32,
    name: String,
    active: bool,
}
```

**生成的 TypeScript 代码**：

```typescript
function isUser(obj: any): obj is User {
    return (
        typeof obj === 'object' && obj !== null &&
        typeof obj.id === 'number' &&
        typeof obj.name === 'string' &&
        typeof obj.active === 'boolean'
    );
}
```

**获取生成的 TypeScript 代码**：

```rust
let ts_guard = User::ts_guard();
println!("{}", ts_guard);
```

#### 4.3.7 typescript_function 宏

`typescript_function` 宏用于从 Rust 函数生成 TypeScript 函数类型定义。

**用法**：

```rust
use typescript_macros::typescript_function;

#[typescript_function]
fn add(a: u32, b: u32) -> u32 {
    a + b
}
```

**生成的 TypeScript 代码**：

```typescript
type addFunction = (a: number, b: number) => number;
```

**获取生成的 TypeScript 代码**：

```rust
let ts_function = add_ts_function_definition();
println!("{}", ts_function);
```

#### 4.3.8 typescript_function_declaration 宏

`typescript_function_declaration` 宏用于从 Rust 函数生成 TypeScript 函数声明。

**用法**：

```rust
use typescript_macros::typescript_function_declaration;

#[typescript_function_declaration]
fn greet(name: String) -> String {
    format!("Hello, {}", name)
}
```

**生成的 TypeScript 代码**：

```typescript
function greet(name: string): string;
```

**获取生成的 TypeScript 代码**：

```rust
let ts_declaration = greet_ts_function_declaration();
println!("{}", ts_declaration);
```

#### 4.3.9 typescript_namespace 宏

`typescript_namespace` 宏用于从 Rust 模块生成 TypeScript 命名空间。

**用法**：

```rust
use typescript_macros::typescript_namespace;

#[typescript_namespace("utils")]
mod utils {
    pub fn multiply(a: u32, b: u32) -> u32 {
        a * b
    }
    
    pub struct Point {
        pub x: f64,
        pub y: f64,
    }
}
```

**生成的 TypeScript 代码**：

```typescript
namespace utils {
    // 模块内容将在这里生成
}
```

**获取生成的 TypeScript 代码**：

```rust
let ts_namespace = utils_ts_namespace();
println!("{}", ts_namespace);
```

### 4.4 字段属性

所有结构体相关的宏都支持以下字段属性：

| 属性 | 描述 | 示例 |
|------|------|------|
| `#[ts(rename = "new_name")]` | 重命名字段 | `#[ts(rename = "user_id")] id: u32` |
| `#[ts(skip)]` | 跳过该字段 | `#[ts(skip)] internal: bool` |
| `#[ts(optional)]` | 标记为可选字段 | `#[ts(optional)] age: Option<u32>` |
| `#[ts(type = "custom_type")]` | 自定义 TypeScript 类型 | `#[ts(type = "Date")] created_at: String` |

**示例**：

```rust
use typescript_macros::TypescriptInterface;

#[derive(TypescriptInterface)]
struct User {
    #[ts(rename = "user_id")]
    id: u32,
    name: String,
    #[ts(optional)]
    age: Option<u32>,
    #[ts(skip)]
    internal_id: u64,
    #[ts(type = "Date")]
    created_at: String,
}
```

生成的 TypeScript 代码：

```typescript
interface User {
    user_id: number;
    name: string;
    age: number | undefined;
    created_at: Date;
}
```

### 4.5 类型映射规则

宏系统会将 Rust 类型映射到 TypeScript 类型，规则如下：

| Rust 类型 | TypeScript 类型 |
|-----------|----------------|
| `u8`, `u16`, `u32`, `u64`, `i8`, `i16`, `i32`, `i64`, `f32`, `f64`, `usize`, `isize` | `number` |
| `bool` | `boolean` |
| `String`, `&str` | `string` |
| `Option<T>` | `T | undefined` |
| `Vec<T>`, `LinkedList<T>`, `VecDeque<T>`, `&[T]` | `T[]` |
| `HashSet<T>`, `BTreeSet<T>` | `Set<T>` |
| `HashMap<K, V>`, `BTreeMap<K, V>` | `Record<K, V>` |
| `()` | `null` |
| 元组 `(A, B, C)` | 数组 `[A, B, C]` |
| 自定义结构体 | 对应生成的 TypeScript 类型 |
| 自定义枚举 | 对应生成的 TypeScript 枚举或联合类型 |

## 5. 示例代码

### 5.1 工具链示例

#### 示例 1：基本编译

```bash
# 编译单个文件
tsc src/index.ts

# 编译到指定目录
tsc --outDir dist src/index.ts

# 带 source map 和声明文件
tsc --sourceMap --declaration src/index.ts
```

#### 示例 2：使用配置文件

**tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "es2020",
    "module": "esm",
    "outDir": "dist",
    "strict": true,
    "sourceMap": true,
    "declaration": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}
```

**执行编译**

```bash
tsc --project tsconfig.json
```

#### 示例 3：执行 TypeScript 脚本

**script.ts**

```typescript
function greet(name: string): string {
    return `Hello, ${name}!`;
}

console.log(greet("World"));
```

**执行脚本**

```bash
tsc --execute script.ts
```

### 5.2 宏系统示例

#### 示例 1：完整的宏使用

```rust
use typescript_macros::{TypescriptClass, TypescriptInterface, TypescriptEnum, TypescriptType, TypescriptUnion, TypescriptGuard, typescript_function, typescript_function_declaration, typescript_namespace};

// 生成 TypeScript 类
#[derive(TypescriptClass)]
struct User {
    id: u32,
    name: String,
    active: bool,
}

// 生成 TypeScript 接口
#[derive(TypescriptInterface)]
struct Point {
    x: f64,
    y: f64,
}

// 生成 TypeScript 枚举
#[derive(TypescriptEnum)]
enum Color {
    Red,
    Green,
    Blue,
}

// 生成 TypeScript 类型别名
#[derive(TypescriptType)]
struct Product {
    id: u32,
    name: String,
    price: f64,
}

// 生成 TypeScript 联合类型
#[derive(TypescriptUnion)]
enum Shape {
    Circle { radius: f64 },
    Rectangle { width: f64, height: f64 },
}

// 生成 TypeScript 类型守卫
#[derive(TypescriptGuard)]
struct Person {
    name: String,
    age: u32,
}

// 生成 TypeScript 函数类型定义
#[typescript_function]
fn add(a: u32, b: u32) -> u32 {
    a + b
}

// 生成 TypeScript 函数声明
#[typescript_function_declaration]
fn greet(name: String) -> String {
    format!("Hello, {}", name)
}

// 生成 TypeScript 命名空间
#[typescript_namespace("utils")]
mod utils {
    pub fn multiply(a: u32, b: u32) -> u32 {
        a * b
    }
}

fn main() {
    // 打印生成的 TypeScript 代码
    println!("=== TypeScript Class ===");
    println!("{}", User::ts_class_definition());
    
    println!("\n=== TypeScript Interface ===");
    println!("{}", Point::ts_interface_definition());
    
    println!("\n=== TypeScript Enum ===");
    println!("{}", Color::ts_enum_definition());
    
    println!("\n=== TypeScript Type ===");
    println!("{}", Product::ts_type_definition());
    
    println!("\n=== TypeScript Union ===");
    println!("{}", Shape::ts_union_definition());
    
    println!("\n=== TypeScript Guard ===");
    println!("{}", Person::ts_guard());
    
    println!("\n=== TypeScript Function ===");
    println!("{}", add_ts_function_definition());
    
    println!("\n=== TypeScript Function Declaration ===");
    println!("{}", greet_ts_function_declaration());
    
    println!("\n=== TypeScript Namespace ===");
    println!("{}", utils_ts_namespace());
}
```

#### 示例 2：使用字段属性

```rust
use typescript_macros::TypescriptInterface;

#[derive(TypescriptInterface)]
struct User {
    #[ts(rename = "user_id")]
    id: u32,
    name: String,
    #[ts(optional)]
    age: Option<u32>,
    #[ts(skip)]
    internal_id: u64,
    #[ts(type = "Date")]
    created_at: String,
}

fn main() {
    println!("{}", User::ts_interface_definition());
}
```

## 6. 常见问题

### 1. 编译错误："Only structs are supported for TypescriptInterface"

**原因**：`TypescriptInterface` 宏只支持结构体，不支持 trait。

**解决方案**：使用结构体代替 trait，或者使用其他宏。

### 2. 编译错误："Tuple structs are not supported"

**原因**：所有宏都不支持元组结构体。

**解决方案**：使用命名结构体代替元组结构体。

### 3. 编译错误："Unit structs are not supported"

**原因**：所有宏都不支持单元结构体。

**解决方案**：使用有字段的结构体代替单元结构体。

### 4. 生成的 TypeScript 类型不正确

**原因**：某些 Rust 类型可能没有对应的 TypeScript 类型映射。

**解决方案**：对于复杂类型，宏会默认使用 `any` 类型。如果需要更精确的类型映射，可以使用 `#[ts(type = "custom_type")]` 属性指定自定义类型。

### 5. 字段重命名不生效

**原因**：可能是字段属性语法错误。

**解决方案**：确保使用正确的语法 `#[ts(rename = "new_name")]`。

### 6. 生成的 TypeScript 代码不包含所有字段

**原因**：可能某些字段被标记为 `#[ts(skip)]` 或者字段没有正确命名。

**解决方案**：检查字段属性，确保需要的字段没有被跳过，并且所有字段都有正确的标识符。

### 7. 工具链命令未找到

**原因**：可执行文件未添加到 PATH 环境变量。

**解决方案**：将 `target/release` 目录添加到 PATH 环境变量。

### 8. 宏系统编译错误

**原因**：可能是 Rust 版本不兼容或者依赖配置错误。

**解决方案**：确保使用 Rust 1.60 或更高版本，并正确配置 `Cargo.toml` 文件。

## 7. 最佳实践

1. **使用配置文件**：对于复杂项目，建议使用 `tsconfig.json` 配置文件来管理编译选项。

2. **合理使用宏**：根据需要选择合适的宏，例如：
   - 对于需要在 TypeScript 中实例化的类型，使用 `TypescriptClass`
   - 对于纯数据结构，使用 `TypescriptInterface` 或 `TypescriptType`
   - 对于有多种变体的类型，使用 `TypescriptUnion`

3. **使用字段属性**：合理使用字段属性来定制生成的 TypeScript 类型，例如：
   - 使用 `#[ts(rename)]` 来使用符合 TypeScript 命名规范的字段名
   - 使用 `#[ts(optional)]` 来标记可选字段
   - 使用 `#[ts(type)]` 来指定自定义 TypeScript 类型

4. **类型安全**：利用宏系统生成的 TypeScript 类型来确保 Rust 和 TypeScript 之间的类型安全。

5. **性能优化**：对于大型项目，考虑使用 `tsup` 打包工具来优化输出代码。

6. **代码组织**：将相关的类型定义和函数放在同一个模块中，使用 `typescript_namespace` 宏来生成对应的 TypeScript 命名空间。

## 8. 贡献指南

如果您发现任何问题或有改进建议，请提交 Issue 或 Pull Request 到 GitHub 仓库。

### 贡献流程

1. Fork 仓库
2. 创建特性分支
3. 提交更改
4. 运行测试
5. 提交 Pull Request

### 代码规范

- 遵循 Rust 代码规范
- 为所有公开 API 添加文档注释
- 编写测试用例
- 确保代码通过所有测试

## 9. 许可证

Rusty TypeScript 使用 MIT 许可证。

```
MIT License

Copyright (c) 2026 Rusty TypeScript Contributors

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```