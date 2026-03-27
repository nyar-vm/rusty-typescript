//! JIT 编译模块
//!
//! 提供即时编译功能，将热点代码编译为机器码，提高执行性能。
//!
//! # 模块结构
//!
//! - [`compiler`] - JIT 编译器核心，负责热点检测和函数编译
//! - [`optimizer`] - JIT 优化器，提供指令级别的优化
//! - [`executor`] - JIT 执行器，执行 JIT 编译后的代码

mod compiler;
mod executor;
mod optimizer;

pub use compiler::{HotFunctionInfo, JITCompiler};
pub use executor::JITExecutor;
pub use optimizer::{JITOptimizer, OptimizationLevel};
