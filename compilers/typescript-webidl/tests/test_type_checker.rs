//! WebIDL 类型检查器测试
//!
//! 测试 WebIDL 类型检查器的功能，验证类型检查的正确性。

use typescript_webidl::{parse, type_check};

/// 测试基本类型检查
#[test]
fn test_basic_type_checking() {
    let idl = r#"
        interface TestInterface {
            attribute string name;
            attribute long age;
            void doSomething();
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析接口应该成功");

    if let Ok(root) = result {
        let check_result = type_check(&root);
        assert!(check_result.is_ok(), "类型检查应该成功");
    }
}

/// 测试继承接口的类型检查
#[test]
fn test_inherited_interface_type_checking() {
    let idl = r#"
        interface BaseInterface {
            attribute string baseProperty;
        }
        
        interface DerivedInterface : BaseInterface {
            attribute string derivedProperty;
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析继承接口应该成功");

    if let Ok(root) = result {
        let check_result = type_check(&root);
        assert!(check_result.is_ok(), "继承接口的类型检查应该成功");
    }
}

/// 测试复杂类型的类型检查
#[test]
fn test_complex_type_checking() {
    let idl = r#"
        enum TestEnum {
            "value1",
            "value2"
        }
        
        dictionary TestDictionary {
            string name;
            TestEnum type;
        }
        
        interface TestInterface {
            attribute sequence<string> names;
            attribute Promise<TestDictionary> result;
            attribute (string or long) value;
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析复杂类型应该成功");

    if let Ok(root) = result {
        let check_result = type_check(&root);
        assert!(check_result.is_ok(), "复杂类型的类型检查应该成功");
    }
}

/// 测试类型错误检查
#[test]
fn test_type_error_checking() {
    let idl = r#"
        interface TestInterface {
            attribute non_existent_type invalidProperty;
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析带有错误类型的接口应该成功");

    if let Ok(root) = result {
        let check_result = type_check(&root);
        assert!(check_result.is_err(), "类型检查应该失败，因为使用了不存在的类型");
    }
}

/// 测试操作参数类型检查
#[test]
fn test_operation_param_type_checking() {
    let idl = r#"
        interface TestInterface {
            void doSomething(string param1, long param2);
            string getValue(long index);
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析带有操作参数的接口应该成功");

    if let Ok(root) = result {
        let check_result = type_check(&root);
        assert!(check_result.is_ok(), "操作参数的类型检查应该成功");
    }
}

/// 测试只读属性类型检查
#[test]
fn test_readonly_attribute_type_checking() {
    let idl = r#"
        interface TestInterface {
            readonly attribute string name;
            attribute long age;
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析带有只读属性的接口应该成功");

    if let Ok(root) = result {
        let check_result = type_check(&root);
        assert!(check_result.is_ok(), "只读属性的类型检查应该成功");
    }
}

/// 测试静态操作类型检查
#[test]
fn test_static_operation_type_checking() {
    let idl = r#"
        interface TestInterface {
            static void staticMethod();
            void instanceMethod();
        }
    "#;

    let result = parse(idl);
    assert!(result.is_ok(), "解析带有静态操作的接口应该成功");

    if let Ok(root) = result {
        let check_result = type_check(&root);
        assert!(check_result.is_ok(), "静态操作的类型检查应该成功");
    }
}
