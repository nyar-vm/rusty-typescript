//! 补全提供器模块
//! 
//! 根据不同的上下文生成相应的补全建议。

use super::*;
use oak_lsp::types::CompletionItem;
use crate::lsp::symbols::{Symbol, SymbolKind, SymbolTable};

/// 补全提供器
pub struct CompletionProvider;

impl CompletionProvider {
    /// 提供补全项
    pub fn provide(context: &CompletionContext, symbol_table: Option<&SymbolTable>) -> Vec<CompletionItem> {
        let mut completions = Vec::new();
        
        match context.context_type {
            CompletionContextType::MemberAccess => {
                if let Some(object_name) = &context.object_name {
                    completions.extend(Self::provide_member_completions(object_name, symbol_table));
                }
            },
            CompletionContextType::ObjectLiteral => {
                completions.extend(Self::provide_object_literal_completions(symbol_table));
            },
            CompletionContextType::TypeAnnotation => {
                completions.extend(Self::provide_type_completions(symbol_table));
            },
            CompletionContextType::FunctionParameter => {
                completions.extend(Self::provide_parameter_completions(symbol_table));
            },
            CompletionContextType::Default => {
                completions.extend(Self::provide_default_completions(symbol_table));
            },
        }
        
        completions
    }
    
    /// 提供成员访问补全
    fn provide_member_completions(object_name: &str, symbol_table: Option<&SymbolTable>) -> Vec<CompletionItem> {
        let mut completions = Vec::new();
        
        if let Some(table) = symbol_table {
            if let Some(symbol) = table.find_by_name(object_name) {
                if let Some(ref type_name) = symbol.type_annotation {
                    let members = Self::get_type_members(type_name, table);
                    for member in members {
                        let kind = Self::symbol_kind_to_completion_kind(member.kind);
                        
                        completions.push(CompletionItem {
                            label: member.name.clone(),
                            kind: Some(kind),
                            detail: member.type_annotation.clone(),
                            documentation: member.signature.clone(),
                            insert_text: Some(member.name.clone()),
                        });
                    }
                }
            }
        }
        
        Self::add_builtin_properties(object_name, &mut completions);
        
        completions
    }
    
    /// 提供对象字面量补全
    fn provide_object_literal_completions(symbol_table: Option<&SymbolTable>) -> Vec<CompletionItem> {
        let mut completions = Vec::new();
        
        // Add common object properties
        let common_properties = [
            "id", "name", "value", "type", "length", "size", "data", "items",
            "key", "title", "description", "enabled", "disabled", "visible",
            "hidden", "active", "inactive", "status", "error", "success",
        ];

        for prop in common_properties {
            completions.push(CompletionItem {
                label: prop.to_string(),
                kind: Some(oak_lsp::types::CompletionItemKind::Property),
                detail: Some("Object property".to_string()),
                documentation: None,
                insert_text: Some(format!("{}: ", prop)),
            });
        }

        // Add symbols that could be used as property names
        if let Some(table) = symbol_table {
            for symbol in table.current_scope_symbols() {
                if matches!(symbol.kind, SymbolKind::Variable | SymbolKind::Property) {
                    completions.push(CompletionItem {
                        label: symbol.name.clone(),
                        kind: Some(oak_lsp::types::CompletionItemKind::Property),
                        detail: symbol.type_annotation.clone(),
                        documentation: None,
                        insert_text: Some(format!("{}: ", symbol.name)),
                    });
                }
            }
        }
        
        completions
    }
    
