//! module 模块
//!
use std::collections::HashMap;
use typescript_types::TsValue;

pub struct ModuleInstance {
    /// 模块名
    pub name: String,
    /// 导出的值
    pub exports: HashMap<String, TsValue>,
    /// 是否已加载
    pub loaded: bool,
}

impl ModuleInstance {
    /// 创建新的模块实例
    pub fn new(name: String) -> Self {
        Self { name, exports: HashMap::new(), loaded: false }
    }

    /// 导出值
    pub fn export(&mut self, name: &str, value: TsValue) {
        self.exports.insert(name.to_string(), value);
    }

    /// 获取导出值
    pub fn get_export(&self, name: &str) -> Option<&TsValue> {
        self.exports.get(name)
    }
}

/// 函数
#[derive(Debug, Clone)]