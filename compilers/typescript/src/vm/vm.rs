//! vm 模块
//!
//! 虚拟机模块
//!
//! 提供 TypeScript 代码的执行环境，包括异常处理、模块系统、内置函数库等。

use std::{collections::HashMap, rc::Rc};
use typescript_ir::Program;
use typescript_types::{TsError, TsValue};

use crate::codegen::{BinaryOp, Instruction, UnaryOp};
use crate::vm::{Builtins, CallFrame, ExceptionHandler, Function, ModuleInstance, PerformanceMonitor};

/// 虚拟机
///
/// 执行 TypeScript IR 指令的运行时环境
pub struct VM {
    /// 全局变量
    globals: Vec<(String, TsValue)>,
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
}

/// 调用帧
#[derive(Debug, Clone)]
impl VM {
    /// 创建一个新的虚拟机
    pub fn new(globals: Vec<(String, TsValue)>) -> Self {
        let builtins = Builtins::new();
        let mut vm = Self {
            globals,
            stack: Vec::with_capacity(1024), // 预分配栈空间
            ip: 0,
            functions: HashMap::new(),
            call_stack: Vec::with_capacity(128), // 预分配调用栈空间
            exception_handlers: Vec::with_capacity(64), // 预分配异常处理栈空间
            modules: HashMap::new(),
            builtins,
            perf_monitor: PerformanceMonitor::new(),
        };

        // 初始化全局内置对象
        vm.init_builtins();
        vm
    }

    /// 初始化内置对象
    fn init_builtins(&mut self) {
        self.globals.push(("console".to_string(), self.builtins.console_object()));
        self.globals.push(("Math".to_string(), self.builtins.math_object()));
        self.globals.push(("JSON".to_string(), self.builtins.json_object()));
        self.globals.push(("Date".to_string(), self.builtins.date_constructor.clone()));
        self.globals.push(("RegExp".to_string(), self.builtins.regexp_constructor.clone()));
        self.globals.push(("Map".to_string(), self.builtins.map_constructor.clone()));
        self.globals.push(("Set".to_string(), self.builtins.set_constructor.clone()));
        self.globals.push(("Array".to_string(), self.builtins.array_constructor.clone()));
        self.globals.push(("Object".to_string(), self.builtins.object_constructor.clone()));
        self.globals.push(("String".to_string(), self.builtins.string_constructor.clone()));
        self.globals.push(("Number".to_string(), self.builtins.number_constructor.clone()));
        self.globals.push(("Boolean".to_string(), self.builtins.boolean_constructor.clone()));
        self.globals.push(("Symbol".to_string(), self.builtins.symbol_constructor.clone()));
        self.globals.push(("BigInt".to_string(), self.builtins.bigint_constructor.clone()));
    }

    /// 执行程序
    pub fn execute(&mut self, program: &Program) -> Result<TsValue, TsError> {
        // 生成 VM 指令
        let instructions = crate::codegen::ir_to_vm_instructions(program)?;

        // 执行指令
        self.execute_instructions(&instructions)
    }

