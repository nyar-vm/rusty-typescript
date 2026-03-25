# Basic Types Example

This example demonstrates the basic usage of the Rusty TypeScript compiler and runtime.

## Overview

The `basic_types` example shows how to:
- Create a TypeScript instance
- Execute TypeScript code with basic types (numbers, strings)
- Handle execution results and errors

## Code Structure

- `src/main.rs`: Main Rust file that sets up and uses the TypeScript runtime

## Usage

To run this example:

```bash
# From the project root
cargo run --example basic_types

# Or from the example directory
cd examples/basic_types
cargo run
```

## What It Does

The example executes the following TypeScript code:

```typescript
let x = 10; let y = 'hello'; console.log(x, y); x + y
```

This demonstrates:
- Variable declaration with `let`
- Basic types: number (`10`) and string (`'hello'`)
- Console output with `console.log()`
- Type coercion (adding a number and a string)

## Expected Output

You should see output similar to:

```
Test result: "10hello"
```

This shows that the TypeScript code was successfully executed and the result of `x + y` (which is string concatenation) was returned.
