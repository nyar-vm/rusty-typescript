use typescript_lsp::lsp::symbols::collector::{SymbolCollector, SymbolKind};

#[test]
fn test_collect_variable() {
    let mut collector = SymbolCollector::new();
    let source = "const myVar: string = 'hello';";
    collector.collect(source);

    let table = collector.symbol_table();
    let symbol = table.find_by_name("myVar");

    assert!(symbol.is_some());
    let symbol = symbol.unwrap();
    assert_eq!(symbol.name, "myVar");
    assert_eq!(symbol.kind, SymbolKind::Variable);
}

#[test]
fn test_collect_function() {
    let mut collector = SymbolCollector::new();
    let source = "function greet(name: string): string { return name; }";
    collector.collect(source);

    let table = collector.symbol_table();
    let symbol = table.find_by_name("greet");

    assert!(symbol.is_some());
    let symbol = symbol.unwrap();
    assert_eq!(symbol.name, "greet");
    assert_eq!(symbol.kind, SymbolKind::Function);
}

#[test]
fn test_collect_class() {
    let mut collector = SymbolCollector::new();
    let source = "class MyClass { private x: number; }";
    collector.collect(source);

    let table = collector.symbol_table();
    let symbol = table.find_by_name("MyClass");

    assert!(symbol.is_some());
    let symbol = symbol.unwrap();
    assert_eq!(symbol.name, "MyClass");
    assert_eq!(symbol.kind, SymbolKind::Class);
}
