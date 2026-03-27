#![doc = include_str!("readme.md")]

mod compiler;
mod executor;
mod optimizer;
mod performance_test;

pub use compiler::{
    AdaptiveThresholdConfig, CompilationReason, CompileStatus, HotFunctionInfo, HotFunctionPriorityQueue, JITCompiler,
    JITEvent, JITEventCallback, JITStatistics, PriorityEntry,
};
pub use executor::JITExecutor;
pub use optimizer::{JITOptimizer, OptimizationLevel};
pub use performance_test::{PerformanceTestResult, PerformanceTestSuite};
