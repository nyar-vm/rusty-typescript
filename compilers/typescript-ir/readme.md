# TypeScript IR

This module provides the intermediate representation (IR) for the TypeScript compiler. It defines the data structures and operations used during the compilation process.

## Features

- AST to IR conversion
- IR optimization
- IR serialization/deserialization
- Type information preservation

## Usage

```rust
use typescript_ir::{IRModule, IRStatement};

let module = IRModule::new();
// Add statements to the module
```

## Dependencies

- typescript-types: Type definitions for TypeScript
- oak-typescript: Oak framework TypeScript support
