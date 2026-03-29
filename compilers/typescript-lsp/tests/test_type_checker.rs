use typescript_lsp::lsp::diagnostics::type_checker::{
    extract_argument_count, extract_param_count, is_nested_scope, is_valid_identifier, is_valid_type_annotation,
    types_compatible,
};

#[test]
fn test_valid_type_annotation() {
    assert!(is_valid_type_annotation("string"));
    assert!(is_valid_type_annotation("number"));
    assert!(is_valid_type_annotation("boolean"));
    assert!(is_valid_type_annotation("any"));
    assert!(is_valid_type_annotation("string[]"));
    assert!(is_valid_type_annotation("string | number"));
    assert!(is_valid_type_annotation("MyClass"));
}

#[test]
fn test_types_compatible() {
    assert!(types_compatible("string", "string"));
    assert!(types_compatible("any", "string"));
    assert!(types_compatible("string", "any"));
    assert!(types_compatible("string | number", "string"));
    assert!(!types_compatible("string", "number"));
}

#[test]
fn test_valid_identifier() {
    assert!(is_valid_identifier("myVar"));
    assert!(is_valid_identifier("_private"));
    assert!(is_valid_identifier("$jquery"));
    assert!(is_valid_identifier("MyClass123"));
    assert!(!is_valid_identifier("123var"));
    assert!(!is_valid_identifier(""));
}

#[test]
fn test_extract_argument_count() {
    assert_eq!(extract_argument_count("()"), Some(0));
    assert_eq!(extract_argument_count("(a)"), Some(1));
    assert_eq!(extract_argument_count("(a, b)"), Some(2));
    assert_eq!(extract_argument_count("(a, b, c)"), Some(3));
}

#[test]
fn test_extract_param_count() {
    let sig1 = Some("(a: string, b: number) => void".to_string());
    assert_eq!(extract_param_count(&sig1), Some(2));

    let sig2 = Some("() => void".to_string());
    assert_eq!(extract_param_count(&sig2), Some(0));

    let sig3 = Some("(x: any) => any".to_string());
    assert_eq!(extract_param_count(&sig3), Some(1));
}

#[test]
fn test_is_nested_scope() {
    assert!(is_nested_scope(5, 0, 10));
    assert!(!is_nested_scope(15, 0, 10));
    assert!(!is_nested_scope(-1, 0, 10));
}
