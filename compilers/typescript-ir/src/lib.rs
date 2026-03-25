#![warn(missing_docs)]

//! TypeScript IR (Intermediate Representation) 库
//!
//! 这个库提供了 TypeScript 的中间表示，用于编译器优化和代码生成。

mod optimization;
pub mod performance;

// 子模块
pub mod ast_to_ir;
pub mod expression;
pub mod program;
pub mod statement;
pub mod types;
pub mod visitor;

// Reexport oak-typescript AST
pub use oak_typescript::ast;

pub use optimization::*;
pub use performance::*;

// 从子模块重新导出常用类型
pub use types::{AssignmentOp, BinaryOp, PrimitiveType, TypeAnnotation, TypeParameter, UnaryOp};

pub use expression::{Expression, TypedExpression};

pub use statement::{ImportSpecifier, Statement, TypedStatement};

pub use program::{
    CFGNode, Class, ControlFlowGraph, Decorator, DecoratorTarget, EnumMember, Function, InterfaceMember, Method, Module,
    Program, TypedFunction, TypedModule, TypedProgram,
};

pub use visitor::{SimpleVisitor, Visitor};
