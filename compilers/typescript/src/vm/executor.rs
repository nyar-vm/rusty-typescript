//! 指令执行器模块
//!
//! 提供指令执行的主循环逻辑。

use typescript_types::{TsError, TsValue};

use crate::codegen::Instruction;

use super::{
    exception_ops::ExceptionOperations,
    function_ops::{ClassOperations, FunctionOperations},
    memory::MemoryManager,
    object_ops::{ArrayOperations, ObjectOperations},
    perf::PerformanceMonitor,
};

/// 执行上下文
pub struct ExecutionContext<'a> {
    /// 栈
    pub stack: &'a mut Vec<TsValue>,
    /// 内存管理器
    pub memory: &'a mut MemoryManager,
    /// 性能监控
    pub perf_monitor: &'a mut PerformanceMonitor,
    /// 当前指令指针
    pub ip: usize,
}

/// 指令执行器
pub struct InstructionExecutor;

impl InstructionExecutor {
    /// 执行常量压栈指令
    pub fn push_constant(stack: &mut Vec<TsValue>, value: TsValue) {
        stack.push(value);
    }

    /// 执行变量加载指令
    pub fn load_variable(
        name: &str,
        stack: &mut Vec<TsValue>,
        globals: &std::collections::HashMap<String, TsValue>,
        call_stack: &[super::CallFrame],
    ) -> Result<(), TsError> {
        if let Some(frame) = call_stack.last() {
            if let Some(value) = frame.get_local(name) {
                stack.push(value.clone());
                return Ok(());
            }
        }

        if let Some(value) = globals.get(name) {
            stack.push(value.clone());
            return Ok(());
        }

        Err(TsError::ReferenceError(format!("Variable '{}' is not defined", name)))
    }

    /// 执行变量存储指令
    pub fn store_variable(
        name: &str,
        stack: &mut Vec<TsValue>,
        globals: &mut std::collections::HashMap<String, TsValue>,
        call_stack: &mut Vec<super::CallFrame>,
    ) -> Result<(), TsError> {
        if let Some(value) = stack.pop() {
            let value_clone = value.clone();

            if let Some(frame) = call_stack.last_mut() {
                if frame.get_local(name).is_some() {
                    frame.set_local(name, value);
                    stack.push(value_clone);
                    return Ok(());
                }
            }

            globals.insert(name.to_string(), value);
            stack.push(value_clone);
            Ok(())
        }
        else {
            Err(TsError::TypeError("Stack underflow".to_string()))
        }
    }

    /// 执行局部变量加载指令
    pub fn load_local(index: usize, stack: &mut Vec<TsValue>, call_stack: &[super::CallFrame]) -> Result<(), TsError> {
        if let Some(frame) = call_stack.last() {
            if let Some(value) = frame.get_local_by_index(index) {
                stack.push(value.clone());
                return Ok(());
            }
        }
        Err(TsError::ReferenceError(format!("Local variable at index {} not found", index)))
    }

    /// 执行局部变量存储指令
    pub fn store_local(index: usize, stack: &mut Vec<TsValue>, call_stack: &mut Vec<super::CallFrame>) -> Result<(), TsError> {
        if let Some(value) = stack.pop() {
            if let Some(frame) = call_stack.last_mut() {
                frame.set_local_by_index(index, value);
                return Ok(());
            }
        }
        Err(TsError::TypeError("Failed to store local variable".to_string()))
    }

    /// 执行条件跳转指令
    pub fn jump_if_false(offset: i32, current_ip: usize, stack: &mut Vec<TsValue>) -> Result<usize, TsError> {
        if let Some(value) = stack.pop() {
            if !value.to_boolean() { Ok((current_ip as i32 + offset as i32) as usize) } else { Ok(current_ip + 1) }
        }
        else {
            Err(TsError::TypeError("Stack underflow".to_string()))
        }
    }

    /// 执行无条件跳转指令
    pub fn jump(offset: i32, current_ip: usize) -> usize {
        (current_ip as i32 + offset as i32) as usize
    }

