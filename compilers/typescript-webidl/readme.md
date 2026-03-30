# TypeScript WebIDL Compiler

The TypeScript WebIDL Compiler is a tool for converting WebIDL to TypeScript type definitions, built on top of the Oak IDL library.

## ✨ Features

- **WebIDL Parsing**: Uses Oak IDL library to parse WebIDL strings
- **TypeScript Conversion**: Converts parsed WebIDL AST to TypeScript type definitions
- **Error Handling**: Provides detailed error information
- **File Support**: Reads and parses WebIDL from files

## 🚀 Quick Start

### Installation

Add the dependency to your `Cargo.toml` file:

```toml
dependencies = {
    typescript-webidl = { path = "path/to/typescript-webidl" }
}
```

### Usage Example

```rust
use typescript_webidl::{parse, convert_to_typescript};

fn main() {
    let webidl = r#"
        interface TestInterface {
            void test();
        };
    "#;
    
    match parse(webidl) {
        Ok(root) => {
            let typescript = convert_to_typescript(&root);
            println!("{}", typescript);
        }
        Err(error) => {
            println!("Error: {}", error);
        }
    }
}
```

## 📁 Project Structure

- `src/lib.rs`：Main entry file, contains parsing and conversion functions
- `src/converter/`：WebIDL to TypeScript conversion module
- `src/types/`：Type definition module
- `src/type_checker/`：Type checking module
- `tests/`：Test files

## 📚 Dependencies

- **Oak IDL**：For WebIDL parsing
- **Serde**：For serialization and deserialization
- **Regex**：For fallback WebIDL parsing

## ⚠️ Notes

- Due to the Oak IDL library's `parse` function potentially returning an empty `items` vector, the project includes a regex-based fallback parser
- The fallback parser supports basic WebIDL syntax, including interfaces, operations, and parameters
- For complex WebIDL syntax, it is recommended to use a standard WebIDL parser

## 🤝 Contributing

We welcome issues and pull requests!

## 📄 License

MIT