    /// 执行指令序列
    fn execute_instructions(&mut self, instructions: &[Instruction]) -> Result<TsValue, TsError> {
        self.ip = 0;
        self.stack.clear();
        self.call_stack.clear();
        self.exception_handlers.clear();
        self.perf_monitor.start();

        while self.ip < instructions.len() {
            let instruction = &instructions[self.ip];
            self.ip += 1;
            self.perf_monitor.record_instruction();

            match instruction {
                // 常量操作
                Instruction::PushUndefined => {
                    self.stack.push(TsValue::Undefined);
                }
                Instruction::PushNull => {
                    self.stack.push(TsValue::Null);
                }
                Instruction::PushBoolean(b) => {
                    self.stack.push(TsValue::Boolean(*b));
                }
                Instruction::PushNumber(n) => {
                    self.stack.push(TsValue::Number(*n));
                }
                Instruction::PushString(s) => {
                    self.stack.push(TsValue::String(s.clone()));
                }

                // 变量操作
                Instruction::LoadVariable(name) => {
                    self.load_variable(name)?;
                }
                Instruction::StoreVariable(name) => {
                    self.store_variable(name)?;
                }
                Instruction::LoadLocal(index) => {
                    self.load_local(*index)?;
                }
                Instruction::StoreLocal(index) => {
                    self.store_local(*index)?;
                }

                // 对象操作
                Instruction::CreateObject => {
                    self.stack.push(TsValue::Object(vec![]));
                }
                Instruction::GetProperty => {
                    self.get_property()?;
                }
                Instruction::SetProperty => {
                    self.set_property()?;
                }

                // 数组操作
                Instruction::CreateArray => {
                    self.stack.push(TsValue::Array(vec![]));
                }
                Instruction::GetElement => {
                    self.get_element()?;
                }
                Instruction::SetElement => {
                    self.set_element()?;
                }

                // 函数操作
                Instruction::CreateFunction(name, param_count) => {
                    self.create_function(name, *param_count)?;
                }
                Instruction::SetFunctionBody(body) => {
                    self.set_function_body(body)?;
                }
                Instruction::Call(arg_count) => {
                    self.call_function(*arg_count)?;
                }
                Instruction::Return => {
                    return self.handle_return();
                }

                // 类操作
                Instruction::CreateClass(name) => {
                    self.create_class(name)?;
                }
                Instruction::AddMethod(name) => {
                    self.add_method(name)?;
                }
                Instruction::SetClassBody(body) => {
                    self.set_class_body(body)?;
                }

                // 类型操作
                Instruction::CreateTypeAlias(name) => {
                    self.create_type_alias(name)?;
                }
                Instruction::CreateInterface(name) => {
                    self.create_interface(name)?;
                }

                // 二元操作
                Instruction::BinaryOp(op) => {
                    self.binary_op(op)?;
                }

                // 一元操作
                Instruction::UnaryOp(op) => {
                    self.unary_op(op)?;
                }

                // 控制流
                Instruction::Jump(offset) => {
                    self.ip = (self.ip as i32 + *offset as i32) as usize;
                }
                Instruction::JumpIfFalse(offset) => {
                    self.jump_if_false(*offset as i32)?;
                }
                Instruction::JumpLoop(kind) => {
                    self.jump_loop(*kind as u8)?;
                }

                // 异常处理
                Instruction::TryStart { handler_ip, finally_ip, exception_var } => {
                    self.try_start(*handler_ip, *finally_ip, exception_var.clone())?;
                }
                Instruction::TryEnd => {
                    self.try_end()?;
                }
                Instruction::Throw => {
                    self.throw_exception()?;
                }

                // 模块操作
                Instruction::ImportModule { name, alias } => {
                    self.import_module(name, alias.as_deref())?;
                }
                Instruction::Export { name } => {
                    self.export_value(name)?;
                }

                // 栈操作
                Instruction::Pop => {
                    self.stack.pop();
                }
                Instruction::Dup => {
                    if let Some(top) = self.stack.last() {
                        self.stack.push(top.clone());
                    }
                }
                Instruction::Swap => {
                    let len = self.stack.len();
                    if len >= 2 {
                        self.stack.swap(len - 1, len - 2);
                    }
                }
            }
        }

        // 返回栈顶值
        Ok(self.stack.pop().unwrap_or(TsValue::Undefined))
    }

    /// 加载变量
    fn load_variable(&mut self, name: &str) -> Result<(), TsError> {
        // 首先查找局部变量
        if let Some(frame) = self.call_stack.last() {
            if let Some(value) = frame.get_local(name) {
                self.stack.push(value.clone());
                return Ok(());
            }
        }

        // 然后查找全局变量（使用线性查找，因为全局变量数量通常不多）
        for (n, value) in &self.globals {
            if n == name {
                self.stack.push(value.clone());
                return Ok(());
            }
        }

        Err(TsError::ReferenceError(format!("Variable '{}' is not defined", name)))
    }

