# 🔍 typescript-lsp

> Rusty TypeScript Language Server Protocol (LSP) Implementation 🦀🔧

[![Crate](https://img.shields.io/badge/Crate-typescript--lsp-orange?logo=rust)](https://crates.io)
[![Rust Edition](https://img.shields.io/badge/Rust-2024%20Edition-blue)](https://www.rust-lang.org)
[![LSP](https://img.shields.io/badge/LSP-3.17-green)](https://microsoft.github.io/language-server-protocol/)

---

## 📋 Introduction

**typescript-lsp** is the Language Server Protocol (LSP) implementation for Rusty TypeScript, providing intelligent code completion, diagnostics, definition jumping, and other features for editors.

### ✨ Core Features

| Feature | Description | Status |
|---------|-------------|--------|
| 📝 **Code Completion** | Intelligent auto-completion suggestions | 🚧 Planned |
| 🔍 **Go to Definition** | Jump to symbol definitions | 🚧 Planned |
| 🔎 **Find References** | Locate all references to a symbol | 🚧 Planned |
| ⚠️ **Diagnostics** | Real-time errors and warnings | 🚧 Planned |
| 🔄 **Rename** | Safe rename refactoring | 🚧 Planned |
| 📐 **Code Formatting** | Automatic code formatting | 🚧 Planned |
| 💡 **Code Actions** | Quick fixes and code operations | 🚧 Planned |
| 🎨 **Syntax Highlighting** | Semantic syntax highlighting | 🚧 Planned |

---

## 🚀 Quick Start

### Add Dependency

```toml
[dependencies]
typescript-lsp = { path = "../compilers/typescript-lsp" }
```

### Basic Usage

```rust
use typescript_lsp::LspServer;

#[tokio::main]
async fn main() {
    // 🚀 Start LSP server
    let server = LspServer::new();
    
    // Listen on stdin/stdout for LSP communication
    server.run().await;
}
```

### VS Code Integration

```json
// .vscode/settings.json
{
    "typescript.tsdk": "./node_modules/typescript/lib",
    "rusty-typescript.server.path": "./target/release/typescript-lsp"
}
```

---

## 💻 Editor Configuration

### VS Code

After installing the `rusty-typescript` extension, configure in `settings.json`:

```json
{
    "rusty-typescript.trace.server": "verbose",
    "rusty-typescript.server.path": "${workspaceFolder}/target/release/typescript-lsp",
    "[typescript]": {
        "editor.defaultFormatter": "rusty-typescript.rusty-typescript"
    }
}
```

### Neovim (nvim-lspconfig)

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

-- Configure Rusty TypeScript LSP
configs.rusty_typescript = {
    default_config = {
        cmd = { 'typescript-lsp' },
        filetypes = { 'typescript', 'typescriptreact', 'javascript', 'javascriptreact' },
        root_dir = lspconfig.util.root_pattern('package.json', 'tsconfig.json', '.git'),
        settings = {},
    },
}

lspconfig.rusty_typescript.setup{}
```

### Emacs (lsp-mode)

```elisp
(use-package lsp-mode
  :hook ((typescript-mode . lsp)
         (typescript-ts-mode . lsp))
  :commands lsp
  :config
  (add-to-list 'lsp-language-id-configuration '(typescript-ts-mode . "typescript"))
  (lsp-register-client
   (make-lsp-client
    :new-connection (lsp-stdio-connection "typescript-lsp")
    :activation-fn (lsp-activate-on "typescript")
    :server-id 'rusty-typescript)))
```

---

## 🧪 Development

### Run LSP Server

```bash
# Development mode (stdio)
cargo run --package typescript-lsp

# Debug mode
RUST_LOG=debug cargo run --package typescript-lsp
```

### Testing

```bash
# Run unit tests
cargo test --package typescript-lsp

# Integration tests (requires editor)
cargo test --package typescript-lsp --test integration
```

---

## 📚 Dependencies

- **typescript** - Core compiler
- **typescript-types** - Type system
- **typescript-ir** - Intermediate representation
- **oak-lsp** - LSP framework
- **tokio** - Async runtime
- **serde_json** - JSON processing

---

## 🤝 Contributing

We welcome issues and PRs! Please ensure:

1. ✅ Code passes `cargo clippy` checks
2. ✅ Code is formatted with `cargo fmt`
3. ✅ All tests pass with `cargo test`
4. ✅ LSP specification compatibility tests pass

---

## 📄 License

MIT License - see [LICENSE](../../license.md)

---

<div align="center">

**🦀 Intelligent editor support, elevating your development experience 🔍**

</div>
