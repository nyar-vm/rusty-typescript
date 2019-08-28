use typescript_types::{TsError, TsValue};

/// FFI 函数类型
pub type FfiFunction = Box<dyn Fn(&[TsValue]) -> Result<TsValue, TsError>>;

/// FFI 模块
pub struct FfiModule {
    /// 函数映射
    functions: std::collections::HashMap<String, FfiFunction>,
}

impl FfiModule {
    /// 创建一个新的 FFI 模块
    pub fn new() -> Self {
        Self { functions: std::collections::HashMap::new() }
    }

    /// 添加 FFI 函数
    pub fn add_function(&mut self, name: &str, func: FfiFunction) {
        self.functions.insert(name.to_string(), func);
    }

    /// 调用 FFI 函数
    pub fn call_function(&self, name: &str, args: &[TsValue]) -> Result<TsValue, TsError> {
        if let Some(func) = self.functions.get(name) {
            func(args)
        }
        else {
            Err(TsError::ReferenceError(format!("FFI function '{}' not found", name)))
        }
    }

    /// 检查函数是否存在
    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }
}

/// FFI 管理器
pub struct FfiManager {
    /// 模块映射
    modules: std::collections::HashMap<String, FfiModule>,
}

impl FfiManager {
    /// 创建一个新的 FFI 管理器
    pub fn new() -> Self {
        Self { modules: std::collections::HashMap::new() }
    }

    /// 添加 FFI 模块
    pub fn add_module(&mut self, name: &str, module: FfiModule) {
        self.modules.insert(name.to_string(), module);
    }

    /// 获取 FFI 模块
    pub fn get_module(&self, name: &str) -> Option<&FfiModule> {
        self.modules.get(name)
    }

    /// 调用 FFI 函数
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
        // 创建标准模块
        let mut std_module = FfiModule::new();

        // 添加标准函数
        std_module.add_function(
            "print",
            Box::new(|args| {
                for arg in args {
                    print!("{}", arg.to_string());
                }
                Ok(TsValue::Undefined)
            }),
        );

        std_module.add_function(
            "println",
            Box::new(|args| {
                for arg in args {
                    println!("{}", arg.to_string());
                }
                Ok(TsValue::Undefined)
            }),
        );

        std_module.add_function(
            "read_line",
            Box::new(|_| {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                Ok(TsValue::String(input.trim_end().to_string()))
            }),
        );

        std_module.add_function(
            "exit",
            Box::new(|args| {
                if let Some(TsValue::Number(code)) = args.get(0) {
                    std::process::exit(*code as i32);
                }
                else {
                    std::process::exit(0);
                }
            }),
        );

        // 添加标准模块
        self.add_module("std", std_module);
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
