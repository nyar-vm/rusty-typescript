use typescript_lsp::lsp::inlay_hints::{InlayHint, InlayHintKind, InlayHintProvider}; use typescript_lsp::lsp::symbols::SymbolTable;

#[test]
fn test_inlay_hint_creation() {
    let hint = InlayHint::parameter_name(10, "name");
    assert_eq!(hint.position, 10);
    assert_eq!(hint.label, "name:");
    assert_eq!(hint.kind, InlayHintKind::ParameterName);

    let hint = InlayHint::type_inference(20, "string");
    assert_eq!(hint.label, ": string");
    assert_eq!(hint.kind, InlayHintKind::TypeInference);

    let hint = InlayHint::enum_value(30, "0");
    assert_eq!(hint.label, " = 0");
    assert_eq!(hint.kind, InlayHintKind::EnumMemberValue);
}

#[test]
fn test_extract_parameter_names() {
    let provider = InlayHintProvider::new(SymbolTable::new());
    let params = provider.extract_parameter_names("(name: string, age: number) => void");
    assert_eq!(params, vec!["name", "age"]);

    let params = provider.extract_parameter_names("() => void");
    assert!(params.is_empty());
}

#[test]
fn test_extract_variable_name() {
    let provider = InlayHintProvider::new(SymbolTable::new());

    assert_eq!(
        provider.extract_variable_name("const myVar = 123"),
        Some("myVar".to_string())
    );
    assert_eq!(
        provider.extract_variable_name("let count = 0"),
        Some("count".to_string())
    );
    assert_eq!(
        provider.extract_variable_name("var _private = true"),
        Some("_private".to_string())
    );
}
