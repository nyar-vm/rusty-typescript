/// WebIDL 到 TypeScript 转换测试
use typescript_webidl::{convert_to_typescript, parse};

#[test]
fn test_convert_interface() {
    // 测试转换接口
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
        let typescript = convert_to_typescript(&root);
        println!("转换结果: {}", typescript);

        assert!(!typescript.is_empty(), "转换结果应该非空");
        assert!(typescript.contains("interface TestInterface"), "转换结果应该包含接口定义");
        assert!(typescript.contains("name"), "转换结果应该包含属性 name");
        assert!(typescript.contains("age"), "转换结果应该包含属性 age");
        assert!(typescript.contains("doSomething()"), "转换结果应该包含方法 doSomething");
        assert!(typescript.contains("getName()"), "转换结果应该包含方法 getName");
    }
}

#[test]
fn test_convert_struct() {
    // 测试转换结构体
    let idl = r#"
        dictionary TestStruct {
            string name;
            long age;
            boolean active;
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析结构体应该成功");

    if let Ok(root) = result {
        let typescript = convert_to_typescript(&root);
        println!("转换结果: {}", typescript);

        assert!(!typescript.is_empty(), "转换结果应该非空");
        assert!(typescript.contains("interface TestStruct"), "转换结果应该包含结构体定义");
        assert!(typescript.contains("name: string"), "转换结果应该包含字段 name");
        assert!(typescript.contains("age: number"), "转换结果应该包含字段 age");
        assert!(typescript.contains("active: boolean"), "转换结果应该包含字段 active");
    }
}

#[test]
fn test_convert_enum() {
    // 测试转换枚举
    let idl = r#"
        enum TestEnum {
            "value1",
            "value2",
            "value3"
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析枚举应该成功");

    if let Ok(root) = result {
        let typescript = convert_to_typescript(&root);
        println!("转换结果: {}", typescript);

        assert!(!typescript.is_empty(), "转换结果应该非空");
        assert!(typescript.contains("enum TestEnum"), "转换结果应该包含枚举定义");
        assert!(typescript.contains("value1"), "转换结果应该包含枚举值 value1");
        assert!(typescript.contains("value2"), "转换结果应该包含枚举值 value2");
        assert!(typescript.contains("value3"), "转换结果应该包含枚举值 value3");
    }
}

#[test]
fn test_convert_multiple_items() {
    // 测试转换多个项
    let idl = r#"
        enum TestEnum {
            "value1",
            "value2"
        }
        
        dictionary TestStruct {
            string name;
            TestEnum type;
        }
        
        interface TestInterface {
            attribute TestStruct data;
            TestEnum getType();
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析多个项应该成功");

    if let Ok(root) = result {
        let typescript = convert_to_typescript(&root);
        println!("转换结果: {}", typescript);

        assert!(!typescript.is_empty(), "转换结果应该非空");
        assert!(typescript.contains("enum TestEnum"), "转换结果应该包含枚举定义");
        assert!(typescript.contains("interface TestStruct"), "转换结果应该包含结构体定义");
        assert!(typescript.contains("interface TestInterface"), "转换结果应该包含接口定义");
    }
}

#[test]
fn test_convert_complex_types() {
    // 测试转换复杂类型
    let idl = r#"
        interface TestInterface {
            attribute sequence<string> names;
            attribute Promise<string> result;
            attribute (string or long) value;
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析复杂类型应该成功");

    if let Ok(root) = result {
        let typescript = convert_to_typescript(&root);
        println!("转换结果: {}", typescript);

        assert!(!typescript.is_empty(), "转换结果应该非空");
        assert!(typescript.contains("names: Array<string>"), "转换结果应该包含序列类型");
        assert!(typescript.contains("result: Promise<string>"), "转换结果应该包含泛型类型");
        assert!(typescript.contains("value: (string | long)"), "转换结果应该包含联合类型");
    }
}

#[test]
fn test_convert_empty_interface() {
    // 测试转换空接口
    let idl = r#"
        interface EmptyInterface {
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析空接口应该成功");

    if let Ok(root) = result {
        let typescript = convert_to_typescript(&root);
        println!("转换结果: {}", typescript);

        assert!(!typescript.is_empty(), "转换结果应该非空");
        assert!(typescript.contains("interface EmptyInterface"), "转换结果应该包含空接口定义");
    }
}

#[test]
fn test_convert_empty_dictionary() {
    // 测试转换空结构体
    let idl = r#"
        dictionary EmptyDictionary {
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析空结构体应该成功");

    if let Ok(root) = result {
        let typescript = convert_to_typescript(&root);
        println!("转换结果: {}", typescript);

        assert!(!typescript.is_empty(), "转换结果应该非空");
        assert!(typescript.contains("interface EmptyDictionary"), "转换结果应该包含空结构体定义");
    }
}

#[test]
fn test_convert_empty_enum() {
    // 测试转换空枚举
    let idl = r#"
        enum EmptyEnum {
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析空枚举应该成功");

    if let Ok(root) = result {
        let typescript = convert_to_typescript(&root);
        println!("转换结果: {}", typescript);

        assert!(!typescript.is_empty(), "转换结果应该非空");
        assert!(typescript.contains("enum EmptyEnum"), "转换结果应该包含空枚举定义");
    }
}

#[test]
fn test_convert_dictionary_with_optional_members() {
    // 测试转换带有可选属性的结构体
    let idl = r#"
        dictionary TestDictionary {
            string name;
            long? age;
            boolean? active;
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析带有可选属性的结构体应该成功");

    if let Ok(root) = result {
        let typescript = convert_to_typescript(&root);
        println!("转换结果: {}", typescript);

        assert!(!typescript.is_empty(), "转换结果应该非空");
        assert!(typescript.contains("name: string"), "转换结果应该包含必填属性 name");
        assert!(typescript.contains("age?: number"), "转换结果应该包含可选属性 age");
        assert!(typescript.contains("active?: boolean"), "转换结果应该包含可选属性 active");
    }
}

#[test]
fn test_convert_dictionary_with_default_values() {
    // 测试转换带有默认值的属性
    let idl = r#"
        dictionary TestDictionary {
            string name = "default";
            long age = 18;
            boolean active = true;
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析带有默认值的属性应该成功");

    if let Ok(root) = result {
        let typescript = convert_to_typescript(&root);
        println!("转换结果: {}", typescript);

        assert!(!typescript.is_empty(), "转换结果应该非空");
        assert!(typescript.contains("name: string = \"default\""), "转换结果应该包含带有默认值的属性 name");
        assert!(typescript.contains("age: number = 18"), "转换结果应该包含带有默认值的属性 age");
        assert!(typescript.contains("active: boolean = true"), "转换结果应该包含带有默认值的属性 active");
    }
}

#[test]
fn test_convert_interface_with_modifiers() {
    // 测试转换带有修饰符的操作
    let idl = r#"
        interface TestInterface {
            readonly attribute string name;
            static void staticMethod();
            string getValue();
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析带有修饰符的操作应该成功");

    if let Ok(root) = result {
        let typescript = convert_to_typescript(&root);
        println!("转换结果: {}", typescript);

        assert!(!typescript.is_empty(), "转换结果应该非空");
        assert!(typescript.contains("readonly name: string"), "转换结果应该包含只读属性 name");
        assert!(typescript.contains("staticMethod(): void"), "转换结果应该包含静态方法 staticMethod");
        assert!(typescript.contains("getValue(): string"), "转换结果应该包含普通方法 getValue");
    }
}

#[test]
fn test_convert_typedef() {
    // 测试转换类型别名
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
        let typescript = convert_to_typescript(&root);
        println!("转换结果: {}", typescript);

        assert!(!typescript.is_empty(), "转换结果应该非空");
        assert!(typescript.contains("type UTF8String = string"), "转换结果应该包含类型别名 UTF8String");
        assert!(typescript.contains("type Int64 = number"), "转换结果应该包含类型别名 Int64");
        assert!(typescript.contains("name: UTF8String"), "转换结果应该使用类型别名 UTF8String");
        assert!(typescript.contains("getValue(): Int64"), "转换结果应该使用类型别名 Int64");
    }
}

#[test]
fn test_convert_const() {
    // 测试转换常量
    let idl = r#"
        const long MAX_VALUE = 100;
        const string DEFAULT_NAME = "test";
        
        interface TestInterface {
            long getMaxValue();
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析常量应该成功");

    if let Ok(root) = result {
        let typescript = convert_to_typescript(&root);
        println!("转换结果: {}", typescript);

        assert!(!typescript.is_empty(), "转换结果应该非空");
        assert!(typescript.contains("const MAX_VALUE: number = 100"), "转换结果应该包含常量 MAX_VALUE");
        assert!(typescript.contains("const DEFAULT_NAME: string = \"test\""), "转换结果应该包含常量 DEFAULT_NAME");
    }
}

#[test]
fn test_convert_nested_types() {
    // 测试转换嵌套的复杂类型
    let idl = r#"
        dictionary InnerDictionary {
            string innerName;
        }
        
        dictionary OuterDictionary {
            string outerName;
            InnerDictionary inner;
            sequence<InnerDictionary> innerList;
        }
        
        interface TestInterface {
            attribute OuterDictionary data;
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析嵌套的复杂类型应该成功");

    if let Ok(root) = result {
        let typescript = convert_to_typescript(&root);
        println!("转换结果: {}", typescript);

        assert!(!typescript.is_empty(), "转换结果应该非空");
        assert!(typescript.contains("interface InnerDictionary"), "转换结果应该包含内部结构体定义");
        assert!(typescript.contains("interface OuterDictionary"), "转换结果应该包含外部结构体定义");
        assert!(typescript.contains("inner: InnerDictionary"), "转换结果应该包含内部结构体类型的字段");
        assert!(typescript.contains("innerList: Array<InnerDictionary>"), "转换结果应该包含内部结构体序列类型的字段");
    }
}
