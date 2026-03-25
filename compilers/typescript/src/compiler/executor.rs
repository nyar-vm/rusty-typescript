//! 执行器模块
//!
//! 负责执行编译后的代码，包括解释执行和 JIT 编译执行。

use crate::{
    compiler::{CompilationResult, CompilationStage},
    jit::JITExecutor,
};
use std::{collections::HashMap, sync::Arc};
use typescript_ir::Program;
use typescript_types::{TsError, TsValue};

/// 执行器
#[derive(Debug, Clone)]
pub struct Executor {
    /// 当前编译阶段
    current_stage: CompilationStage,
    /// JIT 执行器
    jit_executor: Arc<JITExecutor>,
    /// 全局变量
    globals: HashMap<String, TsValue>,
}

impl Executor {
    /// 创建新的执行器
    pub fn new() -> Self {
        Self { current_stage: CompilationStage::Execution, jit_executor: Arc::new(JITExecutor::new()), globals: HashMap::new() }
    }

    /// 执行编译后的代码
    pub fn execute(&mut self, ir: &Program) -> CompilationResult<TsValue> {
        self.current_stage = CompilationStage::Execution;

        // 使用 JIT 执行器执行代码
        let mut executor = (*self.jit_executor).clone();
        match executor.execute(ir, &self.globals) {
            Ok(value) => CompilationResult::Success(value),
            Err(error) => CompilationResult::Error(error),
        }
    }

    /// 设置全局变量
    pub fn set_global(&mut self, name: String, value: TsValue) {
        self.globals.insert(name, value);
    }

    /// 获取全局变量
    pub fn get_global(&self, name: &str) -> Option<&TsValue> {
        self.globals.get(name)
    }

    /// 获取当前编译阶段
    pub fn current_stage(&self) -> CompilationStage {
        self.current_stage
    }

    /// 重置执行器
    pub fn reset(&mut self) {
        self.current_stage = CompilationStage::Execution;
        self.jit_executor = Arc::new(JITExecutor::new());
        self.globals.clear();
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}
