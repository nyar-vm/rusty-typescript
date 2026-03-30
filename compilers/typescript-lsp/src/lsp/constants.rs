//! TypeScript 语言相关常量
//!
//! 集中管理 TypeScript 语言的关键字、内置方法等常量，
//! 避免代码重复，提高维护性。

use oak_lsp::types::CompletionItem;

/// TypeScript 关键字列表
pub const TYPESCRIPT_KEYWORDS: &[&str] = &[
    "abstract",
    "any",
    "as",
    "async",
    "await",
    "boolean",
    "break",
    "case",
    "catch",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "enum",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "implements",
    "import",
    "in",
    "infer",
    "interface",
    "let",
    "module",
    "namespace",
    "never",
    "new",
    "null",
    "number",
    "object",
    "package",
    "private",
    "protected",
    "public",
    "readonly",
    "require",
    "return",
    "static",
    "string",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "type",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

/// String 类型内置方法
pub const STRING_METHODS: &[&str] = &[
    "charAt",
    "charCodeAt",
    "concat",
    "endsWith",
    "includes",
    "indexOf",
    "lastIndexOf",
    "length",
    "localeCompare",
    "match",
    "padEnd",
    "padStart",
    "repeat",
    "replace",
    "replaceAll",
    "search",
    "slice",
    "split",
    "startsWith",
    "substring",
    "toLowerCase",
    "toUpperCase",
    "trim",
    "trimStart",
    "trimEnd",
];

/// Number 类型内置方法
pub const NUMBER_METHODS: &[&str] = &["toExponential", "toFixed", "toLocaleString", "toPrecision", "toString"];

/// Array 类型内置方法
pub const ARRAY_METHODS: &[&str] = &[
    "concat",
    "every",
    "fill",
    "filter",
    "find",
    "findIndex",
    "flat",
    "flatMap",
    "forEach",
    "includes",
    "indexOf",
    "join",
    "lastIndexOf",
    "length",
    "map",
    "pop",
    "push",
    "reduce",
    "reduceRight",
    "reverse",
    "shift",
    "slice",
    "some",
    "sort",
    "splice",
    "unshift",
];

/// Object 类型内置方法
pub const OBJECT_METHODS: &[&str] =
    &["hasOwnProperty", "isPrototypeOf", "propertyIsEnumerable", "toLocaleString", "toString", "valueOf"];

/// 获取关键字补全项
pub fn get_keyword_completions() -> Vec<CompletionItem> {
    TYPESCRIPT_KEYWORDS
        .iter()
        .map(|&keyword| CompletionItem {
            label: keyword.to_string(),
            kind: Some(oak_lsp::types::CompletionItemKind::Keyword),
            detail: Some("Keyword".to_string()),
            documentation: None,
            insert_text: Some(keyword.to_string()),
        })
        .collect()
}

/// 获取指定类型的内置方法补全项
pub fn get_builtin_method_completions(type_name: &str) -> Vec<CompletionItem> {
    let methods = if type_name == "string" || type_name == "String" {
        STRING_METHODS
    }
    else if type_name == "number" || type_name == "Number" {
        NUMBER_METHODS
    }
    else if type_name.ends_with("[]") || type_name == "Array" {
        ARRAY_METHODS
    }
    else if type_name == "object" || type_name == "Object" {
        OBJECT_METHODS
    }
    else {
        return Vec::new();
    };

    methods
        .iter()
        .map(|&method| CompletionItem {
            label: method.to_string(),
            kind: Some(oak_lsp::types::CompletionItemKind::Method),
            detail: Some(format!("{} method", type_name)),
            documentation: None,
            insert_text: Some(method.to_string()),
        })
        .collect()
}
