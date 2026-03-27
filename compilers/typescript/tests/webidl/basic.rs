use typescript::TypeScript;

#[test]
fn test_webidl_processing() {
    let mut ts = TypeScript::new();

    // 测试 WebIDL 处理
    let webidl = r#"
        interface Person {
            attribute string name;
            attribute unsigned long age;
            void sayHello();
        };
    "#;

    let result = ts.process_webidl(webidl);
    assert!(result.is_ok(), "WebIDL processing failed: {:?}", result);

    let typescript_defs = ts.get_webidl_typescript_definitions();
    println!("Generated TypeScript definitions:");
    println!("{}", typescript_defs);

    // 验证生成的 TypeScript 定义包含预期内容
    assert!(typescript_defs.contains("interface Person"));
    assert!(typescript_defs.contains("name: string"));
    assert!(typescript_defs.contains("age: number"));
    assert!(typescript_defs.contains("sayHello(): void"));
}

#[test]
fn test_webidl_integration() {
    let mut ts = TypeScript::new();

    // 测试包含 WebIDL 的脚本执行
    let script = r#"
// WebIDL
interface Calculator {
    long add(long a, long b);
    long subtract(long a, long b);
};

// TypeScript
class CalculatorImpl implements Calculator {
    add(a: number, b: number): number {
        return a + b;
    }
    subtract(a: number, b: number): number {
        return a - b;
    }
}

const calc = new CalculatorImpl();
calc.add(5, 3) + calc.subtract(10, 4);
"#;

    let result = ts.execute_script(script);
    assert!(result.is_ok(), "Script execution failed: {:?}", result);

    let value = result.unwrap();
    println!("Script execution result: {}", value);

    // 验证计算结果: 5+3=8, 10-4=6, 8+6=14
    assert_eq!(value.to_string(), "14");
}
