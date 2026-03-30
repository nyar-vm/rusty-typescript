//! 跨平台运行时测试
//!
//! 测试跨平台运行时功能。

use typescript::{create_runtime, run_script};
use typescript_types::TsValue;

#[test]
fn test_basic_typescript_execution() {
    let mut runtime = create_runtime();
    let script = "let x = 10; let y = 20; x + y";
    let result = run_script(&mut runtime, script).unwrap();
    assert_eq!(result, "30");
}

#[test]
fn test_console_log() {
    let mut runtime = create_runtime();
    let script = "console.log('Hello, world!'); 42";
    let result = run_script(&mut runtime, script).unwrap();
    assert_eq!(result, "42");
}

#[test]
fn test_module_management() {
    let mut runtime = create_runtime();
    // 测试模块导入
    runtime.import_module("test", None).unwrap();
    // 测试设置全局变量
    runtime.set_global("testVar", TsValue::Number(42.0));
    let script = "testVar + 8";
    let result = run_script(&mut runtime, script).unwrap();
    assert_eq!(result, "50");
}

#[test]
fn test_memory_management() {
    let mut runtime = create_runtime();
    // 执行垃圾回收
    runtime.garbage_collect();
    // 获取内存使用情况
    let memory_usage = runtime.get_memory_usage();
    assert!(memory_usage.contains("Memory usage:"));
}

#[test]
fn test_compile_options() {
    let mut runtime = create_runtime();
    // 获取当前编译选项
    let options = runtime.get_compile_options();
    assert_eq!(options.strict, false);
    assert_eq!(options.enable_jit, true);
}

#[test]
fn test_runtime_status() {
    let runtime = create_runtime();
    let status = runtime.get_status();
    assert!(status.contains("TypeScript Runtime Status"));
    assert!(status.contains("Globals:"));
    assert!(status.contains("Modules:"));
    assert!(status.contains("Objects:"));
}
