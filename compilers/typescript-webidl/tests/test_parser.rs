//! 全面的 WebIDL 解析器测试
//!
//! 测试 WebIDL 解析器对各种语法结构的支持。

use typescript_webidl::parse;

#[test]
fn test_parse_interface_with_attributes() {
    let idl = r#"
        interface TestInterface {
            attribute string name;
            attribute long age;
            readonly attribute boolean active;
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析带属性的接口应该成功");
}

#[test]
fn test_parse_interface_with_operations() {
    let idl = r#"
        interface TestInterface {
            void doSomething();
            long calculate(long a, long b);
            string getName();
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析带操作的接口应该成功");
}

#[test]
fn test_parse_interface_with_inheritance() {
    let idl = r#"
        interface BaseInterface {
            attribute string baseProperty;
        }
        
        interface DerivedInterface : BaseInterface {
            attribute string derivedProperty;
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析带继承的接口应该成功");
}

#[test]
fn test_parse_multiple_interfaces() {
    let idl = r#"
        interface Interface1 {
            attribute string prop1;
        }
        
        interface Interface2 {
            attribute string prop2;
        }
        
        interface Interface3 {
            attribute string prop3;
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析多个接口应该成功");
}

#[test]
fn test_parse_complex_idl() {
    let idl = r#"
        interface ComplexInterface {
            // 属性
            attribute string name;
            attribute long age;
            readonly attribute boolean active;
            
            // 操作
            void initialize();
            long calculate(long a, long b);
            string getName();
            boolean isActive();
        }
        
        interface AnotherInterface {
            attribute DOMString url;
            void load(URLRequest request);
            Promise<Response> fetch(RequestInfo input);
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析复杂的 IDL 应该成功");
}
