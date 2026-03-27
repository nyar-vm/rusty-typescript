//! JIT 优化器模块
//!
//! 提供指令级别的优化功能，包括常量折叠、死代码消除、公共子表达式消除等。

use std::collections::{HashMap, HashSet};

use crate::codegen::{BinaryOp, Instruction};

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
        self.constant_folding(instructions);
        self.dead_code_elimination(instructions);
    }

    /// 应用中级优化
    fn apply_medium_optimizations(&self, instructions: &mut Vec<Instruction>) {
        self.common_subexpression_elimination(instructions);
        self.loop_invariant_code_motion(instructions);
    }

    /// 应用高级优化
    fn apply_high_optimizations(&self, instructions: &mut Vec<Instruction>) {
        self.function_inlining(instructions);
        self.register_allocation(instructions);
    }

    /// 常量折叠
    fn constant_folding(&self, instructions: &mut Vec<Instruction>) {
        let mut i = 0;
        while i < instructions.len() {
            match &instructions[i] {
                Instruction::PushNumber(n1) => {
                    if i + 2 < instructions.len() {
                        if let Instruction::PushNumber(ref n2) = instructions[i + 1] {
                            match &instructions[i + 2] {
                                Instruction::BinaryOp(op) => {
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
                                    instructions[i] = Instruction::PushNumber(result);
                                    instructions.remove(i + 1);
                                    instructions.remove(i + 1);
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
                                    let result = match op {
                                        BinaryOp::And => *b1 && *b2,
                                        BinaryOp::Or => *b1 || *b2,
                                        BinaryOp::Eq => *b1 == *b2,
                                        BinaryOp::Neq => *b1 != *b2,
                                        _ => continue,
                                    };
                                    instructions[i] = Instruction::PushBoolean(result);
                                    instructions.remove(i + 1);
                                    instructions.remove(i + 1);
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
                                    let result = format!("{}{}", s1, s2);
                                    instructions[i] = Instruction::PushString(result);
                                    instructions.remove(i + 1);
                                    instructions.remove(i + 1);
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
                                let result = match op {
                                    crate::codegen::UnaryOp::Not => !*b,
                                    _ => continue,
                                };
                                instructions[i - 1] = Instruction::PushBoolean(result);
                                instructions.remove(i);
                                i = i.saturating_sub(1);
                                continue;
                            }
                            Instruction::PushNumber(n) => {
                                let result = match op {
                                    crate::codegen::UnaryOp::Neg => -*n,
                                    crate::codegen::UnaryOp::Pos => *n,
                                    crate::codegen::UnaryOp::BitNot => (!(*n as i64)) as f64,
                                    _ => continue,
                                };
                                instructions[i - 1] = Instruction::PushNumber(result);
                                instructions.remove(i);
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
        let mut i = 0;
        while i < instructions.len() {
            match &instructions[i] {
                Instruction::Jump(offset) => {
                    let jump_to = (i as i32 + 1 + *offset as i32) as usize;
                    if jump_to > i + 1 {
                        let start = i + 1;
                        let end = jump_to;
                        if start < end && end <= instructions.len() {
                            instructions.drain(start..end);
                            continue;
                        }
                    }
                }
                Instruction::JumpIfFalse(offset) => {
                    if i > 0 {
                        if let Instruction::PushBoolean(b) = instructions[i - 1] {
                            if b {
                                instructions.remove(i);
                                continue;
                            }
                            else {
                                let jump_to = (i as i32 + 1 + *offset as i32) as usize;
                                if jump_to > i + 1 {
                                    let start = i + 1;
                                    let end = jump_to;
                                    if start < end && end <= instructions.len() {
                                        instructions.drain(start..end);
                                    }
                                }
                                instructions.remove(i);
                                instructions.remove(i - 1);
                                i = i.saturating_sub(1);
                                continue;
                            }
                        }
                        else if let Instruction::PushNumber(n) = instructions[i - 1] {
                            let condition = n != 0.0;
                            if condition {
                                instructions.remove(i);
                                continue;
                            }
                            else {
                                let jump_to = (i as i32 + 1 + *offset as i32) as usize;
                                if jump_to > i + 1 {
                                    let start = i + 1;
                                    let end = jump_to;
                                    if start < end && end <= instructions.len() {
                                        instructions.drain(start..end);
                                    }
                                }
                                instructions.remove(i);
                                instructions.remove(i - 1);
                                i = i.saturating_sub(1);
                                continue;
                            }
                        }
                        else if let Instruction::PushString(s) = &instructions[i - 1] {
                            let condition = !s.is_empty();
                            if condition {
                                instructions.remove(i);
                                continue;
                            }
                            else {
                                let jump_to = (i as i32 + 1 + *offset as i32) as usize;
                                if jump_to > i + 1 {
                                    let start = i + 1;
                                    let end = jump_to;
                                    if start < end && end <= instructions.len() {
                                        instructions.drain(start..end);
                                    }
                                }
                                instructions.remove(i);
                                instructions.remove(i - 1);
                                i = i.saturating_sub(1);
                                continue;
                            }
                        }
                        else if let Instruction::PushUndefined = instructions[i - 1] {
                            let jump_to = (i as i32 + 1 + *offset as i32) as usize;
                            if jump_to > i + 1 {
                                let start = i + 1;
                                let end = jump_to;
                                if start < end && end <= instructions.len() {
                                    instructions.drain(start..end);
                                }
                            }
                            instructions.remove(i);
                            instructions.remove(i - 1);
                            i = i.saturating_sub(1);
                            continue;
                        }
                        else if let Instruction::PushNull = instructions[i - 1] {
                            let jump_to = (i as i32 + 1 + *offset as i32) as usize;
                            if jump_to > i + 1 {
                                let start = i + 1;
                                let end = jump_to;
                                if start < end && end <= instructions.len() {
                                    instructions.drain(start..end);
                                }
                            }
                            instructions.remove(i);
                            instructions.remove(i - 1);
                            i = i.saturating_sub(1);
                            continue;
                        }
                    }
                }
                Instruction::Pop => {
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
                        instructions[i] = Instruction::Pop;
                        instructions.drain(i + 1..j);
                        continue;
                    }
                }
                Instruction::Return => {
                    if i + 1 < instructions.len() {
                        instructions.drain(i + 1..);
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
        let mut expr_map: HashMap<String, usize> = HashMap::new();
        let mut i = 0;

        while i < instructions.len() {
            let instr = instructions[i].clone();
            match &instr {
                Instruction::BinaryOp(_op) => {
                    if i >= 2 {
                        let expr_key = format!("{:?}_{:?}_{:?}", instructions[i - 2], instructions[i - 1], instr);

                        if expr_map.contains_key(&expr_key) {
                            instructions[i - 2] = Instruction::Jump(2);
                            instructions[i - 1] = Instruction::Pop;
                            instructions[i] = Instruction::Dup;
                        }
                        else {
                            expr_map.insert(expr_key, i);
                        }
                    }
                }
                Instruction::UnaryOp(_op) => {
                    if i >= 1 {
                        let expr_key = format!("{:?}_{:?}", instructions[i - 1], instr);

                        if expr_map.contains_key(&expr_key) {
                            instructions[i - 1] = Instruction::Jump(1);
                            instructions[i] = Instruction::Dup;
                        }
                        else {
                            expr_map.insert(expr_key, i);
                        }
                    }
                }
                Instruction::GetProperty => {
                    if i >= 2 {
                        let expr_key = format!("{:?}_{:?}_{:?}", instructions[i - 2], instructions[i - 1], instr);

                        if expr_map.contains_key(&expr_key) {
                            instructions[i - 2] = Instruction::Jump(2);
                            instructions[i - 1] = Instruction::Pop;
                            instructions[i] = Instruction::Dup;
                        }
                        else {
                            expr_map.insert(expr_key, i);
                        }
                    }
                }
                Instruction::GetElement => {
                    if i >= 2 {
                        let expr_key = format!("{:?}_{:?}_{:?}", instructions[i - 2], instructions[i - 1], instr);

                        if expr_map.contains_key(&expr_key) {
                            instructions[i - 2] = Instruction::Jump(2);
                            instructions[i - 1] = Instruction::Pop;
                            instructions[i] = Instruction::Dup;
                        }
                        else {
                            expr_map.insert(expr_key, i);
                        }
                    }
                }
                Instruction::Call(arg_count) => {
                    if i >= *arg_count as usize + 1 {
                        let mut expr_key = format!("Call_{}", arg_count);
                        for j in 0..*arg_count as usize + 1 {
                            expr_key.push_str(&format!("_{:?}", instructions[i - j]));
                        }

                        if expr_map.contains_key(&expr_key) {
                            let start_pos = i - *arg_count as usize;
                            let jump_offset = (*arg_count as usize + 1).try_into().unwrap();
                            instructions[start_pos] = Instruction::Jump(jump_offset);
                            for j in 1..*arg_count as usize + 1 {
                                instructions[start_pos + j] = Instruction::Pop;
                            }
                            instructions[i] = Instruction::Dup;
                        }
                        else {
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
        let mut i = 0;
        while i < instructions.len() {
            match &instructions[i] {
                Instruction::Jump(offset) => {
                    let jump_to = (i as i32 + 1 + *offset as i32) as usize;
                    if jump_to < i {
                        let loop_start = jump_to;
                        let loop_end = i;

                        let mut invariant_instructions = Vec::new();
                        let mut j = loop_start;

                        while j < loop_end {
                            if self.is_invariant_instruction(instructions, j, loop_start, loop_end) {
                                invariant_instructions.push((j, instructions[j].clone()));
                            }
                            j += 1;
                        }

                        if !invariant_instructions.is_empty() {
                            let mut to_move = Vec::new();
                            let mut remove_indices = Vec::new();

                            let mut processed = HashSet::new();
                            for (idx, instr) in invariant_instructions {
                                if !processed.contains(&idx) {
                                    to_move.push(instr);
                                    remove_indices.push(idx);
                                    processed.insert(idx);
                                }
                            }

                            let to_move_len = to_move.len();
                            let remove_indices_len = remove_indices.len();

                            remove_indices.sort_by(|a, b| b.cmp(a));
                            for idx in &remove_indices {
                                instructions.remove(*idx);
                            }

                            instructions.splice(loop_start..loop_start, to_move);

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
            Instruction::LoadVariable(_) => true,
            Instruction::StoreVariable(_) => false,
            Instruction::LoadLocal(idx) => !self.is_local_modified_in_loop(instructions, *idx, loop_start, loop_end),
            Instruction::StoreLocal(_) => false,

            Instruction::PushUndefined
            | Instruction::PushNull
            | Instruction::PushBoolean(_)
            | Instruction::PushNumber(_)
            | Instruction::PushString(_) => true,

            Instruction::BinaryOp(_) | Instruction::UnaryOp(_) => {
                self.are_operands_invariant(instructions, instr_pos, loop_start, loop_end)
            }

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
        let mut i = 0;
        while i < instructions.len() {
            match &instructions[i] {
                Instruction::Call(arg_count) => {
                    if self.should_inline_function(instructions, i, *arg_count) {
                        self.inline_function(instructions, i, *arg_count);
                    }
                }
                _ => {}
            }
            i += 1;
        }

        self.optimize_tail_recursion(instructions);
    }

    /// 检查是否应该内联函数
    fn should_inline_function(&self, instructions: &[Instruction], call_pos: usize, arg_count: u32) -> bool {
        if arg_count > 5 {
            return false;
        }

        if let Some((function_body, param_count)) = self.find_function_definition(instructions, call_pos) {
            if function_body.len() < 100 && param_count == arg_count {
                return true;
            }
        }

        false
    }

    /// 查找函数定义
    fn find_function_definition(&self, instructions: &[Instruction], call_pos: usize) -> Option<(Vec<Instruction>, u32)> {
        for i in (0..call_pos).rev() {
            match &instructions[i] {
                Instruction::SetFunctionBody(body) => {
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
        if let Some((function_body, param_count)) = self.find_function_definition(instructions, call_pos) {
            if param_count == arg_count {
                let mut inline_instructions = vec![];

                for i in 0..arg_count as usize {
                    inline_instructions.push(Instruction::StoreLocal(i));
                }

                inline_instructions.extend(function_body);

                instructions.splice(call_pos - arg_count as usize..=call_pos, inline_instructions);
            }
        }
    }

    /// 优化尾递归
    fn optimize_tail_recursion(&self, instructions: &mut Vec<Instruction>) {
        let mut i = 0;
        while i < instructions.len() {
            if let Instruction::Call(arg_count) = instructions[i] {
                if i + 1 < instructions.len() && matches!(instructions[i + 1], Instruction::Return) {
                    self.replace_tail_recursion_with_loop(instructions, i, arg_count);
                }
            }
            i += 1;
        }
    }

    /// 将尾递归替换为循环
    fn replace_tail_recursion_with_loop(&self, instructions: &mut Vec<Instruction>, call_pos: usize, _arg_count: u32) {
        let mut loop_instructions = vec![];

        loop_instructions.push(Instruction::Jump(0));

        instructions.splice(call_pos..call_pos + 2, loop_instructions);
    }

    /// 寄存器分配
    fn register_allocation(&self, instructions: &mut Vec<Instruction>) {
        let mut registers: HashMap<String, usize> = HashMap::new();
        let mut register_count = 0;
        let max_registers = 8;

        let mut live_variables = self.analyze_liveness(instructions);

        self.reorder_instructions(instructions);
        self.optimize_stack_operations(instructions);

        let mut i = 0;
        while i < instructions.len() {
            match &instructions[i] {
                Instruction::LoadVariable(name) => {
                    if !registers.contains_key(name) {
                        if register_count < max_registers {
                            registers.insert(name.to_string(), register_count);
                            register_count += 1;
                        }
                        else {
                            self.spill_register(&mut registers, &live_variables, i);
                            registers.insert(name.to_string(), register_count - 1);
                        }
                    }
                }
                Instruction::StoreVariable(name) => {
                    if !registers.contains_key(name) {
                        if register_count < max_registers {
                            registers.insert(name.to_string(), register_count);
                            register_count += 1;
                        }
                        else {
                            self.spill_register(&mut registers, &live_variables, i);
                            registers.insert(name.to_string(), register_count - 1);
                        }
                    }
                }
                Instruction::LoadLocal(idx) => {
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
        let mut reordered = vec![];
        let mut i = 0;

        while i < instructions.len() {
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
                for instr in parallel_instructions {
                    reordered.push(instr);
                }
                i = j;
            }
            else {
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
                    instructions[i] = Instruction::Pop;
                    instructions.drain(i + 1..j);
                    continue;
                }
            }

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
                    instructions[i] = Instruction::Dup;
                    instructions.drain(i + 1..j);
                    continue;
                }
            }

            i += 1;
        }
    }
}
