//! NAPI 相关功能测试模块
//!
//! 测试 TsValue 与 NAPI 值之间的转换、错误处理、模块管理和 FFI 桥接功能。
//!
//! 注意：由于 NAPI JavaScript 转换功能需要 Node.js 运行时环境，
//! 本测试文件主要测试 FFI 模块的核心功能和类型转换 trait。

use std::sync::Arc;
use typescript::ffi::{FfiFunction, FfiManager, FfiModule, FromRust, ToRust};
use typescript_types::{TsError, TsValue};

/// 测试 TsError TypeError 的 Display 实现
///
/// 验证 TypeError 能够正确显示错误信息
#[test]
fn test_ts_error_type_error_display() {
    let error = TsError::TypeError("expected string".to_string());
    let display_str = format!("{}", error);
    assert!(display_str.contains("TypeError"));
    assert!(display_str.contains("expected string"));
}

/// 测试 TsError ReferenceError 的 Display 实现
#[test]
fn test_ts_error_reference_error_display() {
    let error = TsError::ReferenceError("variable not defined".to_string());
    let display_str = format!("{}", error);
    assert!(display_str.contains("ReferenceError"));
    assert!(display_str.contains("variable not defined"));
}

/// 测试 TsError SyntaxError 的 Display 实现
#[test]
fn test_ts_error_syntax_error_display() {
    let error = TsError::SyntaxError("unexpected token".to_string());
    let display_str = format!("{}", error);
    assert!(display_str.contains("SyntaxError"));
    assert!(display_str.contains("unexpected token"));
}

/// 测试 TsError RangeError 的 Display 实现
#[test]
fn test_ts_error_range_error_display() {
    let error = TsError::RangeError("index out of bounds".to_string());
    let display_str = format!("{}", error);
    assert!(display_str.contains("RangeError"));
    assert!(display_str.contains("index out of bounds"));
}

/// 测试 TsError Other 的 Display 实现
#[test]
fn test_ts_error_other_display() {
    let error = TsError::Other("generic error".to_string());
    let display_str = format!("{}", error);
    assert!(display_str.contains("Error"));
    assert!(display_str.contains("generic error"));
}

/// 测试 FfiModule 创建和函数添加
///
/// 验证 FfiModule 能够正确创建并添加函数
#[test]
fn test_ffi_module_creation() {
    let mut module = FfiModule::new();
    assert!(!module.has_function("test"));

    let func: FfiFunction = Arc::new(|_args| Ok(TsValue::Number(42.0)));
    module.add_function("test", func);

    assert!(module.has_function("test"));
}

/// 测试 FfiModule 函数调用
///
/// 验证 FfiModule 能够正确调用已注册的函数
#[test]
fn test_ffi_module_call_function() {
    let mut module = FfiModule::new();

    let add_func: FfiFunction = Arc::new(|args| {
        let a = args
            .get(0)
            .and_then(|v| match v {
                TsValue::Number(n) => Some(*n),
                _ => None,
            })
            .unwrap_or(0.0);

        let b = args
            .get(1)
            .and_then(|v| match v {
                TsValue::Number(n) => Some(*n),
                _ => None,
            })
            .unwrap_or(0.0);

        Ok(TsValue::Number(a + b))
    });

    module.add_function("add", add_func);

    let result = module.call_function("add", &[TsValue::Number(3.0), TsValue::Number(5.0)]);
    assert!(result.is_ok());

    if let TsValue::Number(n) = result.unwrap() {
        assert!((n - 8.0).abs() < f64::EPSILON);
    }
    else {
        panic!("Expected Number result");
    }
}

/// 测试 FfiModule 调用不存在的函数
///
/// 验证调用不存在的函数时返回正确的错误
#[test]
fn test_ffi_module_call_nonexistent_function() {
    let module = FfiModule::new();
    let result = module.call_function("nonexistent", &[]);

    assert!(result.is_err());
    if let Err(TsError::ReferenceError(msg)) = result {
        assert!(msg.contains("nonexistent"));
    }
    else {
        panic!("Expected ReferenceError");
    }
}

/// 测试 FfiManager 创建和模块添加
///
/// 验证 FfiManager 能够正确创建并添加模块
#[test]
fn test_ffi_manager_creation() {
    let mut manager = FfiManager::new();
    assert!(manager.get_module("test").is_none());

    let module = FfiModule::new();
    manager.add_module("test", module);

    assert!(manager.get_module("test").is_some());
}

