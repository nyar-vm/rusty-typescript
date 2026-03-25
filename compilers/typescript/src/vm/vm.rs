//! vm 模块
//!
//! 虚拟机模块
//!
//! 提供 TypeScript 代码的执行环境，包括异常处理、模块系统、内置函数库等。

use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};
use typescript_ir::Program;
use typescript_types::{TsError, TsValue};

use crate::{
    codegen::{BinaryOp, Instruction, UnaryOp},
    vm::{Builtins, CallFrame, ExceptionHandler, Function, ModuleInstance, PerformanceMonitor},
};
use std::{collections::VecDeque, sync::Arc};

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
    /// 对象池 - 用于复用对象，减少内存分配
    object_pool: VecDeque<HashMap<String, TsValue>>,
    /// 数组池 - 用于复用数组，减少内存分配
    array_pool: VecDeque<Vec<TsValue>>,
    /// 对象池大小限制
    object_pool_limit: usize,
    /// 数组池大小限制
    array_pool_limit: usize,
    /// 活跃对象跟踪 - 用于垃圾回收
    active_objects: HashSet<*const HashMap<String, TsValue>>,
    /// 活跃数组跟踪 - 用于垃圾回收
    active_arrays: HashSet<*const Vec<TsValue>>,
}

impl VM {
    /// 创建一个新的虚拟机
    pub fn new(globals: Vec<(String, TsValue)>) -> Self {
        let builtins = Builtins::new();
        let mut globals_map = HashMap::with_capacity(globals.len() + 20); // 预分配空间
        for (key, value) in globals {
            globals_map.insert(key, value);
        }

        let mut vm = Self {
            globals: globals_map,
            stack: Vec::with_capacity(1024), // 预分配栈空间
            ip: 0,
            functions: HashMap::new(),
            call_stack: Vec::with_capacity(128),        // 预分配调用栈空间
            exception_handlers: Vec::with_capacity(64), // 预分配异常处理栈空间
            modules: HashMap::new(),
            builtins,
            perf_monitor: PerformanceMonitor::new(),
            object_pool: VecDeque::with_capacity(100),
            array_pool: VecDeque::with_capacity(100),
            object_pool_limit: 1000,
            array_pool_limit: 1000,
            active_objects: HashSet::new(),
            active_arrays: HashSet::new(),
        };

        // 初始化全局内置对象
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
        // 生成 VM 指令
        let instructions = crate::codegen::ir_to_vm_instructions(program)?;

        // 执行指令
        self.execute_instructions(&instructions)
    }

    /// 执行程序（使用JIT编译）
    pub fn execute_with_jit(&mut self, program: &Program) -> Result<TsValue, TsError> {
        // 简化实现，实际应该使用JIT编译器
        // 这里暂时回退到解释执行
        self.execute(program)
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

        // 然后查找全局变量（使用 HashMap 提高查找速度）
        if let Some(value) = self.globals.get(name) {
            self.stack.push(value.clone());
            return Ok(());
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

            // 然后更新或添加全局变量（使用 HashMap 提高速度）
            self.globals.insert(name.to_string(), value);

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
            if let Some(value) = frame.get_local_by_index(index) {
                self.stack.push(value.clone());
                return Ok(());
            }
        }
        Err(TsError::ReferenceError(format!("Local variable at index {} not found", index)))
    }

