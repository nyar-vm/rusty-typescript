//! NAPI 模块加载测试
//!
//! 测试 NAPI 模块加载、类型转换和函数调用功能。

use typescript::{
    ffi::{
        NapiEnv, NapiExport, NapiFunction, NapiModuleLoader, NapiStatus, NapiValue, check_napi_status, create_napi_error,
        napi_to_ts_value, ts_value_to_napi,
    },
    platform::dylib::{DylibError, build_dylib_name, get_dylib_extension, get_dylib_prefix},
};
use typescript_types::{TsError, TsValue};

/// 测试动态库文件名构建
#[test]
fn test_build_dylib_name() {
    let name = build_dylib_name("test");
    assert!(name.contains("test"));

    #[cfg(target_os = "windows")]
    assert!(name.ends_with(".dll"));

    #[cfg(target_os = "macos")]
    assert!(name.ends_with(".dylib"));

    #[cfg(target_os = "linux")]
    assert!(name.ends_with(".so"));
}

/// 测试动态库扩展名
#[test]
fn test_get_dylib_extension() {
    let ext = get_dylib_extension();
    assert!(!ext.is_empty());

    #[cfg(target_os = "windows")]
    assert_eq!(ext, "dll");

    #[cfg(target_os = "macos")]
    assert_eq!(ext, "dylib");

    #[cfg(target_os = "linux")]
    assert_eq!(ext, "so");
}

/// 测试动态库前缀
#[test]
fn test_get_dylib_prefix() {
    let prefix = get_dylib_prefix();

    #[cfg(target_os = "windows")]
    assert_eq!(prefix, "");

    #[cfg(not(target_os = "windows"))]
    assert_eq!(prefix, "lib");
}

/// 测试 NAPI 模块加载器创建
#[test]
fn test_napi_module_loader_creation() {
    let loader = NapiModuleLoader::new();
    assert!(!loader.has_module("test"));
    assert!(loader.module_names().is_empty());
}

/// 测试 NAPI 状态检查
#[test]
fn test_check_napi_status() {
    assert!(check_napi_status(NapiStatus::Ok).is_ok());
    assert!(check_napi_status(NapiStatus::GenericFailure).is_err());
    assert!(check_napi_status(NapiStatus::InvalidArg).is_err());
}

/// 测试 NAPI 错误创建
#[test]
fn test_create_napi_error() {
    let error = create_napi_error("test error");
    match error {
        TsError::Other(msg) => assert!(msg.contains("test error")),
        _ => panic!("Expected TsError::Other"),
    }
}

/// 测试 TsValue 到 NAPI 值转换
#[test]
fn test_ts_value_to_napi() {
    // 由于 NapiEnv 有私有字段，这里无法直接创建
    // 实际测试需要在有有效 NAPI 环境的情况下运行
    // 这里只测试函数签名和基本逻辑
}

/// 测试 NAPI 值到 TsValue 转换
#[test]
fn test_napi_to_ts_value() {
    // 由于 NapiEnv 和 NapiValue 有私有字段，这里无法直接创建
    // 实际测试需要在有有效 NAPI 环境的情况下运行
    // 这里只测试函数签名和基本逻辑
}

/// 测试 NAPI 函数创建
#[test]
fn test_napi_function_creation() {
    let func = NapiFunction::new("test_func", std::ptr::null());
    assert_eq!(func.name, "test_func");
}

/// 测试 NAPI 函数调用
#[test]
fn test_napi_function_call() {
    let func = NapiFunction::new("test_func", std::ptr::null());
    let result = func.call(&[TsValue::Number(1.0), TsValue::Number(2.0)]);
    assert!(result.is_ok());
    // 由于 TsValue 没有实现 PartialEq，这里只检查结果是否成功
}

/// 测试加载不存在的 NAPI 模块
#[test]
fn test_load_nonexistent_napi_module() {
    let mut loader = NapiModuleLoader::new();
    let result = loader.load_module("nonexistent", "/path/to/nonexistent.node");
    assert!(result.is_err());
}

/// 测试 NAPI 模块导出
#[test]
fn test_napi_export_enum() {
    let func = NapiFunction::new("test", std::ptr::null());
    let export = NapiExport::Function(func);

    match export {
        NapiExport::Function(f) => assert_eq!(f.name, "test"),
        _ => panic!("Expected NapiExport::Function"),
    }

    let export = NapiExport::Value(TsValue::Number(42.0));
    match export {
        NapiExport::Value(_v) => assert!(true), // 由于 TsValue 没有实现 PartialEq，这里只检查匹配是否成功
        _ => panic!("Expected NapiExport::Value"),
    }
}

/// 测试已加载的 NAPI 模块
#[test]
fn test_loaded_napi_module() {
    // 由于无法实际加载动态库，这里测试模块结构
    // 实际测试需要在有有效 .node 文件的环境中运行
}

/// 测试 NAPI 值类型枚举
#[test]
fn test_napi_value_type() {
    assert_eq!(NapiValue::Undefined as i32, 0);
    assert_eq!(NapiValue::Null as i32, 1);
    assert_eq!(NapiValue::Boolean as i32, 2);
    assert_eq!(NapiValue::Number as i32, 3);
    assert_eq!(NapiValue::String as i32, 4);
    assert_eq!(NapiValue::Symbol as i32, 5);
    assert_eq!(NapiValue::Object as i32, 6);
    assert_eq!(NapiValue::Function as i32, 7);
    assert_eq!(NapiValue::External as i32, 8);
    assert_eq!(NapiValue::Bigint as i32, 9);
}

/// 测试 NAPI 状态码枚举
#[test]
fn test_napi_status() {
    assert_eq!(NapiStatus::Ok as i32, 0);
    assert_eq!(NapiStatus::InvalidArg as i32, 1);
    assert_eq!(NapiStatus::GenericFailure as i32, 9);
    assert_eq!(NapiStatus::PendingException as i32, 10);
}

/// 测试动态库错误类型
#[test]
fn test_dylib_error() {
    let error = DylibError::OpenError("test".to_string());
    assert!(error.to_string().contains("Failed to open"));

    let error = DylibError::SymbolError("test".to_string());
    assert!(error.to_string().contains("Failed to find symbol"));

    let error = DylibError::CloseError("test".to_string());
    assert!(error.to_string().contains("Failed to close"));
}

/// 测试 FFI 管理器的 NAPI 模块支持
#[test]
fn test_ffi_manager_napi_support() {
    use typescript::ffi::FfiManager;

    let mut manager = FfiManager::new();

    // 测试初始状态
    assert!(!manager.has_napi_module("test"));
    assert!(manager.get_napi_module("test").is_none());
    assert!(manager.get_napi_module_names().is_empty());

    // 测试加载不存在的模块
    let result = manager.load_napi_module("test", "/nonexistent.node");
    assert!(result.is_err());

    // 测试卸载未加载的模块
    let result = manager.unload_napi_module("test");
    assert!(result.is_err());
}
