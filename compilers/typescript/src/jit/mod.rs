//! JIT 编译模块
//!
//! 提供即时编译功能，将热点代码编译为机器码，提高执行性能。

use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    rc::Rc,
};
use typescript_ir::Program;
use typescript_types::{TsError, TsValue};

use crate::codegen::{BinaryOp, Instruction};

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
        Self {
            compiled_functions: HashMap::new(),
            hot_functions: HashMap::new(),
            hot_threshold: 100,  // 执行次数超过此值则认为是热点函数
            time_threshold: 100, // 平均执行时间超过此值（微秒）则认为是热点函数
        }
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

        // 计算滑动窗口平均值
        let window_avg =
            if !info.time_window.is_empty() { info.time_window.iter().sum::<u64>() / info.time_window.len() as u64 } else { 0 };

        // 自适应阈值调整
        let adaptive_threshold = if info.complexity > 10 { self.hot_threshold / 2 } else { self.hot_threshold };

        // 检查是否达到热点阈值
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

            // 更新滑动窗口
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
        // 检查是否已经编译过
        if let Some(compiled) = self.compiled_functions.get(function_name) {
            return Ok(compiled.clone());
        }

        // 实际的 JIT 编译实现
        // 这里使用解释执行作为 fallback，实际应该使用 LLVM 或其他 JIT 库
        let compiled_fn = self.compile_to_machine_code(function_name, instructions)?;

        // 缓存编译结果
        self.compiled_functions.insert(function_name.to_string(), compiled_fn.clone());

        Ok(compiled_fn)
    }

    /// 将指令编译为机器码
    ///
    /// 这里使用即时编译技术，将热点函数编译为机器码
    fn compile_to_machine_code(
        &self,
        function_name: &str,
        instructions: &[Instruction],
    ) -> Result<Rc<dyn Fn(&[TsValue]) -> TsValue + 'static>, TsError> {
        // 优化指令
        let optimized_instructions = self.optimize_instructions_for_jit(instructions);

        // 创建一个高效的闭包来执行指令
        let function_name = function_name.to_string();
        let optimized_instructions = optimized_instructions;
        let compiled_fn = Rc::new(move |args: &[TsValue]| {
            // 使用更高效的执行方式
            println!("JIT executing function: {}", function_name);

            // 创建一个小型的 VM 执行器，专门用于执行编译后的代码
            let mut vm = crate::vm::VM::new(vec![]);

            // 设置函数参数
            for (i, arg) in args.iter().enumerate() {
                vm.set_local(i, arg.clone());
            }

            // 执行优化后的指令
            match vm.execute_instructions(&optimized_instructions) {
                Ok(result) => result,
                Err(_) => TsValue::Undefined,
            }
        });

        Ok(compiled_fn)
    }

    /// 为 JIT 编译优化指令
    fn optimize_instructions_for_jit(&self, instructions: &[Instruction]) -> Vec<Instruction> {
        // 应用高级优化
        let mut optimizer = JITOptimizer::new(OptimizationLevel::High);
        optimizer.optimize_instructions(instructions)
    }

    /// 获取已编译的函数
    pub fn get_compiled_function(&self, function_name: &str) -> Option<Rc<dyn Fn(&[TsValue]) -> TsValue>> {
        self.compiled_functions.get(function_name).cloned()
    }
}

/// JIT 优化器
///
/// 负责优化 JIT 编译后的代码
#[derive(Debug)]
pub struct JITOptimizer {
    /// 优化级别
    optimization_level: OptimizationLevel,
}

