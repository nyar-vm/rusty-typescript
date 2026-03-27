//! vm 模块
//!
//! 虚拟机模块
//!
//! 提供 TypeScript 代码的执行环境，包括异常处理、模块系统、内置函数库等。

use std::collections::HashMap;

use typescript_ir::Program;
use typescript_types::{TsError, TsValue};

use crate::{
    codegen::Instruction,
    vm::{Builtins, CallFrame, ExceptionHandler, Function, ModuleInstance, PerformanceMonitor},
};

use super::{
    exception_ops::ExceptionOperations,
    executor::InstructionExecutor,
    function_ops::{ClassOperations, FunctionOperations, TypeOperations},
    memory::MemoryManager,
    module_ops::ModuleOperations,
    object_ops::{ArrayOperations, ObjectOperations},
    operations::{BinaryOperations, UnaryOperations},
};

/// 虚拟机
///
/// 执行 TypeScript IR 指令的运行时环境
pub struct VM {
    /// 全局变量（使用 HashMap 提高查找速度）
    globals: HashMap<String, TsValue>,
    /// 栈
    stack: Vec<TsValue>,
    /// 指令指针
    ip: usize,
    /// 函数环境
    functions: HashMap<String, Function>,
    /// 调用栈
    call_stack: Vec<CallFrame>,
    /// 异常处理栈
    exception_handlers: Vec<ExceptionHandler>,
    /// 模块缓存
    modules: HashMap<String, ModuleInstance>,
    /// 内置对象
    builtins: Builtins,
    /// 性能监控
    perf_monitor: PerformanceMonitor,
    /// 内存管理器
    memory: MemoryManager,
}

impl VM {
    /// 创建一个新的虚拟机
    pub fn new(globals: Vec<(String, TsValue)>) -> Self {
        let builtins = Builtins::new();
        let mut globals_map = HashMap::with_capacity(globals.len() + 20);
        for (key, value) in globals {
            globals_map.insert(key, value);
        }

        let mut vm = Self {
            globals: globals_map,
            stack: Vec::with_capacity(1024),
            ip: 0,
            functions: HashMap::new(),
            call_stack: Vec::with_capacity(128),
            exception_handlers: Vec::with_capacity(64),
            modules: HashMap::new(),
            builtins,
            perf_monitor: PerformanceMonitor::new(),
            memory: MemoryManager::new(),
        };

        vm.init_builtins();
        vm
    }

    /// 初始化内置对象
    fn init_builtins(&mut self) {
        self.globals.insert("console".to_string(), self.builtins.console_object());
        self.globals.insert("Math".to_string(), self.builtins.math_object());
        self.globals.insert("JSON".to_string(), self.builtins.json_object());
        self.globals.insert("Date".to_string(), self.builtins.date_constructor.clone());
        self.globals.insert("RegExp".to_string(), self.builtins.regexp_constructor.clone());
        self.globals.insert("Map".to_string(), self.builtins.map_constructor.clone());
        self.globals.insert("Set".to_string(), self.builtins.set_constructor.clone());
        self.globals.insert("Array".to_string(), self.builtins.array_constructor.clone());
        self.globals.insert("Object".to_string(), self.builtins.object_constructor.clone());
        self.globals.insert("String".to_string(), self.builtins.string_constructor.clone());
        self.globals.insert("Number".to_string(), self.builtins.number_constructor.clone());
        self.globals.insert("Boolean".to_string(), self.builtins.boolean_constructor.clone());
        self.globals.insert("Symbol".to_string(), self.builtins.symbol_constructor.clone());
        self.globals.insert("BigInt".to_string(), self.builtins.bigint_constructor.clone());
    }

    /// 执行程序
    pub fn execute(&mut self, program: &Program) -> Result<TsValue, TsError> {
        let instructions = crate::codegen::ir_to_vm_instructions(program)?;
        self.execute_instructions(&instructions)
    }

    /// 执行程序（使用JIT编译）
    pub fn execute_with_jit(&mut self, program: &Program) -> Result<TsValue, TsError> {
        self.execute(program)
    }