    /// 提供类型注解补全
    fn provide_type_completions(symbol_table: Option<&SymbolTable>) -> Vec<CompletionItem> {
        let mut completions = Vec::new();
        
        // Add built-in types
        let builtin_types = [
            "string", "number", "boolean", "object", "array", "any", "void",
            "null", "undefined", "never", "unknown", "symbol", "bigint",
        ];

        for type_name in builtin_types {
            completions.push(CompletionItem {
                label: type_name.to_string(),
                kind: Some(oak_lsp::types::CompletionItemKind::Keyword),
                detail: Some("Built-in type".to_string()),
                documentation: None,
                insert_text: Some(type_name.to_string()),
            });
        }

        // Add user-defined types from symbol table
        if let Some(table) = symbol_table {
            for symbol in table.current_scope_symbols() {
                if matches!(symbol.kind, SymbolKind::Class | SymbolKind::Interface | SymbolKind::TypeAlias | SymbolKind::Enum) {
                    completions.push(CompletionItem {
                        label: symbol.name.clone(),
                        kind: Some(Self::symbol_kind_to_completion_kind(symbol.kind)),
                        detail: Some("User-defined type".to_string()),
                        documentation: None,
                        insert_text: Some(symbol.name.clone()),
                    });
                }
            }
        }
        
        completions
    }
    
    /// 提供函数参数补全
    fn provide_parameter_completions(symbol_table: Option<&SymbolTable>) -> Vec<CompletionItem> {
        let mut completions = Vec::new();
        
        // Add common parameter names
        let common_params = [
            "param", "value", "input", "output", "data", "options", "config",
            "callback", "handler", "event", "error", "result", "context",
            "index", "item", "key", "value", "name", "id",
        ];

        for param in common_params {
            completions.push(CompletionItem {
                label: param.to_string(),
                kind: Some(oak_lsp::types::CompletionItemKind::Variable),
                detail: Some("Parameter".to_string()),
                documentation: None,
                insert_text: Some(param.to_string()),
            });
        }

        // Add symbols that could be used as parameter types
        if let Some(table) = symbol_table {
            for symbol in table.current_scope_symbols() {
                if matches!(symbol.kind, SymbolKind::Class | SymbolKind::Interface | SymbolKind::TypeAlias | SymbolKind::Enum) {
                    completions.push(CompletionItem {
                        label: symbol.name.clone(),
                        kind: Some(Self::symbol_kind_to_completion_kind(symbol.kind)),
                        detail: Some("Type".to_string()),
                        documentation: None,
                        insert_text: Some(symbol.name.clone()),
                    });
                }
            }
        }
        
        completions
    }
    
    /// 提供默认补全
    fn provide_default_completions(symbol_table: Option<&SymbolTable>) -> Vec<CompletionItem> {
        let mut completions = Vec::new();
        
        // Add symbols from symbol table
        if let Some(table) = symbol_table {
            for symbol in table.current_scope_symbols() {
                let kind = Self::symbol_kind_to_completion_kind(symbol.kind);
                let mut detail = String::new();
                
                if let Some(ref type_ann) = symbol.type_annotation {
                    detail = format!("{}{}", symbol.name, type_ann);
                }
                
                let insert_text = if symbol.kind == SymbolKind::Function {
                    if let Some(ref sig) = symbol.signature {
                        Some(format!("{}{}", symbol.name, sig))
                    } else {
                        Some(format!("{}()", symbol.name))
                    }
                } else {
                    Some(symbol.name.clone())
                };
                
                completions.push(CompletionItem {
                    label: symbol.name.clone(),
                    kind: Some(kind),
                    detail: if detail.is_empty() { None } else { Some(detail) },
                    documentation: symbol.signature.clone(),
                    insert_text,
                });
            }
        }
        
        // Add keywords
        Self::add_keyword_completions(&mut completions);
        
        completions
    }
    