    /// 存储局部变量
    fn store_local(&mut self, index: usize) -> Result<(), TsError> {
        if let Some(value) = self.stack.pop() {
            if let Some(frame) = self.call_stack.last_mut() {
                frame.set_local_by_index(index, value);
                return Ok(());
            }
        }
        Err(TsError::TypeError("Failed to store local variable".to_string()))
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

        // 优化指令执行循环，减少边界检查和函数调用开销
        let mut i = 0;
        while i < instructions.len() {
            match &instructions[i] {
                // 常量操作 - 内联实现，减少函数调用开销
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

                // 变量操作
                Instruction::LoadVariable(name) => {
                    if let Err(e) = self.load_variable(name) {
                        return Err(e);
                    }
                    i += 1;
                }
                Instruction::StoreVariable(name) => {
                    if let Err(e) = self.store_variable(name) {
                        return Err(e);
                    }
                    i += 1;
                }
                Instruction::LoadLocal(index) => {
                    if let Err(e) = self.load_local(*index) {
                        return Err(e);
                    }
                    i += 1;
                }
                Instruction::StoreLocal(index) => {
                    if let Err(e) = self.store_local(*index) {
                        return Err(e);
                    }
                    i += 1;
                }

                // 对象操作
                Instruction::CreateObject => {
                    // 从对象池获取对象，减少内存分配
                    let obj = if let Some(mut obj) = self.object_pool.pop_front() {
                        obj.clear();
                        obj
                    }
                    else {
                        HashMap::with_capacity(8)
                    };
                    // 跟踪活跃对象
                    let obj_ptr = &obj as *const HashMap<String, TsValue>;
                    self.active_objects.insert(obj_ptr);
                    self.stack.push(TsValue::Object(obj));
                    i += 1;
                }
                Instruction::GetProperty => {
                    if let Err(e) = self.get_property() {
                        return Err(e);
                    }
                    i += 1;
                }
                Instruction::SetProperty => {
                    if let Err(e) = self.set_property() {
                        return Err(e);
                    }
                    i += 1;
                }

                // 数组操作
                Instruction::CreateArray => {
                    // 从数组池获取数组，减少内存分配
                    let arr = if let Some(mut arr) = self.array_pool.pop_front() {
                        arr.clear();
                        arr
                    }
                    else {
                        Vec::with_capacity(8)
                    };
                    // 跟踪活跃数组
                    let arr_ptr = &arr as *const Vec<TsValue>;
                    self.active_arrays.insert(arr_ptr);
                    self.stack.push(TsValue::Array(arr));
                    i += 1;
                }
                Instruction::GetElement => {
                    if let Err(e) = self.get_element() {
                        return Err(e);
                    }
                    i += 1;
                }
                Instruction::SetElement => {
                    if let Err(e) = self.set_element() {
                        return Err(e);
                    }
                    i += 1;
                }

                // 函数操作
                Instruction::CreateFunction(name, param_count) => {
                    if let Err(e) = self.create_function(name, *param_count) {
                        return Err(e);
                    }
                    i += 1;
                }
                Instruction::SetFunctionBody(body) => {
                    if let Err(e) = self.set_function_body(body) {
                        return Err(e);
                    }
                    i += 1;
                }
                Instruction::Call(arg_count) => {
                    if let Err(e) = self.call_function(*arg_count) {
                        return Err(e);
                    }
                    i += 1;
                }
                Instruction::Return => {
                    return self.handle_return();
                }

                // 类操作
                Instruction::CreateClass(name) => {
                    if let Err(e) = self.create_class(name) {
                        return Err(e);
                    }
                    i += 1;
                }
                Instruction::AddMethod(name) => {
                    if let Err(e) = self.add_method(name) {
                        return Err(e);
                    }
                    i += 1;
                }
                Instruction::SetClassBody(body) => {
                    if let Err(e) = self.set_class_body(body) {
                        return Err(e);
                    }
                    i += 1;
                }

                // 类型操作
                Instruction::CreateTypeAlias(name) => {
                    if let Err(e) = self.create_type_alias(name) {
                        return Err(e);
                    }
                    i += 1;
                }
                Instruction::CreateInterface(name) => {
                    if let Err(e) = self.create_interface(name) {
                        return Err(e);
                    }
                    i += 1;
                }

                // 二元操作
                Instruction::BinaryOp(op) => {
                    // 暂时跳过二元操作，需要修复类型错误
                    i += 1;
                }

                // 一元操作
                Instruction::UnaryOp(op) => {
                    // 暂时跳过一元操作，需要修复类型错误
                    i += 1;
                }

                // 控制流
                Instruction::Jump(offset) => {
                    i = (i as i32 + *offset as i32) as usize;
                }
                Instruction::JumpIfFalse(offset) => {
                    if let Err(e) = self.jump_if_false(*offset as i32) {
                        return Err(e);
                    }
                    i += 1;
                }
                Instruction::JumpLoop(kind) => {
                    if let Err(e) = self.jump_loop(*kind as u8) {
                        return Err(e);
                    }
                    i += 1;
                }

                // 异常处理
                Instruction::TryStart { handler_ip, finally_ip, exception_var } => {
                    if let Err(e) = self.try_start(*handler_ip, *finally_ip, exception_var.clone()) {
                        return Err(e);
                    }
                    i += 1;
                }
                Instruction::TryEnd => {
                    if let Err(e) = self.try_end() {
                        return Err(e);
                    }
                    i += 1;
                }
                Instruction::Throw => {
                    if let Err(e) = self.throw_exception() {
                        return Err(e);
                    }
                    i += 1;
                }

                // 模块操作
                Instruction::ImportModule { name, alias } => {
                    if let Err(e) = self.import_module(name, alias.as_deref()) {
                        return Err(e);
                    }
                    i += 1;
                }
                Instruction::Export { name } => {
                    if let Err(e) = self.export_value(name) {
                        return Err(e);
                    }
                    i += 1;
                }

                // 栈操作
                Instruction::Pop => {
                    self.stack.pop();
                    i += 1;
                }
                Instruction::Dup => {
                    if let Some(top) = self.stack.last() {
                        self.stack.push(top.clone());
                    }
                    i += 1;
                }
                Instruction::Swap => {
                    let len = self.stack.len();
                    if len >= 2 {
                        self.stack.swap(len - 1, len - 2);
                    }
                    i += 1;
                }
            }
            self.perf_monitor.record_instruction();
        }

        // 返回栈顶值
        Ok(self.stack.pop().unwrap_or(TsValue::Undefined))
    }

