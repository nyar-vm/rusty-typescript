use std::sync::{Arc, Mutex};
use typescript_types::{TsError, TsValue};

/// 对象池
///
/// 用于管理和重用频繁使用的对象，减少内存分配和回收的开销
struct ObjectPool<T> {
    pool: Mutex<Vec<T>>,
    max_size: usize,
}

impl<T> ObjectPool<T> {
    /// 创建新的对象池
    ///
    /// # 参数
    /// - `max_size`: 对象池的最大容量
    pub fn new(max_size: usize) -> Self {
        Self {
            pool: Mutex::new(Vec::with_capacity(max_size)),
            max_size,
        }
    }

    /// 从对象池获取一个对象
    ///
    /// # 参数
    /// - `default`: 如果对象池为空，使用此函数创建新对象
    pub fn get<F>(&self, default: F) -> T
    where
        F: Fn() -> T,
    {
        let mut pool = self.pool.lock().unwrap();
        pool.pop().unwrap_or_else(default)
    }

    /// 将对象归还到对象池
    ///
    /// # 参数
    /// - `item`: 要归还的对象
    pub fn put(&self, item: T) {
        let mut pool = self.pool.lock().unwrap();
        if pool.len() < self.max_size {
            pool.push(item);
        }
    }
}

/// 全局对象池实例
lazy_static::lazy_static! {
    /// TsValue 数组对象池
    pub static ref TS_VALUE_ARRAY_POOL: ObjectPool<Vec<TsValue>> = ObjectPool::new(100);
    /// TsValue 对象池
    pub static ref TS_VALUE_OBJECT_POOL: ObjectPool<Vec<(String, TsValue)>> = ObjectPool::new(100);
    /// TsValue Map 条目池
    pub static ref TS_VALUE_MAP_POOL: ObjectPool<Vec<(TsValue, TsValue)>> = ObjectPool::new(100);
}

/// NAPI 模块加载器
pub mod napi;

pub use napi::{
    LoadedNapiModule, NapiEnv, NapiExport, NapiFunction, NapiModule, NapiModuleLoader, NapiStatus, NapiValue,
    NapiValueType, check_napi_status, create_napi_error, napi_to_ts_value, ts_value_to_napi,
};

/// 性能测试模块
pub mod benchmark;
pub use benchmark::{run_ffi_benchmark, run_napi_benchmark};

/// FFI 函数类型
///
/// 使用 Arc 包装以支持线程安全和克隆
pub type FfiFunction = Arc<dyn Fn(&[TsValue]) -> Result<TsValue, TsError> + Send + Sync>;

/// FFI 模块
pub struct FfiModule {
    /// 函数映射
    functions: std::collections::HashMap<String, FfiFunction>,
    /// 模块名称
    name: String,
}

impl FfiModule {
    /// 创建一个新的 FFI 模块
    ///
    /// # 参数
    /// - `name`: 模块名称
    pub fn new(name: &str) -> Self {
        Self {
            functions: std::collections::HashMap::new(),
            name: name.to_string(),
        }
    }

    /// 添加 FFI 函数
    ///
    /// # 参数
    /// - `name`: 函数名称
    /// - `func`: 函数实现
    pub fn add_function(&mut self, name: &str, func: FfiFunction) {
        self.functions.insert(name.to_string(), func);
    }

    /// 调用 FFI 函数
    ///
    /// # 参数
    /// - `name`: 函数名称
    /// - `args`: 函数参数
    ///
    /// # 返回
    /// - 成功时返回函数执行结果
    /// - 失败时返回错误信息
    pub fn call_function(&self, name: &str, args: &[TsValue]) -> Result<TsValue, TsError> {
        if let Some(func) = self.functions.get(name) {
            func(args)
        }
        else {
            Err(TsError::ReferenceError(format!("FFI function '{}' not found in module '{}'", name, self.name)))
        }
    }

    /// 检查函数是否存在
    ///
    /// # 参数
    /// - `name`: 函数名称
    ///
    /// # 返回
    /// - 如果函数存在返回 true，否则返回 false
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// 获取模块名称
    ///
    /// # 返回
    /// - 模块名称
    pub fn name(&self) -> &str {
        &self.name
    }

    /// 获取所有函数名称
    ///
    /// # 返回
    /// - 函数名称列表
    pub fn get_functions(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }
}

/// FFI 管理器
pub struct FfiManager {
    /// 模块映射
    modules: std::collections::HashMap<String, FfiModule>,
    /// NAPI 模块加载器
    napi_loader: NapiModuleLoader,
}

