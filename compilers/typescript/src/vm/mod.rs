use crate::codegen::{BinaryOp, Instruction, UnaryOp};
use std::rc::Rc;
use typescript_types::{TsError, TsValue};

/// 虚拟机
pub struct VM {
    /// 全局变量
    globals: Vec<(String, TsValue)>,
    /// 栈
    stack: Vec<TsValue>,
    /// 指令指针
    ip: usize,
    /// 函数环境
    functions: std::collections::HashMap<String, Function>,
}

/// 函数
struct Function {
    /// 函数名
    name: String,
    /// 参数数量
    param_count: u32,
    /// 函数体指令
    body: Vec<Instruction>,
}

impl VM {
    /// 创建一个新的虚拟机
    pub fn new(globals: Vec<(String, TsValue)>) -> Self {
        Self { globals, stack: vec![], ip: 0, functions: std::collections::HashMap::new() }
    }

    /// 执行程序
    pub fn execute(&mut self, program: &typescript_ir::Program) -> Result<TsValue, TsError> {
        // 生成 VM 指令
        let instructions = crate::codegen::ir_to_vm_instructions(program);

        // 执行指令
        self.execute_instructions(&instructions)
    }

    /// 执行指令序列
    fn execute_instructions(&mut self, instructions: &[Instruction]) -> Result<TsValue, TsError> {
        self.ip = 0;
        self.stack.clear();

        while self.ip < instructions.len() {
            let instruction = &instructions[self.ip];
            self.ip += 1;

            // 调试输出
            println!("Executing instruction: {:?}, Stack: {:?}", instruction, self.stack);

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
                    // 打印调试信息
                    println!("Loading variable: '{}', globals: {:?}", name, self.globals);

                    // 查找全局变量
                    if let Some(value) = self.globals.iter().find(|(n, _)| n == name) {
                        self.stack.push(value.1.clone());
                        println!("Found variable '{}' with value: {:?}", name, value.1);
                    }
                    else {
                        return Err(TsError::ReferenceError(format!("Variable '{}' is not defined", name)));
                    }
                }
                Instruction::StoreVariable(name) => {
                    if let Some(value) = self.stack.pop() {
                        // 克隆值，以便在多个地方使用
                        let value_clone = value.clone();

                        // 查找并更新全局变量
                        if let Some((_, val)) = self.globals.iter_mut().find(|(n, _)| n == name) {
                            *val = value;
                        }
                        else {
                            // 添加新的全局变量
                            self.globals.push((name.clone(), value));
                        }
                        // 将值重新压回栈，以便赋值表达式可以返回值
                        self.stack.push(value_clone);
                    }
                    else {
                        return Err(TsError::TypeError("Stack underflow".to_string()));
                    }
                }

                // 对象操作
                Instruction::CreateObject => {
                    self.stack.push(TsValue::Object(vec![]));
                }
                Instruction::GetProperty => {
                    if let (Some(property), Some(object)) = (self.stack.pop(), self.stack.pop()) {
                        match (object, property) {
                            (TsValue::Object(props), TsValue::String(key)) => {
                                if let Some(value) = props.iter().find(|(k, _)| k == &key) {
                                    self.stack.push(value.1.clone());
                                }
                                else {
                                    self.stack.push(TsValue::Undefined);
                                }
                            }
                            _ => {
                                return Err(TsError::TypeError("Cannot get property of non-object".to_string()));
                            }
                        }
                    }
                    else {
                        return Err(TsError::TypeError("Stack underflow".to_string()));
                    }
                }
                Instruction::SetProperty => {
                    if let (Some(property), Some(value), Some(object)) = (self.stack.pop(), self.stack.pop(), self.stack.pop())
                    {
                        match (object, property) {
                            (TsValue::Object(mut props), TsValue::String(key)) => {
                                // 查找并更新属性
                                if let Some((_, val)) = props.iter_mut().find(|(k, _)| k == &key) {
                                    *val = value;
                                }
                                else {
                                    // 添加新属性
                                    props.push((key, value));
                                }
                                self.stack.push(TsValue::Object(props));
                            }
                            _ => {
                                return Err(TsError::TypeError("Cannot set property of non-object".to_string()));
                            }
                        }
                    }
                    else {
                        return Err(TsError::TypeError("Stack underflow".to_string()));
                    }
                }

                // 数组操作
                Instruction::CreateArray => {
                    self.stack.push(TsValue::Array(vec![]));
                }
                Instruction::GetElement => {
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
                            }
                            _ => {
                                return Err(TsError::TypeError("Cannot get element of non-array".to_string()));
                            }
                        }
                    }
                    else {
                        return Err(TsError::TypeError("Stack underflow".to_string()));
                    }
                }
                Instruction::SetElement => {
                    if let (Some(index), Some(value), Some(array)) = (self.stack.pop(), self.stack.pop(), self.stack.pop()) {
                        match (array, index) {
                            (TsValue::Array(mut elements), TsValue::Number(idx)) => {
                                let idx = idx as usize;
                                if idx < elements.len() {
                                    elements[idx] = value;
                                }
                                else {
                                    // 扩展数组
                                    elements.resize(idx + 1, TsValue::Undefined);
                                    elements[idx] = value;
                                }
                                self.stack.push(TsValue::Array(elements));
                            }
                            _ => {
                                return Err(TsError::TypeError("Cannot set element of non-array".to_string()));
                            }
                        }
                    }
                    else {
                        return Err(TsError::TypeError("Stack underflow".to_string()));
                    }
                }

                // 函数操作
                Instruction::CreateFunction(name, param_count) => {
                    self.stack.push(TsValue::Function(Rc::new(|args| {
                        // 这里只是一个占位符，实际函数执行需要更复杂的实现
                        TsValue::Undefined
                    })));
                }
                Instruction::SetFunctionBody(body) => {
                    if let Some(TsValue::Function(_)) = self.stack.last() {
                        // 这里需要将函数体与函数对象关联起来
                        // 暂时不实现
                    }
                }
                Instruction::Call(arg_count) => {
                    if let Some(callee) = self.stack.pop() {
                        match callee {
                            TsValue::Function(func) => {
                                // 提取参数
                                let mut args = vec![];
                                for _ in 0..*arg_count {
                                    if let Some(arg) = self.stack.pop() {
                                        args.push(arg);
                                    }
                                    else {
                                        return Err(TsError::TypeError("Stack underflow".to_string()));
                                    }
                                }
                                // 反转参数顺序
                                args.reverse();
                                // 调用函数
                                let result = func(&args);
                                // 将结果压回栈
                                self.stack.push(result);
                            }
                            _ => {
                                return Err(TsError::TypeError("Cannot call non-function".to_string()));
                            }
                        }
                    }
                    else {
                        return Err(TsError::TypeError("Stack underflow".to_string()));
                    }
                }
                Instruction::Return => {
                    // 从栈中弹出返回值
                    let result = self.stack.pop().unwrap_or(TsValue::Undefined);
                    return Ok(result);
                }

                // 类操作
                Instruction::CreateClass(name) => {
                    // 创建类对象
                    let class_obj = TsValue::Object(vec![
                        ("name".to_string(), TsValue::String(name.clone())),
                        ("prototype".to_string(), TsValue::Object(vec![])),
                    ]);
                    self.stack.push(class_obj);
                }
                Instruction::AddMethod(name) => {
                    // 创建方法对象
                    let method_obj = TsValue::Function(Rc::new(|args| {
                        // 这里只是一个占位符，实际方法执行需要更复杂的实现
                        TsValue::Undefined
                    }));
                    self.stack.push(method_obj);
                }
                Instruction::SetClassBody(body) => {
                    if let Some(TsValue::Object(class_obj)) = self.stack.pop() {
                        // 这里需要将类体与类对象关联起来
                        // 暂时不实现，直接将类对象压回栈
                        self.stack.push(TsValue::Object(class_obj));
                    }
                }

                // 类型操作
                Instruction::CreateTypeAlias(name) => {
                    // 创建类型别名对象
                    let type_alias_obj = TsValue::Object(vec![("name".to_string(), TsValue::String(name.clone()))]);
                    self.stack.push(type_alias_obj);
                }
                Instruction::CreateInterface(name) => {
                    // 创建接口对象
                    let interface_obj = TsValue::Object(vec![("name".to_string(), TsValue::String(name.clone()))]);
                    self.stack.push(interface_obj);
                }

                // 二元操作
                Instruction::BinaryOp(op) => {
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
                            _ => TsValue::Undefined, // 其他操作暂时不实现
                        };
                        self.stack.push(result);
                    }
                    else {
                        return Err(TsError::TypeError("Stack underflow".to_string()));
                    }
                }

                // 一元操作
                Instruction::UnaryOp(op) => {
                    if let Some(value) = self.stack.pop() {
                        let result = match op {
                            UnaryOp::Not => TsValue::Boolean(!value.to_boolean()),
                            UnaryOp::Neg => TsValue::Number(-value.to_number()),
                            UnaryOp::Pos => TsValue::Number(value.to_number()),
                            UnaryOp::TypeOf => TsValue::String(self.type_of(&value)),
                            _ => TsValue::Undefined, // 其他操作暂时不实现
                        };
                        self.stack.push(result);
                    }
                    else {
                        return Err(TsError::TypeError("Stack underflow".to_string()));
                    }
                }

                // 控制流
                Instruction::Jump(offset) => {
                    self.ip = (self.ip as i32 + *offset as i32) as usize;
                }
                Instruction::JumpIfFalse(offset) => {
                    if let Some(value) = self.stack.pop() {
                        if !value.to_boolean() {
                            self.ip = (self.ip as i32 + *offset as i32) as usize;
                        }
                    }
                    else {
                        return Err(TsError::TypeError("Stack underflow".to_string()));
                    }
                }
                Instruction::JumpLoop(kind) => {
                    // 0: continue, 1: break
                    // 暂时简化处理，直接跳转到循环结束
                    // 实际实现需要维护循环栈
                    self.ip += 1; // 跳过当前指令
                }

                // 栈操作
                Instruction::Pop => {
                    self.stack.pop();
                }
            }
        }

        // 返回栈顶值
        Ok(self.stack.pop().unwrap_or(TsValue::Undefined))
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
        // 简化实现，实际需要检查类型和值
        TsValue::Boolean(left.to_string() == right.to_string())
    }

    /// 二元严格不等于
    fn binary_strict_neq(&self, left: TsValue, right: TsValue) -> TsValue {
        // 简化实现，实际需要检查类型和值
        TsValue::Boolean(left.to_string() != right.to_string())
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
}

/// 导入 typescript_ir
use typescript_ir;
