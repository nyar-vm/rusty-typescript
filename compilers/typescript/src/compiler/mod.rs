//! TypeScript 编译器架构模块
//!
//! 定义了 TypeScript 编译器的多层级架构，包括词法分析、语法分析、语义分析、
//! 中间表示生成、优化和代码生成等层级。

pub mod cache;
pub mod codegen;
pub mod context;
pub mod executor;
pub mod ir_generator;
pub mod main;
pub mod optimizer;
pub mod parallel;
pub mod semantic;

use std::sync::Arc;
use typescript_types::{TsError, TsValue};

/// 编译层级枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompilationStage {
    /// 词法分析
    LexicalAnalysis,
    /// 语法分析
    SyntaxAnalysis,
    /// 语义分析
    SemanticAnalysis,
    /// 中间表示生成
    IRGeneration,
    /// 优化
    Optimization,
    /// 代码生成
    CodeGeneration,
    /// 执行
    Execution,
}

/// 编译结果类型
#[derive(Debug, Clone)]
pub enum CompilationResult<T>
where
    T: Clone,
{
    /// 成功
    Success(T),
    /// 失败
    Error(TsError),
    /// 跳过（增量编译时）
    Skipped,
}

/// 编译器接口
pub trait Compiler {
    /// 编译方法
    fn compile(&mut self, input: &str) -> CompilationResult<TsValue>;

    /// 重置编译器状态
    fn reset(&mut self);

    /// 获取当前编译阶段
    fn current_stage(&self) -> CompilationStage;
}

/// 编译单元
#[derive(Debug, Clone)]
pub struct CompilationUnit {
    /// 源文件路径
    pub source_path: Option<String>,
    /// 源文件内容
    pub source_content: String,
    /// 编译结果
    pub result: Option<CompilationResult<TsValue>>,
    /// 依赖项
    pub dependencies: Vec<String>,
}

impl CompilationUnit {
    /// 创建新的编译单元
    pub fn new(source_content: String) -> Self {
        Self { source_path: None, source_content, result: None, dependencies: Vec::new() }
    }

    /// 创建带路径的编译单元
    pub fn with_path(path: String, source_content: String) -> Self {
        Self { source_path: Some(path), source_content, result: None, dependencies: Vec::new() }
    }
}

/// 编译任务
#[derive(Debug, Clone)]
pub struct CompilationTask {
    /// 任务 ID
    pub id: String,
    /// 编译单元
    pub unit: CompilationUnit,
    /// 优先级
    pub priority: u32,
}

impl CompilationTask {
    /// 创建新的编译任务
    pub fn new(id: String, unit: CompilationUnit, priority: u32) -> Self {
        Self { id, unit, priority }
    }
}