    /// 存储变量
    fn store_variable(&mut self, name: &str) -> Result<(), TsError> {
        if let Some(value) = self.stack.pop() {
            let value_clone = value.clone();

            // 首先尝试更新局部变量
            if let Some(frame) = self.call_stack.last_mut() {
                if frame.get_local(name).is_some() {
                    frame.set_local(name, value);
                    self.stack.push(value_clone);
                    return Ok(());
                }
            }

            // 然后尝试更新全局变量
            if let Some((_, val)) = self.globals.iter_mut().find(|(n, _)| n == name) {
                *val = value;
            }
            else {
                // 添加新的全局变量
                self.globals.push((name.to_string(), value));
            }

            self.stack.push(value_clone);
            Ok(())
        }
        else {
            Err(TsError::TypeError("Stack underflow".to_string()))
        }
    }

    /// 加载局部变量
    fn load_local(&mut self, index: usize) -> Result<(), TsError> {
        if let Some(frame) = self.call_stack.last() {
            if let Some(name) = frame.locals.keys().nth(index) {
                if let Some(value) = frame.get_local(name) {
                    self.stack.push(value.clone());
                    return Ok(());
                }
            }
        }
        Err(TsError::ReferenceError(format!("Local variable at index {} not found", index)))
    }

    /// 存储局部变量
    fn store_local(&mut self, index: usize) -> Result<(), TsError> {
        if let Some(value) = self.stack.pop() {
            if let Some(frame) = self.call_stack.last_mut() {
                if let Some(name) = frame.locals.keys().nth(index).cloned() {
                    frame.set_local(&name, value);
                    return Ok(());
                }
            }
        }
        Err(TsError::TypeError("Failed to store local variable".to_string()))
    }

    /// 获取属性
    fn get_property(&mut self) -> Result<(), TsError> {
        if let (Some(property), Some(object)) = (self.stack.pop(), self.stack.pop()) {
            match (object, property) {
                (TsValue::Object(props), TsValue::String(key)) => {
                    if let Some((_, value)) = props.iter().find(|(k, _)| k == &key) {
                        self.stack.push(value.clone());
                    }
                    else {
                        self.stack.push(TsValue::Undefined);
                    }
                    Ok(())
                }
                (TsValue::Array(elements), TsValue::String(key)) if key == "length" => {
                    self.stack.push(TsValue::Number(elements.len() as f64));
                    Ok(())
                }
                _ => Err(TsError::TypeError("Cannot get property of non-object".to_string())),
            }
        }
        else {
            Err(TsError::TypeError("Stack underflow".to_string()))
        }
    }

    /// 设置属性
    fn set_property(&mut self) -> Result<(), TsError> {
        if let (Some(property), Some(value), Some(object)) = (self.stack.pop(), self.stack.pop(), self.stack.pop()) {
            match (object, property) {
                (TsValue::Object(mut props), TsValue::String(key)) => {
                    if let Some((_, val)) = props.iter_mut().find(|(k, _)| k == &key) {
                        *val = value;
                    }
                    else {
                        props.push((key, value));
                    }
                    self.stack.push(TsValue::Object(props));
                    Ok(())
                }
                _ => Err(TsError::TypeError("Cannot set property of non-object".to_string())),
            }
        }
        else {
            Err(TsError::TypeError("Stack underflow".to_string()))
        }
    }

    /// 获取数组元素
    fn get_element(&mut self) -> Result<(), TsError> {
        if let (Some(index), Some(array)) = (self.stack.pop(), self.stack.pop()) {
            match (array, index) {
                (TsValue::Array(elements), TsValue::Number(idx)) => {
                    let idx = idx as usize;
                    if idx < elements.len() {
                        self.stack.push(elements[idx].clone());
                    }
                    else {
                        self.stack.push(TsValue::Undefined);
                    }
                    Ok(())
                }
                (TsValue::String(s), TsValue::Number(idx)) => {
                    let idx = idx as usize;
                    if let Some(ch) = s.chars().nth(idx) {
                        self.stack.push(TsValue::String(ch.to_string()));
                    }
                    else {
                        self.stack.push(TsValue::Undefined);
                    }
                    Ok(())
                }
                _ => Err(TsError::TypeError("Cannot get element of non-array".to_string())),
            }
        }
        else {
            Err(TsError::TypeError("Stack underflow".to_string()))
        }
    }