    /// 加载变量
    fn load_variable(&mut self, name: &str) -> Result<(), TsError> {
        InstructionExecutor::load_variable(name, &mut self.stack, &self.globals, &self.call_stack)
    }

    /// 存储变量
    fn store_variable(&mut self, name: &str) -> Result<(), TsError> {
        InstructionExecutor::store_variable(name, &mut self.stack, &mut self.globals, &mut self.call_stack)
    }

    /// 加载局部变量
    fn load_local(&mut self, index: usize) -> Result<(), TsError> {
        InstructionExecutor::load_local(index, &mut self.stack, &self.call_stack)
    }

    /// 存储局部变量
    fn store_local(&mut self, index: usize) -> Result<(), TsError> {
        InstructionExecutor::store_local(index, &mut self.stack, &mut self.call_stack)
    }

    /// 为JIT编译的函数设置局部变量
    pub fn set_local(&mut self, index: usize, value: TsValue) {
        if let Some(frame) = self.call_stack.last_mut() {
            frame.set_local_by_index(index, value);
        }
    }

    /// 执行指令序列（公开方法，供JIT使用）
    pub fn execute_instructions(&mut self, instructions: &[Instruction]) -> Result<TsValue, TsError> {
        self.ip = 0;
        self.stack.clear();
        self.call_stack.clear();
        self.exception_handlers.clear();
        self.perf_monitor.start();

        let mut i = 0;
        while i < instructions.len() {
            match &instructions[i] {
                Instruction::PushUndefined => {
                    self.stack.push(TsValue::Undefined);
                    i += 1;
                }
                Instruction::PushNull => {
                    self.stack.push(TsValue::Null);
                    i += 1;
                }
                Instruction::PushBoolean(b) => {
                    self.stack.push(TsValue::Boolean(*b));
                    i += 1;
                }
                Instruction::PushNumber(n) => {
                    self.stack.push(TsValue::Number(*n));
                    i += 1;
                }
                Instruction::PushString(s) => {
                    self.stack.push(TsValue::String(s.clone()));
                    i += 1;
                }
                Instruction::Push(value) => {
                    self.stack.push(value.clone());
                    i += 1;
                }

                Instruction::LoadVariable(name) => {
                    self.load_variable(name)?;
                    i += 1;
                }
                Instruction::StoreVariable(name) => {
                    self.store_variable(name)?;
                    i += 1;
                }
                Instruction::LoadLocal(index) => {
                    self.load_local(*index)?;
                    i += 1;
                }
                Instruction::StoreLocal(index) => {
                    self.store_local(*index)?;
                    i += 1;
                }

                Instruction::CreateObject | Instruction::CreateObject(_size) => {
                    ObjectOperations::create_object(&mut self.memory, &mut self.stack);
                    i += 1;
                }
                Instruction::GetProperty | Instruction::GetMember => {
                    let property = self.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
                    let object = self.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
                    ObjectOperations::get_property(object, property, &mut self.stack)?;
                    i += 1;
                }
                Instruction::SetProperty | Instruction::SetMember => {
                    let property = self.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
                    let value = self.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
                    let object = self.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
                    ObjectOperations::set_property(object, property, value, &mut self.stack)?;
                    i += 1;
                }

                Instruction::CreateArray(_size) => {
                    ArrayOperations::create_array(&mut self.memory, &mut self.stack);
                    i += 1;
                }
                Instruction::GetIndex => {
                    let index = self.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
                    let array = self.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
                    ArrayOperations::get_element(array, index, &mut self.stack)?;
                    i += 1;
                }
                Instruction::SetIndex => {
                    let index = self.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
                    let value = self.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
                    let array = self.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
                    ArrayOperations::set_element(array, index, value, &mut self.memory, &mut self.stack)?;
                    i += 1;
                }

                Instruction::CreateFunction(name, param_count) => {
                    FunctionOperations::create_function(name, *param_count, &mut self.functions, &mut self.stack)?;
                    i += 1;
                }
                Instruction::Call(arg_count) => {
                    FunctionOperations::call_function(
                        *arg_count,
                        &mut self.stack,
                        &mut self.perf_monitor,
                        self.call_stack.len(),
                    )?;
                    i += 1;
                }
                Instruction::Return => {
                    return self.handle_return();
                }

                Instruction::CreateClass(name) => {
                    ClassOperations::create_class(name, &mut self.stack)?;
                    i += 1;
                }
                Instruction::AddMethod(name) => {
                    ClassOperations::add_method(&mut self.stack)?;
                    i += 1;
                }

                Instruction::BinaryOp(_op) => {
                    i += 1;
                }
                Instruction::UnaryOp(_op) => {
                    i += 1;
                }

                Instruction::Jump(offset) => {
                    i = InstructionExecutor::jump(*offset as i32, i);
                }
                Instruction::JumpIfFalse(offset) => {
                    i = InstructionExecutor::jump_if_false(*offset as i32, i, &mut self.stack)?;
                }
                Instruction::JumpLoop(_kind) => {
                    i += 1;
                }

                Instruction::TryStart(_handler) => {
                    i += 1;
                }
                Instruction::TryEnd => {
                    ExceptionOperations::try_end(&mut self.exception_handlers)?;
                    i += 1;
                }
                Instruction::Throw => {
                    ExceptionOperations::throw_exception(&mut self.stack)?;
                    i += 1;
                }
                Instruction::Catch(_handler) => {
                    i += 1;
                }
                Instruction::Finally => {
                    i += 1;
                }

                Instruction::Pop => {
                    self.stack.pop();
                    i += 1;
                }
            }
            self.perf_monitor.record_instruction();
        }

        Ok(self.stack.pop().unwrap_or(TsValue::Undefined))
    }