impl FfiManager {
    /// 创建一个新的 FFI 管理器
    pub fn new() -> Self {
        Self {
            modules: std::collections::HashMap::new(),
            napi_loader: NapiModuleLoader::new(),
        }
    }

    /// 加载 NAPI 模块
    ///
    /// # 参数
    /// - `name`: 模块名称
    /// - `path`: 模块文件路径（.node 文件）
    ///
    /// # 返回
    /// - 成功时返回加载的模块
    /// - 失败时返回错误
    pub fn load_napi_module(&mut self, name: &str, path: &str) -> Result<&LoadedNapiModule, TsError> {
        self.napi_loader.load_module(name, path)
    }

    /// 获取已加载的 NAPI 模块
    pub fn get_napi_module(&self, name: &str) -> Option<&LoadedNapiModule> {
        self.napi_loader.get_module(name)
    }

    /// 检查 NAPI 模块是否已加载
    pub fn has_napi_module(&self, name: &str) -> bool {
        self.napi_loader.has_module(name)
    }

    /// 卸载 NAPI 模块
    pub fn unload_napi_module(&mut self, name: &str) -> Result<(), TsError> {
        self.napi_loader.unload_module(name)
    }

    /// 获取所有已加载的 NAPI 模块名称
    pub fn get_napi_module_names(&self) -> Vec<&String> {
        self.napi_loader.module_names()
    }

    /// 添加 FFI 模块
    ///
    /// # 参数
    /// - `name`: 模块名称
    /// - `module`: 模块实例
    pub fn add_module(&mut self, name: &str, module: FfiModule) {
        self.modules.insert(name.to_string(), module);
    }

    /// 获取 FFI 模块
    ///
    /// # 参数
    /// - `name`: 模块名称
    ///
    /// # 返回
    /// - 如果模块存在返回模块引用，否则返回 None
    pub fn get_module(&self, name: &str) -> Option<&FfiModule> {
        self.modules.get(name)
    }

    /// 调用 FFI 函数
    ///
    /// # 参数
    /// - `module_name`: 模块名称
    /// - `function_name`: 函数名称
    /// - `args`: 函数参数
    ///
    /// # 返回
    /// - 成功时返回函数执行结果
    /// - 失败时返回错误信息
    pub fn call_function(&self, module_name: &str, function_name: &str, args: &[TsValue]) -> Result<TsValue, TsError> {
        if let Some(module) = self.modules.get(module_name) {
            module.call_function(function_name, args)
        }
        else {
            Err(TsError::ReferenceError(format!("FFI module '{}' not found", module_name)))
        }
    }

    /// 注册标准 FFI 函数
    pub fn register_std_functions(&mut self) {
        let mut std_module = FfiModule::new("std");

        std_module.add_function(
            "print",
            Arc::new(|args| {
                for arg in args {
                    print!("{}", arg.to_string());
                }
                Ok(TsValue::Undefined)
            }),
        );

        std_module.add_function(
            "println",
            Arc::new(|args| {
                for arg in args {
                    println!("{}", arg.to_string());
                }
                Ok(TsValue::Undefined)
            }),
        );

        std_module.add_function(
            "read_line",
            Arc::new(|_| {
                let mut input = String::new();
                match std::io::stdin().read_line(&mut input) {
                    Ok(_) => Ok(TsValue::String(input.trim_end().to_string())),
                    Err(e) => Err(TsError::Other(format!("Failed to read line: {}", e))),
                }
            }),
        );

        std_module.add_function(
            "exit",
            Arc::new(|args| {
                if let Some(TsValue::Number(code)) = args.get(0) {
                    std::process::exit(*code as i32);
                }
                else {
                    std::process::exit(0);
                }
            }),
        );

        std_module.add_function(
            "throw_error",
            Arc::new(|args| {
                if let Some(TsValue::String(msg)) = args.get(0) {
                    Err(TsError::Other(msg.clone()))
                }
                else {
                    Err(TsError::TypeError("Expected error message".to_string()))
                }
            }),
        );

        self.add_module("std", std_module);
    }

    /// 获取所有模块名称
    ///
    /// # 返回
    /// - 模块名称列表
    pub fn get_modules(&self) -> Vec<String> {
        self.modules.keys().cloned().collect()
    }

    /// 移除 FFI 模块
    ///
    /// # 参数
    /// - `name`: 模块名称
    pub fn remove_module(&mut self, name: &str) {
        self.modules.remove(name);
    }
}

/// 从 Rust 类型转换为 TypeScript 值
pub trait FromRust<T> {
    /// 从 Rust 类型转换为 TypeScript 值
    fn from_rust(value: T) -> TsValue;
}

