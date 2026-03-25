//! 优化器模块
//!
//! 负责对中间表示（IR）进行优化，提高代码执行效率。

use crate::compiler::{CompilationResult, CompilationStage};
use typescript_ir::{Program, TypedProgram, optimize_full};
use typescript_types::TsError;

/// 优化级别
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum OptimizationLevel {
    /// 无优化
    None,
    /// 基本优化
    Basic,
    /// 中级优化
    Medium,
    /// 高级优化
    High,
}

/// 优化器
#[derive(Debug, Clone)]
pub struct Optimizer {
    /// 当前编译阶段
    current_stage: CompilationStage,
    /// 优化级别
    optimization_level: OptimizationLevel,
}

impl Optimizer {
    /// 创建新的优化器
    pub fn new() -> Self {
        Self { current_stage: CompilationStage::Optimization, optimization_level: OptimizationLevel::High }
    }

    /// 创建指定优化级别的优化器
    pub fn with_level(level: OptimizationLevel) -> Self {
        Self { current_stage: CompilationStage::Optimization, optimization_level: level }
    }

    /// 优化中间表示
    pub fn optimize(&mut self, ir: &Program) -> CompilationResult<Program> {
        self.current_stage = CompilationStage::Optimization;

        // 根据优化级别选择不同的优化策略
        let optimized_ir = optimize_full(ir);

        CompilationResult::Success(optimized_ir)
    }

    /// 获取当前编译阶段
    pub fn current_stage(&self) -> CompilationStage {
        self.current_stage
    }

    /// 获取优化级别
    pub fn optimization_level(&self) -> OptimizationLevel {
        self.optimization_level
    }

    /// 设置优化级别
    pub fn set_optimization_level(&mut self, level: OptimizationLevel) {
        self.optimization_level = level;
    }

    /// 重置优化器
    pub fn reset(&mut self) {
        self.current_stage = CompilationStage::Optimization;
        self.optimization_level = OptimizationLevel::High;
    }
}

impl Default for Optimizer {
    fn default() -> Self {
        Self::new()
    }
}