    /// 获取属性
    fn get_property(&mut self) -> Result<(), TsError> {
        let property = self.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
        let object = self.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
        ObjectOperations::get_property(object, property, &mut self.stack)
    }

    /// 设置属性
    fn set_property(&mut self) -> Result<(), TsError> {
        let property = self.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
        let value = self.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
        let object = self.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
        ObjectOperations::set_property(object, property, value, &mut self.stack)
    }

    /// 获取数组元素
    fn get_element(&mut self) -> Result<(), TsError> {
        let index = self.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
        let array = self.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
        ArrayOperations::get_element(array, index, &mut self.stack)
    }

    /// 设置数组元素
    fn set_element(&mut self) -> Result<(), TsError> {
        let index = self.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
        let value = self.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
        let array = self.stack.pop().ok_or_else(|| TsError::TypeError("Stack underflow".to_string()))?;
        ArrayOperations::set_element(array, index, value, &mut self.memory, &mut self.stack)
    }

    /// 创建函数
    fn create_function(&mut self, name: &str, param_count: u32) -> Result<(), TsError> {
        FunctionOperations::create_function(name, param_count, &mut self.functions, &mut self.stack)
    }

    /// 设置函数体
    fn set_function_body(&mut self, _body: &[Instruction]) -> Result<(), TsError> {
        Ok(())
    }

    /// 调用函数
    fn call_function(&mut self, arg_count: u32) -> Result<(), TsError> {
        FunctionOperations::call_function(arg_count, &mut self.stack, &mut self.perf_monitor, self.call_stack.len())
    }

    /// 处理返回
    fn handle_return(&mut self) -> Result<TsValue, TsError> {
        let result = self.stack.pop().unwrap_or(TsValue::Undefined);
        if let Some(frame) = self.call_stack.pop() {
            self.ip = frame.return_ip;
        }
        Ok(result)
    }

    /// 创建类
    fn create_class(&mut self, name: &str) -> Result<(), TsError> {
        ClassOperations::create_class(name, &mut self.stack)
    }

    /// 添加方法
    fn add_method(&mut self, _name: &str) -> Result<(), TsError> {
        ClassOperations::add_method(&mut self.stack)
    }

    /// 设置类体
    fn set_class_body(&mut self, _body: &[Instruction]) -> Result<(), TsError> {
        if let Some(TsValue::Object(class_obj)) = self.stack.pop() {
            self.stack.push(TsValue::Object(class_obj));
        }
        Ok(())
    }

