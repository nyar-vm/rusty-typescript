use typescript_lsp::lsp::diagnostics::syntax_checker::{
    check_bracket_matching, check_keyword_usage, check_quote_matching, check_switch_statements, check_try_catch_statements,
    check_unused_imports,
};

#[test]
fn test_bracket_matching() {
    let content = "function test() { return [1, 2, 3]; }";
    let diagnostics = check_bracket_matching(content);
    assert!(diagnostics.is_empty());

    let content = "function test() { return [1, 2, 3); }";
    let diagnostics = check_bracket_matching(content);
    assert!(!diagnostics.is_empty());
}

#[test]
fn test_unclosed_bracket() {
    let content = "function test() { return [1, 2, 3; }";
    let diagnostics = check_bracket_matching(content);
    assert!(!diagnostics.is_empty());
}

#[test]
fn test_quote_matching() {
    let content = r#"const s = "hello world";"#;
    let diagnostics = check_quote_matching(content);
    assert!(diagnostics.is_empty());

    let content = r#"const s = "hello world';"#;
    let diagnostics = check_quote_matching(content);
    assert!(!diagnostics.is_empty());
}

#[test]
fn test_keyword_usage() {
    let content = "class { }";
    let diagnostics = check_keyword_usage(content);
    assert!(!diagnostics.is_empty());

    let content = "class MyClass { }";
    let diagnostics = check_keyword_usage(content);
    assert!(diagnostics.is_empty());
}

#[test]
fn test_switch_statements() {
    let content = "switch (x) { case 1: break; default: break; }";
    let diagnostics = check_switch_statements(content);
    assert!(diagnostics.is_empty());

    let content = "switch (x) { }";
    let diagnostics = check_switch_statements(content);
    assert!(!diagnostics.is_empty());
}

#[test]
fn test_try_catch_statements() {
    let content = "try { } catch (e) { }";
    let diagnostics = check_try_catch_statements(content);
    assert!(diagnostics.is_empty());

    let content = "try { }";
    let diagnostics = check_try_catch_statements(content);
    assert!(!diagnostics.is_empty());
}

#[test]
fn test_unused_imports() {
    let content = "import { unused } from 'module';\nconst used = 1;";
    let diagnostics = check_unused_imports(content);
    assert!(!diagnostics.is_empty());

    let content = "import { used } from 'module';\nconst x = used;";
    let diagnostics = check_unused_imports(content);
    assert!(diagnostics.is_empty());
}
