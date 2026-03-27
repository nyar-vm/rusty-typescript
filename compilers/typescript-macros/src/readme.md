# TypeScript macros for Rust

This crate provides procedural macros for generating TypeScript type definitions
from Rust code, enabling seamless type-safe interaction between Rust and TypeScript.

## 可用宏

- `TypescriptClass` - 为 Rust 结构体生成 TypeScript 类定义
- `TypescriptInterface` - 为 Rust 结构体生成 TypeScript 接口定义
- `TypescriptEnum` - 为 Rust 枚举生成 TypeScript 枚举定义
- `TypescriptType` - 为 Rust 类型定义生成 TypeScript 类型别名
- `TypescriptUnion` - 为 Rust 枚举生成 TypeScript 联合类型
- `TypescriptGuard` - 为 Rust 类型生成 TypeScript 类型守卫
- `typescript_function` - 为 Rust 函数生成 TypeScript 函数类型定义
- `typescript_function_declaration` - 为 Rust 函数生成 TypeScript 函数声明
- `typescript_namespace` - 为 Rust 模块生成 TypeScript 命名空间

## 使用示例

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
```
