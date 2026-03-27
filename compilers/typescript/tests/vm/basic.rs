//! 虚拟机测试模块
//!
//! 测试虚拟机的基本功能、内置函数和操作。

use typescript::vm::{Builtins, CallFrame, ModuleInstance, PerformanceMonitor, VM};
use typescript_types::TsValue;

/// 测试虚拟机基本功能
///
/// 验证虚拟机的基本初始化和状态
#[test]
fn test_vm_basic() {
    let vm = VM::new(vec![]);
    assert_eq!(vm.call_stack_depth(), 0);
}

/// 测试内置函数
///
/// 验证内置函数的正确性
#[test]
fn test_builtins() {
    let builtins = Builtins::new();

    // 测试 console.log
    if let TsValue::Function(log) = builtins.console.get("log").unwrap() {
        let result = log(&[TsValue::String("test".to_string())]);
        assert!(result.is_undefined());
    }

    // 测试 Math.abs
    if let TsValue::Function(abs) = builtins.math.get("abs").unwrap() {
        let result = abs(&[TsValue::Number(-42.0)]);
        assert!(result.is_number());
        assert_eq!(result.to_number(), 42.0);
    }

    // 测试 Math.PI
    if let TsValue::Number(pi) = builtins.math.get("PI").unwrap() {
        assert!((*pi - std::f64::consts::PI).abs() < f64::EPSILON);
    }
}

/// 测试调用帧
///
/// 验证调用帧的局部变量管理
#[test]
fn test_call_frame() {
    let mut frame = CallFrame::new("test".to_string(), 0, 0);
    frame.set_local("x", TsValue::Number(42.0));
    if let Some(value) = frame.get_local("x") {
        assert!(value.is_number());
        assert_eq!(value.to_number(), 42.0);
    }
    else {
        panic!("Expected local variable 'x' to exist");
    }
}

/// 测试模块实例
///
/// 验证模块实例的导出功能
#[test]
fn test_module_instance() {
    let mut module = ModuleInstance::new("test".to_string());
    module.export("value", TsValue::Number(42.0));
    if let Some(value) = module.get_export("value") {
        assert!(value.is_number());
        assert_eq!(value.to_number(), 42.0);
    }
    else {
        panic!("Expected export 'value' to exist");
    }
}

/// 测试性能监控
///
/// 验证性能监控器的功能
#[test]
fn test_performance_monitor() {
    let mut monitor = PerformanceMonitor::new();
    monitor.start();
    monitor.record_instruction();
    monitor.record_instruction();
    monitor.record_call(1);

    assert_eq!(monitor.instruction_count, 2);
    assert_eq!(monitor.call_count, 1);
    assert_eq!(monitor.max_stack_depth, 1);
    assert!(monitor.elapsed_us() > 0);
}

/// 测试二元操作
///
/// 验证虚拟机的二元操作功能
#[test]
fn test_binary_operations() {
    let vm = VM::new(vec![]);

    // 测试加法
    let result = vm.binary_add(TsValue::Number(1.0), TsValue::Number(2.0)).unwrap();
    assert!(result.is_number());
    assert_eq!(result.to_number(), 3.0);

    // 测试减法
    let result = vm.binary_sub(TsValue::Number(5.0), TsValue::Number(3.0)).unwrap();
    assert!(result.is_number());
    assert_eq!(result.to_number(), 2.0);

    // 测试严格等于
    let result = vm.binary_strict_eq(TsValue::Number(1.0), TsValue::Number(1.0));
    assert!(result.is_boolean());
    assert_eq!(result.to_boolean(), true);

    let result = vm.binary_strict_eq(TsValue::Number(1.0), TsValue::String("1".to_string()));
    assert!(result.is_boolean());
    assert_eq!(result.to_boolean(), false);
}