/// 测试 FfiManager 跨模块函数调用
///
/// 验证 FfiManager 能够正确调用不同模块中的函数
#[test]
fn test_ffi_manager_call_function() {
    let mut manager = FfiManager::new();
    let mut math_module = FfiModule::new();

    let multiply_func: FfiFunction = Arc::new(|args| {
        let a = args
            .get(0)
            .and_then(|v| match v {
                TsValue::Number(n) => Some(*n),
                _ => None,
            })
            .unwrap_or(0.0);

        let b = args
            .get(1)
            .and_then(|v| match v {
                TsValue::Number(n) => Some(*n),
                _ => None,
            })
            .unwrap_or(0.0);

        Ok(TsValue::Number(a * b))
    });

    math_module.add_function("multiply", multiply_func);
    manager.add_module("math", math_module);

    let result = manager.call_function("math", "multiply", &[TsValue::Number(4.0), TsValue::Number(7.0)]);
    assert!(result.is_ok());

    if let TsValue::Number(n) = result.unwrap() {
        assert!((n - 28.0).abs() < f64::EPSILON);
    }
    else {
        panic!("Expected Number result");
    }
}

/// 测试 FfiManager 调用不存在模块的函数
///
/// 验证调用不存在模块中的函数时返回正确的错误
#[test]
fn test_ffi_manager_call_nonexistent_module() {
    let manager = FfiManager::new();
    let result = manager.call_function("nonexistent", "func", &[]);

    assert!(result.is_err());
    if let Err(TsError::ReferenceError(msg)) = result {
        assert!(msg.contains("nonexistent"));
    }
    else {
        panic!("Expected ReferenceError");
    }
}

/// 测试 FfiManager 注册标准函数
///
/// 验证 FfiManager 能够正确注册标准库函数
#[test]
fn test_ffi_manager_register_std_functions() {
    let mut manager = FfiManager::new();
    manager.register_std_functions();

    assert!(manager.get_module("std").is_some());

    let std_module = manager.get_module("std").unwrap();
    assert!(std_module.has_function("print"));
    assert!(std_module.has_function("println"));
    assert!(std_module.has_function("read_line"));
    assert!(std_module.has_function("exit"));
}

/// 测试 FromRust trait 实现 - bool
///
/// 验证 bool 类型能够正确转换为 TsValue
#[test]
fn test_from_rust_bool() {
    let true_value = TsValue::from_rust(true);
    assert!(matches!(true_value, TsValue::Boolean(true)));

    let false_value = TsValue::from_rust(false);
    assert!(matches!(false_value, TsValue::Boolean(false)));
}

/// 测试 ToRust trait 实现 - bool
///
/// 验证 TsValue 能够正确转换为 bool 类型
#[test]
fn test_to_rust_bool() {
    let ts_value = TsValue::Boolean(true);
    let result: bool = TsValue::to_rust(&ts_value).unwrap();
    assert!(result);

    let ts_value = TsValue::Boolean(false);
    let result: bool = TsValue::to_rust(&ts_value).unwrap();
    assert!(!result);

    let ts_value = TsValue::String("not a bool".to_string());
    let result = TsValue::to_rust::<bool>(&ts_value);
    assert!(result.is_err());
}

/// 测试 FromRust trait 实现 - f64
///
/// 验证 f64 类型能够正确转换为 TsValue
#[test]
fn test_from_rust_f64() {
    let value = TsValue::from_rust(3.14159);
    if let TsValue::Number(n) = value {
        assert!((n - 3.14159).abs() < f64::EPSILON);
    }
    else {
        panic!("Expected Number");
    }
}

/// 测试 ToRust trait 实现 - f64
///
/// 验证 TsValue 能够正确转换为 f64 类型
#[test]
fn test_to_rust_f64() {
    let ts_value = TsValue::Number(2.71828);
    let result: f64 = TsValue::to_rust(&ts_value).unwrap();
    assert!((result - 2.71828).abs() < f64::EPSILON);
}

/// 测试 FromRust trait 实现 - String
///
/// 验证 String 类型能够正确转换为 TsValue
#[test]
fn test_from_rust_string() {
    let value = TsValue::from_rust("hello world".to_string());
    assert!(matches!(value, TsValue::String(s) if s == "hello world"));
}

/// 测试 ToRust trait 实现 - String
///
/// 验证 TsValue 能够正确转换为 String 类型
#[test]
fn test_to_rust_string() {
    let ts_value = TsValue::String("test string".to_string());
    let result: String = TsValue::to_rust(&ts_value).unwrap();
    assert_eq!(result, "test string");
}

