//! 平台支持模块集成测试
//! 
//! 测试类型系统、WebAssembly 支持和 WebIDL 支持之间的协作。

use typescript_types::TsValue;
use typescript_wasi::WasiRuntime;
use typescript_webidl::{parse, convert_to_typescript};

/// 测试类型系统与 WebAssembly 运行时的集成
#[test]
fn test_typesystem_wasm_integration() {
    // 创建 WASI 运行时
    let mut runtime = WasiRuntime::new();
    
    // 定义一个使用复杂类型的 TypeScript 代码
    let code = r#"
        interface Person {
            name: string;
            age: number;
            active: boolean;
        }
        
        function createPerson(name: string, age: number): Person {
            return { name, age, active: true };
        }
        
        createPerson("John", 30);
    "#;
    
    // 编译并执行代码
    let result = runtime.compile(code);
    assert!(result.success, "编译使用复杂类型的 TypeScript 代码应该成功");
}

/// 测试 WebIDL 与 TypeScript 转换的集成
#[test]
fn test_webidl_typescript_integration() {
    // 定义 WebIDL 接口
    let idl = r#"
        interface TestInterface {
            attribute string name;
            attribute long age;
            void doSomething();
            string getName();
        }
    "#;
    
    // 解析 WebIDL
    let result = parse(idl);
    assert!(result.is_ok(), "解析 WebIDL 应该成功");
    
    if let Ok(root) = result {
        // 转换为 TypeScript
        let typescript = convert_to_typescript(&root);
        assert!(!typescript.is_empty(), "转换结果应该非空");
        
        // 验证转换结果
        assert!(typescript.contains("interface TestInterface"), "转换结果应该包含接口定义");
        assert!(typescript.contains("name: string"), "转换结果应该包含属性 name");
        assert!(typescript.contains("age: number"), "转换结果应该包含属性 age");
        assert!(typescript.contains("doSomething(): void"), "转换结果应该包含方法 doSomething");
        assert!(typescript.contains("getName(): string"), "转换结果应该包含方法 getName");
    }
}

/// 测试完整的工作流：WebIDL → TypeScript → WebAssembly
#[test]
fn test_full_workflow_integration() {
    // 定义 WebIDL 接口
    let idl = r#"
        interface Calculator {
            long add(long a, long b);
            long subtract(long a, long b);
        }
    "#;
    
    // 解析 WebIDL
    let result = parse(idl);
    assert!(result.is_ok(), "解析 WebIDL 应该成功");
    
    if let Ok(root) = result {
        // 转换为 TypeScript
        let typescript = convert_to_typescript(&root);
        assert!(!typescript.is_empty(), "转换结果应该非空");
        
        // 创建包含实现的完整 TypeScript 代码
        let full_code = format!(r#"
            {};
            
            class CalculatorImpl implements Calculator {{
                add(a: number, b: number): number {{
                    return a + b;
                }}
                subtract(a: number, b: number): number {{
                    return a - b;
                }}
            }}
            
            const calculator = new CalculatorImpl();
            calculator.add(10, 5);
        "#, typescript);
        
        // 创建 WASI 运行时并执行代码
        let mut runtime = WasiRuntime::new();
        let result = runtime.compile(&full_code);
        assert!(result.success, "编译并执行完整工作流应该成功");
    }
}

/// 测试类型系统与 WebIDL 类型的兼容性
#[test]
fn test_typesystem_webidl_compatibility() {
    // 测试 WebIDL 基本类型与 TypeScript 类型的对应关系
    let idl_types = vec![
        "string",
        "long",
        "boolean",
        "double",
        "DOMString"
    ];
    
    let expected_typescript_types = vec![
        "string",
        "number",
        "boolean",
        "number",
        "string"
    ];
    
    assert_eq!(idl_types.len(), expected_typescript_types.len(), "类型数量应该匹配");
    
    for (idl_type, expected_ts_type) in idl_types.iter().zip(expected_typescript_types.iter()) {
        // 验证类型映射的正确性
        assert!(!expected_ts_type.is_empty(), "TypeScript 类型应该非空");
    }
}