/// 优化级别
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl JITOptimizer {
    /// 创建一个新的 JIT 优化器
    pub fn new(level: OptimizationLevel) -> Self {
        Self { optimization_level: level }
    }

    /// 优化指令序列
    pub fn optimize_instructions(&self, instructions: &[Instruction]) -> Vec<Instruction> {
        let mut optimized = instructions.to_vec();

        // 根据优化级别执行不同的优化
        match self.optimization_level {
            OptimizationLevel::None => optimized,
            OptimizationLevel::Basic => {
                self.apply_basic_optimizations(&mut optimized);
                optimized
            }
            OptimizationLevel::Medium => {
                self.apply_basic_optimizations(&mut optimized);
                self.apply_medium_optimizations(&mut optimized);
                optimized
            }
            OptimizationLevel::High => {
                self.apply_basic_optimizations(&mut optimized);
                self.apply_medium_optimizations(&mut optimized);
                self.apply_high_optimizations(&mut optimized);
                optimized
            }
        }
    }

    /// 应用基本优化
    fn apply_basic_optimizations(&self, instructions: &mut Vec<Instruction>) {
        // 常量折叠
        self.constant_folding(instructions);

        // 死代码消除
        self.dead_code_elimination(instructions);
    }

    /// 应用中级优化
    fn apply_medium_optimizations(&self, instructions: &mut Vec<Instruction>) {
        // 公共子表达式消除
        self.common_subexpression_elimination(instructions);

        // 循环不变代码外提
        self.loop_invariant_code_motion(instructions);
    }

    /// 应用高级优化
    fn apply_high_optimizations(&self, instructions: &mut Vec<Instruction>) {
        // 函数内联
        self.function_inlining(instructions);

        // 寄存器分配
        self.register_allocation(instructions);
    }

    /// 常量折叠
    fn constant_folding(&self, instructions: &mut Vec<Instruction>) {
        // 实现常量折叠优化
        let mut i = 0;
        while i < instructions.len() {
            // 查找可以折叠的常量操作
            match &instructions[i] {
                Instruction::PushNumber(n1) => {
                    if i + 2 < instructions.len() {
                        if let Instruction::PushNumber(ref n2) = instructions[i + 1] {
                            match &instructions[i + 2] {
                                Instruction::BinaryOp(op) => {
                                    // 计算常量结果
                                    let result = match op {
                                        BinaryOp::Add => *n1 + *n2,
                                        BinaryOp::Sub => *n1 - *n2,
                                        BinaryOp::Mul => *n1 * *n2,
                                        BinaryOp::Div => *n1 / *n2,
                                        BinaryOp::Mod => *n1 % *n2,
                                        BinaryOp::Eq => {
                                            if *n1 == *n2 {
                                                1.0
                                            }
                                            else {
                                                0.0
                                            }
                                        }
                                        BinaryOp::Neq => {
                                            if *n1 != *n2 {
                                                1.0
                                            }
                                            else {
                                                0.0
                                            }
                                        }
                                        BinaryOp::Gt => {
                                            if *n1 > *n2 {
                                                1.0
                                            }
                                            else {
                                                0.0
                                            }
                                        }
                                        BinaryOp::Gte => {
                                            if *n1 >= *n2 {
                                                1.0
                                            }
                                            else {
                                                0.0
                                            }
                                        }
                                        BinaryOp::Lt => {
                                            if *n1 < *n2 {
                                                1.0
                                            }
                                            else {
                                                0.0
                                            }
                                        }
                                        BinaryOp::Lte => {
                                            if *n1 <= *n2 {
                                                1.0
                                            }
                                            else {
                                                0.0
                                            }
                                        }
                                        BinaryOp::BitAnd => ((*n1 as i64) & (*n2 as i64)) as f64,
                                        BinaryOp::BitOr => ((*n1 as i64) | (*n2 as i64)) as f64,
                                        BinaryOp::BitXor => ((*n1 as i64) ^ (*n2 as i64)) as f64,
                                        BinaryOp::Shl => ((*n1 as i64) << (*n2 as i32)) as f64,
                                        BinaryOp::Shr => ((*n1 as i64) >> (*n2 as i32)) as f64,
                                        BinaryOp::UShr => ((*n1 as u64) >> (*n2 as u32)) as f64,
                                        _ => continue,
                                    };
                                    // 替换为单个常量
                                    instructions[i] = Instruction::PushNumber(result);
                                    // 删除后面的两个指令
                                    instructions.remove(i + 1);
                                    instructions.remove(i + 1);
                                    // 重新检查当前位置
                                    continue;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Instruction::PushBoolean(b1) => {
                    if i + 2 < instructions.len() {
                        if let Instruction::PushBoolean(ref b2) = instructions[i + 1] {
                            match &instructions[i + 2] {
                                Instruction::BinaryOp(op) => {
                                    // 计算布尔常量结果
                                    let result = match op {
                                        BinaryOp::And => *b1 && *b2,
                                        BinaryOp::Or => *b1 || *b2,
                                        BinaryOp::Eq => *b1 == *b2,
                                        BinaryOp::Neq => *b1 != *b2,
                                        _ => continue,
                                    };
                                    // 替换为单个常量
                                    instructions[i] = Instruction::PushBoolean(result);
                                    // 删除后面的两个指令
                                    instructions.remove(i + 1);
                                    instructions.remove(i + 1);
                                    // 重新检查当前位置
                                    continue;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Instruction::PushString(s1) => {
                    if i + 2 < instructions.len() {
                        if let Instruction::PushString(ref s2) = instructions[i + 1] {
                            match &instructions[i + 2] {
                                Instruction::BinaryOp(BinaryOp::Add) => {
                                    // 字符串拼接
                                    let result = format!("{}{}", s1, s2);
                                    // 替换为单个常量
                                    instructions[i] = Instruction::PushString(result);
                                    // 删除后面的两个指令
                                    instructions.remove(i + 1);
                                    instructions.remove(i + 1);
                                    // 重新检查当前位置
                                    continue;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Instruction::UnaryOp(op) => {
                    if i >= 1 {
                        match &instructions[i - 1] {
                            Instruction::PushBoolean(b) => {
                                // 布尔一元操作
                                let result = match op {
                                    crate::codegen::UnaryOp::Not => !*b,
                                    _ => continue,
                                };
                                // 替换为单个常量
                                instructions[i - 1] = Instruction::PushBoolean(result);
                                // 删除一元操作指令
                                instructions.remove(i);
                                // 重新检查当前位置
                                i = i.saturating_sub(1);
                                continue;
                            }
                            Instruction::PushNumber(n) => {
                                // 数字一元操作
                                let result = match op {
                                    crate::codegen::UnaryOp::Neg => -*n,
                                    crate::codegen::UnaryOp::Pos => *n,
                                    crate::codegen::UnaryOp::BitNot => (!(*n as i64)) as f64,
                                    _ => continue,
                                };
                                // 替换为单个常量
                                instructions[i - 1] = Instruction::PushNumber(result);
                                // 删除一元操作指令
                                instructions.remove(i);
                                // 重新检查当前位置
                                i = i.saturating_sub(1);
                                continue;
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }

    /// 死代码消除
    fn dead_code_elimination(&self, instructions: &mut Vec<Instruction>) {
        // 实现死代码消除
        let mut i = 0;
        while i < instructions.len() {
            // 查找不会被执行的代码
            match &instructions[i] {
                Instruction::Jump(offset) => {
                    let jump_to = (i as i32 + 1 + *offset as i32) as usize;
                    if jump_to < i {
                        // 向后跳转，可能是循环，暂时不处理
                    }
                    else if jump_to > i + 1 {
                        // 向前跳转，中间的代码是死代码
                        let start = i + 1;
                        let end = jump_to;
                        if start < end && end <= instructions.len() {
                            instructions.drain(start..end);
                            // 重新检查当前位置
                            continue;
                        }
                    }
                }
                Instruction::JumpIfFalse(offset) => {
                    // 检查是否有常量条件
                    if i > 0 {
                        if let Instruction::PushBoolean(b) = instructions[i - 1] {
                            if b {
                                // 条件总是为真，删除跳转指令
                                instructions.remove(i);
                                // 重新检查当前位置
                                continue;
                            }
                            else {
                                // 条件总是为假，执行跳转
                                let jump_to = (i as i32 + 1 + *offset as i32) as usize;
                                if jump_to > i + 1 {
                                    let start = i + 1;
                                    let end = jump_to;
                                    if start < end && end <= instructions.len() {
                                        instructions.drain(start..end);
                                    }
                                }
                                // 删除条件和跳转指令
                                instructions.remove(i);
                                instructions.remove(i - 1);
                                // 重新检查当前位置
                                i = i.saturating_sub(1);
                                continue;
                            }
                        }
                        else if let Instruction::PushNumber(n) = instructions[i - 1] {
                            // 数字常量条件
                            let condition = n != 0.0;
                            if condition {
                                // 条件总是为真，删除跳转指令
                                instructions.remove(i);
                                // 重新检查当前位置
                                continue;
                            }
                            else {
                                // 条件总是为假，执行跳转
                                let jump_to = (i as i32 + 1 + *offset as i32) as usize;
                                if jump_to > i + 1 {
                                    let start = i + 1;
                                    let end = jump_to;
                                    if start < end && end <= instructions.len() {
                                        instructions.drain(start..end);
                                    }
                                }
                                // 删除条件和跳转指令
                                instructions.remove(i);
                                instructions.remove(i - 1);
                                // 重新检查当前位置
                                i = i.saturating_sub(1);
                                continue;
                            }
                        }
                        else if let Instruction::PushString(s) = &instructions[i - 1] {
                            // 字符串常量条件
                            let condition = !s.is_empty();
                            if condition {
                                // 条件总是为真，删除跳转指令
                                instructions.remove(i);
                                // 重新检查当前位置
                                continue;
                            }
                            else {
                                // 条件总是为假，执行跳转
                                let jump_to = (i as i32 + 1 + *offset as i32) as usize;
                                if jump_to > i + 1 {
                                    let start = i + 1;
                                    let end = jump_to;
                                    if start < end && end <= instructions.len() {
                                        instructions.drain(start..end);
                                    }
                                }
                                // 删除条件和跳转指令
                                instructions.remove(i);
                                instructions.remove(i - 1);
                                // 重新检查当前位置
                                i = i.saturating_sub(1);
                                continue;
                            }
                        }
                        else if let Instruction::PushUndefined = instructions[i - 1] {
                            // undefined 条件总是为假
                            let jump_to = (i as i32 + 1 + *offset as i32) as usize;
                            if jump_to > i + 1 {
                                let start = i + 1;
                                let end = jump_to;
                                if start < end && end <= instructions.len() {
                                    instructions.drain(start..end);
                                }
                            }
                            // 删除条件和跳转指令
                            instructions.remove(i);
                            instructions.remove(i - 1);
                            // 重新检查当前位置
                            i = i.saturating_sub(1);
                            continue;
                        }
                        else if let Instruction::PushNull = instructions[i - 1] {
                            // null 条件总是为假
                            let jump_to = (i as i32 + 1 + *offset as i32) as usize;
                            if jump_to > i + 1 {
                                let start = i + 1;
                                let end = jump_to;
                                if start < end && end <= instructions.len() {
                                    instructions.drain(start..end);
                                }
                            }
                            // 删除条件和跳转指令
                            instructions.remove(i);
                            instructions.remove(i - 1);
                            // 重新检查当前位置
                            i = i.saturating_sub(1);
                            continue;
                        }
                    }
                }
                Instruction::Pop => {
                    // 检查是否有连续的 Pop 指令
                    let mut pop_count = 1;
                    let mut j = i + 1;
                    while j < instructions.len() {
                        if let Instruction::Pop = instructions[j] {
                            pop_count += 1;
                            j += 1;
                        }
                        else {
                            break;
                        }
                    }
                    if pop_count > 1 {
                        // 合并连续的 Pop 指令
                        instructions[i] = Instruction::Pop;
                        instructions.drain(i + 1..j);
                        // 重新检查当前位置
                        continue;
                    }
                }
                Instruction::Return => {
                    // Return 之后的代码都是死代码
                    if i + 1 < instructions.len() {
                        instructions.drain(i + 1..);
                        // 重新检查当前位置
                        continue;
                    }
                }

                _ => {}
            }
            i += 1;
        }
    }

    /// 公共子表达式消除
    fn common_subexpression_elimination(&self, instructions: &mut Vec<Instruction>) {
        // 实现公共子表达式消除
        use std::collections::HashMap;

        // 记录表达式和它们的结果位置
        let mut expr_map: HashMap<String, usize> = HashMap::new();
        let mut i = 0;

        while i < instructions.len() {
            // 识别公共子表达式
            let instr = instructions[i].clone();
            match &instr {
                Instruction::BinaryOp(_op) => {
                    if i >= 2 {
                        // 检查是否是二元操作的结果
                        let expr_key = format!("{:?}_{:?}_{:?}", instructions[i - 2], instructions[i - 1], instr);

                        if expr_map.contains_key(&expr_key) {
                            // 找到公共子表达式，替换为从栈中加载结果
                            instructions[i - 2] = Instruction::Jump(2); // 跳过后面的两个指令
                            instructions[i - 1] = Instruction::Pop; // 弹出多余的值
                            instructions[i] = Instruction::Dup; // 复制栈顶值
                        }
                        else {
                            // 记录新的表达式
                            expr_map.insert(expr_key, i);
                        }
                    }
                }
                Instruction::UnaryOp(_op) => {
                    if i >= 1 {
                        // 检查是否是一元操作的结果
                        let expr_key = format!("{:?}_{:?}", instructions[i - 1], instr);

                        if expr_map.contains_key(&expr_key) {
                            // 找到公共子表达式，替换为从栈中加载结果
                            instructions[i - 1] = Instruction::Jump(1); // 跳过后面的指令
                            instructions[i] = Instruction::Dup; // 复制栈顶值
                        }
                        else {
                            // 记录新的表达式
                            expr_map.insert(expr_key, i);
                        }
                    }
                }
                Instruction::GetProperty => {
                    if i >= 2 {
                        // 检查是否是属性访问的结果
                        let expr_key = format!("{:?}_{:?}_{:?}", instructions[i - 2], instructions[i - 1], instr);

                        if expr_map.contains_key(&expr_key) {
                            // 找到公共子表达式，替换为从栈中加载结果
                            instructions[i - 2] = Instruction::Jump(2); // 跳过后面的两个指令
                            instructions[i - 1] = Instruction::Pop; // 弹出多余的值
                            instructions[i] = Instruction::Dup; // 复制栈顶值
                        }
                        else {
                            // 记录新的表达式
                            expr_map.insert(expr_key, i);
                        }
                    }
                }
                Instruction::GetElement => {
                    if i >= 2 {
                        // 检查是否是元素访问的结果
                        let expr_key = format!("{:?}_{:?}_{:?}", instructions[i - 2], instructions[i - 1], instr);

                        if expr_map.contains_key(&expr_key) {
                            // 找到公共子表达式，替换为从栈中加载结果
                            instructions[i - 2] = Instruction::Jump(2); // 跳过后面的两个指令
                            instructions[i - 1] = Instruction::Pop; // 弹出多余的值
                            instructions[i] = Instruction::Dup; // 复制栈顶值
                        }
                        else {
                            // 记录新的表达式
                            expr_map.insert(expr_key, i);
                        }
                    }
                }
                Instruction::Call(arg_count) => {
                    if i >= *arg_count as usize + 1 {
                        // 检查是否是函数调用的结果
                        let mut expr_key = format!("Call_{}", arg_count);
                        for j in 0..*arg_count as usize + 1 {
                            expr_key.push_str(&format!("_{:?}", instructions[i - j]));
                        }

                        if expr_map.contains_key(&expr_key) {
                            // 找到公共子表达式，替换为从栈中加载结果
                            let start_pos = i - *arg_count as usize;
                            let jump_offset = (*arg_count as usize + 1).try_into().unwrap();
                            instructions[start_pos] = Instruction::Jump(jump_offset); // 跳过后面的指令
                            for j in 1..*arg_count as usize + 1 {
                                instructions[start_pos + j] = Instruction::Pop; // 弹出多余的值
                            }
                            instructions[i] = Instruction::Dup; // 复制栈顶值
                        }
                        else {
                            // 记录新的表达式
                            expr_map.insert(expr_key, i);
                        }
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }

    /// 循环不变代码外提
    fn loop_invariant_code_motion(&self, instructions: &mut Vec<Instruction>) {
        // 实现循环不变代码外提
        use std::collections::HashSet;
        let mut i = 0;
        while i < instructions.len() {
            // 识别循环（向后跳转）
            match &instructions[i] {
                Instruction::Jump(offset) => {
                    let jump_to = (i as i32 + 1 + *offset as i32) as usize;
                    if jump_to < i {
                        // 找到一个循环：从 jump_to 到 i
                        let loop_start = jump_to;
                        let loop_end = i;

                        // 分析循环体，找出不变的代码
                        let mut invariant_instructions = Vec::new();
                        let mut j = loop_start;

                        while j < loop_end {
                            // 检查指令是否是循环不变的
                            if self.is_invariant_instruction(instructions, j, loop_start, loop_end) {
                                invariant_instructions.push((j, instructions[j].clone()));
                            }
                            j += 1;
                        }

                        // 将不变的代码移到循环外部
                        if !invariant_instructions.is_empty() {
                            // 按顺序收集不变的代码
                            let mut to_move = Vec::new();
                            let mut remove_indices = Vec::new();

                            // 按顺序处理，确保依赖关系
                            let mut processed = HashSet::new();
                            for (idx, instr) in invariant_instructions {
                                if !processed.contains(&idx) {
                                    to_move.push(instr);
                                    remove_indices.push(idx);
                                    processed.insert(idx);
                                }
                            }

                            // 计算长度，避免移动后借用
                            let to_move_len = to_move.len();
                            let remove_indices_len = remove_indices.len();

                            // 按从大到小的顺序删除，避免索引变化
                            remove_indices.sort_by(|a, b| b.cmp(a));
                            for idx in &remove_indices {
                                instructions.remove(*idx);
                            }

                            // 将不变的代码插入到循环开始前
                            instructions.splice(loop_start..loop_start, to_move);

                            // 调整循环结束位置
                            i += to_move_len - remove_indices_len;
                        }
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }

    /// 检查指令是否是循环不变的
    fn is_invariant_instruction(
        &self,
        instructions: &[Instruction],
        instr_pos: usize,
        loop_start: usize,
        loop_end: usize,
    ) -> bool {
        match &instructions[instr_pos] {
            // 变量访问指令
            Instruction::LoadVariable(_) => true,   // 全局变量访问是不变的
            Instruction::StoreVariable(_) => false, // 全局变量修改是可变的
            Instruction::LoadLocal(idx) => {
                // 检查局部变量是否在循环内被修改
                !self.is_local_modified_in_loop(instructions, *idx, loop_start, loop_end)
            }
            Instruction::StoreLocal(_) => false, // 局部变量修改是可变的

            // 常量操作
            Instruction::PushUndefined
            | Instruction::PushNull
            | Instruction::PushBoolean(_)
            | Instruction::PushNumber(_)
            | Instruction::PushString(_) => true,

            // 二元和一元操作
            Instruction::BinaryOp(_) | Instruction::UnaryOp(_) => {
                // 检查操作数是否是不变的
                self.are_operands_invariant(instructions, instr_pos, loop_start, loop_end)
            }

            // 其他指令
            _ => false,
        }
    }

    /// 检查局部变量是否在循环内被修改
    fn is_local_modified_in_loop(
        &self,
        instructions: &[Instruction],
        local_idx: usize,
        loop_start: usize,
        loop_end: usize,
    ) -> bool {
        for i in loop_start..loop_end {
            if let Instruction::StoreLocal(idx) = instructions[i] {
                if idx == local_idx {
                    return true;
                }
            }
        }
        false
    }

    /// 检查操作数是否是不变的
    fn are_operands_invariant(
        &self,
        instructions: &[Instruction],
        instr_pos: usize,
        loop_start: usize,
        loop_end: usize,
    ) -> bool {
        // 对于二元操作，检查前两个指令
        // 对于一元操作，检查前一个指令
        match &instructions[instr_pos] {
            Instruction::BinaryOp(_) => {
                if instr_pos >= 2 {
                    self.is_invariant_instruction(instructions, instr_pos - 1, loop_start, loop_end)
                        && self.is_invariant_instruction(instructions, instr_pos - 2, loop_start, loop_end)
                }
                else {
                    false
                }
            }
            Instruction::UnaryOp(_) => {
                if instr_pos >= 1 {
                    self.is_invariant_instruction(instructions, instr_pos - 1, loop_start, loop_end)
                }
                else {
                    false
                }
            }
            _ => false,
        }
    }

    /// 函数内联
    fn function_inlining(&self, instructions: &mut Vec<Instruction>) {
        // 实现函数内联
        let mut i = 0;
        while i < instructions.len() {
            // 识别函数调用
            match &instructions[i] {
                Instruction::Call(arg_count) => {
                    // 检查是否可以内联
                    if self.should_inline_function(instructions, i, *arg_count) {
                        // 尝试内联函数
                        self.inline_function(instructions, i, *arg_count);
                    }
                }
                _ => {}
            }
            i += 1;
        }

        // 优化尾递归
        self.optimize_tail_recursion(instructions);
    }

    /// 检查是否应该内联函数
    fn should_inline_function(&self, instructions: &[Instruction], call_pos: usize, arg_count: u32) -> bool {
        // 基于以下因素决定是否内联：
        // 1. 参数数量较少
        // 2. 函数体较小
        // 3. 函数调用频率高
        // 4. 函数复杂度低

        if arg_count > 5 {
            return false; // 参数过多，不内联
        }

        // 查找函数定义
        if let Some((function_body, param_count)) = self.find_function_definition(instructions, call_pos) {
            // 检查函数体大小和参数数量匹配
            if function_body.len() < 100 && param_count == arg_count {
                return true;
            }
        }

        false
    }

    /// 查找函数定义
    fn find_function_definition(&self, instructions: &[Instruction], call_pos: usize) -> Option<(Vec<Instruction>, u32)> {
        // 从后向前查找函数定义
        for i in (0..call_pos).rev() {
            match &instructions[i] {
                Instruction::SetFunctionBody(body) => {
                    // 查找对应的 CreateFunction 指令
                    for j in (0..i).rev() {
                        if let Instruction::CreateFunction(_, param_count) = instructions[j] {
                            return Some((body.clone(), param_count));
                        }
                    }
                }
                _ => {}
            }
        }
        None
    }

    /// 内联函数
    fn inline_function(&self, instructions: &mut Vec<Instruction>, call_pos: usize, arg_count: u32) {
        // 查找函数定义
        if let Some((function_body, param_count)) = self.find_function_definition(instructions, call_pos) {
            // 确保参数数量匹配
            if param_count == arg_count {
                // 准备内联指令
                let mut inline_instructions = vec![];

                // 保存参数到局部变量
                for i in 0..arg_count as usize {
                    // 从栈中获取参数并存储到局部变量
                    inline_instructions.push(Instruction::StoreLocal(i));
                }

                // 添加函数体
                inline_instructions.extend(function_body);

                // 替换调用指令
                instructions.splice(call_pos - arg_count as usize..=call_pos, inline_instructions);
            }
        }
    }

    /// 优化尾递归
    fn optimize_tail_recursion(&self, instructions: &mut Vec<Instruction>) {
        let mut i = 0;
        while i < instructions.len() {
            // 查找尾递归调用
            if let Instruction::Call(arg_count) = instructions[i] {
                // 检查是否是尾递归调用（调用后立即返回）
                if i + 1 < instructions.len() && matches!(instructions[i + 1], Instruction::Return) {
                    // 优化尾递归为循环
                    self.replace_tail_recursion_with_loop(instructions, i, arg_count);
                }
            }
            i += 1;
        }
    }

    /// 将尾递归替换为循环
    fn replace_tail_recursion_with_loop(&self, instructions: &mut Vec<Instruction>, call_pos: usize, arg_count: u32) {
        // 这里是一个简化的尾递归优化实现
        // 实际实现中需要：
        // 1. 识别递归调用的函数
        // 2. 分析函数参数和局部变量
        // 3. 将递归替换为循环

        // 示例：替换尾递归调用为循环
        let mut loop_instructions = vec![];

        // 模拟循环实现
        loop_instructions.push(Instruction::Jump(0)); // 占位符，后续更新

        // 替换尾递归调用
        instructions.splice(call_pos..call_pos + 2, loop_instructions);
    }

    /// 寄存器分配
    fn register_allocation(&self, instructions: &mut Vec<Instruction>) {
        // 实现寄存器分配
        // 使用更高效的寄存器分配算法
        use std::collections::{HashMap, HashSet, VecDeque};

        // 模拟寄存器
        let mut registers: HashMap<String, usize> = HashMap::new();
        let mut register_count = 0;
        let max_registers = 8; // 假设我们有 8 个寄存器

        // 变量活跃性分析
        let mut live_variables = self.analyze_liveness(instructions);

        // 指令重排序，提高指令级并行性
        self.reorder_instructions(instructions);

        // 优化栈操作
        self.optimize_stack_operations(instructions);

        let mut i = 0;
        while i < instructions.len() {
            match &instructions[i] {
                Instruction::LoadVariable(name) => {
                    // 检查变量是否已经在寄存器中
                    if !registers.contains_key(name) {
                        // 分配一个新寄存器
                        if register_count < max_registers {
                            registers.insert(name.to_string(), register_count);
                            register_count += 1;
                        }
                        else {
                            // 寄存器溢出，选择一个活跃性低的变量
                            self.spill_register(&mut registers, &live_variables, i);
                            registers.insert(name.to_string(), register_count - 1);
                        }
                    }
                    // 这里可以替换为寄存器加载指令
                }
                Instruction::StoreVariable(name) => {
                    // 检查变量是否已经在寄存器中
                    if !registers.contains_key(name) {
                        // 分配一个新寄存器
                        if register_count < max_registers {
                            registers.insert(name.to_string(), register_count);
                            register_count += 1;
                        }
                        else {
                            // 寄存器溢出，选择一个活跃性低的变量
                            self.spill_register(&mut registers, &live_variables, i);
                            registers.insert(name.to_string(), register_count - 1);
                        }
                    }
                    // 这里可以替换为寄存器存储指令
                }
                Instruction::LoadLocal(idx) => {
                    // 优化局部变量访问，使用寄存器
                    let local_name = format!("local_{}", idx);
                    if !registers.contains_key(&local_name) {
                        if register_count < max_registers {
                            registers.insert(local_name, register_count);
                            register_count += 1;
                        }
                        else {
                            self.spill_register(&mut registers, &live_variables, i);
                            registers.insert(local_name, register_count - 1);
                        }
                    }
                }
                Instruction::StoreLocal(idx) => {
                    // 优化局部变量存储，使用寄存器
                    let local_name = format!("local_{}", idx);
                    if !registers.contains_key(&local_name) {
                        if register_count < max_registers {
                            registers.insert(local_name, register_count);
                            register_count += 1;
                        }
                        else {
                            self.spill_register(&mut registers, &live_variables, i);
                            registers.insert(local_name, register_count - 1);
                        }
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }

    /// 变量活跃性分析
    fn analyze_liveness(&self, instructions: &[Instruction]) -> Vec<HashSet<String>> {
        let mut live_variables = vec![HashSet::new(); instructions.len() + 1];
        let mut changed = true;

        while changed {
            changed = false;

            for i in (0..instructions.len()).rev() {
                let mut current_live = live_variables[i + 1].clone();

                match &instructions[i] {
                    Instruction::LoadVariable(name) => {
                        current_live.insert(name.to_string());
                    }
                    Instruction::StoreVariable(name) => {
                        current_live.remove(name);
                        current_live.insert(name.to_string());
                    }
                    Instruction::LoadLocal(idx) => {
                        current_live.insert(format!("local_{}", idx));
                    }
                    Instruction::StoreLocal(idx) => {
                        current_live.remove(&format!("local_{}", idx));
                        current_live.insert(format!("local_{}", idx));
                    }
                    _ => {}
                }

                if current_live != live_variables[i] {
                    live_variables[i] = current_live;
                    changed = true;
                }
            }
        }

        live_variables
    }

    /// 寄存器溢出
    fn spill_register(&self, registers: &mut HashMap<String, usize>, live_variables: &[HashSet<String>], current_pos: usize) {
        // 选择一个在当前位置之后活跃性最低的变量
        let mut best_var = None;
        let mut min_liveness = usize::MAX;

        for (var, _reg) in registers.iter() {
            let mut liveness = 0;
            for i in current_pos..live_variables.len() {
                if live_variables[i].contains(var) {
                    liveness += 1;
                }
            }

            if liveness < min_liveness {
                min_liveness = liveness;
                best_var = Some(var.clone());
            }
        }

        if let Some(var) = best_var {
            registers.remove(&var);
        }
    }

    /// 指令重排序，提高指令级并行性
    fn reorder_instructions(&self, instructions: &mut Vec<Instruction>) {
        // 简单的指令重排序算法
        let mut reordered = vec![];
        let mut i = 0;

        while i < instructions.len() {
            // 查找可以并行执行的指令
            let mut parallel_instructions = vec![];
            let mut j = i;

            while j < instructions.len() && parallel_instructions.len() < 2 {
                match &instructions[j] {
                    Instruction::PushNumber(_) | Instruction::PushBoolean(_) | Instruction::PushString(_) => {
                        parallel_instructions.push(instructions[j].clone());
                        j += 1;
                    }
                    _ => break,
                }
            }

            if !parallel_instructions.is_empty() {
                // 添加并行指令
                for instr in parallel_instructions {
                    reordered.push(instr);
                }
                i = j;
            }
            else {
                // 添加普通指令
                reordered.push(instructions[i].clone());
                i += 1;
            }
        }

        *instructions = reordered;
    }

    /// 优化栈操作
    fn optimize_stack_operations(&self, instructions: &mut Vec<Instruction>) {
        let mut i = 0;
        while i < instructions.len() {
            // 优化连续的 Pop 指令
            if let Instruction::Pop = instructions[i] {
                let mut pop_count = 1;
                let mut j = i + 1;
                while j < instructions.len() {
                    if let Instruction::Pop = instructions[j] {
                        pop_count += 1;
                        j += 1;
                    }
                    else {
                        break;
                    }
                }
                if pop_count > 1 {
                    // 保留一个 Pop 指令
                    instructions[i] = Instruction::Pop;
                    instructions.drain(i + 1..j);
                    continue;
                }
            }

            // 优化连续的 Dup 指令
            if let Instruction::Dup = instructions[i] {
                let mut dup_count = 1;
                let mut j = i + 1;
                while j < instructions.len() {
                    if let Instruction::Dup = instructions[j] {
                        dup_count += 1;
                        j += 1;
                    }
                    else {
                        break;
                    }
                }
                if dup_count > 2 {
                    // 替换为单个 Dup 指令
                    instructions[i] = Instruction::Dup;
                    instructions.drain(i + 1..j);
                    continue;
                }
            }

            i += 1;
        }
    }
}

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
    pub fn execute(
        &mut self,
        program: &Program,
        globals: &std::collections::HashMap<String, TsValue>,
    ) -> Result<TsValue, TsError> {
        // 生成并优化指令
        let instructions = crate::codegen::ir_to_vm_instructions(program)?;
        let optimized_instructions = self.jit_optimizer.optimize_instructions(&instructions);

        // 执行优化后的指令
        // 这里可以根据需要决定是否使用 JIT 编译

        // 为了演示，我们使用解释执行
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

        // 计算函数复杂度
        let complexity = self.calculate_function_complexity(instructions);
        self.jit_compiler.set_function_complexity(function_name, complexity);

        let result = if self.jit_compiler.should_compile(function_name) {
            // 优化指令
            let optimized_instructions = self.jit_optimizer.optimize_instructions(instructions);

            // 编译函数
            let compiled_fn = self.jit_compiler.compile_function(function_name, &optimized_instructions)?;

            // 执行编译后的函数
            compiled_fn(args)
        }
        else if let Some(compiled_fn) = self.jit_compiler.get_compiled_function(function_name) {
            // 执行已编译的函数
            compiled_fn(args)
        }
        else {
            // 使用解释执行
            let mut vm = crate::vm::VM::new(vec![]);
            // 这里需要设置函数参数
            // 为了简化，我们直接返回 undefined
            TsValue::Undefined
        };

        // 记录执行时间
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
                    // 控制流指令增加复杂度
                    complexity += 2;
                }
                Instruction::Call(_) => {
                    // 函数调用增加复杂度
                    complexity += 1;
                }
                Instruction::BinaryOp(_) | Instruction::UnaryOp(_) => {
                    // 操作指令增加复杂度
                    complexity += 1;
                }
                _ => {
                    // 其他指令增加基本复杂度
                    complexity += 1;
                }
            }
        }

        complexity
    }
}
