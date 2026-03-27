use std::time::Instant;
use typescript_macros::TypescriptClass;

// 定义一个复杂的结构体来测试宏的性能
#[derive(TypescriptClass)]
struct ComplexStruct {
    // 基本类型
    id: u32,
    name: String,
    active: bool,
    // 集合类型
    items: Vec<String>,
    numbers: Vec<u32>,
    unique_ids: std::collections::HashSet<u32>,
    // 映射类型
    user_map: std::collections::HashMap<String, u32>,
    // 元组类型
    coordinates: (f64, f64, f64),
    // 可选类型
    optional_field: Option<String>,
    // 嵌套类型
    nested: Vec<Option<std::collections::HashMap<String, Vec<u32>>>>,
}

#[test]
fn benchmark_macro_expansion() {
    let iterations = 1000;
    let start = Instant::now();

    for _ in 0..iterations {
        // 访问生成的常量来触发宏展开
        let _ = ComplexStruct::TS_CLASS_DEFINITION;
    }

    let duration = start.elapsed().as_millis();
    println!("Macro expansion took {}ms for {} iterations", duration, iterations);

    // 验证性能是否在合理范围内
    assert!(duration < 500, "Performance test failed: total time {}ms exceeds 500ms", duration);
}
