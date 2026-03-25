# 🌐 typescript-wasi

> Rusty TypeScript WASI Runtime Support 🦀🌍

[![Crate](https://img.shields.io/badge/Crate-typescript--wasi-orange?logo=rust)](https://crates.io)
[![Rust Edition](https://img.shields.io/badge/Rust-2024%20Edition-blue)](https://www.rust-lang.org)
[![WASI](https://img.shields.io/badge/WASI-Preview%202-green)](https://wasi.dev/)

---

## 📋 Introduction

**typescript-wasi** is the WASI (WebAssembly System Interface) runtime support package for Rusty TypeScript, allowing TypeScript code to safely execute system calls in WebAssembly environments.

### ✨ Planned Features

| Feature | Description | Status |
|---------|-------------|--------|
| 🌐 **WASI Runtime** | Complete WASI implementation | 🚧 Planned |
| 📁 **Filesystem** | Secure file access | 🚧 Planned |
| 🕐 **Clock/Time** | Time-related operations | 🚧 Planned |
| 🌐 **Network** | HTTP client support | 🚧 Planned |
| 🔐 **Random** | Cryptographically secure random numbers | 🚧 Planned |
| 👤 **Environment** | Environment access control | 🚧 Planned |
| 🖥️ **Command Line** | Argument passing support | 🚧 Planned |

---

## 🚀 Quick Start

### Add Dependency

```toml
[dependencies]
typescript-wasi = { path = "../compilers/typescript-wasi" }
```

### Basic Usage

```rust
use typescript_wasi::WasiRuntime;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 🌐 Create WASI runtime
    let mut runtime = WasiRuntime::new()
        .preopen_dir("./data")?  // Pre-open directory
        .env("RUST_LOG", "debug")  // Set environment variable
        .args(&["script.ts", "arg1", "arg2"]);  // Pass arguments
    
    // 📝 Load and execute TypeScript WASM module
    let wasm_bytes = std::fs::read("typescript.wasm")?;
    let result = runtime.run(&wasm_bytes)?;
    
    println!("✅ Execution result: {:?}", result);
    Ok(())
}
```

### Sandbox Configuration

```rust
use typescript_wasi::{WasiRuntime, WasiPermissions};

fn main() {
    // 🔐 Configure permissions
    let permissions = WasiPermissions::new()
        .allow_read(true)
        .allow_write(true)
        .allow_network(false)  // Disable network
        .allow_env(&["HOME", "USER"]);  // Allow specific env vars
    
    let runtime = WasiRuntime::new()
        .permissions(permissions)
        .preopen_dir("./sandbox");
    
    // Execute in restricted environment
    let result = runtime.run(&wasm_bytes);
}
```

---

## 🔧 WASI Feature Support

### 📁 Filesystem

```rust
use typescript_wasi::WasiRuntime;

let runtime = WasiRuntime::new()
    // Map host directories to guest
    .preopen_dir("./data")
    .preopen_dir_with_name("./logs", "/var/log");

// TypeScript code:
// const content = Deno.readTextFileSync("/data/input.txt");
// Deno.writeTextFileSync("/var/log/output.txt", "Hello WASI!");
```

### 🕐 Clock and Time

```rust
// TypeScript code:
const now = Date.now();
console.log(`Current timestamp: ${now}`);

const start = performance.now();
// ... perform operations
const end = performance.now();
console.log(`Time taken: ${end - start}ms`);
```

### 🌐 Network (Restricted)

```rust
let runtime = WasiRuntime::new()
    .permissions(
        WasiPermissions::new()
            .allow_network(true)
            .allow_hosts(&["api.example.com", "cdn.example.com"])
    );

// TypeScript code:
// const response = await fetch("https://api.example.com/data");
// const data = await response.json();
```

### 🔐 Random Numbers

```rust
// TypeScript code:
const randomBytes = crypto.getRandomValues(new Uint8Array(32));
console.log("Random bytes:", randomBytes);
```

---

## 🧪 Use Cases

### 1. Plugin Systems

```rust
use typescript_wasi::WasiRuntime;

/// Secure plugin execution environment
fn execute_plugin(plugin_wasm: &[u8], input: &str) -> Result<String> {
    let mut runtime = WasiRuntime::new()
        .permissions(WasiPermissions::readonly())  // Read-only permissions
        .preopen_dir("./plugins/data");
    
    let result = runtime.run(plugin_wasm)?;
    Ok(result.output)
}
```

### 2. Edge Computing

```rust
use typescript_wasi::WasiRuntime;

/// Execute user code at edge nodes
fn edge_function(handler_wasm: &[u8], request: Request) -> Result<Response> {
    let mut runtime = WasiRuntime::new()
        .permissions(
            WasiPermissions::new()
                .allow_network(true)
                .max_execution_time(Duration::from_secs(30))
                .max_memory(128 * 1024 * 1024),  // 128MB
        );
    
    // Inject request data
    runtime.set_env("REQUEST_BODY", &request.body)?;
    
    let result = runtime.run(handler_wasm)?;
    Ok(Response::new(result.output))
}
```

### 3. Code Sandboxing

```rust
use typescript_wasi::{WasiRuntime, WasiPermissions};

/// Fully isolated code execution
fn sandbox_execute(untrusted_code: &[u8]) -> Result<String> {
    let mut runtime = WasiRuntime::new()
        .permissions(WasiPermissions::none())  // No permissions
        .max_memory(64 * 1024 * 1024)  // 64MB memory limit
        .max_execution_time(Duration::from_secs(5));
    
    runtime.run(untrusted_code)
        .map(|r| r.output)
        .map_err(|e| e.into())
}
```

---

## 🔧 Development

### Build WASM Modules

```bash
# Compile TypeScript to WASM
cargo build --package typescript --target wasm32-wasi --release

# Optimize WASM size
wasm-opt -O3 target/wasm32-wasi/release/typescript.wasm -o typescript.wasm
```

### Testing

```bash
# Run WASI tests
cargo test --package typescript-wasi

# Integration tests
cargo test --package typescript-wasi --test integration

# With wasmtime backend
cargo test --package typescript-wasi --features wasmtime

# With wasmer backend
cargo test --package typescript-wasi --features wasmer
```

---

## 📚 Dependencies

- **typescript** - Core compiler (WASM target)
- **typescript-types** - Type system
- **wasmtime** - WASM runtime (optional)
- **wasmer** - WASM runtime (optional)
- **wasi-common** - WASI standard implementation
- **cap-std** - Capability-safe standard library
- **tokio** - Async runtime

### Feature Flags

```toml
[features]
default = ["wasmtime"]
wasmtime = ["dep:wasmtime", "dep:wasmtime-wasi"]
wasmer = ["dep:wasmer", "dep:wasmer-wasi"]
```

---

## 🤝 Contributing

We welcome issues and PRs! Please ensure:

1. ✅ Code passes `cargo clippy` checks
2. ✅ Sandbox security tests pass
3. ✅ WASI specification compatibility tests pass
4. ✅ Performance benchmark tests are added

---

## 📄 License

MIT License - see [LICENSE](../../license.md)

---

<div align="center">

**🦀 Secure WebAssembly runtime, making TypeScript everywhere 🌐**

</div>
