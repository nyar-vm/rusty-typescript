//! 函数和类操作模块
//!
//! 提供函数创建、调用和类定义等操作。

use std::{collections::HashMap, rc::Rc};

use typescript_types::{TsError, TsValue};

use super::{function::Function, perf::PerformanceMonitor};

/// 函数操作执行器
pub struct FunctionOperations;

impl FunctionOperations {
    /// 创建函数
    pub fn create_function(
        name: &str,
        param_count: u32,
        functions: &mut HashMap<String, Function>,
        stack: &mut Vec<TsValue>,
    ) -> Result<(), TsError> {
        let func = Function::new(name.to_string(), param_count, vec![]);
        functions.insert(name.to_string(), func);
        stack.push(TsValue::Function(Rc::new(|_| TsValue::Undefined)));
        Ok(())
    }

    /// 调用函数
    pub fn call_function(
        arg_count: u32,
        stack: &mut Vec<TsValue>,
        perf_monitor: &mut PerformanceMonitor,
        call_stack_depth: usize,
    ) -> Result<(), TsError> {
        if let Some(callee) = stack.pop() {
            match callee {
                TsValue::Function(func) => {
                    let arg_count_usize = arg_count as usize;
                    let stack_len = stack.len();

                    if stack_len < arg_count_usize {
                        return Err(TsError::TypeError("Stack underflow".to_string()));
                    }

                    let start_idx = stack_len - arg_count_usize;
                    let args = &stack[start_idx..stack_len];

                    perf_monitor.record_call(call_stack_depth);

                    let result = func(args);

                    stack.truncate(start_idx);
                    stack.push(result);
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
    pub fn handle_return(stack: &mut Vec<TsValue>, return_ip: Option<usize>) -> Result<(TsValue, Option<usize>), TsError> {
        let result = stack.pop().unwrap_or(TsValue::Undefined);
        Ok((result, return_ip))
    }
}

/// 类操作执行器
pub struct ClassOperations;

impl ClassOperations {
    /// 创建类
    pub fn create_class(name: &str, stack: &mut Vec<TsValue>) -> Result<(), TsError> {
        let mut class_obj_map = HashMap::new();
        class_obj_map.insert("name".to_string(), TsValue::String(name.to_string()));
        class_obj_map.insert("prototype".to_string(), TsValue::Object(HashMap::new()));
        let class_obj = TsValue::Object(class_obj_map);
        stack.push(class_obj);
        Ok(())
    }

    /// 添加方法
    pub fn add_method(stack: &mut Vec<TsValue>) -> Result<(), TsError> {
        stack.push(TsValue::Function(Rc::new(|_| TsValue::Undefined)));
        Ok(())
    }

    /// 设置类体
    pub fn set_class_body(stack: &mut Vec<TsValue>) -> Result<(), TsError> {
        if let Some(TsValue::Object(class_obj)) = stack.pop() {
            stack.push(TsValue::Object(class_obj));
        }
        Ok(())
    }
}

/// 类型操作执行器
pub struct TypeOperations;

impl TypeOperations {
    /// 创建类型别名
    pub fn create_type_alias(name: &str, stack: &mut Vec<TsValue>) -> Result<(), TsError> {
        let mut type_alias_map = HashMap::new();
        type_alias_map.insert("name".to_string(), TsValue::String(name.to_string()));
        let type_alias_obj = TsValue::Object(type_alias_map);
        stack.push(type_alias_obj);
        Ok(())
    }

    /// 创建接口
    pub fn create_interface(name: &str, stack: &mut Vec<TsValue>) -> Result<(), TsError> {
        let mut interface_map = HashMap::new();
        interface_map.insert("name".to_string(), TsValue::String(name.to_string()));
        let interface_obj = TsValue::Object(interface_map);
        stack.push(interface_obj);
        Ok(())
    }
}
