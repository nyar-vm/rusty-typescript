//! 性能监控测试模块
//!
//! 测试性能监控器、热点检测和性能阈值功能。

use std::{thread, time::Duration};
use typescript_ir::performance::{PerformanceMonitor, PerformanceThreshold, Profiler};

/// 测试性能监控器
///
/// 验证性能监控器能够正确记录和报告性能数据
#[test]
fn test_performance_monitor() {
    let mut monitor = PerformanceMonitor::new();
    monitor.start();

    // 模拟一些操作
    for _ in 0..100 {
        monitor.record_instruction();
    }

    monitor.record_function_start("test_func");
    thread::sleep(Duration::from_millis(1));
    monitor.record_function_end("test_func", Duration::from_millis(1));

    monitor.record_allocation(1024);
    monitor.record_gc();

    let report = monitor.generate_report();
    assert!(report.instruction_count >= 100);
    assert_eq!(report.function_call_count, 1);
    assert_eq!(report.allocation_count, 1);
    assert_eq!(report.gc_count, 1);
}

/// 测试热点检测
///
/// 验证热点检测功能能够正确识别和排序热点
#[test]
fn test_hot_spots() {
    let mut monitor = PerformanceMonitor::new();
    monitor.start();

    // 记录热点
    for _ in 0..10 {
        monitor.record_hot_spot("loop_body", Duration::from_micros(100));
    }
    for _ in 0..5 {
        monitor.record_hot_spot("condition_check", Duration::from_micros(50));
    }

    let report = monitor.generate_report();
    assert_eq!(report.hot_spots.len(), 2);
    assert_eq!(report.hot_spots[0].location, "loop_body");
}

/// 测试函数统计
///
/// 验证函数统计功能能够正确记录和分析函数调用
#[test]
fn test_function_stats() {
    let mut monitor = PerformanceMonitor::new();
    monitor.start();

    // 记录函数调用
    monitor.record_function_start("func_a");
    monitor.record_function_end("func_a", Duration::from_millis(10));

    monitor.record_function_start("func_a");
    monitor.record_function_end("func_a", Duration::from_millis(20));

    monitor.record_function_start("func_b");
    monitor.record_function_end("func_b", Duration::from_millis(5));

    let report = monitor.generate_report();
    assert_eq!(report.slowest_functions.len(), 2);

    let func_a = report.slowest_functions.iter().find(|f| f.name == "func_a").unwrap();
    assert_eq!(func_a.call_count, 2);
    assert_eq!(func_a.total_time, Duration::from_millis(30));
}

/// 测试性能阈值
///
/// 验证性能阈值检查功能能够正确检测超出阈值的情况
#[test]
fn test_threshold() {
    let threshold = PerformanceThreshold::strict();
    let mut monitor = PerformanceMonitor::new();
    monitor.start();

    // 执行超过阈值的操作
    for _ in 0..(threshold.max_instructions + 1) {
        monitor.record_instruction();
    }

    assert!(monitor.is_threshold_exceeded(&threshold));
}

/// 测试性能分析器
///
/// 验证性能分析器能够正确记录函数调用和指令执行
#[test]
fn test_profiler() {
    let mut profiler = Profiler::new();
    profiler.start();

    profiler.enter_function("outer");
    for _ in 0..10 {
        profiler.record_instruction();
    }
    profiler.enter_function("inner");
    for _ in 0..5 {
        profiler.record_instruction();
    }
    profiler.exit_function();
    profiler.exit_function();

    let report = profiler.report();
    assert_eq!(report.function_call_count, 2);
    assert!(report.instruction_count >= 15);
}
