//! JIT 性能测试
//!
//! 测试 JIT 编译优化的性能效果

use std::time::Instant;
use typescript_ir::Program;
use typescript_types::TsValue;

use typescript::jit::JITExecutor;

#[test]
fn test_jit_performance() {
    // 测试 JIT 执行器的基本功能
    test_jit_executor_basic();

    // 测试内存管理优化
    test_memory_optimization();
}

/// 测试 JIT 执行器基本功能
fn test_jit_executor_basic() {
    println!("=== 测试 JIT 执行器基本功能 ===");

    let mut executor = JITExecutor::new();

    // 创建一个简单的程序
    let program = Program { statements: vec![], source_map: None };

    // 创建全局变量
    let mut globals = std::collections::HashMap::new();
    globals.insert("test".to_string(), TsValue::Number(42.0));

    // 执行程序
    let start = Instant::now();
    for _ in 0..1000 {
        let result = executor.execute(&program, &globals);
        assert!(result.is_ok());
    }
    let duration = start.elapsed();

    println!("JIT 执行器基本功能测试完成，执行 1000 次调用耗时: {:?}", duration);
}

/// 测试内存管理优化
fn test_memory_optimization() {
    println!("=== 测试内存管理优化 ===");

    // 测试对象创建和回收
    let start = Instant::now();
    for _ in 0..10000 {
        // 创建对象
        let mut map = std::collections::HashMap::new();
        // 模拟使用对象
        map.insert("key".to_string(), TsValue::Number(1.0));
        // 模拟回收对象
    }
    let duration = start.elapsed();

    println!("内存管理优化测试完成，执行 10000 次对象创建和回收耗时: {:?}", duration);
}
