# Test WebIDL Lexer Example

This example demonstrates lexing WebIDL using the oak-idl lexer.

## Overview

The `test-webidl-lexer` example shows how to:
- Use the oak-idl lexer to tokenize WebIDL
- Work with lexer sessions and caching
- Inspect token results

## Code Structure

- `src/main.rs`: Main Rust file that demonstrates WebIDL lexing

## Usage

To run this example:

```bash
# From the project root
cargo run --example test-webidl-lexer

# Or from the example directory
cd examples/test-webidl-lexer
cargo run
```

## What It Does

The example lexes a simple WebIDL interface definition:

```webidl
interface TestInterface {
    attribute string name;
    attribute long age;
    void doSomething();
    string getName();
}
```

This demonstrates:
- WebIDL interface definition lexing
- Token extraction and inspection
- Working with oak-core lexer infrastructure

## Expected Output

You should see output similar to:
- The original WebIDL input
- Each token with its kind and text content