/// 测试 FromRust trait 实现 - Vec<TsValue>
///
/// 验证 Vec<TsValue> 类型能够正确转换为 TsValue::Array
#[test]
fn test_from_rust_vec() {
    let vec = vec![TsValue::Number(1.0), TsValue::Number(2.0), TsValue::Number(3.0)];
    let value = TsValue::from_rust(vec);

    if let TsValue::Array(arr) = value {
        assert_eq!(arr.len(), 3);
    }
    else {
        panic!("Expected Array");
    }
}

/// 测试 ToRust trait 实现 - Vec<TsValue>
///
/// 验证 TsValue::Array 能够正确转换为 Vec<TsValue>
#[test]
fn test_to_rust_vec() {
    let ts_value = TsValue::Array(vec![TsValue::Number(1.0), TsValue::Number(2.0)]);
    let result: Vec<TsValue> = TsValue::to_rust(&ts_value).unwrap();
    assert_eq!(result.len(), 2);

    let ts_value = TsValue::String("not an array".to_string());
    let result = TsValue::to_rust::<Vec<TsValue>>(&ts_value);
    assert!(result.is_err());
}

/// 测试 TsValue 基本类型检查方法
///
/// 验证 TsValue 的各种 is_* 方法能够正确判断类型
#[test]
fn test_ts_value_type_checks() {
    assert!(TsValue::Undefined.is_undefined());
    assert!(TsValue::Null.is_null());
    assert!(TsValue::Boolean(true).is_boolean());
    assert!(TsValue::Number(42.0).is_number());
    assert!(TsValue::String("test".to_string()).is_string());
    assert!(TsValue::Object(vec![]).is_object());
    assert!(TsValue::Array(vec![]).is_array());
    assert!(TsValue::BigInt(123i128).is_bigint());
    assert!(TsValue::Date(123456789i64).is_date());
    assert!(TsValue::RegExp("test".to_string()).is_regexp());
    assert!(TsValue::Map(vec![]).is_map());
    assert!(TsValue::Set(vec![]).is_set());
    assert!(TsValue::Promise(Box::new(TsValue::Undefined)).is_promise());
}

/// 测试 TsValue 转换为布尔值
///
/// 验证 TsValue::to_boolean 方法能够正确转换
#[test]
fn test_ts_value_to_boolean() {
    assert!(!TsValue::Undefined.to_boolean());
    assert!(!TsValue::Null.to_boolean());
    assert!(TsValue::Boolean(true).to_boolean());
    assert!(!TsValue::Boolean(false).to_boolean());
    assert!(TsValue::Number(1.0).to_boolean());
    assert!(!TsValue::Number(0.0).to_boolean());
    assert!(TsValue::String("hello".to_string()).to_boolean());
    assert!(!TsValue::String("".to_string()).to_boolean());
    assert!(TsValue::Object(vec![]).to_boolean());
    assert!(TsValue::Array(vec![]).to_boolean());
}

/// 测试 TsValue 转换为数字
///
/// 验证 TsValue::to_number 方法能够正确转换
#[test]
fn test_ts_value_to_number() {
    assert!(TsValue::Undefined.to_number().is_nan());
    assert!((TsValue::Null.to_number() - 0.0).abs() < f64::EPSILON);
    assert!((TsValue::Boolean(true).to_number() - 1.0).abs() < f64::EPSILON);
    assert!((TsValue::Boolean(false).to_number() - 0.0).abs() < f64::EPSILON);
    assert!((TsValue::Number(42.0).to_number() - 42.0).abs() < f64::EPSILON);
    assert!((TsValue::String("123".to_string()).to_number() - 123.0).abs() < f64::EPSILON);
    assert!(TsValue::String("not a number".to_string()).to_number().is_nan());
}

/// 测试 TsValue 转换为字符串
///
/// 验证 TsValue::to_string 方法能够正确转换
#[test]
fn test_ts_value_to_string() {
    assert_eq!(TsValue::Undefined.to_string(), "undefined");
    assert_eq!(TsValue::Null.to_string(), "null");
    assert_eq!(TsValue::Boolean(true).to_string(), "true");
    assert_eq!(TsValue::Boolean(false).to_string(), "false");
    assert_eq!(TsValue::String("hello".to_string()).to_string(), "hello");
    assert_eq!(TsValue::Object(vec![]).to_string(), "[object Object]");
    assert_eq!(TsValue::Array(vec![]).to_string(), "[]");
}

