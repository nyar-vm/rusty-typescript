#![feature(new_range_api)]

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

#[test]
fn test_formatting() {
    use typescript_lsp::lsp::formatter::{
        ArrowParens, FormatOptions, LineEnding, TrailingComma, format_code, format_code_with_options,
    };

    // 测试基本格式化
    let code = "const x=1;function foo(){return x;}";
    let formatted = format_code(code);
    assert!(formatted.contains("const x = 1;"));
    assert!(formatted.contains("function foo() {"));
    assert!(formatted.contains("return x;"));
    assert!(formatted.contains("}"));

    // 测试自定义格式化选项
    let mut options = FormatOptions::default();
    options.indent_size = 2;
    options.single_quote = true;
    options.trailing_comma = TrailingComma::All;
    options.arrow_parens = ArrowParens::Avoid;
    options.line_ending = LineEnding::CRLF;

    let code = "const x = \"test\"; const arr = [1,2,3]; const func = (x) => x;";
    let formatted = format_code_with_options(code, options);
    assert!(formatted.contains("const x = 'test';"));
    assert!(formatted.contains("const arr = [1, 2, 3,];"));
    assert!(formatted.contains("const func = x => x;"));

    // 测试 TypeScript 特有语法的格式化
    let code = "interface User{id:number;name:string;} class Person implements User{id:number;name:string;constructor(id:number,name:string){this.id=id;this.name=name;}}";
    let formatted = format_code(code);
    assert!(formatted.contains("interface User {"));
    assert!(formatted.contains("id: number;"));
    assert!(formatted.contains("name: string;"));
    assert!(formatted.contains("class Person implements User {"));
    assert!(formatted.contains("constructor(id: number, name: string) {"));
    assert!(formatted.contains("this.id = id;"));
    assert!(formatted.contains("this.name = name;"));
}

#[test]
fn test_range_formatting() {
    use core::range::Range;
    use typescript_lsp::lsp::formatter::format_range;

    let code = "const x=1; const y=2; const z=3;";
    let range = Range { start: 7, end: 15 };
    let formatted = format_range(code, range.start, range.end, Default::default());
    assert!(formatted.contains("const y = 2;"));
}

/// 测试诊断功能
///
/// 验证诊断信息的创建和操作
#[test]
fn test_diagnostics() {
    use core::range::Range;
    use typescript_lsp::lsp::diagnostics::{Diagnostic, DiagnosticLevel};

    // 测试诊断创建
    let diag = Diagnostic::error("测试错误", ((0usize)..(10usize)).into());
    assert_eq!(diag.level, DiagnosticLevel::Error);
    assert_eq!(diag.message, "测试错误");

    let diag = Diagnostic::warning("测试警告", ((10usize)..(20usize)).into());
    assert_eq!(diag.level, DiagnosticLevel::Warning);

    // 测试诊断添加错误代码
    let diag = Diagnostic::error("测试错误", ((0usize)..(10usize)).into()).with_code("TS1234");
    assert_eq!(diag.code, Some("TS1234".to_string()));
}
