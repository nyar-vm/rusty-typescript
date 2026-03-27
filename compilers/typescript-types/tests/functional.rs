//! 类型系统功能测试
//!
//! 测试类型系统的核心功能，验证各种类型操作的正确性。

use std::collections::HashMap;
use typescript_types::{Conditional, Mapped, TemplateLiteral, TsValue};

/// 测试基本类型操作
#[test]
fn test_basic_type_operations() {
    let bool_type = TsValue::Boolean(true);
    let number_type = TsValue::Number(42.0);
    let string_type = TsValue::String("test".to_string());
    let null_type = TsValue::Null;
    let undefined_type = TsValue::Undefined;
    let any_type = TsValue::Any;
    let unknown_type = TsValue::Unknown;
    let never_type = TsValue::Never;
    let void_type = TsValue::Void;

    assert!(bool_type.is_boolean());
    assert!(number_type.is_number());
    assert!(string_type.is_string());
    assert!(null_type.is_null());
    assert!(undefined_type.is_undefined());
    assert!(any_type.is_any());
    assert!(unknown_type.is_unknown());
    assert!(never_type.is_never());
    assert!(void_type.is_void());

    assert_eq!(bool_type.to_boolean(), true);
    assert_eq!(number_type.to_number(), 42.0);
    assert_eq!(string_type.to_string(), "test");
    assert_eq!(null_type.to_string(), "null");
    assert_eq!(undefined_type.to_string(), "undefined");
}

/// 测试复杂类型操作
#[test]
fn test_complex_type_operations() {
    let obj_type = TsValue::Object(
        vec![
            ("name".to_string(), TsValue::String("test".to_string())),
            ("age".to_string(), TsValue::Number(18.0)),
            ("active".to_string(), TsValue::Boolean(true)),
        ]
        .into_iter()
        .collect(),
    );

    let property_keys = obj_type.get_property_keys();
    assert!(property_keys.contains(&"name".to_string()));
    assert!(property_keys.contains(&"age".to_string()));
    assert!(property_keys.contains(&"active".to_string()));

    let name_type = obj_type.get_property_type("name");
    assert!(name_type.is_some());
    assert!(name_type.unwrap().is_string());

    let array_type = TsValue::Array(vec![TsValue::Number(1.0), TsValue::Number(2.0), TsValue::Number(3.0)]);
    assert!(array_type.is_array());

    let union_type = TsValue::Union(vec![TsValue::Number(1.0), TsValue::String("test".to_string()), TsValue::Boolean(true)]);
    assert!(union_type.is_union());

    let tuple_type = TsValue::Tuple(vec![TsValue::String("test".to_string()), TsValue::Number(42.0), TsValue::Boolean(true)]);
    assert!(tuple_type.is_tuple());
}

/// 测试类型赋值检查
#[test]
fn test_type_assignability() {
    let number_type = TsValue::Number(42.0);
    let same_number_type = TsValue::Number(42.0);
    let different_number_type = TsValue::Number(100.0);
    let string_type = TsValue::String("test".to_string());

    assert!(number_type.is_assignable_to(&same_number_type));
    assert!(number_type.is_assignable_to(&different_number_type));
    assert!(!number_type.is_assignable_to(&string_type));

    let any_type = TsValue::Any;
    let unknown_type = TsValue::Unknown;
    let never_type = TsValue::Never;

    assert!(any_type.is_assignable_to(&number_type));
    assert!(number_type.is_assignable_to(&unknown_type));
    assert!(never_type.is_assignable_to(&number_type));
    assert!(!number_type.is_assignable_to(&never_type));

    let nullable_string = TsValue::Nullable(Box::new(TsValue::String("test".to_string())));
    let null_type = TsValue::Null;
    let string_type = TsValue::String("test".to_string());

    assert!(nullable_string.is_assignable_to(&null_type));
    assert!(string_type.is_assignable_to(&nullable_string));
}

/// 测试条件类型
#[test]
fn test_conditional_types() {
    let check_type = TsValue::Number(42.0);
    let extends_type = TsValue::Number(0.0);
    let true_type = TsValue::String("true".to_string());
    let false_type = TsValue::String("false".to_string());

    let conditional = Conditional::new(check_type.clone(), extends_type.clone(), true_type.clone(), false_type.clone());

    let conditional_type = TsValue::Conditional(conditional);
    assert!(conditional_type.is_conditional());

    let result = conditional_type.evaluate_conditional(&check_type, &extends_type);
    assert!(result.is_some());
    assert!(result.unwrap());
}

