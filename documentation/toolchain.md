# Rusty TypeScript 工具链使用指南

本文档提供了 Rusty TypeScript 工具链的使用指南，包括 typescript-tools 和 typescript-macros 的安装、配置和使用方法。

## 目录

- [安装](#安装)
- [typescript-tools](#typescript-tools)
  - [tsc 命令](#tsc-命令)
  - [tsup 命令](#tsup-命令)
- [typescript-macros](#typescript-macros)
  - [TypescriptClass 宏](#typescriptclass-宏)
  - [TypescriptFunction 宏](#typescriptfunction-宏)
  - [TypescriptInterface 宏](#typescriptinterface-宏)
  - [TypescriptEnum 宏](#typescriptenum-宏)
  - [TypescriptType 宏](#typescripttype-宏)
- [示例](#示例)
- [常见问题](#常见问题)

## 安装

### 从源码构建

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

## typescript-tools

### tsc 命令

`tsc` 命令是 Rusty TypeScript 的编译器命令，用于编译 TypeScript 文件。

#### 基本用法

```bash
tsc [options] [files]
```

#### 常用选项

| 选项 | 描述 | 示例 |
|------|------|------|
| `--target` | 指定编译目标版本 | `tsc --target es2018 file.ts` |
| `--module` | 指定模块系统 | `tsc --module commonjs file.ts` |
| `--outDir` | 指定输出目录 | `tsc --outDir dist file.ts` |
| `--sourceMap` | 生成源映射文件 | `tsc --sourceMap file.ts` |
| `--execute` | 直接执行编译后的代码 | `tsc --execute file.ts` |

#### 示例

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

### tsup 命令

`tsup` 命令是 Rusty TypeScript 的打包工具，用于将 TypeScript 代码打包为单个文件。

#### 基本用法

```bash
tsup [options] [files]
```

#### 常用选项

| 选项 | 描述 | 示例 |
|------|------|------|
| `--outDir` | 指定输出目录 | `tsup --outDir dist file.ts` |
| `--format` | 指定输出格式（esm、cjs） | `tsup --format esm file.ts` |
| `--minify` | 压缩输出代码 | `tsup --minify file.ts` |
| `--sourceMap` | 生成源映射文件 | `tsup --sourceMap file.ts` |
| `--watch` | 启用监视模式 | `tsup --watch file.ts` |

#### 示例

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

## typescript-macros

`typescript-macros` 是 Rusty TypeScript 提供的 procedural macros，用于从 Rust 代码生成 TypeScript 类型定义。

### 安装

在 `Cargo.toml` 文件中添加依赖：

```toml
[dependencies]
typescript-macros = { path = "compilers/typescript-macros" }
```

### TypescriptClass 宏

`TypescriptClass` 宏用于从 Rust 结构体生成 TypeScript 类定义。

#### 用法

```rust
use typescript_macros::TypescriptClass;

#[derive(TypescriptClass)]
struct User {
    id: u32,
    name: String,
    active: bool,
}
```

#### 生成的 TypeScript 代码

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

#### 获取生成的 TypeScript 代码

```rust
let ts_class = User::ts_class_definition();
println!("{}", ts_class);
```

### TypescriptFunction 宏

`TypescriptFunction` 宏用于从 Rust 函数生成 TypeScript 函数类型定义。

#### 用法

```rust
use typescript_macros::TypescriptFunction;

#[TypescriptFunction]
fn add(a: u32, b: u32) -> u32 {
    a + b
}
```

#### 生成的 TypeScript 代码

```typescript
type addFunction = (a: number, b: number) => number;
```

#### 获取生成的 TypeScript 代码

```rust
let ts_function = add_ts_function_definition();
println!("{}", ts_function);
```

### TypescriptInterface 宏

`TypescriptInterface` 宏用于从 Rust 结构体生成 TypeScript 接口定义。

#### 用法

```rust
use typescript_macros::TypescriptInterface;

#[derive(TypescriptInterface)]
struct User {
    id: u32,
    name: String,
    active: bool,
}
```

#### 生成的 TypeScript 代码

```typescript
interface User {
    id: number;
    name: string;
    active: boolean;
}
```

#### 获取生成的 TypeScript 代码

```rust
let ts_interface = User::ts_interface_definition();
println!("{}", ts_interface);
```

### TypescriptEnum 宏

`TypescriptEnum` 宏用于从 Rust 枚举生成 TypeScript 枚举定义。

#### 用法

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

#### 生成的 TypeScript 代码

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

#### 获取生成的 TypeScript 代码

```rust
let ts_color_enum = Color::ts_enum_definition();
println!("{}", ts_color_enum);

let ts_direction_enum = Direction::ts_enum_definition();
println!("{}", ts_direction_enum);
```

### TypescriptType 宏

`TypescriptType` 宏用于从 Rust 结构体生成 TypeScript 类型别名。

#### 用法

```rust
use typescript_macros::TypescriptType;

#[derive(TypescriptType)]
struct User {
    id: u32,
    name: String,
    active: bool,
}
```

#### 生成的 TypeScript 代码

```typescript
type User = {
    id: number;
    name: string;
    active: boolean;
};
```

#### 获取生成的 TypeScript 代码

```rust
let ts_type = User::ts_type_definition();
println!("{}", ts_type);
```

## 示例

### 完整示例

```rust
use typescript_macros::{TypescriptClass, TypescriptFunction, TypescriptInterface, TypescriptEnum, TypescriptType};

// 定义一个用户结构体
#[derive(TypescriptClass, TypescriptInterface, TypescriptType)]
struct User {
    id: u32,
    name: String,
    active: bool,
}

// 定义一个颜色枚举
#[derive(TypescriptEnum)]
enum Color {
    Red,
    Green,
    Blue,
}

// 定义一个方向枚举
#[derive(TypescriptEnum)]
enum Direction {
    Up = "UP",
    Down = "DOWN",
    Left = "LEFT",
    Right = "RIGHT",
}

// 定义一个添加函数
#[TypescriptFunction]
fn add(a: u32, b: u32) -> u32 {
    a + b
}

fn main() {
    // 打印生成的 TypeScript 代码
    println!("=== TypeScript Class ===");
    println!("{}", User::ts_class_definition());
    
    println!("\n=== TypeScript Interface ===");
    println!("{}", User::ts_interface_definition());
    
    println!("\n=== TypeScript Type ===");
    println!("{}", User::ts_type_definition());
    
    println!("\n=== TypeScript Enum (Color) ===");
    println!("{}", Color::ts_enum_definition());
    
    println!("\n=== TypeScript Enum (Direction) ===");
    println!("{}", Direction::ts_enum_definition());
    
    println!("\n=== TypeScript Function ===");
    println!("{}", add_ts_function_definition());
}
```

## 常见问题

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

**解决方案**：对于复杂类型，宏会默认使用 `any` 类型。如果需要更精确的类型映射，可以手动定义 TypeScript 类型。

## 贡献

如果您发现任何问题或有改进建议，请提交 Issue 或 Pull Request 到 GitHub 仓库。

## 许可证

Rusty TypeScript 工具链使用 MIT 许可证。
