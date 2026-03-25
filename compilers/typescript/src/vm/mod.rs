#![doc = include_str!("readme.md")]

pub mod builtins;
pub mod exception;
pub mod frame;
pub mod function;
pub mod module;
pub mod perf;
pub mod types;
pub mod vm;

// 重新导出主要类型
pub use builtins::Builtins;
pub use exception::ExceptionHandler;
pub use frame::CallFrame;
pub use function::Function;
pub use module::ModuleInstance;
pub use perf::PerformanceMonitor;
pub use types::*;
pub use vm::VM;
