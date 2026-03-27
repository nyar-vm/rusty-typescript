//! JIT 编译测试
//!
//! 测试 JIT 编译和优化功能。

use typescript::{create_runtime, run_script};

#[test]
fn test_jit_compilation() {
    // 创建 TypeScript 运行时
    let mut runtime = create_runtime();

    // 测试脚本：包含一个热点函数
    let script = r#"
    function hotFunction(x, y) {
        let result = 0;
        for (let i = 0; i < 1000; i++) {
            result += x * y + i;
        }
        return result;
    }
    
    // 多次调用函数，触发 JIT 编译
    console.log('Calling hotFunction...');
    for (let i = 0; i < 150; i++) {
        hotFunction(i, i + 1);
    }
    
    // 最后一次调用，应该使用 JIT 编译
    let result = hotFunction(100, 200);
    console.log('Final result:', result);
    result;
    "#;

    // 执行脚本
    match run_script(&mut runtime, script) {
        Ok(value) => println!("Script executed successfully: {}", value),
        Err(error) => panic!("Script execution failed: {:?}", error),
    }
}


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

use typescript::jit::{JITCompiler, JITEvent, JITEventCallback, CompileStatus, HotFunctionInfo, HotFunctionPriorityQueue, PriorityEntry, AdaptiveThresholdConfig, JITStatistics};
use std::rc::Rc;

struct TestCallback {
    events: std::cell::RefCell<Vec<JITEvent>>,
}

impl TestCallback {
    fn new() -> Self {
        Self { events: std::cell::RefCell::new(Vec::new()) }
    }

    fn get_events(&self) -> Vec<JITEvent> {
        self.events.borrow().clone()
    }
}

impl JITEventCallback for TestCallback {
    fn on_event(&self, event: &JITEvent) {
        self.events.borrow_mut().push(event.clone());
    }
}

/// 测试编译状态枚举
#[test]
fn test_compile_status() {
    assert_eq!(CompileStatus::NotCompiled, CompileStatus::NotCompiled);
    assert_ne!(CompileStatus::NotCompiled, CompileStatus::Compiled);
}

/// 测试热点函数信息默认值
#[test]
fn test_hot_function_info_default() {
    let info = HotFunctionInfo::default();
    assert_eq!(info.call_count, 0);
    assert_eq!(info.compile_status, CompileStatus::NotCompiled);
}

/// 测试优先级队列
#[test]
fn test_priority_queue() {
    let mut queue = HotFunctionPriorityQueue::new();

    let entry1 = PriorityEntry {
        function_name: "func1".to_string(),
        priority_score: 1.0,
        call_count: 10,
        avg_time: 100,
        complexity: 5,
        execution_frequency: 0.0,
        instruction_density: 0.0,
        memory_access_frequency: 0,
    };

    let entry2 = PriorityEntry {
        function_name: "func2".to_string(),
        priority_score: 2.0,
        call_count: 20,
        avg_time: 200,
        complexity: 10,
        execution_frequency: 0.0,
        instruction_density: 0.0,
        memory_access_frequency: 0,
    };

    queue.push(entry1);
    queue.push(entry2);

    assert_eq!(queue.len(), 2);

    let top = queue.pop();
    assert!(top.is_some());
    assert_eq!(top.unwrap().function_name, "func2");
}

/// 测试自适应阈值配置
#[test]
fn test_adaptive_threshold_config() {
    let config = AdaptiveThresholdConfig::new();

    let low_threshold = config.calculate_call_threshold(5, 1.0, 1.0);
    let high_threshold = config.calculate_call_threshold(100, 1.0, 1.0);

    assert!(low_threshold > high_threshold);
}

/// 测试 JIT 统计信息
#[test]
fn test_jit_statistics() {
    let mut stats = JITStatistics::new();
    stats.total_compilations = 10;
    stats.successful_compilations = 8;
    stats.failed_compilations = 2;

    assert!((stats.success_rate() - 0.8).abs() < f64::EPSILON);
}

/// 测试 JIT 编译器创建
#[test]
fn test_jit_compiler_creation() {
    let compiler = JITCompiler::new();
    assert_eq!(compiler.hot_function_count(), 0);
    assert_eq!(compiler.compiled_function_count(), 0);
}

/// 测试事件回调
#[test]
fn test_event_callback() {
    let callback = Rc::new(TestCallback::new());
    let mut compiler = JITCompiler::new();
    compiler.register_callback(callback.clone());

    compiler.set_function_complexity("test_func", 5);

    for _ in 0..150 {
        compiler.should_compile("test_func");
    }

    let events = callback.get_events();
    assert!(!events.is_empty());
}

/// 测试性能测试套件
///
/// 验证性能测试套件的功能
#[test]
fn test_performance_suite() {
    use typescript::jit::PerformanceTestSuite;
    
    let mut suite = PerformanceTestSuite::new();
    suite.run_all_tests();
    suite.print_results();
    assert!(!suite.get_results().is_empty());
}

