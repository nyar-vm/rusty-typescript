# 📋 typescript-types

> Rusty TypeScript Type System Core 🦀🔷

[![Crate](https://img.shields.io/badge/Crate-typescript--types-orange?logo=rust)](https://crates.io)
[![Rust Edition](https://img.shields.io/badge/Rust-2024%20Edition-blue)](https://www.rust-lang.org)

---

## 📋 Introduction

**typescript-types** is the core type system package for Rusty TypeScript, defining the representation, type conversion, and error handling mechanisms for all TypeScript values.

### ✨ Core Features

| Feature | Description | Status |
|---------|-------------|--------|
| 🔷 **TsValue** | Complete TypeScript value type system | ✅ Ready |
| ⚠️ **TsError** | Standard error type definitions | ✅ Ready |
| 🔄 **ToTsValue** | Rust type to TS value conversion trait | ✅ Ready |
| 🧬 **Complex Types** | Support for functions, objects, arrays, promises, and more | ✅ Ready |
| 📊 **Type Checking** | Runtime type checking methods | ✅ Ready |
| 🎯 **Type Conversion** | Safe type-to-type conversions | ✅ Ready |

---

## 🚀 Quick Start

### Add Dependency

```toml
[dependencies]
typescript-types = { path = "../compilers/typescript-types" }
```

### Basic Usage

```rust
use typescript_types::{TsValue, TsError, ToTsValue};

fn main() {
    // 🔢 Create number
    let num = TsValue::Number(42.0);
    println!("Number: {}", num.to_string()); // "42"
    
    // 📝 Create string
    let str = TsValue::String("Hello, TypeScript! 🦀".to_string());
    println!("String: {}", str.to_string());
    
    // ✅ Create boolean
    let bool_val = TsValue::Boolean(true);
    println!("Boolean: {}", bool_val.to_boolean()); // true
    
    // 📦 Create object
    let obj = TsValue::Object(vec![
        ("name".to_string(), TsValue::String("Rusty".to_string())),
        ("version".to_string(), TsValue::Number(1.0)),
    ]);
    
    // 📚 Create array
    let arr = TsValue::Array(vec![
        TsValue::Number(1.0),
        TsValue::Number(2.0),
        TsValue::Number(3.0),
    ]);
    
    // 🔍 Type checking
    assert!(num.is_number());
    assert!(str.is_string());
    assert!(bool_val.is_boolean());
    assert!(obj.is_object());
    assert!(arr.is_array());
}
```

### Rust Type Conversion

```rust
use typescript_types::ToTsValue;

fn main() {
    // Auto convert Rust types to TsValue
    let num: f64 = 3.14;
    let ts_num = num.to_ts_value();
    // TsValue::Number(3.14)
    
    let text = "Hello";
    let ts_str = text.to_ts_value();
    // TsValue::String("Hello".to_string())
    
    let flag = true;
    let ts_bool = flag.to_ts_value();
    // TsValue::Boolean(true)
    
    // Vec conversion
    let vec = vec![1, 2, 3];
    let ts_arr = vec.to_ts_value();
    // TsValue::Array([...])
    
    // Option conversion
    let maybe: Option<i32> = Some(42);
    let ts_opt = maybe.to_ts_value();
    // TsValue::Number(42.0)
    
    let none: Option<i32> = None;
    let ts_none = none.to_ts_value();
    // TsValue::Null
}
```

### Error Handling

```rust
use typescript_types::TsError;

fn may_fail() -> Result<TsValue, TsError> {
    // Type error
    Err(TsError::TypeError(
        "Expected number, got string".to_string()
    ))
    
    // Reference error
    // Err(TsError::ReferenceError("x is not defined".to_string()))
    
    // Syntax error
    // Err(TsError::SyntaxError("Unexpected token".to_string()))
    
    // Range error
    // Err(TsError::RangeError("Index out of bounds".to_string()))
}
```

---

## 💡 Usage Examples

### Creating Complex Objects

```rust
use typescript_types::TsValue;

fn create_user() -> TsValue {
    TsValue::Object(vec![
        ("id".to_string(), TsValue::Number(1.0)),
        ("name".to_string(), TsValue::String("Alice".to_string())),
        ("active".to_string(), TsValue::Boolean(true)),
        ("tags".to_string(), TsValue::Array(vec![
            TsValue::String("admin".to_string()),
            TsValue::String("developer".to_string()),
        ])),
        ("metadata".to_string(), TsValue::Object(vec![
            ("created".to_string(), TsValue::Date(1700000000)),
            ("version".to_string(), TsValue::Number(2.0)),
        ])),
    ])
}
```

### Function Values

```rust
use typescript_types::{TsValue, ToTsValue};
use std::rc::Rc;

fn create_add_function() -> TsValue {
    TsValue::Function(Rc::new(|args: &[TsValue]| {
        if args.len() >= 2 {
            let a = args[0].to_number();
            let b = args[1].to_number();
            TsValue::Number(a + b)
        } else {
            TsValue::Error("Expected 2 arguments".to_string())
        }
    }))
}

// Usage
let add = create_add_function();
if let TsValue::Function(f) = add {
    let result = f(&[TsValue::Number(10.0), TsValue::Number(20.0)]);
    println!("10 + 20 = {}", result.to_number()); // 30
}
```

### Promise Simulation

```rust
use typescript_types::TsValue;

fn create_promise() -> TsValue {
    TsValue::Promise(Box::new(TsValue::String("Resolved!".to_string())))
}

let promise = create_promise();
assert!(promise.is_promise());
```

---

## 🧪 Testing

```bash
# Run all tests
cargo test --package typescript-types

# View output
cargo test --package typescript-types -- --nocapture
```

---

## 📚 Dependencies

- **No internal dependencies** - this is a foundational package

---

## 🤝 Contributing

We welcome issues and PRs! Please ensure:

1. ✅ Code passes `cargo clippy` checks
2. ✅ Code is formatted with `cargo fmt`
3. ✅ New types include complete method implementations
4. ✅ Corresponding unit tests are added

---

## 📄 License

MIT License - see [LICENSE](../../license.md)

---

<div align="center">

**🦀 Types are the soul of programs, let's treat them well 🔷**

</div>
