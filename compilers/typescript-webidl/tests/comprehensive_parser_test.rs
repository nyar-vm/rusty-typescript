/// 全面的 WebIDL 解析测试
use typescript_webidl::parse;

#[test]
fn test_parse_interface_with_attributes() {
    // 测试带有属性的接口
    let idl = r#"
        interface TestInterface {
            attribute string name;
            attribute long age;
            attribute boolean active;
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析带有属性的接口应该成功");
}

#[test]
fn test_parse_interface_with_operations() {
    // 测试带有操作的接口
    let idl = r#"
        interface TestInterface {
            void doSomething();
            string getName();
            long add(long a, long b);
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析带有操作的接口应该成功");
}

#[test]
fn test_parse_struct() {
    // 测试结构体
    let idl = r#"
        dictionary TestStruct {
            string name;
            long age;
            boolean active;
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析结构体应该成功");
}

#[test]
fn test_parse_enum() {
    // 测试枚举
    let idl = r#"
        enum TestEnum {
            "value1",
            "value2",
            "value3"
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析枚举应该成功");
}

#[test]
fn test_parse_union_type() {
    // 测试联合类型
    let idl = r#"
        interface TestInterface {
            attribute (string or long) value;
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析联合类型应该成功");
}

#[test]
fn test_parse_array_type() {
    // 测试数组类型
    let idl = r#"
        interface TestInterface {
            attribute sequence<string> names;
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析数组类型应该成功");
}

#[test]
fn test_parse_generic_type() {
    // 测试泛型类型
    let idl = r#"
        interface TestInterface {
            attribute Promise<string> result;
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析泛型类型应该成功");
}

#[test]
fn test_parse_multiple_items() {
    // 测试多个项
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
}

#[test]
#[ignore]
fn test_parse_invalid_syntax() {
    // 测试无效语法
    let invalid_idl = r#"
        interface TestInterface {
            attribute string name
        }
    "#;

    let result = parse(invalid_idl);
    assert!(result.is_err(), "解析无效语法应该失败");
}

#[test]
fn test_parse_complex_webidl() {
    // 测试复杂的 WebIDL
    let complex_idl = r#"
        enum MediaType {
            "audio",
            "video",
            "image"
        }
        
        dictionary MediaOptions {
            MediaType type;
            boolean autoplay;
            sequence<string> sources;
        }
        
        interface MediaPlayer {
            attribute MediaOptions options;
            void play();
            void pause();
            Promise<void> load(string url);
            (string or Error) getError();
        }
    "#;

    let result = parse(complex_idl);
    assert!(result.is_ok(), "解析复杂 WebIDL 应该成功");
}

#[test]
fn test_parse_empty_interface() {
    // 测试空接口
    let idl = r#"
        interface EmptyInterface {
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析空接口应该成功");
}

#[test]
fn test_parse_empty_dictionary() {
    // 测试空结构体
    let idl = r#"
        dictionary EmptyDictionary {
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析空结构体应该成功");
}

#[test]
fn test_parse_empty_enum() {
    // 测试空枚举
    let idl = r#"
        enum EmptyEnum {
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析空枚举应该成功");
}

#[test]
fn test_parse_dictionary_with_optional_members() {
    // 测试带有可选属性的结构体
    let idl = r#"
        dictionary TestDictionary {
            string name;
            long? age;
            boolean? active;
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析带有可选属性的结构体应该成功");
}

#[test]
fn test_parse_dictionary_with_default_values() {
    // 测试带有默认值的属性
    let idl = r#"
        dictionary TestDictionary {
            string name = "default";
            long age = 18;
            boolean active = true;
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析带有默认值的属性应该成功");
}

#[test]
fn test_parse_interface_with_modifiers() {
    // 测试带有修饰符的操作
    let idl = r#"
        interface TestInterface {
            readonly attribute string name;
            static void staticMethod();
            string getValue();
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析带有修饰符的操作应该成功");
}

#[test]
fn test_parse_typedef() {
    // 测试类型别名
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
}

#[test]
fn test_parse_const() {
    // 测试常量
    let idl = r#"
        const long MAX_VALUE = 100;
        const string DEFAULT_NAME = "test";
        
        interface TestInterface {
            long getMaxValue();
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析常量应该成功");
}

#[test]
fn test_parse_nested_types() {
    // 测试嵌套的复杂类型
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
}
