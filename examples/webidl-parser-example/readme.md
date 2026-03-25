# WebIDL Parser Example

This example demonstrates comprehensive usage of the WebIDL parser and TypeScript converter.

## Overview

The `webidl-parser-example` shows how to:
- Parse simple and complex WebIDL definitions
- Convert WebIDL to TypeScript type definitions
- Read WebIDL from files and write TypeScript output
- Handle parsing errors gracefully

## Code Structure

- `src/main.rs`: Main Rust file with multiple WebIDL parsing examples
- `src/sample.idl`: Sample WebIDL file for demonstration

## Usage

To run this example:

```bash
# From the project root
cargo run --example webidl-parser-example

# Or from the example directory
cd examples/webidl-parser-example
cargo run
```

## What It Does

The example includes four demonstrations:

1. **Simple WebIDL String**: Parses a basic Point interface and converts it
2. **Complex Types**: Demonstrates dictionaries, enums, and interfaces with methods
3. **File I/O**: Reads from sample.idl and writes TypeScript to sample.d.ts
4. **Error Handling**: Shows how invalid WebIDL is handled

## Expected Output

You should see:
- Multiple WebIDL parsing examples with their TypeScript conversions
- A generated `sample.d.ts` file from the input WebIDL
- Demonstration of error handling for invalid syntax