    /// 添加关键字补全
    fn add_keyword_completions(completions: &mut Vec<CompletionItem>) {
        let keywords = [
            "abstract", "any", "as", "async", "await", "boolean", "break", "case", "catch",
            "class", "const", "continue", "debugger", "default", "delete", "do", "else", "enum",
            "export", "extends", "false", "finally", "for", "function", "if", "implements",
            "import", "in", "infer", "interface", "let", "module", "namespace", "never", "new",
            "null", "number", "object", "package", "private", "protected", "public", "readonly",
            "require", "return", "static", "string", "super", "switch", "this", "throw", "true",
            "try", "type", "typeof", "var", "void", "while", "with", "yield",
        ];

        for keyword in keywords {
            completions.push(CompletionItem {
                label: keyword.to_string(),
                kind: Some(oak_lsp::types::CompletionItemKind::Keyword),
                detail: Some("Keyword".to_string()),
                documentation: None,
                insert_text: Some(keyword.to_string()),
            });
        }
    }
    
    /// 添加内置属性
    fn add_builtin_properties(type_name: &str, completions: &mut Vec<CompletionItem>) {
        let string_methods = [
            "charAt", "charCodeAt", "concat", "endsWith", "includes", "indexOf",
            "lastIndexOf", "length", "localeCompare", "match", "padEnd", "padStart",
            "repeat", "replace", "replaceAll", "search", "slice", "split", "startsWith",
            "substring", "toLowerCase", "toUpperCase", "trim", "trimStart", "trimEnd",
        ];

        let number_methods = [
            "toExponential", "toFixed", "toLocaleString", "toPrecision", "toString",
        ];

        let array_methods = [
            "concat", "every", "fill", "filter", "find", "findIndex", "flat", "flatMap",
            "forEach", "includes", "indexOf", "join", "lastIndexOf", "length", "map",
            "pop", "push", "reduce", "reduceRight", "reverse", "shift", "slice",
            "some", "sort", "splice", "unshift",
        ];

        let object_methods = [
            "hasOwnProperty", "isPrototypeOf", "propertyIsEnumerable",
            "toLocaleString", "toString", "valueOf",
        ];

        let methods = if type_name == "string" || type_name == "String" {
            Some(&string_methods[..])
        } else if type_name == "number" || type_name == "Number" {
            Some(&number_methods[..])
        } else if type_name.ends_with("[]") || type_name == "Array" {
            Some(&array_methods[..])
        } else if type_name == "object" || type_name == "Object" {
            Some(&object_methods[..])
        } else {
            None
        };

        if let Some(methods) = methods {
            for method in methods {
                completions.push(CompletionItem {
                    label: method.to_string(),
                    kind: Some(oak_lsp::types::CompletionItemKind::Method),
                    detail: Some(format!("{} method", type_name)),
                    documentation: None,
                    insert_text: Some(method.to_string()),
                });
            }
        }
    }
    
    /// 获取类型成员
    fn get_type_members(type_name: &str, table: &SymbolTable) -> Vec<&Symbol> {
        let mut members = Vec::new();

        for symbol in table.all_symbols() {
            if matches!(symbol.kind, SymbolKind::Property | SymbolKind::Method) {
                if let Some(ref parent_type) = symbol.type_annotation {
                    if parent_type == type_name {
                        members.push(symbol);
                    }
                }
            }
        }

        members
    }
    
    /// 转换符号种类为补全项种类
    fn symbol_kind_to_completion_kind(kind: SymbolKind) -> oak_lsp::types::CompletionItemKind {
        use oak_lsp::types::CompletionItemKind;
        
        match kind {
            SymbolKind::Variable => CompletionItemKind::Variable,
            SymbolKind::Function => CompletionItemKind::Function,
            SymbolKind::Class => CompletionItemKind::Class,
            SymbolKind::Interface => CompletionItemKind::Interface,
            SymbolKind::TypeAlias => CompletionItemKind::TypeParameter,
            SymbolKind::Enum => CompletionItemKind::Enum,
            SymbolKind::EnumMember => CompletionItemKind::EnumMember,
            SymbolKind::Parameter => CompletionItemKind::Variable,
            SymbolKind::Property => CompletionItemKind::Property,
            SymbolKind::Method => CompletionItemKind::Method,
            SymbolKind::Module => CompletionItemKind::Module,
            SymbolKind::Namespace => CompletionItemKind::Module,
        }
    }
}