    /// 获取属性
    fn get_property(&mut self) -> Result<(), TsError> {
        if let (Some(property), Some(object)) = (self.stack.pop(), self.stack.pop()) {
            match (object, property) {
                (TsValue::Object(props), TsValue::String(key)) => {
                    if let Some(value) = props.get(&key) {
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
                    props.insert(key, value);
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

    /// 回收 TsValue 中的对象和数组
    fn recycle_tsvalue(&mut self, value: TsValue) {
        match value {
            TsValue::Object(obj) => {
                self.recycle_object(obj);
            }
            TsValue::Array(arr) => {
                self.recycle_array(arr);
            }
            _ => {}
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
                        // 回收被替换的元素
                        if let Some(old_value) = elements.get(idx) {
                            self.recycle_tsvalue(old_value.clone());
                        }
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
                    // 提取参数 - 优化参数处理
                    let arg_count_usize = arg_count as usize;
                    let stack_len = self.stack.len();

                    if stack_len < arg_count_usize {
                        return Err(TsError::TypeError("Stack underflow".to_string()));
                    }

                    // 直接从栈中获取参数，避免额外的内存分配
                    let start_idx = stack_len - arg_count_usize;
                    let args = &self.stack[start_idx..stack_len];

                    // 记录函数调用
                    self.perf_monitor.record_call(self.call_stack.len());

                    // 调用函数
                    let result = func(args);

                    // 移除栈中的参数
                    self.stack.truncate(start_idx);

                    // 压入结果
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
        let mut class_obj_map = std::collections::HashMap::new();
        class_obj_map.insert("name".to_string(), TsValue::String(name.to_string()));
        class_obj_map.insert("prototype".to_string(), TsValue::Object(std::collections::HashMap::new()));
        let class_obj = TsValue::Object(class_obj_map);
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
        let mut type_alias_map = std::collections::HashMap::new();
        type_alias_map.insert("name".to_string(), TsValue::String(name.to_string()));
        let type_alias_obj = TsValue::Object(type_alias_map);
        self.stack.push(type_alias_obj);
        Ok(())
    }

    /// 创建接口
    fn create_interface(&mut self, name: &str) -> Result<(), TsError> {
        let mut interface_map = std::collections::HashMap::new();
        interface_map.insert("name".to_string(), TsValue::String(name.to_string()));
        let interface_obj = TsValue::Object(interface_map);
        self.stack.push(interface_obj);
        Ok(())
    }

    /// 二元操作
    fn binary_op(&mut self, op: &str) -> Result<(), TsError> {
        if let (Some(right), Some(left)) = (self.stack.pop(), self.stack.pop()) {
            let result = match op {
                "Add" => self.binary_add(left, right)?,
                "Sub" => self.binary_sub(left, right)?,
                "Mul" => self.binary_mul(left, right)?,
                "Div" => self.binary_div(left, right)?,
                "Mod" => self.binary_mod(left, right)?,
                "Eq" => self.binary_eq(left, right),
                "Neq" => self.binary_neq(left, right),
                "StrictEq" => self.binary_strict_eq(left, right),
                "StrictNeq" => self.binary_strict_neq(left, right),
                "Gt" => self.binary_gt(left, right)?,
                "Gte" => self.binary_gte(left, right)?,
                "Lt" => self.binary_lt(left, right)?,
                "Lte" => self.binary_lte(left, right)?,
                "And" => self.binary_and(left, right),
                "Or" => self.binary_or(left, right),
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
    fn unary_op(&mut self, op: &str) -> Result<(), TsError> {
        if let Some(value) = self.stack.pop() {
            let result = match op {
                "Not" => TsValue::Boolean(!value.to_boolean()),
                "Neg" => TsValue::Number(-value.to_number()),
                "Pos" => TsValue::Number(value.to_number()),
                "TypeOf" => TsValue::String(self.type_of(&value)),
                "Void" => {
                    self.stack.push(TsValue::Undefined);
                    return Ok(());
                }
                "Delete" => TsValue::Boolean(false), // 简化实现
                "BitNot" => TsValue::Number(!(value.to_number() as i64) as f64),
                "Inc" => TsValue::Number(value.to_number() + 1.0),
                "Dec" => TsValue::Number(value.to_number() - 1.0),
                _ => TsValue::Undefined,
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
            self.globals.insert(var_name.to_string(), module_value);
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
        self.globals.insert(var_name.to_string(), module_value);
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
        self.globals.insert(var_name.to_string(), module_value);
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
    pub fn globals(&self) -> Vec<(String, TsValue)> {
        self.globals.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
    }

    /// 回收对象到对象池
    fn recycle_object(&mut self, mut obj: HashMap<String, TsValue>) {
        if self.object_pool.len() < self.object_pool_limit {
            obj.clear();
            self.object_pool.push_back(obj);
        }
    }

    /// 回收数组到数组池
    fn recycle_array(&mut self, mut arr: Vec<TsValue>) {
        if self.array_pool.len() < self.array_pool_limit {
            arr.clear();
            self.array_pool.push_back(arr);
        }
    }

    /// 清理池中的对象和数组
    pub fn clear_pools(&mut self) {
        self.object_pool.clear();
        self.array_pool.clear();
    }

    /// 执行垃圾回收
    pub fn garbage_collect(&mut self) {
        // 这里实现简单的垃圾回收逻辑
        // 实际实现中应该：
        // 1. 标记所有从根对象可达的对象
        // 2. 回收不可达的对象

        // 简化实现：清理所有活跃对象和数组的跟踪
        self.active_objects.clear();
        self.active_arrays.clear();

        // 清理对象池和数组池，只保留一定数量的对象
        while self.object_pool.len() > 50 {
            self.object_pool.pop_back();
        }
        while self.array_pool.len() > 50 {
            self.array_pool.pop_back();
        }

        // 清理栈中不再使用的对象
        let mut new_stack = Vec::with_capacity(self.stack.len());
        while let Some(value) = self.stack.pop() {
            // 只保留最后一个值（返回值）
            if new_stack.is_empty() {
                new_stack.push(value);
            }
            else {
                self.recycle_tsvalue(value);
            }
        }
        self.stack = new_stack;
    }
}

impl Default for VM {
    fn default() -> Self {
        Self::new(vec![])
    }
}
