//! 符号收集器模块
//!
//! 遍历代码收集符号信息。

use super::{Scope, ScopeType, Symbol, SymbolKind, SymbolTable};
use core::range::Range;

/// 符号收集器
///
/// 用于遍历 TypeScript 代码并收集符号信息
#[derive(Debug, Default)]
pub struct SymbolCollector {
    /// 符号表
    symbol_table: SymbolTable,
    /// 当前文档 URI
    current_uri: Option<String>,
}

impl SymbolCollector {
    /// 创建新的符号收集器
    pub fn new() -> Self {
        Self { symbol_table: SymbolTable::new(), current_uri: None }
    }

    /// 设置文档 URI
    pub fn with_uri(mut self, uri: String) -> Self {
        self.current_uri = Some(uri);
        self
    }

    /// 收集代码中的符号
    pub fn collect(&mut self, source: &str) -> &SymbolTable {
        self.symbol_table.clear();
        self.parse_declarations(source);
        &self.symbol_table
    }

    /// 解析声明语句
    fn parse_declarations(&mut self, source: &str) {
        let lines: Vec<&str> = source.lines().collect();
        let mut offset = 0;
        let mut brace_stack: Vec<(usize, ScopeType)> = Vec::new();

        for (line_idx, line) in lines.iter().enumerate() {
            let line_start = offset;
            let line_end = offset + line.len();
            let trimmed = line.trim();

            if trimmed.is_empty() {
                offset = line_end + 1;
                continue;
            }

            self.parse_variable_declaration(trimmed, line_start, line_end);
            self.parse_function_declaration(trimmed, line_start, line_end, source);
            self.parse_class_declaration(trimmed, line_start, line_end, source);
            self.parse_interface_declaration(trimmed, line_start, line_end, source);
            self.parse_type_declaration(trimmed, line_start, line_end);
            self.parse_enum_declaration(trimmed, line_start, line_end, source);

            self.track_braces(trimmed, line_start, &mut brace_stack);

            offset = line_end + 1;
        }
    }

