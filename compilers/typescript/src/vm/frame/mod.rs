//! frame 模块
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
    /// 局部变量索引映射
    pub local_indices: HashMap<String, usize>,
    /// 局部变量值按索引存储
    pub local_values: Vec<TsValue>,
    /// 栈底位置
    pub stack_base: usize,
}

impl CallFrame {
    /// 创建新的调用帧
    pub fn new(function_name: String, return_ip: usize, stack_base: usize) -> Self {
        Self {
            function_name,
            return_ip,
            locals: HashMap::new(),
            local_indices: HashMap::new(),
            local_values: Vec::new(),
            stack_base,
        }
    }

    /// 设置局部变量
    pub fn set_local(&mut self, name: &str, value: TsValue) {
        self.locals.insert(name.to_string(), value.clone());

        // 为变量分配索引
        if !self.local_indices.contains_key(name) {
            let index = self.local_values.len();
            self.local_indices.insert(name.to_string(), index);
            self.local_values.push(value);
        }
        else {
            // 更新已有变量的值
            let index = self.local_indices[name];
            self.local_values[index] = value;
        }
    }

    /// 获取局部变量
    pub fn get_local(&self, name: &str) -> Option<&TsValue> {
        self.locals.get(name)
    }

    /// 通过索引获取局部变量
    pub fn get_local_by_index(&self, index: usize) -> Option<&TsValue> {
        self.local_values.get(index)
    }

    /// 通过索引设置局部变量
    pub fn set_local_by_index(&mut self, index: usize, value: TsValue) {
        if index < self.local_values.len() {
            self.local_values[index] = value;
        }
    }

    /// 获取局部变量的索引
    pub fn get_local_index(&self, name: &str) -> Option<usize> {
        self.local_indices.get(name).copied()
    }
}
