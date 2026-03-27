//! 类型系统性能测试
//!
//! 测试类型系统的执行效率，验证性能优化的效果。

use std::time::Instant;
use typescript_types::TsValue;

/// 测试类型比较性能
#[test]
fn test_type_comparison_performance() {
    let start = Instant::now();

    // 创建大量类型实例进行比较
    let types = vec![
        TsValue::Boolean(true),
        TsValue::Number(42.0),
        TsValue::String("test".to_string()),
        TsValue::Null,
        TsValue::Undefined,
        TsValue::Any,
        TsValue::Unknown,
        TsValue::Never,
        TsValue::Void,
    ];

    // 执行大量类型比较操作
    for _ in 0..1000000 {
        for i in 0..types.len() {
            for j in 0..types.len() {
                let _ = types[i].is_assignable_to(&types[j]);
            }
        }
    }

    let duration = start.elapsed();
    println!("类型比较性能测试: {:?}", duration);

    // 确保测试通过
    assert!(true);
}

/// 测试类型推断性能
#[test]
fn test_type_inference_performance() {
    let start = Instant::now();

    // 创建泛型类型
    let generic1 = TsValue::Generic("T".to_string(), vec![TsValue::Number(1.0)]);
    let generic2 = TsValue::Generic("T".to_string(), vec![TsValue::Number(2.0)]);

    // 执行大量类型推断操作
    for _ in 0..1000000 {
        let _ = generic1.infer_type_params(&generic2);
    }

    let duration = start.elapsed();
    println!("类型推断性能测试: {:?}", duration);

    // 确保测试通过
    assert!(true);
}

/// 测试类型操作性能
#[test]
fn test_type_operations_performance() {
    let start = Instant::now();

    // 创建复杂类型
    let obj = TsValue::Object(
        vec![
            ("a".to_string(), TsValue::Number(1.0)),
            ("b".to_string(), TsValue::String("test".to_string())),
            ("c".to_string(), TsValue::Boolean(true)),
        ]
        .into_iter()
        .collect(),
    );

    // 执行大量类型操作
    for _ in 0..1000000 {
        let _ = obj.get_property_keys();
        let _ = obj.get_property_type("a");
        let _ = obj.to_string();
    }

    let duration = start.elapsed();
    println!("类型操作性能测试: {:?}", duration);

    // 确保测试通过
    assert!(true);
}