/// 测试映射类型
#[test]
fn test_mapped_types() {
    let key_type =
        TsValue::KeyOf(Box::new(TsValue::Object(vec![("a".to_string(), TsValue::Number(1.0))].into_iter().collect())));

    let mapped = Mapped::new("K".to_string(), key_type.clone(), TsValue::String("test".to_string()));

    let mapped_type = TsValue::Mapped(mapped);
    assert!(mapped_type.is_mapped());

    let property_keys = mapped_type.get_property_keys();
    assert!(property_keys.contains(&"a".to_string()));
}

/// 测试模板字面量类型
#[test]
fn test_template_literal_types() {
    let mut template = TemplateLiteral::new();
    template.push_string("Hello ".to_string());
    template.push_type(TsValue::String("World".to_string()));
    template.push_string("!".to_string());

    let template_type = TsValue::TemplateLiteral(template);
    assert!(template_type.is_template_literal());

    let template_str = template_type.to_string();
    assert!(template_str.contains("Hello"));
    assert!(template_str.contains("World"));
    assert!(template_str.contains("!"));
}

/// 测试类型推断
#[test]
fn test_type_inference() {
    let number_type = TsValue::Number(42.0);
    let same_number_type = TsValue::Number(42.0);
    let result = number_type.infer_type_params(&same_number_type);
    assert!(result.success);
    assert!(result.inferred_types.is_empty());

    let generic_type = TsValue::Generic("T".to_string(), vec![TsValue::Number(1.0)]);
    let target_type = TsValue::Generic("T".to_string(), vec![TsValue::Number(2.0)]);
    let result = generic_type.infer_type_params(&target_type);
    assert!(result.success);
}

/// 测试类型参数替换
#[test]
fn test_type_param_substitution() {
    let generic_type = TsValue::Generic("T".to_string(), vec![TsValue::Number(1.0)]);
    let substitutions = std::collections::HashMap::from([("T".to_string(), TsValue::String("test".to_string()))]);
    let substituted = generic_type.substitute_type_params(&substitutions);
    assert!(substituted.is_string());

    let obj_type =
        TsValue::Object(vec![("value".to_string(), TsValue::Generic("T".to_string(), vec![]))].into_iter().collect());
    let substituted_obj = obj_type.substitute_type_params(&substitutions);
    if let TsValue::Object(props) = substituted_obj {
        let value_type = props.get("value").unwrap();
        assert!(value_type.is_string());
    }
    else {
        panic!("Object type substitution failed");
    }
}

/// 测试条件类型创建
#[test]
fn test_conditional_type() {
    let check_type = TsValue::String("hello".to_string());
    let extends_type = TsValue::String("hello".to_string());
    let true_type = TsValue::Boolean(true);
    let false_type = TsValue::Boolean(false);
    let cond = Conditional::new(check_type, extends_type, true_type, false_type);
    let ts_cond = TsValue::Conditional(cond);
    assert!(ts_cond.is_conditional());
}

/// 测试映射类型创建
#[test]
fn test_mapped_type() {
    let constraint = TsValue::KeyOf(Box::new(TsValue::Object(HashMap::new())));
    let value_type = TsValue::String("value".to_string());
    let mapped = Mapped::new("K".to_string(), constraint, value_type);
    let ts_mapped = TsValue::Mapped(mapped);
    assert!(ts_mapped.is_mapped());
}

/// 测试模板字面量创建
#[test]
fn test_template_literal() {
    let mut tpl = TemplateLiteral::new();
    tpl.push_string("hello".to_string());
    tpl.push_string_type();
    let ts_tpl = TsValue::TemplateLiteral(tpl);
    assert!(ts_tpl.is_template_literal());
}

/// 测试类型推断功能
#[test]
fn test_type_inference_basic() {
    let infer_type = TsValue::Infer { type_param: "T".to_string(), constraint: None };
    let target = TsValue::String("hello".to_string());
    let result = infer_type.infer_type_params(&target);
    assert!(result.success);
    assert_eq!(result.inferred_types.get("T"), Some(&TsValue::String("hello".to_string())));
}

/// 测试 keyof 解析
#[test]
fn test_keyof_resolution() {
    let mut props = HashMap::new();
    props.insert("name".to_string(), TsValue::String("".to_string()));
    props.insert("age".to_string(), TsValue::Number(0.0));
    let obj = TsValue::Object(props);
    let keyof = TsValue::KeyOf(Box::new(obj));
    let resolved = keyof.resolve_keyof();
    assert!(resolved.is_union());
}

/// 测试索引访问
#[test]
fn test_indexed_access() {
    let mut props = HashMap::new();
    props.insert("name".to_string(), TsValue::String("".to_string()));
    let obj = TsValue::Object(props);
    let indexed =
        TsValue::IndexedAccess { object_type: Box::new(obj), index_type: Box::new(TsValue::String("name".to_string())) };
    let resolved = indexed.resolve_indexed_access();
    assert!(resolved.is_string());
}