/// 从 TypeScript 值转换为 Rust 类型
pub trait ToRust<T> {
    /// 从 TypeScript 值转换为 Rust 类型
    fn to_rust(value: &TsValue) -> Result<T, TsError>;
}

/// 实现 FromRust  trait for bool
impl FromRust<bool> for TsValue {
    fn from_rust(value: bool) -> TsValue {
        TsValue::Boolean(value)
    }
}

/// 实现 ToRust  trait for bool
impl ToRust<bool> for TsValue {
    fn to_rust(value: &TsValue) -> Result<bool, TsError> {
        match value {
            TsValue::Boolean(b) => Ok(*b),
            _ => Err(TsError::TypeError("Expected boolean".to_string())),
        }
    }
}

/// 实现 FromRust  trait for f64
impl FromRust<f64> for TsValue {
    fn from_rust(value: f64) -> TsValue {
        TsValue::Number(value)
    }
}

/// 实现 ToRust  trait for f64
impl ToRust<f64> for TsValue {
    fn to_rust(value: &TsValue) -> Result<f64, TsError> {
        Ok(value.to_number())
    }
}

/// 实现 FromRust  trait for String
impl FromRust<String> for TsValue {
    fn from_rust(value: String) -> TsValue {
        TsValue::String(value)
    }
}

/// 实现 ToRust  trait for String
impl ToRust<String> for TsValue {
    fn to_rust(value: &TsValue) -> Result<String, TsError> {
        Ok(value.to_string())
    }
}

/// 实现 FromRust  trait for &str
impl FromRust<&str> for TsValue {
    fn from_rust(value: &str) -> TsValue {
        TsValue::String(value.to_string())
    }
}

/// 实现 FromRust  trait for Vec<TsValue>
impl FromRust<Vec<TsValue>> for TsValue {
    fn from_rust(value: Vec<TsValue>) -> TsValue {
        TsValue::Array(value)
    }
}

/// 实现 ToRust  trait for Vec<TsValue>
impl ToRust<Vec<TsValue>> for TsValue {
    fn to_rust(value: &TsValue) -> Result<Vec<TsValue>, TsError> {
        match value {
            TsValue::Array(arr) => Ok(arr.clone()),
            _ => Err(TsError::TypeError("Expected array".to_string())),
        }
    }
}

/// 实现 FromRust  trait for Vec<(String, TsValue)>
impl FromRust<Vec<(String, TsValue)>> for TsValue {
    fn from_rust(value: Vec<(String, TsValue)>) -> TsValue {
        TsValue::Object(value)
    }
}

/// 实现 ToRust  trait for Vec<(String, TsValue)>
impl ToRust<Vec<(String, TsValue)>> for TsValue {
    fn to_rust(value: &TsValue) -> Result<Vec<(String, TsValue)>, TsError> {
        match value {
            TsValue::Object(props) => Ok(props.to_vec()),
            _ => Err(TsError::TypeError("Expected object".to_string())),
        }
    }
}

/// 实现 FromRust  trait for Vec<(TsValue, TsValue)>
impl FromRust<Vec<(TsValue, TsValue)>> for TsValue {
    fn from_rust(value: Vec<(TsValue, TsValue)>) -> TsValue {
        TsValue::Map(value)
    }
}

/// 实现 ToRust  trait for Vec<(TsValue, TsValue)>
impl ToRust<Vec<(TsValue, TsValue)>> for TsValue {
    fn to_rust(value: &TsValue) -> Result<Vec<(TsValue, TsValue)>, TsError> {
        match value {
            TsValue::Map(entries) => Ok(entries.to_vec()),
            _ => Err(TsError::TypeError("Expected map".to_string())),
        }
    }
}

/// 实现 FromRust  trait for i128
impl FromRust<i128> for TsValue {
    fn from_rust(value: i128) -> TsValue {
        TsValue::BigInt(value)
    }
}

/// 实现 ToRust  trait for i128
impl ToRust<i128> for TsValue {
    fn to_rust(value: &TsValue) -> Result<i128, TsError> {
        match value {
            TsValue::BigInt(bi) => Ok(*bi),
            TsValue::Number(n) => Ok(*n as i128),
            _ => Err(TsError::TypeError("Expected bigint or number".to_string())),
        }
    }
}

/// 实现 FromRust  trait for i64
impl FromRust<i64> for TsValue {
    fn from_rust(value: i64) -> TsValue {
        TsValue::Date(value)
    }
}

