//! 虚拟机模块
//!
//! 提供 TypeScript 代码的执行环境，包括异常处理、模块系统、内置函数库等。
//!
//! 本模块已拆分为以下子模块：
//! - types: 类型定义
//! - frame: 调用帧
//! - exception: 异常处理
//! - module: 模块系统
//! - function: 函数
//! - builtins: 内置对象
//! - perf: 性能监控
//! - vm: 主虚拟机

pub mod types;
pub mod frame;
pub mod exception;
pub mod module;
pub mod function;
pub mod builtins;
pub mod perf;
pub mod vm;

// 重新导出主要类型
pub use types::*;
pub use frame::CallFrame;
pub use exception::ExceptionHandler;
pub use module::ModuleInstance;
pub use function::Function;
pub use builtins::Builtins;
pub use perf::PerformanceMonitor;
pub use vm::VM;
