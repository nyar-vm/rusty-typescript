use typescript_macros::{TypescriptClass, TypescriptFunction};

// 测试 TypescriptClass 宏
#[derive(TypescriptClass)]
struct User {
    id: u32,
    name: String,
    active: bool,
}

// 测试 TypescriptFunction 宏
#[TypescriptFunction]
fn add(a: u32, b: u32) -> u32 {
    a + b
}

#[TypescriptFunction]
fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[test]
fn test_typescript_class() {
    // 验证生成的 TypeScript 类定义
    let class_def = User::TS_CLASS_DEFINITION;
    assert!(class_def.contains("class User"));
    assert!(class_def.contains("id: number"));
    assert!(class_def.contains("name: string"));
    assert!(class_def.contains("active: boolean"));
    assert!(class_def.contains("constructor"));
}

#[test]
fn test_typescript_function() {
    // 打印出生成的常量内容
    println!("add_TS_FUNCTION_DEFINITION: {}", add_TS_FUNCTION_DEFINITION);
    println!("greet_TS_FUNCTION_DEFINITION: {}", greet_TS_FUNCTION_DEFINITION);

    // 验证生成的 TypeScript 函数类型定义
    assert!(add_TS_FUNCTION_DEFINITION.contains("type addFunction"));
    assert!(add_TS_FUNCTION_DEFINITION.contains("a: number"));
    assert!(add_TS_FUNCTION_DEFINITION.contains("b: number"));
    assert!(add_TS_FUNCTION_DEFINITION.contains("=> number"));

    assert!(greet_TS_FUNCTION_DEFINITION.contains("type greetFunction"));
    assert!(greet_TS_FUNCTION_DEFINITION.contains("name: string"));
    assert!(greet_TS_FUNCTION_DEFINITION.contains("=> string"));
}
