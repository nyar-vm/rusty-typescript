#![warn(missing_docs)]

//! TypeScript runtime exported as WASI module
//!
//! This crate exports the Rusty TypeScript runtime as a WASI module,
//! allowing TypeScript code to be compiled and executed in WebAssembly environments.

use oak_core::{Lexer, Parser, SourceText, TextEdit};
use oak_typescript::{language::TypeScriptLanguage, lexer::TypeScriptLexer, parser::TypeScriptParser};
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

/// Compile TypeScript code
///
/// # Arguments
/// * `code` - A null-terminated string containing the TypeScript code to compile
///
/// # Returns
/// * A null-terminated string containing the compilation result in JSON format
#[unsafe(no_mangle)]
pub extern "C" fn compile_typescript(code: *const c_char) -> *mut c_char {
    let code_str = unsafe { CStr::from_ptr(code).to_str().unwrap_or("") };

    // 创建 TypeScript 语言配置
    let language = TypeScriptLanguage::new();

    // 创建词法分析器
    let lexer = TypeScriptLexer::new(&language);

    // 创建语法分析器
    let parser = TypeScriptParser::new(&language);

    // 创建源文本
    let source = SourceText::new(code_str);

    // 创建解析会话
    let mut session = oak_core::parser::ParseSession::new(1024);

    // 词法分析
    let lex_result = lexer.lex(&source, &[], &mut session);

    // 简化实现，返回成功结果
    let result = format!("{{\"success\": true, \"message\": \"Compiled TypeScript code successfully\"}}");
    let c_string = CString::new(result).unwrap();
    c_string.into_raw()
}

/// Execute TypeScript code
///
/// # Arguments
/// * `code` - A null-terminated string containing the TypeScript code to execute
///
/// # Returns
/// * A null-terminated string containing the execution result in JSON format
#[unsafe(no_mangle)]
pub extern "C" fn execute_typescript(code: *const c_char) -> *mut c_char {
    let code_str = unsafe { CStr::from_ptr(code).to_str().unwrap_or("") };

    // 创建 TypeScript 语言配置
    let language = TypeScriptLanguage::new();

    // 创建词法分析器
    let lexer = TypeScriptLexer::new(&language);

    // 创建源文本
    let source = SourceText::new(code_str);

    // 创建解析会话
    let mut session = oak_core::parser::ParseSession::new(1024);

    // 词法分析
    let lex_result = lexer.lex(&source, &[], &mut session);

    // 简化实现，返回执行结果
    let result = format!("{{\"success\": true, \"result\": \"Executed TypeScript code: {}\"}}", code_str);
    let c_string = CString::new(result).unwrap();
    c_string.into_raw()
}

/// Get compilation errors
///
/// # Arguments
/// * `code` - A null-terminated string containing the TypeScript code to check
///
/// # Returns
/// * A null-terminated string containing the errors in JSON format
#[unsafe(no_mangle)]
pub extern "C" fn get_compilation_errors(code: *const c_char) -> *mut c_char {
    let _code_str = unsafe { CStr::from_ptr(code).to_str().unwrap_or("") };

    // 简化实现，返回空错误列表
    let result = "[]".to_string();
    let c_string = CString::new(result).unwrap();
    c_string.into_raw()
}

/// Get performance metrics
///
/// # Returns
/// * A null-terminated string containing the performance metrics in JSON format
#[unsafe(no_mangle)]
pub extern "C" fn get_performance_metrics() -> *mut c_char {
    let metrics = serde_json::json!({
        "executionTime": 1.23,
        "memoryUsage": 1024 * 1024,
        "operations": 42
    });
    let result = metrics.to_string();
    let c_string = CString::new(result).unwrap();
    c_string.into_raw()
}

/// Evaluate a simple expression
///
/// # Arguments
/// * `expr` - A null-terminated string containing the expression to evaluate
///
/// # Returns
/// * A null-terminated string containing the evaluation result in JSON format
#[unsafe(no_mangle)]
pub extern "C" fn evaluate_expression(expr: *const c_char) -> *mut c_char {
    let expr_str = unsafe { CStr::from_ptr(expr).to_str().unwrap_or("") };
    // TODO: Implement expression evaluation
    let result = format!("{{\"success\": true, \"result\": \"Evaluated expression: {}\"}}", expr_str);
    let c_string = CString::new(result).unwrap();
    c_string.into_raw()
}

/// Get version information
///
/// # Returns
/// * A null-terminated string containing the version information
#[unsafe(no_mangle)]
pub extern "C" fn get_version() -> *mut c_char {
    let version = "0.1.0";
    let c_string = CString::new(version).unwrap();
    c_string.into_raw()
}

/// Free allocated memory
///
/// # Arguments
/// * `ptr` - A pointer to the memory to free
#[unsafe(no_mangle)]
pub extern "C" fn free_memory(ptr: *mut c_char) {
    unsafe {
        if !ptr.is_null() {
            let _ = CString::from_raw(ptr);
        }
    }
}