    /// 设置数组元素
    fn set_element(&mut self) -> Result<(), TsError> {
        if let (Some(index), Some(value), Some(array)) = (self.stack.pop(), self.stack.pop(), self.stack.pop()) {
            match (array, index) {
                (TsValue::Array(mut elements), TsValue::Number(idx)) => {
                    let idx = idx as usize;
                    if idx < elements.len() {
                        elements[idx] = value;
                    }
                    else {
                        elements.resize(idx + 1, TsValue::Undefined);
                        elements[idx] = value;
                    }
                    self.stack.push(TsValue::Array(elements));
                    Ok(())
                }
                _ => Err(TsError::TypeError("Cannot set element of non-array".to_string())),
            }
        }
        else {
            Err(TsError::TypeError("Stack underflow".to_string()))
        }
    }

    /// 创建函数
    fn create_function(&mut self, name: &str, param_count: u32) -> Result<(), TsError> {
        let func = Function::new(name.to_string(), param_count, vec![]);
        self.functions.insert(name.to_string(), func);
        self.stack.push(TsValue::Function(Rc::new(|_| TsValue::Undefined)));
        Ok(())
    }

    /// 设置函数体
    fn set_function_body(&mut self, _body: &[Instruction]) -> Result<(), TsError> {
        // 简化实现，实际应该将指令与函数关联
        Ok(())
    }

    /// 调用函数
    fn call_function(&mut self, arg_count: u32) -> Result<(), TsError> {
        if let Some(callee) = self.stack.pop() {
            match callee {
                TsValue::Function(func) => {
                    // 提取参数
                    let mut args = vec![];
                    for _ in 0..arg_count {
                        if let Some(arg) = self.stack.pop() {
                            args.push(arg);
                        }
                        else {
                            return Err(TsError::TypeError("Stack underflow".to_string()));
                        }
                    }
                    args.reverse();

                    // 记录函数调用
                    self.perf_monitor.record_call(self.call_stack.len());

                    // 调用函数
                    let result = func(&args);
                    self.stack.push(result);
                    Ok(())
                }
                _ => Err(TsError::TypeError("Cannot call non-function".to_string())),
            }
        }
        else {
            Err(TsError::TypeError("Stack underflow".to_string()))
        }
    }

    /// 处理返回
    fn handle_return(&mut self) -> Result<TsValue, TsError> {
        let result = self.stack.pop().unwrap_or(TsValue::Undefined);

        // 如果有调用帧，恢复状态
        if let Some(frame) = self.call_stack.pop() {
            self.ip = frame.return_ip;
        }

        Ok(result)
    }

    /// 创建类
    fn create_class(&mut self, name: &str) -> Result<(), TsError> {
        let class_obj = TsValue::Object(vec![
            ("name".to_string(), TsValue::String(name.to_string())),
            ("prototype".to_string(), TsValue::Object(vec![])),
        ]);
        self.stack.push(class_obj);
        Ok(())
    }

    /// 添加方法
    fn add_method(&mut self, _name: &str) -> Result<(), TsError> {
        // 简化实现
        self.stack.push(TsValue::Function(Rc::new(|_| TsValue::Undefined)));
        Ok(())
    }

    /// 设置类体
    fn set_class_body(&mut self, _body: &[Instruction]) -> Result<(), TsError> {
        // 简化实现
        if let Some(TsValue::Object(class_obj)) = self.stack.pop() {
            self.stack.push(TsValue::Object(class_obj));
        }
        Ok(())
    }

    /// 创建类型别名
    fn create_type_alias(&mut self, name: &str) -> Result<(), TsError> {
        let type_alias_obj = TsValue::Object(vec![("name".to_string(), TsValue::String(name.to_string()))]);
        self.stack.push(type_alias_obj);
        Ok(())
    }

    /// 创建接口
    fn create_interface(&mut self, name: &str) -> Result<(), TsError> {
        let interface_obj = TsValue::Object(vec![("name".to_string(), TsValue::String(name.to_string()))]);
        self.stack.push(interface_obj);
        Ok(())
    }