/// 测试 TsValue Clone 实现
///
/// 验证 TsValue 能够正确克隆各种类型
#[test]
fn test_ts_value_clone() {
    let original = TsValue::Number(42.0);
    let cloned = original.clone();
    assert!(matches!(cloned, TsValue::Number(n) if (n - 42.0).abs() < f64::EPSILON));

    let original = TsValue::String("test".to_string());
    let cloned = original.clone();
    assert!(matches!(cloned, TsValue::String(s) if s == "test"));

    let original = TsValue::Array(vec![TsValue::Number(1.0), TsValue::Number(2.0)]);
    let cloned = original.clone();
    if let TsValue::Array(arr) = cloned {
        assert_eq!(arr.len(), 2);
    }
    else {
        panic!("Expected Array");
    }
}

/// 测试 TsValue Debug 实现
///
/// 验证 TsValue 的 Debug trait 实现能够正确输出
#[test]
fn test_ts_value_debug() {
    let value = TsValue::Undefined;
    let debug_str = format!("{:?}", value);
    assert_eq!(debug_str, "Undefined");

    let value = TsValue::Number(42.0);
    let debug_str = format!("{:?}", value);
    assert_eq!(debug_str, "Number");

    let value = TsValue::String("test".to_string());
    let debug_str = format!("{:?}", value);
    assert_eq!(debug_str, "String");
}

/// 测试 TsError Debug 实现
///
/// 验证 TsError 的 Debug trait 实现能够正确输出
#[test]
fn test_ts_error_debug() {
    let error = TsError::TypeError("test error".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("TypeError"));

    let error = TsError::ReferenceError("ref error".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("ReferenceError"));
}

/// 测试 FfiModule 多函数注册
///
/// 验证 FfiModule 能够正确注册和调用多个函数
#[test]
fn test_ffi_module_multiple_functions() {
    let mut module = FfiModule::new();

    let add_func: FfiFunction = Arc::new(|args| {
        let a = args
            .get(0)
            .and_then(|v| match v {
                TsValue::Number(n) => Some(*n),
                _ => None,
            })
            .unwrap_or(0.0);
        let b = args
            .get(1)
            .and_then(|v| match v {
                TsValue::Number(n) => Some(*n),
                _ => None,
            })
            .unwrap_or(0.0);
        Ok(TsValue::Number(a + b))
    });

    let sub_func: FfiFunction = Arc::new(|args| {
        let a = args
            .get(0)
            .and_then(|v| match v {
                TsValue::Number(n) => Some(*n),
                _ => None,
            })
            .unwrap_or(0.0);
        let b = args
            .get(1)
            .and_then(|v| match v {
                TsValue::Number(n) => Some(*n),
                _ => None,
            })
            .unwrap_or(0.0);
        Ok(TsValue::Number(a - b))
    });

    module.add_function("add", add_func);
    module.add_function("sub", sub_func);

    assert!(module.has_function("add"));
    assert!(module.has_function("sub"));

    let add_result = module.call_function("add", &[TsValue::Number(5.0), TsValue::Number(3.0)]);
    if let TsValue::Number(n) = add_result.unwrap() {
        assert!((n - 8.0).abs() < f64::EPSILON);
    }

    let sub_result = module.call_function("sub", &[TsValue::Number(5.0), TsValue::Number(3.0)]);
    if let TsValue::Number(n) = sub_result.unwrap() {
        assert!((n - 2.0).abs() < f64::EPSILON);
    }
}

/// 测试 FfiManager 多模块管理
///
/// 验证 FfiManager 能够正确管理多个模块
#[test]
fn test_ffi_manager_multiple_modules() {
    let mut manager = FfiManager::new();

    let mut math_module = FfiModule::new();
    math_module.add_function("pi", Arc::new(|_| Ok(TsValue::Number(std::f64::consts::PI))));

    let mut string_module = FfiModule::new();
    string_module.add_function("empty", Arc::new(|_| Ok(TsValue::String("".to_string()))));

    manager.add_module("math", math_module);
    manager.add_module("string", string_module);

    assert!(manager.get_module("math").is_some());
    assert!(manager.get_module("string").is_some());

    let pi_result = manager.call_function("math", "pi", &[]);
    if let TsValue::Number(n) = pi_result.unwrap() {
        assert!((n - std::f64::consts::PI).abs() < f64::EPSILON);
    }

    let empty_result = manager.call_function("string", "empty", &[]);
    if let TsValue::String(s) = empty_result.unwrap() {
        assert!(s.is_empty());
    }
}