    /// 处理单条指令
    #[allow(clippy::too_many_arguments)]
    pub fn execute_instruction(
        instruction: &Instruction,
        ctx: &mut ExecutionContext,
        globals: &mut std::collections::HashMap<String, TsValue>,
        functions: &mut std::collections::HashMap<String, super::Function>,
        _modules: &mut std::collections::HashMap<String, super::ModuleInstance>,
        call_stack: &mut Vec<super::CallFrame>,
        exception_handlers: &mut Vec<super::ExceptionHandler>,
    ) -> Result<Option<TsValue>, TsError> {
        match instruction {
            Instruction::PushUndefined => {
                ctx.stack.push(TsValue::Undefined);
            }
            Instruction::PushNull => {
                ctx.stack.push(TsValue::Null);
            }
            Instruction::PushBoolean(b) => {
                ctx.stack.push(TsValue::Boolean(*b));
            }
            Instruction::PushNumber(n) => {
                ctx.stack.push(TsValue::Number(*n));
            }
            Instruction::PushString(s) => {
                ctx.stack.push(TsValue::String(s.clone()));
            }

            Instruction::LoadVariable(name) => {
                Self::load_variable(name, ctx.stack, globals, call_stack)?;
            }
            Instruction::StoreVariable(name) => {
                Self::store_variable(name, ctx.stack, globals, call_stack)?;
            }
            Instruction::LoadLocal(index) => {
                Self::load_local(*index, ctx.stack, call_stack)?;
            }
            Instruction::StoreLocal(index) => {
                Self::store_local(*index, ctx.stack, call_stack)?;
            }

            Instruction::CreateObject => {
                ObjectOperations::create_object(ctx.memory, ctx.stack);
            }
            Instruction::GetProperty => {
                let property = ctx.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
                let object = ctx.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
                ObjectOperations::get_property(object, property, ctx.stack)?;
            }
            Instruction::SetProperty => {
                let property = ctx.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
                let value = ctx.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
                let object = ctx.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
                ObjectOperations::set_property(object, property, value, ctx.stack)?;
            }

            Instruction::CreateArray => {
                ArrayOperations::create_array(ctx.memory, ctx.stack);
            }
            Instruction::GetElement => {
                let index = ctx.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
                let array = ctx.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
                ArrayOperations::get_element(array, index, ctx.stack)?;
            }
            Instruction::SetElement => {
                let index = ctx.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
                let value = ctx.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
                let array = ctx.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
                ArrayOperations::set_element(array, index, value, ctx.memory, ctx.stack)?;
            }

            Instruction::CreateFunction(name, param_count) => {
                FunctionOperations::create_function(name, *param_count, functions, ctx.stack)?;
            }
            Instruction::Call(arg_count) => {
                FunctionOperations::call_function(*arg_count, ctx.stack, ctx.perf_monitor, call_stack.len())?;
            }
            Instruction::Return => {
                let result = ctx.stack.pop().unwrap_or(TsValue::Undefined);
                if let Some(frame) = call_stack.pop() {
                    ctx.ip = frame.return_ip;
                }
                return Ok(Some(result));
            }

            Instruction::CreateClass(name) => {
                ClassOperations::create_class(name, ctx.stack)?;
            }
            Instruction::AddMethod(_name) => {
                ClassOperations::add_method(ctx.stack)?;
            }

            Instruction::BinaryOp(_op) => {}
            Instruction::UnaryOp(_op) => {}

            Instruction::Jump(offset) => {
                ctx.ip = Self::jump(*offset as i32, ctx.ip);
                ctx.perf_monitor.record_instruction();
                return Ok(None);
            }
            Instruction::JumpIfFalse(offset) => {
                ctx.ip = Self::jump_if_false(*offset as i32, ctx.ip, ctx.stack)?;
                ctx.perf_monitor.record_instruction();
                return Ok(None);
            }
            Instruction::JumpLoop(_kind) => {}

            Instruction::Throw => {
                return ExceptionOperations::throw_exception(ctx.stack).map(|_| None);
            }
            Instruction::TryStart { handler_ip: _, finally_ip: _, exception_var: _ } => {}
            Instruction::TryEnd => {
                ExceptionOperations::try_end(exception_handlers)?;
            }

            Instruction::Pop => {
                ctx.stack.pop();
            }
            Instruction::Dup => {
                if let Some(value) = ctx.stack.last().cloned() {
                    ctx.stack.push(value);
                }
            }
            Instruction::Swap => {
                if ctx.stack.len() >= 2 {
                    let len = ctx.stack.len();
                    ctx.stack.swap(len - 1, len - 2);
                }
            }
            Instruction::SetFunctionBody(_) => {
                // 简化实现
            }
            Instruction::SetClassBody(_) => {
                // 简化实现
            }
            Instruction::CreateTypeAlias(_) => {
                // 简化实现
            }
            Instruction::CreateInterface(_) => {
                // 简化实现
            }
            Instruction::ImportModule { .. } => {
                // 简化实现
            }
            Instruction::Export { .. } => {
                // 简化实现
            }
        }

        ctx.perf_monitor.record_instruction();
        Ok(None)
    }
}