    /// 跟踪大括号以管理作用域
    fn track_braces(&mut self, line: &str, _line_start: usize, brace_stack: &mut Vec<(usize, ScopeType)>) {
        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '{' => {
                    let scope_type = self.determine_scope_type(&chars.collect::<String>());
                    brace_stack.push((0, scope_type));
                    self.symbol_table.enter_scope(0, scope_type);
                }
                '}' => {
                    if brace_stack.pop().is_some() {
                        self.symbol_table.exit_scope(0);
                    }
                }
                _ => {}
            }
        }
    }

    /// 根据上下文确定作用域类型
    fn determine_scope_type(&self, following_text: &str) -> ScopeType {
        let text = following_text.trim_start();

        if text.starts_with("get ") || text.starts_with("set ") || text.starts_with("constructor") {
            ScopeType::Function
        }
        else if text.starts_with("public") || text.starts_with("private") || text.starts_with("protected") {
            ScopeType::Class
        }
        else {
            ScopeType::Block
        }
    }

    /// 解析变量声明
    fn parse_variable_declaration(&mut self, line: &str, line_start: usize, line_end: usize) {
        let keywords = ["const ", "let ", "var "];

        for keyword in keywords {
            if line.starts_with(keyword) {
                let rest = &line[keyword.len()..];

                if let Some(var_part) = rest.split(|c| c == '=' || c == ';').next() {
                    let var_name = var_part.split(':').next().unwrap_or("").trim();

                    if !var_name.is_empty() && Self::is_valid_identifier(var_name) {
                        let name_start = line_start + keyword.len();
                        let name_end = name_start + var_name.len();

                        let type_annotation = self.extract_type_annotation(rest);
                        let init_expr = self.extract_init_expression(rest);
                        let inferred_type = type_annotation.or_else(|| self.infer_type_from_init(init_expr));

                        let mut symbol =
                            Symbol::new(var_name.to_string(), SymbolKind::Variable, Range::from(line_start..line_end))
                                .with_selection_range(Range::from(name_start..name_end));

                        if let Some(t) = inferred_type {
                            symbol = symbol.with_type(t);
                        }

                        if let Some(uri) = &self.current_uri {
                            symbol = symbol.with_uri(uri.clone());
                        }

                        self.symbol_table.add_symbol(symbol);
                    }
                }
                break;
            }
        }
    }

    /// 解析函数声明
    fn parse_function_declaration(&mut self, line: &str, line_start: usize, line_end: usize, source: &str) {
        if !line.starts_with("function ") && !line.starts_with("async function ") {
            return;
        }

        let is_async = line.starts_with("async ");
        let prefix = if is_async { "async function " } else { "function " };

        let rest = &line[prefix.len()..];

        if let Some(func_part) = rest.split(|c| c == '{' || c == ';').next() {
            let func_name = func_part.split('(').next().unwrap_or("").trim();

            if !func_name.is_empty() && Self::is_valid_identifier(func_name) {
                let name_start = line_start + prefix.len();
                let name_end = name_start + func_name.len();

                let signature = self.extract_function_signature(func_part);
                let return_type = self.extract_return_type(func_part);

                let mut symbol = Symbol::new(func_name.to_string(), SymbolKind::Function, Range::from(line_start..line_end))
                    .with_selection_range(Range::from(name_start..name_end));

                if let Some(sig) = signature {
                    symbol = symbol.with_signature(sig);
                }

                if let Some(ret) = return_type {
                    symbol = symbol.with_type(ret);
                }

                if let Some(uri) = &self.current_uri {
                    symbol = symbol.with_uri(uri.clone());
                }

                self.symbol_table.add_symbol(symbol);

                self.parse_function_parameters(func_part, line_start + prefix.len() + func_name.len());
            }
        }
    }

    /// 解析函数参数
    fn parse_function_parameters(&mut self, params_part: &str, _base_offset: usize) {
        if let Some(params_start) = params_part.find('(') {
            if let Some(params_end) = params_part.find(')') {
                let params_str = &params_part[params_start + 1..params_end];

                for param in params_str.split(',') {
                    let param = param.trim();
                    if param.is_empty() {
                        continue;
                    }

                    let param_name = param.split(':').next().unwrap_or("").trim();

                    if !param_name.is_empty() && Self::is_valid_identifier(param_name) {
                        let type_annotation = self.extract_type_annotation(param);

                        let mut symbol = Symbol::new(param_name.to_string(), SymbolKind::Parameter, Range::from(0..0));

                        if let Some(t) = type_annotation {
                            symbol = symbol.with_type(t);
                        }

                        if let Some(uri) = &self.current_uri {
                            symbol = symbol.with_uri(uri.clone());
                        }

                        self.symbol_table.add_symbol(symbol);
                    }
                }
            }
        }
    }

    /// 解析类声明
    fn parse_class_declaration(&mut self, line: &str, line_start: usize, line_end: usize, source: &str) {
        if !line.starts_with("class ") && !line.starts_with("export class ") && !line.starts_with("abstract class ") {
            return;
        }

        let prefix = if line.starts_with("export class ") {
            "export class "
        }
        else if line.starts_with("abstract class ") {
            "abstract class "
        }
        else {
            "class "
        };

        let rest = &line[prefix.len()..];

        if let Some(class_part) = rest.split(|c| c == '{' || c == ' ' || c == '<').next() {
            let class_name = class_part.trim();

            if !class_name.is_empty() && Self::is_valid_identifier(class_name) {
                let name_start = line_start + prefix.len();
                let name_end = name_start + class_name.len();

                let mut symbol = Symbol::new(class_name.to_string(), SymbolKind::Class, Range::from(line_start..line_end))
                    .with_selection_range(Range::from(name_start..name_end))
                    .with_type(class_name.to_string());

                if line.starts_with("export ") {
                    symbol = symbol.with_exported(true);
                }

                if let Some(uri) = &self.current_uri {
                    symbol = symbol.with_uri(uri.clone());
                }

                self.symbol_table.add_symbol(symbol);
            }
        }
    }

    /// 解析接口声明
    fn parse_interface_declaration(&mut self, line: &str, line_start: usize, line_end: usize, source: &str) {
        if !line.starts_with("interface ") && !line.starts_with("export interface ") {
            return;
        }

        let prefix = if line.starts_with("export interface ") { "export interface " } else { "interface " };

        let rest = &line[prefix.len()..];

        if let Some(interface_part) = rest.split(|c| c == '{' || c == ' ' || c == '<').next() {
            let interface_name = interface_part.trim();

            if !interface_name.is_empty() && Self::is_valid_identifier(interface_name) {
                let name_start = line_start + prefix.len();
                let name_end = name_start + interface_name.len();

                let mut symbol =
                    Symbol::new(interface_name.to_string(), SymbolKind::Interface, Range::from(line_start..line_end))
                        .with_selection_range(Range::from(name_start..name_end))
                        .with_type(interface_name.to_string());

                if line.starts_with("export ") {
                    symbol = symbol.with_exported(true);
                }

                if let Some(uri) = &self.current_uri {
                    symbol = symbol.with_uri(uri.clone());
                }

                self.symbol_table.add_symbol(symbol);
            }
        }
    }

    /// 解析类型别名声明
    fn parse_type_declaration(&mut self, line: &str, line_start: usize, line_end: usize) {
        if !line.starts_with("type ") && !line.starts_with("export type ") {
            return;
        }

        let prefix = if line.starts_with("export type ") { "export type " } else { "type " };

        let rest = &line[prefix.len()..];

        if let Some(type_part) = rest.split('=').next() {
            let type_name = type_part.split('<').next().unwrap_or("").trim();

            if !type_name.is_empty() && Self::is_valid_identifier(type_name) {
                let name_start = line_start + prefix.len();
                let name_end = name_start + type_name.len();

                let mut symbol = Symbol::new(type_name.to_string(), SymbolKind::TypeAlias, Range::from(line_start..line_end))
                    .with_selection_range(Range::from(name_start..name_end));

                if line.starts_with("export ") {
                    symbol = symbol.with_exported(true);
                }

                if let Some(uri) = &self.current_uri {
                    symbol = symbol.with_uri(uri.clone());
                }

                self.symbol_table.add_symbol(symbol);
            }
        }
    }

    /// 解析枚举声明
    fn parse_enum_declaration(&mut self, line: &str, line_start: usize, line_end: usize, source: &str) {
        if !line.starts_with("enum ") && !line.starts_with("export enum ") && !line.starts_with("const enum ") {
            return;
        }

        let prefix = if line.starts_with("export enum ") {
            "export enum "
        }
        else if line.starts_with("const enum ") {
            "const enum "
        }
        else {
            "enum "
        };

        let rest = &line[prefix.len()..];

        if let Some(enum_part) = rest.split(|c| c == '{' || c == ' ').next() {
            let enum_name = enum_part.trim();

            if !enum_name.is_empty() && Self::is_valid_identifier(enum_name) {
                let name_start = line_start + prefix.len();
                let name_end = name_start + enum_name.len();

                let mut symbol = Symbol::new(enum_name.to_string(), SymbolKind::Enum, Range::from(line_start..line_end))
                    .with_selection_range(Range::from(name_start..name_end))
                    .with_type(enum_name.to_string());

                if line.starts_with("export ") {
                    symbol = symbol.with_exported(true);
                }

                if let Some(uri) = &self.current_uri {
                    symbol = symbol.with_uri(uri.clone());
                }

                self.symbol_table.add_symbol(symbol);
            }
        }
    }

    /// 提取类型注解
    fn extract_type_annotation(&self, text: &str) -> Option<String> {
        if let Some(colon_pos) = text.find(':') {
            let type_part = &text[colon_pos + 1..];
            let type_str = type_part.split(|c| c == '=' || c == ';' || c == ',').next().unwrap_or("").trim();

            if !type_str.is_empty() {
                return Some(type_str.to_string());
            }
        }
        None
    }

    /// 提取初始化表达式
    fn extract_init_expression<'a>(&self, text: &'a str) -> &'a str {
        if let Some(eq_pos) = text.find('=') {
            let expr = &text[eq_pos + 1..];
            expr.split(';').next().unwrap_or("").trim()
        }
        else {
            ""
        }
    }

    /// 从初始化表达式推断类型
    fn infer_type_from_init(&self, expr: &str) -> Option<String> {
        let expr = expr.trim();

        if expr.starts_with('"') || expr.starts_with('\'') || expr.starts_with('`') {
            return Some("string".to_string());
        }

        if expr == "true" || expr == "false" {
            return Some("boolean".to_string());
        }

        if expr.parse::<i64>().is_ok() || expr.parse::<f64>().is_ok() {
            return Some("number".to_string());
        }

        if expr == "null" {
            return Some("null".to_string());
        }

        if expr == "undefined" {
            return Some("undefined".to_string());
        }

        if expr.starts_with('[') {
            return Some("any[]".to_string());
        }

        if expr.starts_with('{') {
            return Some("object".to_string());
        }

        if expr.starts_with("function") || expr.contains("=>") {
            return Some("Function".to_string());
        }

        if expr.starts_with("new ") {
            if let Some(class_name) = expr.split_whitespace().nth(1) {
                let name = class_name.split('(').next().unwrap_or("");
                return Some(name.to_string());
            }
        }

        None
    }

    /// 提取函数签名
    fn extract_function_signature(&self, func_part: &str) -> Option<String> {
        if let Some(paren_pos) = func_part.find('(') {
            if let Some(end_pos) = func_part.find(')') {
                let params = &func_part[paren_pos..end_pos + 1];
                let return_type = self.extract_return_type(func_part);

                if let Some(ret) = return_type {
                    return Some(format!("{}: {}", params, ret));
                }
                else {
                    return Some(params.to_string());
                }
            }
        }
        None
    }

    /// 提取返回类型
    fn extract_return_type(&self, func_part: &str) -> Option<String> {
        if let Some(colon_pos) = func_part.find("):") {
            let after_paren = &func_part[colon_pos + 2..];
            let return_type = after_paren.split(|c| c == '{' || c == ';').next().unwrap_or("").trim();

            if !return_type.is_empty() {
                return Some(return_type.to_string());
            }
        }
        None
    }

    /// 检查是否为有效标识符
    fn is_valid_identifier(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        let mut chars = name.chars();

        if let Some(first) = chars.next() {
            if !first.is_alphabetic() && first != '_' && first != '$' {
                return false;
            }
        }

        chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
    }

    /// 获取符号表
    pub fn symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    /// 获取可变符号表
    pub fn symbol_table_mut(&mut self) -> &mut SymbolTable {
        &mut self.symbol_table
    }
}
