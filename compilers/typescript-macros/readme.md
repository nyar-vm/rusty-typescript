# ✨ typescript-macros

> Rusty TypeScript Procedural Macro Library 🦀🔮

[![Crate](https://img.shields.io/badge/Crate-typescript--macros-orange?logo=rust)](https://crates.io)
[![Rust Edition](https://img.shields.io/badge/Rust-2024%20Edition-blue)](https://www.rust-lang.org)
[![Proc-Macro](https://img.shields.io/badge/Proc--Macro-derive%20%7C%20attr-purple)](https://doc.rust-lang.org/reference/procedural-macros.html)

---

## 📋 Introduction

**typescript-macros** is the procedural macro library for Rusty TypeScript, providing compile-time code generation capabilities to simplify TypeScript-related Rust code writing.

### ✨ Implemented Features

| Feature | Description | Status |
|---------|-------------|--------|
| 🏗️ **TypescriptClass** | Generate TypeScript class definitions from Rust structs | ✅ Implemented |
| 📝 **TypescriptFunction** | Generate TypeScript function type definitions from Rust functions | ✅ Implemented |

### ✨ Planned Features

| Feature | Description | Status |
|---------|-------------|--------|
| 🔮 **ts_bindgen** | Generate Rust bindings from TS types | 🚧 Planned |
| 🎨 **ts_embed** | Embed TS code in Rust | 🚧 Planned |
| 🔄 **ts_function** | Auto-generate TS function wrappers | 🚧 Planned |
| 📦 **ts_module** | Module export macros | 🚧 Planned |
| 🧪 **ts_test** | TypeScript testing macros | 🚧 Planned |

---

## 🚀 Quick Start

### Add Dependency

```toml
[dependencies]
typescript-macros = { path = "../compilers/typescript-macros" }
```

### ts_bindgen - Type Binding Generation

```rust
use typescript_macros::ts_bindgen;

// Generate Rust struct from TypeScript interface
#[ts_bindgen(r#"
    interface User {
        id: number;
        name: string;
        email?: string;
        active: boolean;
    }
"#)]
struct User {
    id: f64,
    name: String,
    email: Option<String>,
    active: bool,
}

fn main() {
    let user = User {
        id: 1.0,
        name: "Alice".to_string(),
        email: Some("alice@example.com".to_string()),
        active: true,
    };
    
    println!("👤 User: {:?}", user);
}
```

### ts_embed - Embed TypeScript

```rust
use typescript_macros::ts_embed;
use typescript::TypeScript;

fn main() {
    let mut ts = TypeScript::new();
    
    // Embed and execute TypeScript code at compile time
    let result = ts_embed! {
        r#"
        function fibonacci(n: number): number {
            if (n <= 1) return n;
            return fibonacci(n - 1) + fibonacci(n - 2);
        }
        
        fibonacci(20)
        "#
    };
    
    println!("🎯 20th Fibonacci number: {}", result);
}
```

### TypescriptClass - Generate TypeScript Class Definition

```rust
use typescript_macros::TypescriptClass;

#[derive(TypescriptClass)]
struct User {
    id: u32,
    name: String,
    active: bool,
}

fn main() {
    // Access the generated TypeScript class definition
    println!("TypeScript class definition:");
    println!("{}", User::TS_CLASS_DEFINITION);
}
```

### TypescriptFunction - Generate TypeScript Function Type Definition

```rust
use typescript_macros::TypescriptFunction;

#[TypescriptFunction]
fn add(a: u32, b: u32) -> u32 {
    a + b
}

#[TypescriptFunction]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

fn main() {
    // Access the generated TypeScript function type definitions
    println!("Add function type definition:");
    println!("{}", add::TS_FUNCTION_DEFINITION);
    
    println!("Greet function type definition:");
    println!("{}", greet::TS_FUNCTION_DEFINITION);
}
```

### ts_function - Function Export

```rust
use typescript_macros::ts_function;
use typescript::TypeScript;

struct MyRuntime;

#[ts_function]
impl MyRuntime {
    /// Export Rust function to TypeScript
    #[ts_export]
    fn add(a: f64, b: f64) -> f64 {
        a + b
    }
    
    /// Exported function with documentation
    #[ts_export(name = "greet", description = "Generate greeting")]
    fn generate_greeting(name: String) -> String {
        format!("Hello, {}! 🦀", name)
    }
}

fn main() {
    let mut ts = TypeScript::new();
    MyRuntime::register(&mut ts);
    
    // Now call Rust functions from TypeScript
    let result = ts.execute_script(r#"
        const sum = add(10, 20);
        const greeting = greet("TypeScript");
        console.log(sum, greeting);
    "#);
}
```

---

## 📦 Macro Details

### 🔮 `#[ts_bindgen]`

Generates Rust code from TypeScript type definitions.

#### Supported Types

| TypeScript | Rust |
|------------|------|
| `number` | `f64` |
| `string` | `String` |
| `boolean` | `bool` |
| `T[]` | `Vec<T>` |
| `T \| null` | `Option<T>` |
| `{ [k: string]: T }` | `HashMap<String, T>` |
| `interface` | `struct` |
| `type` | `type alias` |

#### Example

```rust
use typescript_macros::ts_bindgen;

#[ts_bindgen(r#"
    // Complex type definitions
    type Status = "active" | "inactive" | "pending";
    
    interface Config {
        port: number;
        host: string;
        ssl?: {
            cert: string;
            key: string;
        };
        middlewares: string[];
    }
    
    interface ApiResponse<T> {
        data: T;
        status: number;
        message?: string;
    }
"#)]
mod types {
    // Generated code:
    // - Status enum
    // - Config struct
    // - ApiResponse<T> generic struct
}
```

### 🎨 `ts_embed!`

Embeds TypeScript code at compile time.

```rust
use typescript_macros::ts_embed;

// Compile-time execution
const RESULT: f64 = ts_embed!("2 + 2");
// RESULT = 4

// Multi-line code
const GREETING: &str = ts_embed! {
    r#"
    const name = "Rust";
    `Hello from ${name}! 🦀`
    "#
};
```

### 🔄 `#[ts_function]`

Exports Rust functions to the TypeScript runtime.

```rust
use typescript_macros::{ts_function, ts_export};

#[ts_function]
mod math {
    #[ts_export]
    pub fn sqrt(x: f64) -> f64 {
        x.sqrt()
    }
    
    #[ts_export(name = "pow", description = "Calculate power")]
    pub fn power(base: f64, exp: f64) -> f64 {
        base.powf(exp)
    }
}

#[ts_function]
mod string_utils {
    #[ts_export]
    pub fn reverse(s: String) -> String {
        s.chars().rev().collect()
    }
    
    #[ts_export(async)]
    pub async fn fetch_data(url: String) -> String {
        // Async operation
        format!("Data from {}", url)
    }
}
```

---

## 🛠️ Development Guide

### Macro Development Process

```rust
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(TsBindgen, attributes(ts_type))]
pub fn ts_bindgen_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    
    // 1. Parse input
    // 2. Generate code
    // 3. Return TokenStream
    
    let expanded = quote! {
        // Generated code...
    };
    
    TokenStream::from(expanded)
}
```

### Debugging Macros

```bash
# View macro expansion
cargo expand --package typescript-macros

# Debug mode
cargo build --package typescript-macros 2>&1 | head -50
```

---

## 🧪 Testing

```bash
# Run macro tests
cargo test --package typescript-macros

# Test macro expansion
cargo test --package typescript-macros --test expand

# UI tests (compile error checking)
cargo test --package typescript-macros --test ui
```

---

## 📚 Dependencies

- **proc-macro2** - Token processing
- **quote** - Code generation
- **syn** - Rust code parsing
- **typescript-types** - Type system (dev)
- **typescript** - Runtime (dev)

---

## 🤝 Contributing

We welcome issues and PRs! Please ensure:

1. ✅ Macro code passes `cargo clippy` checks
2. ✅ Sufficient test cases are added
3. ✅ UI tests cover error scenarios
4. ✅ Documentation includes usage examples

---

## 📄 License

MIT License - see [LICENSE](../../license.md)

---

<div align="center">

**🦀 Macro magic, making code more concise ✨**

</div>
