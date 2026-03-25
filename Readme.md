# 🦀 Rusty TypeScript

> A blazing-fast TypeScript compiler & toolchain rewritten in Rust ⚡ Faster, safer, more powerful!

[![Rust](https://img.shields.io/badge/Rust-2024%20Edition-orange?logo=rust)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](license.md)
[![Workspace](https://img.shields.io/badge/Workspace-pnpm%20%2B%20Cargo-purple)](https://pnpm.io)

---

## 🎯 What is this?

**Rusty TypeScript** is a high-performance TypeScript compiler and toolchain built from scratch in Rust. It aims to provide full compatibility with the official TypeScript compiler while delivering Rust-level performance and memory safety guarantees.

### ✨ Key Features

| Feature | Description |
|---------|-------------|
| 🚀 **Blazing Fast Compilation** | Rust-native performance eliminates long compilation waits, making your development workflow smoother than ever |
| 🛡️ **Memory Safety** | Zero-cost abstractions ensure no memory leaks or undefined behavior, giving you peace of mind |
| 🔧 **Full Compatibility** | Supports the latest TypeScript features, so you can use all your favorite syntax and patterns |
| 📦 **Modular Design** | Import only what you need, with flexible composition options for your specific use cases |
| 🌐 **Cross-Platform** | Works seamlessly on Windows, macOS, and Linux, so you can code anywhere |
| 🔌 **N-API Support** | Integrates smoothly with the Node.js ecosystem, expanding your tooling options |

---

## 🚀 Quick Start

### Prerequisites

- 🦀 [Rust](https://rustup.rs/) 1.80+
- 📦 [pnpm](https://pnpm.io/) 9.0+

### Build the Project

```bash
# Clone the repository
git clone https://github.com/nyar-vm/rusty-typescript.git
cd rusty-typescript

# Install dependencies
pnpm install

# Build Rust components
cargo build --release

# Build frontend
pnpm build
```

### Usage Example

```rust
use typescript::TypeScript;

fn main() {
    // Create TypeScript runtime
    let mut ts = TypeScript::new();
    
    // Execute TypeScript code
    let result = ts.execute_script(r#"
        let message: string = "Hello, Rusty TypeScript! 🦀";
        console.log(message);
        message
    "#);
    
    match result {
        Ok(value) => println!("✅ Execution successful: {}", value),
        Err(e) => println!("❌ Execution failed: {}", e),
    }
}
```

---

## 📦 Workspace Members

### Compiler Core

| Package | Description | Status |
|---------|-------------|--------|
| [typescript](compilers/typescript) | 🎯 Core compiler & runtime | ✅ Active Development |
| [typescript-ir](compilers/typescript-ir) | 🔄 Intermediate representation & optimization | ✅ Active Development |
| [typescript-types](compilers/typescript-types) | 📋 Type system definitions | ✅ Active Development |
| [typescript-lsp](compilers/typescript-lsp) | 🔍 Language Server Protocol | 🚧 Planned |
| [typescript-macros](compilers/typescript-macros) | ✨ Procedural macros support | 🚧 Planned |
| [typescript-tools](compilers/typescript-tools) | 🛠️ CLI toolset | 🚧 Planned |
| [typescript-wasi](compilers/typescript-wasi) | 🌐 WASI runtime | 🚧 Planned |

### Frontend Applications

| Package | Description | Status |
|---------|-------------|--------|
| [typescript-wasi](frontends/typescript-wasi) | 📦 WASI frontend package | ✅ Available |
| [homepage](frontends/homepage) | 🏠 Official website & playground | ✅ Active Development |

---

## 🛠️ Development Guide

### Common Commands

```bash
# 🦀 Rust Development
cargo build              # Build project
cargo test               # Run tests
cargo clippy             # Code linting
cargo fmt                # Format code

# 📦 Frontend Development
pnpm dev                 # Start development server
pnpm build               # Build production version
pnpm lint                # Code linting

# 🔧 Full Build
pnpm build:all           # Build all packages
```

### Project Configuration

- **Rust**: Using 2024 Edition, minimum version 1.80
- **Node.js**: Managed with pnpm workspace
- **Code Style**: Follows Rust official style guide + Biome configuration

---

## 🤝 Contribution Guide

We welcome all forms of contribution! Whether it's bug reports, feature suggestions, or code submissions.

### Submit an Issue

- 🐛 **Bug Report**: Provide reproduction steps and environment information
- 💡 **Feature Request**: Describe your use case and expected behavior
- 📖 **Documentation Improvement**: Point out unclear or missing documentation

### Submit a PR

1. 🍴 Fork this repository
2. 🌿 Create a feature branch (`git checkout -b feature/amazing-feature`)
3. 💾 Commit your changes (`git commit -m 'Add amazing feature'`)
4. 📤 Push to your branch (`git push origin feature/amazing-feature`)
5. 🔀 Create a Pull Request

---

## 📚 Documentation

- [📖 Architecture Documentation](documentation/architecture.md)
- [✨ Feature Overview](documentation/features.md)
- [🤝 Contribution Guide](documentation/contributing.md)
- [🇨🇳 Chinese Documentation](documentation/zh-hans/)

---

## 📄 License

This project is open source under the [MIT License](license.md).

---

## 🙏 Acknowledgments

- [Oak Framework](https://github.com/ygg-lang/oaks) - Powerful parser generation framework 🌳
- [Rust Community](https://www.rust-lang.org/community) - The best developer community 🦀
- [TypeScript Team](https://www.typescriptlang.org/) - Excellent type system 📘

---

<div align="center">

**Built with ❤️ and 🦀 Rust**

⭐ If this project helps you, please give us a Star!

</div>