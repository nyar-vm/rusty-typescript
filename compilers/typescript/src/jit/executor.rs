//! JIT 执行器模块
//!
//! 使用 JIT 编译的代码执行程序。

use std::{collections::HashMap, fmt::Debug};

use typescript_ir::Program;
use typescript_types::{TsError, TsValue};

use crate::codegen::Instruction;

use super::{
    compiler::JITCompiler,
    optimizer::{JITOptimizer, OptimizationLevel},
};

/// JIT 执行器
///
/// 使用 JIT 编译的代码执行程序
pub struct JITExecutor {
    /// JIT 编译器
    jit_compiler: JITCompiler,
    /// JIT 优化器
    jit_optimizer: JITOptimizer,
}

impl Debug for JITExecutor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JITExecutor").field("jit_compiler", &"JITCompiler").field("jit_optimizer", &"JITOptimizer").finish()
    }
}

impl Clone for JITExecutor {
    fn clone(&self) -> Self {
        Self { jit_compiler: JITCompiler::new(), jit_optimizer: JITOptimizer::new(OptimizationLevel::Medium) }
    }
}

impl JITExecutor {
    /// 创建一个新的 JIT 执行器
    pub fn new() -> Self {
        Self { jit_compiler: JITCompiler::new(), jit_optimizer: JITOptimizer::new(OptimizationLevel::Medium) }
    }

    /// 执行程序
    pub fn execute(&mut self, program: &Program, globals: &HashMap<String, TsValue>) -> Result<TsValue, TsError> {
        let instructions = crate::codegen::ir_to_vm_instructions(program)?;
        let optimized_instructions = self.jit_optimizer.optimize_instructions(&instructions);

        let globals_vec: Vec<(String, TsValue)> = globals.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        let mut vm = crate::vm::VM::new(globals_vec);
        vm.execute_instructions(&optimized_instructions)
    }

    /// 执行函数
    pub fn execute_function(
        &mut self,
        function_name: &str,
        args: &[TsValue],
        instructions: &[Instruction],
    ) -> Result<TsValue, TsError> {
        let start_time = std::time::Instant::now();

        let complexity = self.calculate_function_complexity(instructions);
        self.jit_compiler.set_function_complexity(function_name, complexity);

        let result = if self.jit_compiler.should_compile(function_name) {
            let optimized_instructions = self.jit_optimizer.optimize_instructions(instructions);

            let compiled_fn = self.jit_compiler.compile_function(function_name, &optimized_instructions)?;

            compiled_fn(args)
        }
        else if let Some(compiled_fn) = self.jit_compiler.get_compiled_function(function_name) {
            compiled_fn(args)
        }
        else {
            let mut vm = crate::vm::VM::new(vec![]);
            TsValue::Undefined
        };

        let execution_time = start_time.elapsed().as_micros() as u64;
        self.jit_compiler.record_execution_time(function_name, execution_time);

        Ok(result)
    }

    /// 计算函数复杂度
    fn calculate_function_complexity(&self, instructions: &[Instruction]) -> usize {
        let mut complexity = 1;

        for instr in instructions {
            match instr {
                Instruction::Jump(_) | Instruction::JumpIfFalse(_) | Instruction::JumpLoop(_) => {
                    complexity += 2;
                }
                Instruction::Call(_) => {
                    complexity += 1;
                }
                Instruction::BinaryOp(_) | Instruction::UnaryOp(_) => {
                    complexity += 1;
                }
                _ => {
                    complexity += 1;
                }
            }
        }

        complexity
    }
}
