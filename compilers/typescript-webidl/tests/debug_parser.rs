/// 调试解析器输出
use typescript_webidl::{convert_to_typescript, parse};

#[test]
fn debug_parse_interface() {
    // 测试解析接口
    let idl = r#"
        interface TestInterface {
            attribute string name;
            attribute long age;
            void doSomething();
            string getName();
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析接口应该成功");

    if let Ok(root) = result {
        println!("解析结果: {:?}", root);
        let typescript = convert_to_typescript(&root);
        println!("转换结果: {}", typescript);
    }
}

#[test]
fn debug_parse_interface_with_modifiers() {
    // 测试解析带有修饰符的接口
    let idl = r#"
        interface TestInterface {
            readonly attribute string name;
            static void staticMethod();
            string getValue();
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析接口应该成功");

    if let Ok(root) = result {
        println!("解析结果: {:?}", root);
        let typescript = convert_to_typescript(&root);
        println!("转换结果: {}", typescript);
    }
}

#[test]
fn debug_parse_typedef() {
    // 测试解析类型别名
    let idl = r#"
        typedef string UTF8String;
        typedef long long Int64;
        
        interface TestInterface {
            attribute UTF8String name;
            Int64 getValue();
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析类型别名应该成功");

    if let Ok(root) = result {
        println!("解析结果: {:?}", root);
        let typescript = convert_to_typescript(&root);
        println!("转换结果: {}", typescript);
    }
}
