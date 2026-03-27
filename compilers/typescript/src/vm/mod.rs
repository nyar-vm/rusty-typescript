#![doc = include_str!("readme.md")]

pub mod builtins;
pub mod exception;
pub mod exception_ops;
pub mod executor;
pub mod frame;
pub mod function;
pub mod function_ops;
pub mod memory;
pub mod module;
pub mod module_ops;
pub mod object_ops;
pub mod operations;
pub mod perf;
pub mod types;
pub mod vm;

pub use builtins::Builtins;
pub use exception::ExceptionHandler;
pub use exception_ops::ExceptionOperations;
pub use executor::{ExecutionContext, InstructionExecutor};
pub use frame::CallFrame;
pub use function::Function;
pub use function_ops::{ClassOperations, FunctionOperations, TypeOperations};
pub use memory::MemoryManager;
pub use module::ModuleInstance;
pub use module_ops::ModuleOperations;
pub use object_ops::{ArrayOperations, ObjectOperations};
pub use operations::{BinaryOperations, UnaryOperations};
pub use perf::PerformanceMonitor;
pub use types::*;
pub use vm::VM;
