//! JIT 编译器核心模块
//!
//! 提供热点检测和函数编译功能。

use std::{collections::HashMap, rc::Rc};

use typescript_types::{TsError, TsValue};

use crate::codegen::Instruction;

use super::optimizer::{JITOptimizer, OptimizationLevel};

/// JIT 编译器
///
/// 将热点代码编译为机器码，提高执行性能
pub struct JITCompiler {
    /// 编译缓存，存储已编译的函数
    compiled_functions: HashMap<String, Rc<dyn Fn(&[TsValue]) -> TsValue>>,
    /// 热点函数信息
    hot_functions: HashMap<String, HotFunctionInfo>,
    /// 热点阈值
    hot_threshold: usize,
    /// 时间阈值（微秒）
    time_threshold: u64,
}

/// 热点函数信息
#[derive(Debug, Clone)]
pub struct HotFunctionInfo {
    /// 执行次数
    pub call_count: usize,
    /// 总执行时间（微秒）
    pub total_time: u64,
    /// 平均执行时间（微秒）
    pub avg_time: u64,
    /// 最后执行时间（微秒）
    pub last_time: u64,
    /// 执行时间滑动窗口
    pub time_window: Vec<u64>,
    /// 代码复杂度
    pub complexity: usize,
}

impl JITCompiler {
    /// 创建一个新的 JIT 编译器
    pub fn new() -> Self {
        Self { compiled_functions: HashMap::new(), hot_functions: HashMap::new(), hot_threshold: 100, time_threshold: 100 }
    }

    /// 检查函数是否需要 JIT 编译
    pub fn should_compile(&mut self, function_name: &str) -> bool {
        let info = self.hot_functions.entry(function_name.to_string()).or_insert(HotFunctionInfo {
            call_count: 0,
            total_time: 0,
            avg_time: 0,
            last_time: 0,
            time_window: Vec::with_capacity(10),
            complexity: 1,
        });
        info.call_count += 1;

        let window_avg =
            if !info.time_window.is_empty() { info.time_window.iter().sum::<u64>() / info.time_window.len() as u64 } else { 0 };

        let adaptive_threshold = if info.complexity > 10 { self.hot_threshold / 2 } else { self.hot_threshold };

        let should_compile = (info.call_count >= adaptive_threshold
            || info.avg_time >= self.time_threshold
            || window_avg >= self.time_threshold)
            && !self.compiled_functions.contains_key(function_name);

        should_compile
    }

    /// 记录函数执行时间
    pub fn record_execution_time(&mut self, function_name: &str, execution_time: u64) {
        if let Some(info) = self.hot_functions.get_mut(function_name) {
            info.total_time += execution_time;
            info.avg_time = info.total_time / info.call_count as u64;
            info.last_time = execution_time;

            info.time_window.push(execution_time);
            if info.time_window.len() > 10 {
                info.time_window.remove(0);
            }
        }
    }

    /// 设置函数复杂度
    pub fn set_function_complexity(&mut self, function_name: &str, complexity: usize) {
        let info = self.hot_functions.entry(function_name.to_string()).or_insert(HotFunctionInfo {
            call_count: 0,
            total_time: 0,
            avg_time: 0,
            last_time: 0,
            time_window: Vec::with_capacity(10),
            complexity: 1,
        });
        info.complexity = complexity;
    }

    /// 编译函数为机器码
    pub fn compile_function(
        &mut self,
        function_name: &str,
        instructions: &[Instruction],
    ) -> Result<Rc<dyn Fn(&[TsValue]) -> TsValue>, TsError> {
        if let Some(compiled) = self.compiled_functions.get(function_name) {
            return Ok(compiled.clone());
        }

        let compiled_fn = self.compile_to_machine_code(function_name, instructions)?;

        self.compiled_functions.insert(function_name.to_string(), compiled_fn.clone());

        Ok(compiled_fn)
    }

    /// 将指令编译为机器码
    fn compile_to_machine_code(
        &self,
        function_name: &str,
        instructions: &[Instruction],
    ) -> Result<Rc<dyn Fn(&[TsValue]) -> TsValue + 'static>, TsError> {
        let optimized_instructions = self.optimize_instructions_for_jit(instructions);

        let function_name = function_name.to_string();
        let optimized_instructions = optimized_instructions;
        let compiled_fn = Rc::new(move |args: &[TsValue]| {
            println!("JIT executing function: {}", function_name);

            let mut vm = crate::vm::VM::new(vec![]);

            for (i, arg) in args.iter().enumerate() {
                vm.set_local(i, arg.clone());
            }

            match vm.execute_instructions(&optimized_instructions) {
                Ok(result) => result,
                Err(_) => TsValue::Undefined,
            }
        });

        Ok(compiled_fn)
    }

    /// 为 JIT 编译优化指令
    fn optimize_instructions_for_jit(&self, instructions: &[Instruction]) -> Vec<Instruction> {
        let mut optimizer = JITOptimizer::new(OptimizationLevel::High);
        optimizer.optimize_instructions(instructions)
    }

    /// 获取已编译的函数
    pub fn get_compiled_function(&self, function_name: &str) -> Option<Rc<dyn Fn(&[TsValue]) -> TsValue>> {
        self.compiled_functions.get(function_name).cloned()
    }
}
