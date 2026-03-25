//! frame 模块
//!
use std::collections::HashMap;
use typescript_types::TsValue;

/// 调用帧
#[derive(Debug, Clone)]
pub struct CallFrame {
    /// 函数名
    pub function_name: String,
    /// 返回地址
    pub return_ip: usize,
    /// 局部变量
    pub locals: HashMap<String, TsValue>,
    /// 栈底位置
    pub stack_base: usize,
}

impl CallFrame {
    /// 创建新的调用帧
    pub fn new(function_name: String, return_ip: usize, stack_base: usize) -> Self {
        Self { function_name, return_ip, locals: HashMap::new(), stack_base }
    }

    /// 设置局部变量
    pub fn set_local(&mut self, name: &str, value: TsValue) {
        self.locals.insert(name.to_string(), value);
    }

    /// 获取局部变量
    pub fn get_local(&self, name: &str) -> Option<&TsValue> {
        self.locals.get(name)
    }
}
