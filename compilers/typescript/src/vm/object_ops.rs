//! 对象和数组操作模块
//!
//! 提供对象属性访问、数组元素访问等操作。

use std::collections::HashMap;

use typescript_types::{TsError, TsValue};

use super::memory::MemoryManager;

/// 对象操作执行器
pub struct ObjectOperations;

impl ObjectOperations {
    /// 获取对象属性
    pub fn get_property(object: TsValue, property: TsValue, stack: &mut Vec<TsValue>) -> Result<(), TsError> {
        match (object, property) {
            (TsValue::Object(props), TsValue::String(key)) => {
                if let Some(value) = props.get(&key) {
                    stack.push(value.clone());
                }
                else {
                    stack.push(TsValue::Undefined);
                }
                Ok(())
            }
            (TsValue::Array(elements), TsValue::String(key)) if key == "length" => {
                stack.push(TsValue::Number(elements.len() as f64));
                Ok(())
            }
            _ => Err(TsError::TypeError("Cannot get property of non-object".to_string())),
        }
    }

    /// 设置对象属性
    pub fn set_property(object: TsValue, property: TsValue, value: TsValue, stack: &mut Vec<TsValue>) -> Result<(), TsError> {
        match (object, property) {
            (TsValue::Object(mut props), TsValue::String(key)) => {
                props.insert(key, value);
                stack.push(TsValue::Object(props));
                Ok(())
            }
            _ => Err(TsError::TypeError("Cannot set property of non-object".to_string())),
        }
    }

    /// 创建新对象
    pub fn create_object(memory: &mut MemoryManager, stack: &mut Vec<TsValue>) {
        let obj = memory.acquire_object();
        memory.track_object(&obj);
        stack.push(TsValue::Object(obj));
    }
}

/// 数组操作执行器
pub struct ArrayOperations;

impl ArrayOperations {
    /// 获取数组元素
    pub fn get_element(array: TsValue, index: TsValue, stack: &mut Vec<TsValue>) -> Result<(), TsError> {
        match (array, index) {
            (TsValue::Array(elements), TsValue::Number(idx)) => {
                let idx = idx as usize;
                if idx < elements.len() {
                    stack.push(elements[idx].clone());
                }
                else {
                    stack.push(TsValue::Undefined);
                }
                Ok(())
            }
            (TsValue::String(s), TsValue::Number(idx)) => {
                let idx = idx as usize;
                if let Some(ch) = s.chars().nth(idx) {
                    stack.push(TsValue::String(ch.to_string()));
                }
                else {
                    stack.push(TsValue::Undefined);
                }
                Ok(())
            }
            _ => Err(TsError::TypeError("Cannot get element of non-array".to_string())),
        }
    }

    /// 设置数组元素
    pub fn set_element(
        array: TsValue,
        index: TsValue,
        value: TsValue,
        memory: &mut MemoryManager,
        stack: &mut Vec<TsValue>,
    ) -> Result<(), TsError> {
        match (array, index) {
            (TsValue::Array(mut elements), TsValue::Number(idx)) => {
                let idx = idx as usize;
                if idx < elements.len() {
                    if let Some(old_value) = elements.get(idx) {
                        memory.recycle_tsvalue(old_value.clone());
                    }
                    elements[idx] = value;
                }
                else {
                    elements.resize(idx + 1, TsValue::Undefined);
                    elements[idx] = value;
                }
                stack.push(TsValue::Array(elements));
                Ok(())
            }
            _ => Err(TsError::TypeError("Cannot set element of non-array".to_string())),
        }
    }

    /// 创建新数组
    pub fn create_array(memory: &mut MemoryManager, stack: &mut Vec<TsValue>) {
        let arr = memory.acquire_array();
        memory.track_array(&arr);
        stack.push(TsValue::Array(arr));
    }
}
