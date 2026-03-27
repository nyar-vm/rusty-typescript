//! 模块操作模块
//!
//! 提供模块导入和导出功能。

use std::{collections::HashMap, rc::Rc};

use typescript_types::{TsError, TsValue};

use super::module::ModuleInstance;

/// 模块操作执行器
pub struct ModuleOperations;

impl ModuleOperations {
    /// 导入模块
    pub fn import_module(
        name: &str,
        alias: Option<&str>,
        modules: &mut HashMap<String, ModuleInstance>,
        globals: &mut HashMap<String, TsValue>,
    ) -> Result<(), TsError> {
        if let Some(module) = modules.get(name) {
            let module_value = TsValue::Object(module.exports.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
            let var_name = alias.unwrap_or(name);
            globals.insert(var_name.to_string(), module_value);
            return Ok(());
        }

        if name.ends_with(".node") || name.contains(".node") {
            return Self::import_napi_module(name, alias, modules, globals);
        }

        let mut module = ModuleInstance::new(name.to_string());

        match name {
            "fs" => {
                module.export("readFile", TsValue::Function(Rc::new(|_| TsValue::Undefined)));
                module.export("writeFile", TsValue::Function(Rc::new(|_| TsValue::Undefined)));
            }
            "path" => {
                module.export("join", TsValue::Function(Rc::new(|_| TsValue::Undefined)));
                module.export("resolve", TsValue::Function(Rc::new(|_| TsValue::Undefined)));
            }
            "util" => {
                module.export("inspect", TsValue::Function(Rc::new(|_| TsValue::Undefined)));
            }
            _ => {}
        }

        module.loaded = true;

        let module_value = TsValue::Object(module.exports.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
        let var_name = alias.unwrap_or(name);
        globals.insert(var_name.to_string(), module_value);
        modules.insert(name.to_string(), module);

        Ok(())
    }

    /// 导入 NAPI 模块
    pub fn import_napi_module(
        name: &str,
        alias: Option<&str>,
        modules: &mut HashMap<String, ModuleInstance>,
        globals: &mut HashMap<String, TsValue>,
    ) -> Result<(), TsError> {
        let mut module = ModuleInstance::new(name.to_string());

        module.export("__napi_module__", TsValue::Boolean(true));

        module.loaded = true;

        let module_value = TsValue::Object(module.exports.iter().map(|(k, v)| (k.clone(), v.clone())).collect());
        let var_name = alias.unwrap_or(name);
        globals.insert(var_name.to_string(), module_value);
        modules.insert(name.to_string(), module);

        Ok(())
    }

    /// 导出值
    pub fn export_value(
        name: &str,
        stack: &mut Vec<TsValue>,
        modules: &mut HashMap<String, ModuleInstance>,
    ) -> Result<(), TsError> {
        if let Some(value) = stack.pop() {
            if let Some((_, module)) = modules.iter_mut().last() {
                module.export(name, value);
            }
        }
        Ok(())
    }
}
