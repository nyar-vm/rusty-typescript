# TypeScript Compiler

This is the main TypeScript compiler module for Rusty TypeScript. It provides the core functionality for compiling TypeScript code to various targets.

## Features

- TypeScript code parsing
- Type checking
- IR generation
- Code generation
- Virtual machine execution

## Usage

```rust
use typescript::Compiler;

let mut compiler = Compiler::new();
let result = compiler.compile("let x: number = 42;");
```

## Dependencies

- typescript-types: Type definitions for TypeScript
- typescript-ir: Intermediate representation for TypeScript
- oak-typescript: Oak framework TypeScript support
- oak-core: Oak framework core functionality
