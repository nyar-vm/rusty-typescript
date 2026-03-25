//! function 模块
use crate::codegen::Instruction;
use std::collections::HashMap;
use typescript_types::TsValue;

/// 函数
#[derive(Debug, Clone)]
pub struct Function {
    /// 函数名
    pub name: String,
    /// 参数数量
    pub param_count: u32,
    /// 函数体指令
    pub body: Vec<Instruction>,
    /// 局部变量名
    pub locals: Vec<String>,
    /// 闭包捕获的变量
    pub captures: HashMap<String, TsValue>,
}

impl Function {
    /// 创建新函数
    pub fn new(name: String, param_count: u32, body: Vec<Instruction>) -> Self {
        Self { name, param_count, body, locals: Vec::new(), captures: HashMap::new() }
    }
}
