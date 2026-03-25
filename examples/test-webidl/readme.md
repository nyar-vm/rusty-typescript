# Test WebIDL Example

This example demonstrates parsing and converting WebIDL to TypeScript.

## Overview

The `test-webidl` example shows how to:
- Parse WebIDL definitions using the typescript-webidl crate
- Convert parsed WebIDL to TypeScript type definitions
- Handle parsing and conversion results

## Code Structure

- `src/main.rs`: Main Rust file that demonstrates WebIDL parsing and conversion

## Usage

To run this example:

```bash
# From the project root
cargo run --example test-webidl

# Or from the example directory
cd examples/test-webidl
cargo run
```

## What It Does

The example parses a simple WebIDL enum definition and converts it to TypeScript:

```webidl
enum TestEnum {
    "value1",
    "value2"
}
```

This demonstrates:
- WebIDL enum parsing
- Error handling for invalid WebIDL
- Conversion to TypeScript type definitions

## Expected Output

You should see output similar to:
- The original WebIDL input
- Parse result information
- The converted TypeScript code