/// 测试 TsValue 复杂对象转换
///
/// 验证 TsValue 能够正确处理复杂对象
#[test]
fn test_ts_value_complex_object() {
    let obj = TsValue::Object(vec![
        ("name".to_string(), TsValue::String("Alice".to_string())),
        ("age".to_string(), TsValue::Number(30.0)),
        (
            "hobbies".to_string(),
            TsValue::Array(vec![TsValue::String("reading".to_string()), TsValue::String("coding".to_string())]),
        ),
    ]);

    assert!(obj.is_object());
    assert!(obj.to_boolean());

    let obj_str = obj.to_string();
    assert!(obj_str.contains("[object Object]"));
}

/// 测试 TsValue BigInt 操作
///
/// 验证 TsValue::BigInt 类型的基本操作
#[test]
fn test_ts_value_bigint() {
    let bigint = TsValue::BigInt(12345678901234567890i128);

    assert!(bigint.is_bigint());
    assert!(bigint.to_boolean());

    let num = bigint.to_number();
    assert!((num - 12345678901234567890.0).abs() < 1e10);

    let str = bigint.to_string();
    assert_eq!(str, "12345678901234567890");
}

/// 测试 TsValue Date 操作
///
/// 验证 TsValue::Date 类型的基本操作
#[test]
fn test_ts_value_date() {
    let date = TsValue::Date(1609459200000i64);

    assert!(date.is_date());
    assert!(date.to_boolean());

    let num = date.to_number();
    assert!((num - 1609459200000.0).abs() < f64::EPSILON);

    let str = date.to_string();
    assert!(str.contains("Date"));
}

/// 测试 TsValue RegExp 操作
///
/// 验证 TsValue::RegExp 类型的基本操作
#[test]
fn test_ts_value_regexp() {
    let regexp = TsValue::RegExp("test.*pattern".to_string());

    assert!(regexp.is_regexp());
    assert!(regexp.to_boolean());

    let str = regexp.to_string();
    assert!(str.contains("test.*pattern"));
}

/// 测试 TsValue Map 操作
///
/// 验证 TsValue::Map 类型的基本操作
#[test]
fn test_ts_value_map() {
    let map = TsValue::Map(vec![
        (TsValue::String("key1".to_string()), TsValue::Number(1.0)),
        (TsValue::String("key2".to_string()), TsValue::Number(2.0)),
    ]);

    assert!(map.is_map());
    assert!(map.to_boolean());

    let empty_map = TsValue::Map(vec![]);
    assert!(!empty_map.to_boolean());
}

/// 测试 TsValue Set 操作
///
/// 验证 TsValue::Set 类型的基本操作
#[test]
fn test_ts_value_set() {
    let set = TsValue::Set(vec![TsValue::Number(1.0), TsValue::Number(2.0), TsValue::Number(3.0)]);

    assert!(set.is_set());
    assert!(set.to_boolean());

    let empty_set = TsValue::Set(vec![]);
    assert!(!empty_set.to_boolean());
}

/// 测试 TsValue Promise 操作
///
/// 验证 TsValue::Promise 类型的基本操作
#[test]
fn test_ts_value_promise() {
    let promise = TsValue::Promise(Box::new(TsValue::Number(42.0)));

    assert!(promise.is_promise());
    assert!(promise.to_boolean());

    let str = promise.to_string();
    assert!(str.contains("Promise"));
}

/// 测试 TsValue Union 操作
///
/// 验证 TsValue::Union 类型的基本操作
#[test]
fn test_ts_value_union() {
    let union = TsValue::Union(vec![TsValue::Number(1.0), TsValue::String("test".to_string())]);

    assert!(union.is_union());
    assert!(union.to_boolean());

    let empty_union = TsValue::Union(vec![]);
    assert!(!empty_union.to_boolean());

    let num = union.to_number();
    assert!((num - 1.0).abs() < f64::EPSILON);
}

/// 测试 TsValue Generic 操作
///
/// 验证 TsValue::Generic 类型的基本操作
#[test]
fn test_ts_value_generic() {
    let generic = TsValue::Generic("Array".to_string(), vec![TsValue::String("string".to_string())]);

    assert!(generic.is_generic());
    assert!(generic.to_boolean());

    let str = generic.to_string();
    assert!(str.contains("Array"));
}

/// 测试 TsValue Symbol 操作
///
/// 验证 TsValue::Symbol 类型的基本操作
#[test]
fn test_ts_value_symbol() {
    let symbol = TsValue::Symbol("mySymbol".to_string());

    assert!(symbol.is_symbol());
    assert!(symbol.to_boolean());

    let str = symbol.to_string();
    assert!(str.contains("Symbol"));
    assert!(str.contains("mySymbol"));
}