/// 实现 ToRust  trait for i64
impl ToRust<i64> for TsValue {
    fn to_rust(value: &TsValue) -> Result<i64, TsError> {
        match value {
            TsValue::Date(d) => Ok(*d),
            TsValue::Number(n) => Ok(*n as i64),
            _ => Err(TsError::TypeError("Expected date or number".to_string())),
        }
    }
}

/// 实现 FromRust  trait for Option<TsValue>
impl FromRust<Option<TsValue>> for TsValue {
    fn from_rust(value: Option<TsValue>) -> TsValue {
        value.unwrap_or(TsValue::Null)
    }
}

/// 实现 ToRust  trait for Option<TsValue>
impl ToRust<Option<TsValue>> for TsValue {
    fn to_rust(value: &TsValue) -> Result<Option<TsValue>, TsError> {
        match value {
            TsValue::Null | TsValue::Undefined => Ok(None),
            _ => Ok(Some(value.clone())),
        }
    }
}

/// 实现 FromRust  trait for Result<TsValue, TsError>
impl FromRust<Result<TsValue, TsError>> for TsValue {
    fn from_rust(value: Result<TsValue, TsError>) -> TsValue {
        match value {
            Ok(v) => v,
            Err(e) => TsValue::Error(e.to_string()),
        }
    }
}

/// 实现 ToRust  trait for Result<TsValue, TsError>
impl ToRust<Result<TsValue, TsError>> for TsValue {
    fn to_rust(value: &TsValue) -> Result<Result<TsValue, TsError>, TsError> {
        match value {
            TsValue::Error(msg) => Ok(Err(TsError::Other(msg.clone()))),
            _ => Ok(Ok(value.clone())),
        }
    }
}

/// 辅助函数：从 TypeScript 对象中获取属性
///
/// # 参数
/// - `obj`: TypeScript 对象
/// - `key`: 属性名称
///
/// # 返回
/// - 成功时返回属性值
/// - 失败时返回错误信息
pub fn get_object_property(obj: &TsValue, key: &str) -> Result<TsValue, TsError> {
    match obj {
        TsValue::Object(props) => {
            for (k, v) in props {
                if k == key {
                    return Ok(v.clone());
                }
            }
            Err(TsError::ReferenceError(format!("Property '{}' not found", key)))
        }
        _ => Err(TsError::TypeError("Expected object".to_string())),
    }
}

/// 辅助函数：设置 TypeScript 对象的属性
///
/// # 参数
/// - `obj`: TypeScript 对象
/// - `key`: 属性名称
/// - `value`: 属性值
///
/// # 返回
/// - 成功时返回更新后的对象
/// - 失败时返回错误信息
pub fn set_object_property(obj: &TsValue, key: &str, value: TsValue) -> Result<TsValue, TsError> {
    match obj {
        TsValue::Object(props) => {
            let mut new_props = props.clone();
            let mut found = false;
            for (k, v) in &mut new_props {
                if k == key {
                    *v = value.clone();
                    found = true;
                    break;
                }
            }
            if !found {
                new_props.push((key.to_string(), value));
            }
            Ok(TsValue::Object(new_props))
        }
        _ => Err(TsError::TypeError("Expected object".to_string())),
    }
}

/// 辅助函数：从 TypeScript 数组中获取元素
///
/// # 参数
/// - `arr`: TypeScript 数组
/// - `index`: 索引
///
/// # 返回
/// - 成功时返回元素值
/// - 失败时返回错误信息
pub fn get_array_element(arr: &TsValue, index: usize) -> Result<TsValue, TsError> {
    match arr {
        TsValue::Array(values) => {
            if index < values.len() {
                Ok(values[index].clone())
            }
            else {
                Err(TsError::RangeError(format!("Index {} out of bounds", index)))
            }
        }
        _ => Err(TsError::TypeError("Expected array".to_string())),
    }
}

/// 辅助函数：设置 TypeScript 数组的元素
///
/// # 参数
/// - `arr`: TypeScript 数组
/// - `index`: 索引
/// - `value`: 元素值
///
/// # 返回
/// - 成功时返回更新后的数组
/// - 失败时返回错误信息
pub fn set_array_element(arr: &TsValue, index: usize, value: TsValue) -> Result<TsValue, TsError> {
    match arr {
        TsValue::Array(values) => {
            if index < values.len() {
                let mut new_values = values.clone();
                new_values[index] = value;
                Ok(TsValue::Array(new_values))
            }
            else {
                Err(TsError::RangeError(format!("Index {} out of bounds", index)))
            }
        }
        _ => Err(TsError::TypeError("Expected array".to_string())),
    }
}
