# TypeScript Types

This module provides the type definitions for the TypeScript compiler. It defines the core data types used throughout the compilation process.

## Features

- TypeScript type representations
- Type checking utilities
- Error type definitions
- Serialization support

## Usage

```rust
use typescript_types::{TsValue, TsError};

let value = TsValue::Number(42.0);
// Use the value in the compiler
```

## Dependencies

- serde: Serialization/deserialization support
