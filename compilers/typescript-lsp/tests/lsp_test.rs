use typescript_lsp::TypeScriptLanguageService;

#[test]
fn test_extract_symbol_at_position() {
    let service = TypeScriptLanguageService::new();

    let text = "const x: number = 1;";
    let offset = 6; // Position of 'x'
    let symbol = service.extract_symbol_at_position(text, offset);
    assert_eq!(symbol, Some("x".to_string()));
}

#[test]
fn test_get_offset() {
    let service = TypeScriptLanguageService::new();

    let text = "line 1\nline 2\nline 3";
    let offset = service.get_offset(text, 1, 2); // Line 2, column 2
    assert_eq!(offset, 8); // "line 1\nli" -> 8 characters
}

#[test]
fn test_get_line_and_column() {
    let service = TypeScriptLanguageService::new();

    let text = "line 1\nline 2\nline 3";
    let (line, column) = service.get_line_and_column(text, 8); // Offset 8
    assert_eq!(line, 1); // Line 2 (0-indexed)
    assert_eq!(column, 2); // Column 2 (0-indexed)
}

#[test]
fn test_is_complete_symbol() {
    let service = TypeScriptLanguageService::new();

    let line = "const x = 1;";
    assert!(service.is_complete_symbol(line, 6, 7)); // 'x' at positions 6-7

    let line = "const xy = 1;";
    assert!(!service.is_complete_symbol(line, 6, 7)); // 'x' at positions 6-7 (part of 'xy')
}

#[test]
fn test_infer_variable_type() {
    let service = TypeScriptLanguageService::new();

    assert_eq!(service.infer_variable_type("\"test\""), "string".to_string());
    assert_eq!(service.infer_variable_type("true"), "boolean".to_string());
    assert_eq!(service.infer_variable_type("42"), "number".to_string());
    assert_eq!(service.infer_variable_type("{}"), "object".to_string());
    assert_eq!(service.infer_variable_type("[]"), "any[]".to_string());
    assert_eq!(service.infer_variable_type("null"), "null".to_string());
    assert_eq!(service.infer_variable_type("undefined"), "undefined".to_string());
    assert_eq!(service.infer_variable_type("() => {}"), "Function".to_string());
    assert_eq!(service.infer_variable_type("someValue"), "any".to_string());
}

#[test]
fn test_service_creation() {
    let _service = TypeScriptLanguageService::new();
    // Just verify it can be created without panicking
    assert!(true);
}

#[test]
fn test_parse_format_options() {
    let service = TypeScriptLanguageService::new();

    let options = service.parse_format_options(2, true);
    assert_eq!(options.indent_size, 2);
    assert!(!options.use_tabs);
}
