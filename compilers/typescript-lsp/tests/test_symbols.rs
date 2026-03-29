use oak_core::Range;
use typescript_lsp::lsp::symbols::{ScopeType, Symbol, SymbolKind, SymbolTable};

#[test]
fn test_symbol_creation() {
    let symbol = Symbol::new("myVar".to_string(), SymbolKind::Variable, Range::from(0..10)).with_type("string".to_string());

    assert_eq!(symbol.name, "myVar");
    assert_eq!(symbol.kind, SymbolKind::Variable);
    assert_eq!(symbol.type_annotation, Some("string".to_string()));
}

#[test]
fn test_symbol_table_scope() {
    let mut table = SymbolTable::new();

    table.add_symbol(Symbol::new("globalVar".to_string(), SymbolKind::Variable, Range::from(0..10)));

    table.enter_scope(20, ScopeType::Function);
    table.add_symbol(Symbol::new("localVar".to_string(), SymbolKind::Variable, Range::from(25..35)));

    assert!(table.find_by_name("globalVar").is_some());
    assert!(table.find_by_name("localVar").is_some());

    table.exit_scope(50);

    assert!(table.find_by_name("localVar").is_none());
}