    /// 二元操作
    fn binary_op(&mut self, op: &BinaryOp) -> Result<(), TsError> {
        if let (Some(right), Some(left)) = (self.stack.pop(), self.stack.pop()) {
            let result = match op {
                BinaryOp::Add => self.binary_add(left, right)?,
                BinaryOp::Sub => self.binary_sub(left, right)?,
                BinaryOp::Mul => self.binary_mul(left, right)?,
                BinaryOp::Div => self.binary_div(left, right)?,
                BinaryOp::Mod => self.binary_mod(left, right)?,
                BinaryOp::Eq => self.binary_eq(left, right),
                BinaryOp::Neq => self.binary_neq(left, right),
                BinaryOp::StrictEq => self.binary_strict_eq(left, right),
                BinaryOp::StrictNeq => self.binary_strict_neq(left, right),
                BinaryOp::Gt => self.binary_gt(left, right)?,
                BinaryOp::Gte => self.binary_gte(left, right)?,
                BinaryOp::Lt => self.binary_lt(left, right)?,
                BinaryOp::Lte => self.binary_lte(left, right)?,
                BinaryOp::And => self.binary_and(left, right),
                BinaryOp::Or => self.binary_or(left, right),
                _ => TsValue::Undefined,
            };
            self.stack.push(result);
            Ok(())
        }
        else {
            Err(TsError::TypeError("Stack underflow".to_string()))
        }
    }