    /// 创建类型别名
    fn create_type_alias(&mut self, name: &str) -> Result<(), TsError> {
        TypeOperations::create_type_alias(name, &mut self.stack)
    }

    /// 创建接口
    fn create_interface(&mut self, name: &str) -> Result<(), TsError> {
        TypeOperations::create_interface(name, &mut self.stack)
    }

    /// 二元操作
    fn binary_op(&mut self, op: &str) -> Result<(), TsError> {
        if let (Some(right), Some(left)) = (self.stack.pop(), self.stack.pop()) {
            let result = BinaryOperations::execute(op, left, right)?;
            self.stack.push(result);
            Ok(())
        }
        else {
            Err(TsError::TypeError("Stack underflow".to_string()))
        }
    }

    /// 一元操作
    fn unary_op(&mut self, op: &str) -> Result<(), TsError> {
        if let Some(value) = self.stack.pop() {
            let result = UnaryOperations::execute(op, value)?;
            self.stack.push(result);
            Ok(())
        }
        else {
            Err(TsError::TypeError("Stack underflow".to_string()))
        }
    }

    /// 条件跳转
    fn jump_if_false(&mut self, offset: i32) -> Result<(), TsError> {
        if let Some(value) = self.stack.pop() {
            if !value.to_boolean() {
                self.ip = (self.ip as i32 + offset as i32) as usize;
            }
            Ok(())
        }
        else {
            Err(TsError::TypeError("Stack underflow".to_string()))
        }
    }

    /// 循环跳转
    fn jump_loop(&mut self, _kind: u8) -> Result<(), TsError> {
        Ok(())
    }

    /// 开始 try 块
    fn try_start(
        &mut self,
        handler_ip: usize,
        finally_ip: Option<usize>,
        exception_var: Option<String>,
    ) -> Result<(), TsError> {
        ExceptionOperations::try_start(self.ip, handler_ip, finally_ip, exception_var, &mut self.exception_handlers)
    }

    /// 结束 try 块
    fn try_end(&mut self) -> Result<(), TsError> {
        ExceptionOperations::try_end(&mut self.exception_handlers)
    }

    /// 抛出异常
    fn throw_exception(&mut self) -> Result<(), TsError> {
        ExceptionOperations::throw_exception(&mut self.stack)
    }

    /// 检查是否有异常
    fn check_exception(&self) -> Option<TsError> {
        None
    }

    /// 查找异常处理器
    fn find_exception_handler(&self, ip: usize) -> Option<ExceptionHandler> {
        ExceptionOperations::find_exception_handler(ip, &self.exception_handlers)
    }

    /// 处理异常
    fn handle_exception(&mut self, exception: TsError, handler: &ExceptionHandler) -> Result<(), TsError> {
        let new_ip = ExceptionOperations::handle_exception(exception, handler, &mut self.call_stack)?;
        self.ip = new_ip;
        Ok(())
    }

    /// 导入模块
    fn import_module(&mut self, name: &str, alias: Option<&str>) -> Result<(), TsError> {
        ModuleOperations::import_module(name, alias, &mut self.modules, &mut self.globals)
    }

    /// 导入 NAPI 模块
    fn import_napi_module(&mut self, name: &str, alias: Option<&str>) -> Result<(), TsError> {
        ModuleOperations::import_napi_module(name, alias, &mut self.modules, &mut self.globals)
    }

    /// 导出值
    fn export_value(&mut self, name: &str) -> Result<(), TsError> {
        ModuleOperations::export_value(name, &mut self.stack, &mut self.modules)
    }

    /// 获取性能报告
    pub fn performance_report(&self) -> String {
        self.perf_monitor.report()
    }

    /// 获取调用栈深度
    pub fn call_stack_depth(&self) -> usize {
        self.call_stack.len()
    }

    /// 获取全局变量
    pub fn globals(&self) -> Vec<(String, TsValue)> {
        self.globals.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// 清理池中的对象和数组
    pub fn clear_pools(&mut self) {
        self.memory.clear_pools();
    }

    /// 执行垃圾回收
    pub fn garbage_collect(&mut self) {
        self.memory.garbage_collect(&mut self.stack);
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new(vec![])
    }
}
