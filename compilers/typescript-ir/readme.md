# 🔄 typescript-ir

> Rusty TypeScript Intermediate Representation 🦀⚡️

[![Crate](https://img.shields.io/badge/Crate-typescript--ir-orange?logo=rust)](https://crates.io)
[![Rust Edition](https://img.shields.io/badge/Rust-2024%20Edition-blue)](https://www.rust-lang.org)

---

## 📋 Introduction

**typescript-ir** is the intermediate representation layer for the Rusty TypeScript compiler. It transforms AST into optimized IR and provides various optimization and analysis capabilities, serving as the bridge between parsing and code generation.

### ✨ Core Features

| Feature | Description | Status |
|---------|-------------|--------|
| 🔄 **AST → IR** | Transform Oak AST to internal IR | ✅ Ready |
| ⚡ **Constant Folding** | Compile-time constant expression evaluation | ✅ Ready |
| 🗑️ **Dead Code Elimination** | Remove unreachable code | ✅ Ready |
| 🌊 **Control Flow Graph** | CFG construction and analysis | ✅ Ready |
| 🎯 **Type Information** | Type-aware IR representation | ✅ Ready |
| 🔍 **Visitor Pattern** | Flexible IR traversal | ✅ Ready |

---

## 🚀 Quick Start

### Add Dependency

```toml
[dependencies]
typescript-ir = { path = "../compilers/typescript-ir" }
```

### Basic Usage

```rust
use typescript_ir::{Program, Expression, Statement, BinaryOp};
use typescript_types::TsValue;

fn main() {
    // 📝 Create IR program
    let program = Program {
        statements: vec![
            Statement::VariableDeclaration {
                name: "x".to_string(),
                ty: None,
                initializer: Some(Box::new(Expression::Binary {
                    left: Box::new(Expression::Literal(TsValue::Number(10.0))),
                    op: BinaryOp::Add,
                    right: Box::new(Expression::Literal(TsValue::Number(20.0))),
                })),
            }
        ],
    };
    
    println!("🎯 IR program created successfully: {:?}", program);
}
```

### AST to IR Conversion

```rust
use typescript_ir::ast_to_ir;
use oak_typescript::ast::TypeScriptRoot;

fn compile(ast: &TypeScriptRoot) -> Program {
    // 🔄 Convert Oak AST to IR
    let ir = ast_to_ir::program(ast);
    
    println!("✅ AST to IR conversion completed");
    ir
}
```

### Expression Optimization

```rust
use typescript_ir::{TypedExpression, Expression, TypeAnnotation};

fn optimize_expr(expr: Expression) -> TypedExpression {
    // 🎯 Create typed expression
    let typed = TypedExpression::new(expr, TypeAnnotation::Number);
    
    // ⚡ Perform optimization
    let optimized = typed.optimize();
    
    println!("🚀 Optimization completed, is constant: {}", optimized.is_constant);
    optimized
}
```

---

## ⚡ Optimization Features

### Constant Folding

```rust
use typescript_ir::{TypedExpression, Expression, TypeAnnotation, BinaryOp};
use typescript_types::TsValue;

// Input: 10 + 20 * 2
let expr = Expression::Binary {
    left: Box::new(Expression::Literal(TsValue::Number(10.0))),
    op: BinaryOp::Add,
    right: Box::new(Expression::Binary {
        left: Box::new(Expression::Literal(TsValue::Number(20.0))),
        op: BinaryOp::Mul,
        right: Box::new(Expression::Literal(TsValue::Number(2.0))),
    }),
};

let typed = TypedExpression::new(expr, TypeAnnotation::Number);
let optimized = typed.optimize();

// Output: 50 (constant value)
assert!(optimized.is_constant);
assert_eq!(optimized.constant_value, Some(TsValue::Number(50.0)));
```

### Control Flow Graph

```rust
use typescript_ir::ControlFlowGraph;

// Create CFG
let mut cfg = ControlFlowGraph::new();

// Add nodes
let node_a = cfg.add_node();
let node_b = cfg.add_node();
let node_c = cfg.add_node();

// Add edges
cfg.add_edge(node_a, node_b);
cfg.add_edge(node_b, node_c);

println!("🌊 CFG node count: {}", cfg.nodes.len());
```

---

## 🔍 Visitor Pattern

```rust
use typescript_ir::{Visitor, Expression, Statement, Program};

/// Custom visitor
struct MyVisitor {
    result: Vec<String>,
}

impl Visitor<()> for MyVisitor {
    fn visit_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Identifier(name) => {
                self.result.push(format!("Found identifier: {}", name));
            }
            _ => {}
        }
    }
    
    fn visit_statement(&mut self, stmt: &Statement) {
        // Handle statements...
    }
    
    fn visit_program(&mut self, program: &Program) {
        for stmt in &program.statements {
            self.visit_statement(stmt);
        }
    }
    
    // ... other methods
}
```

---

## 🧪 Testing

```bash
# Run all tests
cargo test --package typescript-ir

# Run optimization tests
cargo test --package typescript-ir -- optimization

# View output
cargo test --package typescript-ir -- --nocapture
```

---

## 📚 Dependencies

- **typescript-types** - Type definitions
- **oak-typescript** - AST definitions (re-export)
- **serde** - Serialization support

---

## 🤝 Contributing

We welcome issues and PRs! Please ensure:

1. ✅ Code passes `cargo clippy` checks
2. ✅ Code is formatted with `cargo fmt`
3. ✅ All tests pass with `cargo test`
4. ✅ New features include corresponding tests

---

## 📄 License

MIT License - see [LICENSE](../../license.md)

---

<div align="center">

**🦀 Intermediate representation, the bridge between compilation 🔄**

</div>
