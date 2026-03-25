//! 性能基准测试
//!
//! 测试 TypeScript 代码执行性能，验证优化效果

use std::time::{Duration, Instant};
use typescript::create_runtime;
use typescript_types::TsValue;

/// 性能基准测试
#[test]
fn test_performance_benchmark() {
    println!("=== TypeScript 性能基准测试 ===\n");

    // 测试算术运算性能
    test_arithmetic_performance();

    // 测试函数调用性能
    test_function_call_performance();

    // 测试循环性能
    test_loop_performance();

    // 测试对象操作性能
    test_object_performance();

    // 测试数组操作性能
    test_array_performance();

    // 测试内存管理性能
    test_memory_performance();

    println!("\n=== 性能基准测试完成 ===");
}

/// 测试算术运算性能
fn test_arithmetic_performance() {
    println!("--- 算术运算性能测试 ---");

    let mut runtime = create_runtime();

    // 简单的算术运算
    let script = r#"
        let result = 0;
        for (let i = 0; i < 10000; i++) {
            result = result + i * 2 - i / 2 + i % 10;
        }
        result
    "#;

    let start = Instant::now();
    match runtime.execute_script(script) {
        Ok(result) => {
            let duration = start.elapsed();
            println!("算术运算测试完成: {:?}", duration);
            println!("结果: {:?}", result);

            // 验证性能目标：执行时间应该小于 100ms
            assert!(duration < Duration::from_millis(100), "算术运算性能不达标: {:?}", duration);
        }
        Err(error) => {
            panic!("算术运算测试失败: {:?}", error);
        }
    }
}

/// 测试函数调用性能
fn test_function_call_performance() {
    println!("\n--- 函数调用性能测试 ---");

    let mut runtime = create_runtime();

    // 递归函数调用
    let script = r#"
        function fibonacci(n) {
            if (n <= 1) return n;
            return fibonacci(n - 1) + fibonacci(n - 2);
        }
        fibonacci(20)
    "#;

    let start = Instant::now();
    match runtime.execute_script(script) {
        Ok(result) => {
            let duration = start.elapsed();
            println!("函数调用测试完成: {:?}", duration);
            println!("结果: {:?}", result);

            // 验证性能目标：执行时间应该小于 500ms
            assert!(duration < Duration::from_millis(500), "函数调用性能不达标: {:?}", duration);
        }
        Err(error) => {
            panic!("函数调用测试失败: {:?}", error);
        }
    }
}

/// 测试循环性能
fn test_loop_performance() {
    println!("\n--- 循环性能测试 ---");

    let mut runtime = create_runtime();

    // 嵌套循环
    let script = r#"
        let sum = 0;
        for (let i = 0; i < 100; i++) {
            for (let j = 0; j < 100; j++) {
                sum = sum + i * j;
            }
        }
        sum
    "#;

    let start = Instant::now();
    match runtime.execute_script(script) {
        Ok(result) => {
            let duration = start.elapsed();
            println!("循环测试完成: {:?}", duration);
            println!("结果: {:?}", result);

            // 验证性能目标：执行时间应该小于 200ms
            assert!(duration < Duration::from_millis(200), "循环性能不达标: {:?}", duration);
        }
        Err(error) => {
            panic!("循环测试失败: {:?}", error);
        }
    }
}

/// 测试对象操作性能
fn test_object_performance() {
    println!("\n--- 对象操作性能测试 ---");

    let mut runtime = create_runtime();

    // 对象创建和属性访问
    let script = r#"
        let obj = {};
        for (let i = 0; i < 1000; i++) {
            obj["key" + i] = i * 2;
        }
        let sum = 0;
        for (let i = 0; i < 1000; i++) {
            sum = sum + obj["key" + i];
        }
        sum
    "#;

    let start = Instant::now();
    match runtime.execute_script(script) {
        Ok(result) => {
            let duration = start.elapsed();
            println!("对象操作测试完成: {:?}", duration);
            println!("结果: {:?}", result);

            // 验证性能目标：执行时间应该小于 300ms
            assert!(duration < Duration::from_millis(300), "对象操作性能不达标: {:?}", duration);
        }
        Err(error) => {
            panic!("对象操作测试失败: {:?}", error);
        }
    }
}

/// 测试数组操作性能
fn test_array_performance() {
    println!("\n--- 数组操作性能测试 ---");

    let mut runtime = create_runtime();

    // 数组操作
    let script = r#"
        let arr = [];
        for (let i = 0; i < 1000; i++) {
            arr.push(i);
        }
        let sum = 0;
        for (let i = 0; i < arr.length; i++) {
            sum = sum + arr[i];
        }
        sum
    "#;

    let start = Instant::now();
    match runtime.execute_script(script) {
        Ok(result) => {
            let duration = start.elapsed();
            println!("数组操作测试完成: {:?}", duration);
            println!("结果: {:?}", result);

            // 验证性能目标：执行时间应该小于 250ms
            assert!(duration < Duration::from_millis(250), "数组操作性能不达标: {:?}", duration);
        }
        Err(error) => {
            panic!("数组操作测试失败: {:?}", error);
        }
    }
}

/// 测试内存管理性能
fn test_memory_performance() {
    println!("\n--- 内存管理性能测试 ---");

    let mut runtime = create_runtime();

    // 大量对象创建和销毁
    let script = r#"
        for (let i = 0; i < 1000; i++) {
            let obj = { value: i, data: "test" + i };
            let arr = [1, 2, 3, 4, 5];
        }
        "done"
    "#;

    let start = Instant::now();
    match runtime.execute_script(script) {
        Ok(result) => {
            let duration = start.elapsed();
            println!("内存管理测试完成: {:?}", duration);
            println!("结果: {:?}", result);

            // 验证性能目标：执行时间应该小于 200ms
            assert!(duration < Duration::from_millis(200), "内存管理性能不达标: {:?}", duration);
        }
        Err(error) => {
            panic!("内存管理测试失败: {:?}", error);
        }
    }
}

/// 性能对比测试
#[test]
fn test_performance_comparison() {
    println!("=== 性能对比测试 ===\n");

    let mut runtime = create_runtime();

    // 测试用例：计算斐波那契数列
    let script = r#"
        function fibonacci(n) {
            if (n <= 1) return n;
            return fibonacci(n - 1) + fibonacci(n - 2);
        }
        fibonacci(15)
    "#;

    // 预热
    for _ in 0..5 {
        let _ = runtime.execute_script(script);
    }

    // 正式测试
    let iterations = 10;
    let mut total_duration = Duration::new(0, 0);

    for _ in 0..iterations {
        let start = Instant::now();
        let _ = runtime.execute_script(script);
        total_duration += start.elapsed();
    }

    let avg_duration = total_duration / iterations;
    println!("平均执行时间: {:?}", avg_duration);

    // 验证性能目标：平均执行时间应该小于 50ms
    assert!(avg_duration < Duration::from_millis(50), "性能对比测试不达标: {:?}", avg_duration);
}