    /// 一元操作
    fn unary_op(&mut self, op: &UnaryOp) -> Result<(), TsError> {
        if let Some(value) = self.stack.pop() {
            let result = match op {
                UnaryOp::Not => TsValue::Boolean(!value.to_boolean()),
                UnaryOp::Neg => TsValue::Number(-value.to_number()),
                UnaryOp::Pos => TsValue::Number(value.to_number()),
                UnaryOp::TypeOf => TsValue::String(self.type_of(&value)),
                UnaryOp::Void => {
                    self.stack.push(TsValue::Undefined);
                    return Ok(());
                }
                UnaryOp::Delete => TsValue::Boolean(false), // 简化实现
                UnaryOp::BitNot => TsValue::Number(!(value.to_number() as i64) as f64),
                UnaryOp::Inc => TsValue::Number(value.to_number() + 1.0),
                UnaryOp::Dec => TsValue::Number(value.to_number() - 1.0),
            };
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
        // 简化实现
        Ok(())
    }

    /// 开始 try 块
    fn try_start(
        &mut self,
        handler_ip: usize,
        finally_ip: Option<usize>,
        exception_var: Option<String>,
    ) -> Result<(), TsError> {
        let handler = ExceptionHandler {
            start_ip: self.ip,
            end_ip: handler_ip,
            handler_ip,
            exception_var,
            has_finally: finally_ip.is_some(),
            finally_ip,
        };
        self.exception_handlers.push(handler);
        Ok(())
    }

    /// 结束 try 块
    fn try_end(&mut self) -> Result<(), TsError> {
        self.exception_handlers.pop();
        Ok(())
    }

    /// 抛出异常
    fn throw_exception(&mut self) -> Result<(), TsError> {
        if let Some(value) = self.stack.pop() {
            Err(TsError::Other(format!("Exception: {}", value.to_string())))
        }
        else {
            Err(TsError::TypeError("Stack underflow".to_string()))
        }
    }

    /// 检查是否有异常
    fn check_exception(&self) -> Option<TsError> {
        // 简化实现，实际应该在执行过程中捕获异常
        // 这里返回 None，因为异常是通过 Result 传递的
        None
    }

    /// 查找异常处理器
    fn find_exception_handler(&self, ip: usize) -> Option<ExceptionHandler> {
        for handler in &self.exception_handlers {
            if handler.start_ip <= ip && ip < handler.end_ip {
                return Some(handler.clone());
            }
        }
        None
    }

    /// 处理异常
    fn handle_exception(&mut self, exception: TsError, handler: &ExceptionHandler) -> Result<(), TsError> {
        // 设置异常变量
        if let Some(var_name) = &handler.exception_var {
            let error_value = TsValue::Error(format!("{}", exception));
            if let Some(frame) = self.call_stack.last_mut() {
                frame.set_local(var_name, error_value);
            }
        }

        // 跳转到处理器
        self.ip = handler.handler_ip;
        Ok(())
    }

    /// 导入模块
    fn import_module(&mut self, name: &str, alias: Option<&str>) -> Result<(), TsError> {
        // 检查模块是否已缓存
        if let Some(module) = self.modules.get(name) {
            let module_value = TsValue::Object(module.exports.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
            let var_name = alias.unwrap_or(name);
            self.globals.push((var_name.to_string(), module_value));
            return Ok(());
        }

        // 检查是否是 NAPI 模块（.node 文件）
        if name.ends_with(".node") || name.contains(".node") {
            return self.import_napi_module(name, alias);
        }

        // 简化实现，实际应该加载模块文件
        // 模拟模块加载过程
        let mut module = ModuleInstance::new(name.to_string());
        
        // 为常见模块添加一些默认导出
        match name {
            "fs" => {
                // 模拟 fs 模块
                module.export("readFile", TsValue::Function(std::rc::Rc::new(|_| TsValue::Undefined)));
                module.export("writeFile", TsValue::Function(std::rc::Rc::new(|_| TsValue::Undefined)));
            }
            "path" => {
                // 模拟 path 模块
                module.export("join", TsValue::Function(std::rc::Rc::new(|_| TsValue::Undefined)));
                module.export("resolve", TsValue::Function(std::rc::Rc::new(|_| TsValue::Undefined)));
            }
            "util" => {
                // 模拟 util 模块
                module.export("inspect", TsValue::Function(std::rc::Rc::new(|_| TsValue::Undefined)));
            }
            _ => {
                // 空模块
            }
        }
        
        module.loaded = true;

        let module_value = TsValue::Object(module.exports.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
        let var_name = alias.unwrap_or(name);
        self.globals.push((var_name.to_string(), module_value));
        self.modules.insert(name.to_string(), module);

        Ok(())
    }

    /// 导入 NAPI 模块
    fn import_napi_module(&mut self, name: &str, alias: Option<&str>) -> Result<(), TsError> {
        // 这里简化实现，实际应该通过 FFI 管理器加载 NAPI 模块
        // 由于 VM 不直接持有 FFI 管理器，这里创建一个模拟的 NAPI 模块
        let mut module = ModuleInstance::new(name.to_string());
        
        // 添加一个模拟的 NAPI 模块标记
        module.export("__napi_module__", TsValue::Boolean(true));
        
        // TODO: 实际实现中，这里应该调用 FFI 管理器加载 NAPI 模块
        // 并将导出的函数和对象转换为 TsValue
        
        module.loaded = true;

        let module_value = TsValue::Object(module.exports.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
        let var_name = alias.unwrap_or(name);
        self.globals.push((var_name.to_string(), module_value));
        self.modules.insert(name.to_string(), module);

        Ok(())
    }

    /// 导出值
    fn export_value(&mut self, name: &str) -> Result<(), TsError> {
        if let Some(value) = self.stack.pop() {
            // 简化实现，实际应该导出到当前模块
            // 这里我们假设当前模块是最后一个添加的模块
            if let Some((module_name, module)) = self.modules.iter_mut().last() {
                module.export(name, value);
            }
        }
        Ok(())
    }

    /// 二元加法
    fn binary_add(&self, left: TsValue, right: TsValue) -> Result<TsValue, TsError> {
        match (&left, &right) {
            (TsValue::Number(a), TsValue::Number(b)) => Ok(TsValue::Number(*a + *b)),
            (TsValue::String(a), TsValue::String(b)) => Ok(TsValue::String(format!("{}{}", a, b))),
            (TsValue::String(a), b) => Ok(TsValue::String(format!("{}{}", a, b.to_string()))),
            (a, TsValue::String(b)) => Ok(TsValue::String(format!("{}{}", a.to_string(), b))),
            _ => Ok(TsValue::Number(left.to_number() + right.to_number())),
        }
    }

    /// 二元减法
    fn binary_sub(&self, left: TsValue, right: TsValue) -> Result<TsValue, TsError> {
        Ok(TsValue::Number(left.to_number() - right.to_number()))
    }

    /// 二元乘法
    fn binary_mul(&self, left: TsValue, right: TsValue) -> Result<TsValue, TsError> {
        Ok(TsValue::Number(left.to_number() * right.to_number()))
    }

    /// 二元除法
    fn binary_div(&self, left: TsValue, right: TsValue) -> Result<TsValue, TsError> {
        Ok(TsValue::Number(left.to_number() / right.to_number()))
    }

    /// 二元取模
    fn binary_mod(&self, left: TsValue, right: TsValue) -> Result<TsValue, TsError> {
        Ok(TsValue::Number(left.to_number() % right.to_number()))
    }

    /// 二元等于
    fn binary_eq(&self, left: TsValue, right: TsValue) -> TsValue {
        TsValue::Boolean(left.to_string() == right.to_string())
    }

    /// 二元不等于
    fn binary_neq(&self, left: TsValue, right: TsValue) -> TsValue {
        TsValue::Boolean(left.to_string() != right.to_string())
    }

    /// 二元严格等于
    fn binary_strict_eq(&self, left: TsValue, right: TsValue) -> TsValue {
        match (&left, &right) {
            (TsValue::Undefined, TsValue::Undefined) => TsValue::Boolean(true),
            (TsValue::Null, TsValue::Null) => TsValue::Boolean(true),
            (TsValue::Boolean(a), TsValue::Boolean(b)) => TsValue::Boolean(*a == *b),
            (TsValue::Number(a), TsValue::Number(b)) => TsValue::Boolean(*a == *b),
            (TsValue::String(a), TsValue::String(b)) => TsValue::Boolean(*a == *b),
            _ => TsValue::Boolean(false),
        }
    }

    /// 二元严格不等于
    fn binary_strict_neq(&self, left: TsValue, right: TsValue) -> TsValue {
        TsValue::Boolean(!self.binary_strict_eq(left, right).to_boolean())
    }

    /// 二元大于
    fn binary_gt(&self, left: TsValue, right: TsValue) -> Result<TsValue, TsError> {
        Ok(TsValue::Boolean(left.to_number() > right.to_number()))
    }

    /// 二元大于等于
    fn binary_gte(&self, left: TsValue, right: TsValue) -> Result<TsValue, TsError> {
        Ok(TsValue::Boolean(left.to_number() >= right.to_number()))
    }

    /// 二元小于
    fn binary_lt(&self, left: TsValue, right: TsValue) -> Result<TsValue, TsError> {
        Ok(TsValue::Boolean(left.to_number() < right.to_number()))
    }

    /// 二元小于等于
    fn binary_lte(&self, left: TsValue, right: TsValue) -> Result<TsValue, TsError> {
        Ok(TsValue::Boolean(left.to_number() <= right.to_number()))
    }

    /// 二元逻辑与
    fn binary_and(&self, left: TsValue, right: TsValue) -> TsValue {
        if left.to_boolean() { right } else { left }
    }

    /// 二元逻辑或
    fn binary_or(&self, left: TsValue, right: TsValue) -> TsValue {
        if left.to_boolean() { left } else { right }
    }

    /// 获取类型
    fn type_of(&self, value: &TsValue) -> String {
        match value {
            TsValue::Undefined => "undefined".to_string(),
            TsValue::Null => "object".to_string(),
            TsValue::Boolean(_) => "boolean".to_string(),
            TsValue::Number(_) => "number".to_string(),
            TsValue::String(_) => "string".to_string(),
            TsValue::Object(_) => "object".to_string(),
            TsValue::Array(_) => "object".to_string(),
            TsValue::Function(_) => "function".to_string(),
            TsValue::Error(_) => "object".to_string(),
            TsValue::Union(_) => "object".to_string(),
            TsValue::Generic(_, _) => "object".to_string(),
            TsValue::Symbol(_) => "symbol".to_string(),
            TsValue::BigInt(_) => "bigint".to_string(),
            TsValue::Date(_) => "object".to_string(),
            TsValue::RegExp(_) => "object".to_string(),
            TsValue::Map(_) => "object".to_string(),
            TsValue::Set(_) => "object".to_string(),
            TsValue::Promise(_) => "object".to_string(),
            TsValue::Iterable(_) => "object".to_string(),
        }
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
    pub fn globals(&self) -> &[(String, TsValue)] {
        &self.globals
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new(vec![])
    }
}


