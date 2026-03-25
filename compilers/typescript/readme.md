# 🎯 typescript

> Rusty TypeScript Core Compiler & Runtime 🦀⚡️

[![Crate](https://img.shields.io/badge/Crate-typescript-orange?logo=rust)](https://crates.io)
[![Rust Edition](https://img.shields.io/badge/Rust-2024%20Edition-blue)](https://www.rust-lang.org)

---

## 📋 Introduction

This is the core compiler package for **Rusty TypeScript**, providing full TypeScript compilation and execution capabilities. It integrates the complete compilation pipeline from lexical analysis and parsing to IR transformation and virtual machine execution.

### ✨ Core Features

| Module | Description | Status |
|--------|-------------|--------|
| 🔤 **Lexer/Parser** | Oak-based lexical and syntax analysis | ✅ Ready |
| 🔄 **IR** | Intermediate representation transformation | ✅ Ready |
| 🎯 **VM** | Virtual machine execution engine | ✅ Ready |
| 🔍 **Type Checker** | Type checking system | 🚧 In Development |
| 📤 **Codegen** | Code generation | 🚧 In Development |
| 🔌 **FFI/NAPI** | Node.js integration | ✅ Ready |
| 🗑️ **GC** | Garbage collector | ✅ Ready |

---

## 🚀 Quick Start

### Add Dependency

```toml
[dependencies]
typescript = { path = "../compilers/typescript" }
```

### Basic Usage

```rust
use typescript::TypeScript;

fn main() {
    // 🎬 Create runtime
    let mut ts = TypeScript::new();
    
    // 📝 Execute TypeScript code
    let result = ts.execute_script(r#"
        // Variable declaration
        let count: number = 42;
        
        // Function definition
        function greet(name: string): string {
            return `Hello, ${name}! 👋`;
        }
        
        // Call function
        console.log(greet("Rusty TypeScript"));
        
        // Return result
        count * 2
    "#);
    
    match result {
        Ok(value) => println!("✅ Result: {}", value),
        Err(e) => eprintln!("❌ Error: {}", e),
    }
}
```

### N-API Integration (Node.js)

```javascript
const { TypeScriptRuntime } = require('./index.node');

// Create runtime
const runtime = new TypeScriptRuntime();

// Execute code
const result = runtime.executeScript(`
    const fib = (n: number): number => {
        if (n <= 1) return n;
        return fib(n - 1) + fib(n - 2);
    };
    fib(10)
`);

console.log('🎉 Result:', result);
```

---

## 🔧 Advanced Usage

### Custom Global Variables

```rust
use typescript::TypeScript;
use typescript_types::{TsValue, ToTsValue};

let mut ts = TypeScript::new();

// Inject custom function
ts.register_global("myApi", |args: &[TsValue]| {
    println!("🎉 Called with args: {:?}", args);
    TsValue::String("Hello from Rust! 🦀".to_string())
});
```

### Error Handling

```rust
match ts.execute_script("invalid syntax !!!") {
    Ok(value) => println!("Result: {}", value),
    Err(TsError::SyntaxError(msg)) => {
        eprintln!("Syntax error: {}", msg);
    }
    Err(TsError::TypeError(msg)) => {
        eprintln!("Type error: {}", msg);
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

---

## 🧪 Testing

```bash
# Run all tests
cargo test --package typescript

# Run specific test
cargo test --package typescript -- test_lexer_parser

# Debug mode
cargo test --package typescript -- --nocapture
```

---

## 📚 Dependencies

- **typescript-types** - Type definitions
- **typescript-ir** - Intermediate representation
- **oak-typescript** - Parser (external)
- **oak-core** - Parsing core (external)
- **napi** - Node.js bindings
- **napi-derive** - NAPI macros

---

## 🤝 Contributing

We welcome issues and PRs! Please ensure:

1. ✅ Code passes `cargo clippy` checks
2. ✅ Code is formatted with `cargo fmt`
3. ✅ All tests pass with `cargo test`
4. ✅ Necessary documentation comments are added

---

## 📄 License

MIT License - see [LICENSE](../../license.md)

---

<div align="center">

**🦀 Built with Rust, optimized for performance ⚡️**

</div>
