#![doc = include_str!("readme.md")]

use core::range::Range;
use oak_core::Arc;
use oak_lsp::{
    LanguageService, WorkspaceManager,
    types::{
        CodeAction, CompletionItem, Diagnostic, DocumentHighlight, FoldingRange, Hover, InitializeParams, InlayHint,
        LocationRange, SemanticTokens, SignatureHelp, SourcePosition, StructureItem, TextEdit, WorkspaceEdit, WorkspaceSymbol,
    },
};
use oak_vfs::{MemoryVfs, Vfs};
use rayon::prelude::*;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    sync::{Mutex, RwLock},
    thread,
};

pub mod completion;
pub mod constants;
pub mod diagnostics;
pub mod formatter;
pub mod highlighter;
pub mod inlay_hints;
pub mod symbols;

use diagnostics::{DiagnosticAnalyzer, DiagnosticLevel};
use formatter::{FormatOptions, format_code_with_options, format_range as format_code_range};
use inlay_hints::InlayHintProvider;
use symbols::{Symbol, SymbolCollector, SymbolKind, SymbolTable};

/// TypeScript language service implementing oak-lsp's LanguageService trait.
pub struct TypeScriptLanguageService {
    vfs: MemoryVfs,
    workspace: WorkspaceManager,
    documents: RwLock<HashMap<String, String>>,
    symbol_tables: RwLock<HashMap<String, SymbolTable>>,
    /// 跟踪文件内容的哈希，用于检测变化
    document_hashes: RwLock<HashMap<String, u64>>,
    /// 跟踪文件的修改范围，用于增量诊断
    document_changes: RwLock<HashMap<String, Option<Range<usize>>>>,
    /// 补全排序器
    completion_sorter: Mutex<completion::CompletionSorter>,
    /// 补全使用频率
    completion_usage: Mutex<HashMap<String, u32>>,
    /// 补全缓存，键为 (uri, offset, prefix)，值为补全结果
    completion_cache: Mutex<HashMap<(String, usize, String), Vec<CompletionItem>>>,
    /// 诊断结果缓存
    diagnostic_cache: Mutex<HashMap<String, Vec<Diagnostic>>>,
    /// 格式化结果缓存
    format_cache: Mutex<HashMap<(String, FormatOptions), String>>,
}

impl TypeScriptLanguageService {
    /// Creates a new `TypeScriptLanguageService`.
    pub fn new() -> Self {
        Self {
            vfs: MemoryVfs::new(),
            workspace: WorkspaceManager::new(),
            documents: RwLock::new(HashMap::new()),
            symbol_tables: RwLock::new(HashMap::new()),
            document_hashes: RwLock::new(HashMap::new()),
            document_changes: RwLock::new(HashMap::new()),
            completion_sorter: Mutex::new(completion::CompletionSorter::new()),
            completion_usage: Mutex::new(HashMap::new()),
            completion_cache: Mutex::new(HashMap::new()),
            diagnostic_cache: Mutex::new(HashMap::new()),
            format_cache: Mutex::new(HashMap::new()),
        }
    }

    /// Get document content by URI
    fn get_document(&self, uri: &str) -> Option<Cow<str>> {
        let documents = self.documents.read().unwrap();
        documents.get(uri).map(|s| Cow::Owned(s.clone()))
    }

    /// Set document content by URI
    fn set_document(&self, uri: String, content: String) {
        let mut documents = self.documents.write().unwrap();
        documents.insert(uri.clone(), content.clone());
        // Also update VFS
        self.vfs.write_file(&uri, content);

        // 标记整个文件为已修改，以便进行全面诊断
        let mut changes = self.document_changes.write().unwrap();
        changes.insert(uri.clone(), None);

        // 清除相关缓存
        self.clear_caches(&uri);
    }

    /// Clear caches for a document
    fn clear_caches(&self, uri: &str) {
        // 清除诊断缓存
        let mut diagnostic_cache = self.diagnostic_cache.lock().unwrap();
        diagnostic_cache.remove(uri);

        // 清除格式化缓存
        let mut format_cache = self.format_cache.lock().unwrap();
        format_cache.retain(|(cached_uri, _), _| cached_uri != uri);

        // 清除补全缓存
        let mut completion_cache = self.completion_cache.lock().unwrap();
        completion_cache.retain(|(cached_uri, _, _), _| cached_uri != uri);
    }

    /// Set document content with change range
    fn set_document_with_change(&self, uri: String, content: String, change_range: Range<usize>) {
        let mut documents = self.documents.write().unwrap();
        documents.insert(uri.clone(), content.clone());
        // Also update VFS
        self.vfs.write_file(&uri, content);

        // 记录修改范围，用于增量诊断
        let mut changes = self.document_changes.write().unwrap();
        changes.insert(uri.clone(), Some(change_range));

        // 清除相关缓存
        self.clear_caches(&uri);
    }

    /// Calculate hash for content
    fn calculate_hash(&self, content: &str) -> u64 {
        use std::{
            collections::hash_map::DefaultHasher,
            hash::{Hash, Hasher},
        };

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    /// Update symbol table for a document (only if content has changed)
    fn update_symbol_table(&self, uri: &str, content: &str) {
        let current_hash = self.calculate_hash(content);

        // Check if content has changed (read-only operation)
        {
            let hashes = self.document_hashes.read().unwrap();
            if let Some(previous_hash) = hashes.get(uri) {
                if *previous_hash == current_hash {
                    // Content hasn't changed, no need to update symbol table
                    return;
                }
            }
        }

        // Get change range for incremental update
        let change_range = {
            let changes = self.document_changes.read().unwrap();
            changes.get(uri).cloned().unwrap_or(None)
        };

        // Content has changed, update symbol table
        let mut collector = SymbolCollector::new();
        if !uri.is_empty() {
            collector = collector.with_uri(uri.to_string());
        }

        // Use incremental collection if possible
        match change_range {
            Some(range) => {
                // Incremental collection (only process changed range)
                collector.collect_incremental(content, range);
            }
            None => {
                // Full collection (process entire file)
                collector.collect(content);
            }
        }

        // Write operations
        let mut tables = self.symbol_tables.write().unwrap();
        tables.insert(uri.to_string(), collector.symbol_table().clone());

        let mut hashes = self.document_hashes.write().unwrap();
        hashes.insert(uri.to_string(), current_hash);
    }

    /// Get symbol table for a document
    fn get_symbol_table(&self, uri: &str) -> Option<SymbolTable> {
        let tables = self.symbol_tables.read().unwrap();
        tables.get(uri).cloned()
    }

    /// Get all document URIs
    fn get_all_document_uris(&self) -> Vec<String> {
        let tables = self.symbol_tables.read().unwrap();
        tables.keys().cloned().collect()
    }

    /// Parse member access expression (e.g., obj.property)
    fn parse_member_access(&self, text: &str, offset: usize) -> Option<String> {
        let before_cursor = &text[..offset];

        if let Some(dot_pos) = before_cursor.rfind('.') {
            let before_dot = &before_cursor[..dot_pos];
            let identifier = self.extract_identifier_before(before_dot)?;
            return Some(identifier);
        }
        None
    }

    /// Extract identifier before a position
    fn extract_identifier_before(&self, text: &str) -> Option<String> {
        let mut end = text.len();

        while end > 0 {
            let ch = text.chars().nth(end - 1)?;
            if ch.is_whitespace() {
                end -= 1;
            }
            else {
                break;
            }
        }

        if end == 0 {
            return None;
        }

        let mut start = end;
        while start > 0 {
            let ch = text.chars().nth(start - 1)?;
            if ch.is_alphanumeric() || ch == '_' || ch == '$' {
                start -= 1;
            }
            else {
                break;
            }
        }

        if start < end { Some(text[start..end].to_string()) } else { None }
    }

    /// Check if cursor is after a dot (member access)
    fn is_member_access_context(&self, text: &str, offset: usize) -> bool {
        if offset == 0 {
            return false;
        }

        let before_cursor = &text[..offset];
        let trimmed = before_cursor.trim_end();
        trimmed.ends_with('.')
    }

    /// Check if cursor is in an object literal context (e.g., { | })
    fn is_object_literal_context(&self, text: &str, offset: usize) -> bool {
        let before_cursor = &text[..offset];

        // Check if there's an opening brace without a closing brace
        let open_braces = before_cursor.chars().filter(|&c| c == '{').count();
        let close_braces = before_cursor.chars().filter(|&c| c == '}').count();

        if open_braces <= close_braces {
            return false;
        }

        // Check if we're after a comma or opening brace
        let trimmed = before_cursor.trim_end();
        trimmed.ends_with('{') || trimmed.ends_with(',')
    }

    /// Check if cursor is in a type annotation context (e.g., let x: |)
    fn is_type_annotation_context(&self, text: &str, offset: usize) -> bool {
        let before_cursor = &text[..offset];

        // Look for a colon before the cursor
        if let Some(colon_pos) = before_cursor.rfind(':') {
            // Check if there's no semicolon or assignment after the colon
            let after_colon = &before_cursor[colon_pos + 1..];
            let trimmed = after_colon.trim();

            // Check if we're not in a string or comment
            !self.is_in_string_or_comment(text, offset) && trimmed.is_empty()
        }
        else {
            false
        }
    }

    /// Check if cursor is in a function parameter context (e.g., function f(|))
    fn is_function_parameter_context(&self, text: &str, offset: usize) -> bool {
        let before_cursor = &text[..offset];

        // Look for an opening parenthesis without a closing parenthesis
        let open_parens = before_cursor.chars().filter(|&c| c == '(').count();
        let close_parens = before_cursor.chars().filter(|&c| c == ')').count();

        if open_parens <= close_parens {
            return false;
        }

        // Check if the last opening parenthesis is part of a function definition
        let mut paren_count = 0;
        for (i, c) in before_cursor.char_indices().rev() {
            match c {
                ')' => paren_count += 1,
                '(' => {
                    paren_count -= 1;
                    if paren_count < 0 {
                        // Found the matching opening parenthesis
                        let before_paren = &before_cursor[..i];
                        let trimmed = before_paren.trim_end();
                        // Check if it's part of a function definition
                        return trimmed.ends_with("function")
                            || trimmed.ends_with("=>")
                            || trimmed.ends_with("=")
                            || trimmed.ends_with(",");
                    }
                }
                _ => {}
            }
        }

        false
    }

    /// Check if position is in a string or comment
    fn is_in_string_or_comment(&self, text: &str, offset: usize) -> bool {
        // Simplified implementation - in a real LSP, this would be more sophisticated
        let mut in_string = false;
        let mut in_comment = false;
        let mut string_delimiter = '"';

        for (i, c) in text.char_indices() {
            if i >= offset {
                break;
            }

            if !in_comment && !in_string && c == '/' {
                // Check for comment
                if let Some(next) = text.chars().nth(i + 1) {
                    if next == '/' {
                        in_comment = true;
                    }
                    else if next == '*' {
                        in_comment = true;
                    }
                }
            }
            else if in_comment && c == '*' {
                // Check for end of multi-line comment
                if let Some(next) = text.chars().nth(i + 1) {
                    if next == '/' {
                        in_comment = false;
                    }
                }
            }
            else if in_comment && c == '\n' {
                // End of single-line comment
                in_comment = false;
            }
            else if !in_comment && (c == '"' || c == '\'') {
                // Check for string
                if !in_string {
                    in_string = true;
                    string_delimiter = c;
                }
                else if c == string_delimiter {
                    // Check if not escaped
                    let mut escaped = false;
                    let mut j = i - 1;
                    while j >= 0 {
                        if let Some(ch) = text.chars().nth(j) {
                            if ch == '\\' {
                                escaped = !escaped;
                            }
                            else {
                                break;
                            }
                        }
                        j -= 1;
                    }
                    if !escaped {
                        in_string = false;
                    }
                }
            }
        }

        in_string || in_comment
    }

    /// Convert symbol kind to completion item kind
    fn symbol_kind_to_completion_kind(&self, kind: SymbolKind) -> oak_lsp::types::CompletionItemKind {
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

    /// Add keyword completions
    fn add_keyword_completions(&self, completions: &mut Vec<CompletionItem>) {
        completions.extend(constants::get_keyword_completions());
    }

    /// Get member completions for an object
    fn get_member_completions(&self, object_name: &str, uri: &str) -> Vec<CompletionItem> {
        let mut completions = Vec::new();

        if let Some(table) = self.get_symbol_table(uri) {
            if let Some(symbol) = table.find_by_name(object_name) {
                if let Some(ref type_name) = symbol.type_annotation {
                    let members = self.get_type_members(type_name, &table);
                    for member in members {
                        let kind = self.symbol_kind_to_completion_kind(member.kind);

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

        self.add_builtin_properties(object_name, &mut completions);

        completions
    }

    /// Get members of a type
    fn get_type_members(&self, type_name: &str, table: &SymbolTable) -> Vec<Symbol> {
        let mut members = Vec::new();

        for symbol in table.all_symbols() {
            if matches!(symbol.kind, SymbolKind::Property | SymbolKind::Method) {
                if let Some(ref parent_type) = symbol.type_annotation {
                    if parent_type == type_name {
                        members.push(symbol.clone());
                    }
                }
            }
        }

        members
    }

    /// Add built-in properties for common types
    fn add_builtin_properties(&self, type_name: &str, completions: &mut Vec<CompletionItem>) {
        completions.extend(constants::get_builtin_method_completions(type_name));
    }

    /// Get file content from VFS
    fn get_file_content(&self, uri: &str) -> Option<Cow<str>> {
        self.vfs.get_source(uri).map(|source| Cow::Owned(source.text().to_string()))
    }

    /// Extract symbol at position
    pub fn extract_symbol_at_position(&self, text: &str, offset: usize) -> Option<String> {
        let mut start = offset;
        let mut end = offset;
        let chars: Vec<char> = text.chars().collect();

        // Search backwards
        while start > 0 {
            let ch = chars[start - 1];
            if ch.is_alphanumeric() || ch == '_' || ch == '$' {
                start -= 1;
            }
            else {
                break;
            }
        }

        // Search forwards
        while end < chars.len() {
            let ch = chars[end];
            if ch.is_alphanumeric() || ch == '_' || ch == '$' {
                end += 1;
            }
            else {
                break;
            }
        }

        if start < end { Some(chars[start..end].iter().collect()) } else { None }
    }

    /// Get offset from line and column
    pub fn get_offset(&self, text: &str, line: usize, column: usize) -> usize {
        // Precompute line offsets for faster lookup
        let line_offsets: Vec<usize> = text.char_indices().filter(|&(_, c)| c == '\n').map(|(i, _)| i + 1).collect();

        // Calculate offset for the given line
        let line_start = if line == 0 {
            0
        }
        else if line <= line_offsets.len() {
            line_offsets[line - 1]
        }
        else {
            return text.len();
        };

        // Calculate column offset
        let _current_offset = line_start;
        let mut current_col = 0;

        for (i, _c) in text[line_start..].char_indices() {
            if current_col >= column {
                return line_start + i;
            }
            current_col += 1;
        }

        text.len()
    }

    /// Get line and column from offset
    pub fn get_line_and_column(&self, text: &str, offset: usize) -> (usize, usize) {
        // Precompute line offsets for faster lookup
        let line_offsets: Vec<usize> = text.char_indices().filter(|&(_, c)| c == '\n').map(|(i, _)| i + 1).collect();

        // Find the line containing the offset
        let line = line_offsets.iter().filter(|&&o| o <= offset).count();

        // Calculate column
        let line_start = if line == 0 { 0 } else { line_offsets[line - 1] };

        let mut col = 0;
        for (i, c) in text[line_start..offset].char_indices() {
            col += 1;
        }

        (line, col)
    }

    /// Check if it's a complete symbol
    pub fn is_complete_symbol(&self, line: &str, start: usize, end: usize) -> bool {
        // Check previous character
        if start > 0 {
            let prev_char = line.chars().nth(start - 1).unwrap_or(' ');
            if prev_char.is_alphanumeric() || prev_char == '_' || prev_char == '$' {
                return false;
            }
        }

        // Check next character
        if end < line.len() {
            let next_char = line.chars().nth(end).unwrap_or(' ');
            if next_char.is_alphanumeric() || next_char == '_' || next_char == '$' {
                return false;
            }
        }

        true
    }

    /// Parse format options
    pub fn parse_format_options(&self, tab_size: u32, insert_spaces: bool) -> FormatOptions {
        FormatOptions::default()
    }

    /// Infer variable type
    pub fn infer_variable_type(&self, init_expr: &str) -> String {
        let expr = init_expr.trim();

        if expr.starts_with('"') || expr.starts_with('\'') || expr.starts_with('`') {
            return "string".to_string();
        }
        else if expr == "true" || expr == "false" {
            return "boolean".to_string();
        }
        else if expr.starts_with('{') && expr.ends_with('}') {
            // Try to infer object type with properties
            return self.infer_object_type(expr);
        }
        else if expr.starts_with('[') && expr.ends_with(']') {
            // Try to infer array type
            return self.infer_array_type(expr);
        }
        else if expr.parse::<i64>().is_ok() || expr.parse::<f64>().is_ok() {
            return "number".to_string();
        }
        else if expr == "null" {
            return "null".to_string();
        }
        else if expr == "undefined" {
            return "undefined".to_string();
        }
        else if expr.starts_with("function") || expr.contains("=>") {
            return self.infer_function_type(expr);
        }
        else if expr.starts_with("new ") {
            // Infer type from constructor
            if let Some(class_name) = expr.split_whitespace().nth(1) {
                let name = class_name.split('(').next().unwrap_or("");
                return name.to_string();
            }
        }
        else if expr.starts_with("this.") {
            // Infer type from this property access
            return "any".to_string();
        }

        "any".to_string()
    }

    /// Infer object type from object literal
    fn infer_object_type(&self, expr: &str) -> String {
        // Simplified object type inference
        // In a real implementation, we would parse the object literal and infer property types
        "object".to_string()
    }

    /// Infer array type from array literal
    fn infer_array_type(&self, expr: &str) -> String {
        // Simplified array type inference
        // In a real implementation, we would analyze array elements to infer a more specific type
        "any[]".to_string()
    }

    /// Infer function type from function expression
    fn infer_function_type(&self, expr: &str) -> String {
        // Simplified function type inference
        // In a real implementation, we would parse function parameters and return type
        "Function".to_string()
    }

    /// Update completion usage frequency
    fn update_completion_usage(&self, item: &str) {
        let mut usage = self.completion_usage.lock().unwrap();
        *usage.entry(item.to_string()).or_insert(0) += 1;
    }

    /// 检查行是否是符号的定义行
    fn is_definition_line(&self, line: &str, symbol_name: &str) -> bool {
        let trimmed = line.trim();

        // 检查常见的定义模式
        let definition_patterns = [
            format!("const {}", symbol_name),
            format!("let {}", symbol_name),
            format!("var {}", symbol_name),
            format!("function {}", symbol_name),
            format!("class {}", symbol_name),
            format!("interface {}", symbol_name),
            format!("type {}", symbol_name),
            format!("enum {}", symbol_name),
            format!("export const {}", symbol_name),
            format!("export let {}", symbol_name),
            format!("export var {}", symbol_name),
            format!("export function {}", symbol_name),
            format!("export class {}", symbol_name),
            format!("export interface {}", symbol_name),
            format!("export type {}", symbol_name),
            format!("export enum {}", symbol_name),
        ];

        // 检查是否匹配任何定义模式
        if definition_patterns.iter().any(|p| trimmed.starts_with(p)) {
            return true;
        }

        // 检查属性定义模式
        let property_patterns = [format!("{}:", symbol_name), format!("{} =", symbol_name)];

        property_patterns.iter().any(|p| trimmed.contains(p))
    }

    /// 在单个文件中查找引用
    fn find_references_in_file(&self, uri: &str, content: &str, symbol_name: &str) -> Vec<LocationRange> {
        let mut references = Vec::new();
        let mut reference_set = HashSet::new();

        /// 首先尝试使用符号表来查找引用
        if let Some(symbol_table) = self.get_symbol_table(uri) {
            for symbol in symbol_table.find_all_by_name(symbol_name) {
                let location = LocationRange { uri: Arc::from(uri), range: symbol.range.clone() };
                let key = (uri, symbol.range.start, symbol.range.end);
                if reference_set.insert(key) {
                    references.push(location);
                }
            }
        }

        /// 然后使用基于文本的查找作为补充
        for (line_idx, line) in content.lines().enumerate() {
            let mut current_pos = 0;
            while let Some(start) = line[current_pos..].find(symbol_name) {
                let actual_start = current_pos + start;
                let end = actual_start + symbol_name.len();

                if self.is_complete_symbol(line, actual_start, end) {
                    let start_offset = self.get_offset(content, line_idx, actual_start);
                    let end_offset = self.get_offset(content, line_idx, end);

                    /// 检查是否已经添加过这个引用
                    let key = (uri, start_offset, end_offset);
                    if reference_set.insert(key) {
                        references
                            .push(LocationRange { uri: Arc::from(uri), range: Range { start: start_offset, end: end_offset } });
                    }
                }

                current_pos = end;
            }
        }

        references
    }

    /// 检查是否是写引用（赋值）
    fn is_write_reference(&self, line: &str, symbol_start: usize) -> bool {
        let after_symbol = &line[symbol_start..];

        /// 检查后面是否有赋值操作符
        let trimmed = after_symbol.trim_start();
        trimmed.starts_with('=') && !trimmed.starts_with("==")
    }

    /// 转换偏移量为位置
    fn offset_to_position(&self, content: &str, offset: usize) -> oak_lsp::types::SourcePosition {
        let (line, character) = self.get_line_and_column(content, offset);
        oak_lsp::types::SourcePosition { line: line as u32, column: character as u32, length: 0, offset: offset }
    }

    /// Calculate relevance score for a completion item
    fn calculate_relevance(&self, item: &CompletionItem, prefix: &str) -> f64 {
        let label = item.label.as_str();

        // Base score based on prefix match
        let mut score = if label.starts_with(prefix) {
            1.0
        }
        else if label.contains(prefix) {
            0.7
        }
        else if self.fuzzy_match(label, prefix) {
            0.5
        }
        else {
            0.3
        };

        // Adjust score based on item kind
        if let Some(kind) = item.kind {
            match kind {
                oak_lsp::types::CompletionItemKind::Function => score += 0.2,
                oak_lsp::types::CompletionItemKind::Variable => score += 0.15,
                oak_lsp::types::CompletionItemKind::Class => score += 0.1,
                _ => {}
            }
        }

        // Adjust score based on usage frequency
        let usage = self.completion_usage.lock().unwrap();
        if let Some(freq) = usage.get(label) {
            score += (*freq as f64) * 0.01;
        }

        score
    }

    /// Fuzzy match algorithm
    /// Returns true if the prefix is a subsequence of the label
    fn fuzzy_match(&self, label: &str, prefix: &str) -> bool {
        if prefix.is_empty() {
            return true;
        }

        let mut label_iter = label.chars();

        for prefix_char in prefix.chars() {
            match label_iter.find(|&c| c.eq_ignore_ascii_case(&prefix_char)) {
                Some(_) => continue,
                None => return false,
            }
        }

        true
    }

    /// Sort completions by relevance and usage frequency
    fn sort_completions(&self, completions: &mut Vec<CompletionItem>, prefix: &str) {
        completions.sort_by(|a, b| {
            let score_a = self.calculate_relevance(a, prefix);
            let score_b = self.calculate_relevance(b, prefix);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// Extract prefix before cursor for completion
    fn extract_completion_prefix(&self, text: &str, offset: usize) -> String {
        let before_cursor = &text[..offset];
        let mut prefix = String::new();

        for c in before_cursor.chars().rev() {
            if c.is_alphanumeric() || c == '_' || c == '$' {
                prefix.insert(0, c);
            }
            else {
                break;
            }
        }

        prefix
    }
}

impl Default for TypeScriptLanguageService {
    fn default() -> Self {
        Self::new()
    }
}

/// Placeholder language type for TypeScript
pub struct TypeScriptLang;

/// Placeholder token type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeScriptTokenType;

impl oak_core::language::TokenType for TypeScriptTokenType {
    type Role = TypeScriptTokenRole;

    const END_OF_STREAM: Self = TypeScriptTokenType;

    fn role(&self) -> Self::Role {
        TypeScriptTokenRole
    }
}

/// Placeholder token role
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeScriptTokenRole;

impl oak_core::language::TokenRole for TypeScriptTokenRole {
    fn universal(&self) -> oak_core::language::UniversalTokenRole {
        oak_core::language::UniversalTokenRole::Name
    }

    fn name(&self) -> &str {
        "name"
    }
}

/// Placeholder element type
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeScriptElementType;

impl oak_core::language::ElementType for TypeScriptElementType {
    type Role = TypeScriptElementRole;

    fn role(&self) -> Self::Role {
        TypeScriptElementRole
    }
}

/// Placeholder element role
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeScriptElementRole;

impl oak_core::language::ElementRole for TypeScriptElementRole {
    fn universal(&self) -> oak_core::language::UniversalElementRole {
        oak_core::language::UniversalElementRole::Expression
    }

    fn name(&self) -> &str {
        "expression"
    }
}

/// Placeholder typed root
#[derive(Debug)]
pub struct TypeScriptRoot;

impl oak_core::language::Language for TypeScriptLang {
    const NAME: &'static str = "typescript";
    const CATEGORY: oak_core::language::LanguageCategory = oak_core::language::LanguageCategory::Programming;

    type TokenType = TypeScriptTokenType;
    type ElementType = TypeScriptElementType;
    type TypedRoot = TypeScriptRoot;
}

impl LanguageService for TypeScriptLanguageService {
    type Lang = TypeScriptLang;
    type Vfs = MemoryVfs;

    fn vfs(&self) -> &Self::Vfs {
        &self.vfs
    }

    fn workspace(&self) -> &WorkspaceManager {
        &self.workspace
    }

    fn hover(&self, uri: &str, range: Range<usize>) -> impl std::future::Future<Output = Option<Hover>> + Send + '_ {
        let uri = uri.to_string();
        async move {
            let content = self.get_document(&uri).or_else(|| self.get_file_content(&uri))?;

            if let Some(symbol_name) = self.extract_symbol_at_position(&content, range.start) {
                Some(Hover { contents: format!("**{}**\n\nType: any", symbol_name), range: Some(range) })
            }
            else {
                None
            }
        }
    }

    fn completion<'a>(
        &'a self,
        uri: &'a str,
        offset: usize,
    ) -> impl std::future::Future<Output = Vec<CompletionItem>> + Send + 'a {
        async move {
            let _span = tracing::span!(tracing::Level::DEBUG, "completion", uri = uri, offset = offset).entered();

            let content = match self.get_document(uri).or_else(|| self.get_file_content(uri)) {
                Some(c) => c,
                None => return vec![],
            };

            self.update_symbol_table(uri, &content);

            // 提取补全前缀
            let prefix = self.extract_completion_prefix(&content, offset);

            // 检查缓存
            let cache_key = (uri.to_string(), offset, prefix.clone());
            {
                let cache = self.completion_cache.lock().unwrap();
                if let Some(cached) = cache.get(&cache_key) {
                    tracing::debug!("Completion cache hit for uri: {}, offset: {}, prefix: {}", uri, offset, prefix);
                    return cached.clone();
                }
            }

            let mut completions = Vec::new();

            if self.is_member_access_context(&content, offset) {
                if let Some(object_name) = self.parse_member_access(&content, offset) {
                    let member_completions = self.get_member_completions(&object_name, uri);

                    // 缓存结果
                    let mut cache = self.completion_cache.lock().unwrap();
                    cache.insert(cache_key, member_completions.clone());
                    return member_completions;
                }
            }

            if let Some(table) = self.get_symbol_table(uri) {
                for symbol in table.current_scope_symbols() {
                    let kind = self.symbol_kind_to_completion_kind(symbol.kind);
                    let mut detail = String::new();

                    if let Some(ref type_ann) = symbol.type_annotation {
                        detail = format!(": {}", type_ann);
                    }

                    let insert_text = if symbol.kind == SymbolKind::Function {
                        if let Some(ref sig) = symbol.signature {
                            Some(format!("{}{}", symbol.name, sig))
                        }
                        else {
                            Some(format!("{}()", symbol.name))
                        }
                    }
                    else {
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

            self.add_keyword_completions(&mut completions);

            // 缓存结果
            let mut cache = self.completion_cache.lock().unwrap();
            cache.insert(cache_key, completions.clone());

            tracing::debug!(
                "Completion generated {} items for uri: {}, offset: {}, prefix: {}",
                completions.len(),
                uri,
                offset,
                prefix
            );
            completions
        }
    }

    fn definition<'a>(
        &'a self,
        uri: &'a str,
        range: Range<usize>,
    ) -> impl std::future::Future<Output = Vec<LocationRange>> + Send + 'a {
        async move {
            let content = match self.get_document(uri).or_else(|| self.get_file_content(uri)) {
                Some(c) => c,
                None => return vec![],
            };

            let symbol_name = match self.extract_symbol_at_position(&content, range.start) {
                Some(s) => s,
                None => return vec![],
            };

            let mut locations = Vec::new();

            /// 首先尝试从当前文件的符号表获取定义位置
            if let Some(symbol_table) = self.get_symbol_table(uri) {
                if let Some(symbol) = symbol_table.find_by_name(&symbol_name) {
                    locations.push(LocationRange { uri: Arc::from(uri), range: symbol.range.clone() });
                    return locations;
                }
            }

            /// 跨文件查找符号定义
            let all_uris = self.get_all_document_uris();
            for other_uri in all_uris {
                if other_uri == uri {
                    continue;
                }

                if let Some(symbol_table) = self.get_symbol_table(&other_uri) {
                    if let Some(symbol) = symbol_table.find_by_name(&symbol_name) {
                        locations.push(LocationRange { uri: Arc::from(other_uri), range: symbol.range.clone() });
                        return locations;
                    }
                }
            }

            /// 回退到基于文本的查找（当前文件）
            let lines: Vec<&str> = content.lines().collect();
            for (line_idx, line) in lines.iter().enumerate() {
                if self.is_definition_line(line, &symbol_name) {
                    let line_offset = self.get_offset(&content, line_idx, 0);
                    let line_end = self.get_offset(&content, line_idx, line.len());
                    locations.push(LocationRange { uri: Arc::from(uri), range: Range { start: line_offset, end: line_end } });
                    break;
                }
            }

            locations
        }
    }

    fn references<'a>(
        &'a self,
        uri: &'a str,
        range: Range<usize>,
    ) -> impl std::future::Future<Output = Vec<LocationRange>> + Send + 'a {
        async move {
            let content = match self.get_document(uri).or_else(|| self.get_file_content(uri)) {
                Some(c) => c,
                None => return vec![],
            };

            let symbol_name = match self.extract_symbol_at_position(&content, range.start) {
                Some(s) => s,
                None => return vec![],
            };

            let mut references = Vec::new();

            /// 1. 先在当前文件中查找引用
            references.extend(self.find_references_in_file(uri, &content, &symbol_name));

            /// 2. 然后在所有其他文件中查找引用（并行处理）
            let all_uris = self.get_all_document_uris();
            let other_uris: Vec<String> = all_uris.into_iter().filter(|u| u != uri).collect();

            // 并行处理其他文件
            let other_references: Vec<LocationRange> = other_uris
                .par_iter()
                .flat_map(|other_uri| {
                    if let Some(other_content) = self.get_document(other_uri).or_else(|| self.get_file_content(other_uri)) {
                        self.find_references_in_file(other_uri, &other_content, &symbol_name)
                    }
                    else {
                        vec![]
                    }
                })
                .collect();

            references.extend(other_references);

            references
        }
    }

    fn document_symbols<'a>(&'a self, uri: &'a str) -> impl std::future::Future<Output = Vec<StructureItem>> + Send + 'a {
        async move {
            let content = match self.get_document(uri).or_else(|| self.get_file_content(uri)) {
                Some(c) => c,
                None => return vec![],
            };

            let mut symbols = Vec::new();
            let lines: Vec<&str> = content.lines().collect();

            for (line_idx, line) in lines.iter().enumerate() {
                let line_offset = self.get_offset(&content, line_idx, 0);
                let line_end = self.get_offset(&content, line_idx, line.len());
                let range = Range { start: line_offset, end: line_end };

                if line.starts_with("class ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let class_name = parts[1].trim();
                        let name_offset = self.get_offset(&content, line_idx, 6);
                        let name_end = self.get_offset(&content, line_idx, 6 + class_name.len());
                        symbols.push(StructureItem {
                            name: class_name.to_string(),
                            detail: None,
                            role: oak_core::language::UniversalElementRole::Typing,
                            kind: oak_lsp::types::SymbolKind::Class,
                            range,
                            selection_range: Range { start: name_offset, end: name_end },
                            deprecated: false,
                            children: vec![],
                        });
                    }
                }
                else if line.starts_with("function ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let function_name = parts[1].split('(').next().unwrap_or("").trim();
                        let name_offset = self.get_offset(&content, line_idx, 9);
                        let name_end = self.get_offset(&content, line_idx, 9 + function_name.len());
                        symbols.push(StructureItem {
                            name: function_name.to_string(),
                            detail: None,
                            role: oak_core::language::UniversalElementRole::Definition,
                            kind: oak_lsp::types::SymbolKind::Function,
                            range,
                            selection_range: Range { start: name_offset, end: name_end },
                            deprecated: false,
                            children: vec![],
                        });
                    }
                }
                else if line.starts_with("const ") || line.starts_with("let ") || line.starts_with("var ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let variable_name = parts[1].split(|c| c == '=' || c == ';' || c == ':').next().unwrap_or("").trim();
                        let keyword_len = parts[0].len();
                        let name_offset = self.get_offset(&content, line_idx, keyword_len + 1);
                        let name_end = self.get_offset(&content, line_idx, keyword_len + 1 + variable_name.len());
                        symbols.push(StructureItem {
                            name: variable_name.to_string(),
                            detail: None,
                            role: oak_core::language::UniversalElementRole::Binding,
                            kind: oak_lsp::types::SymbolKind::Variable,
                            range,
                            selection_range: Range { start: name_offset, end: name_end },
                            deprecated: false,
                            children: vec![],
                        });
                    }
                }
            }

            symbols
        }
    }

    fn formatting<'a>(&'a self, uri: &'a str) -> impl std::future::Future<Output = Vec<TextEdit>> + Send + 'a {
        async move {
            let content = match self.get_document(uri).or_else(|| self.get_file_content(uri)) {
                Some(c) => c,
                None => return vec![],
            };

            let options = FormatOptions::default();
            let cache_key = (uri.to_string(), options.clone());

            // 检查缓存
            {
                let cache = self.format_cache.lock().unwrap();
                if let Some(cached) = cache.get(&cache_key) {
                    if *cached != content {
                        let end_offset = content.len();
                        return vec![TextEdit { range: Range { start: 0, end: end_offset }, new_text: cached.clone() }];
                    }
                    else {
                        return vec![];
                    }
                }
            }

            let formatted_text = format_code_with_options(&content, options);

            // 缓存结果
            let mut cache = self.format_cache.lock().unwrap();
            cache.insert(cache_key, formatted_text.clone());

            if formatted_text != content {
                let end_offset = content.len();
                vec![TextEdit { range: Range { start: 0, end: end_offset }, new_text: formatted_text }]
            }
            else {
                vec![]
            }
        }
    }

    fn range_formatting<'a>(
        &'a self,
        uri: &'a str,
        range: Range<usize>,
    ) -> impl std::future::Future<Output = Vec<TextEdit>> + Send + 'a {
        async move {
            let content = match self.get_document(uri).or_else(|| self.get_file_content(uri)) {
                Some(c) => c,
                None => return vec![],
            };

            let options = FormatOptions::default();
            let formatted_text = format_code_range(&content, range.start, range.end, options);

            if formatted_text != content {
                vec![TextEdit { range: Range { start: 0, end: content.len() }, new_text: formatted_text }]
            }
            else {
                vec![]
            }
        }
    }

    fn initialize<'a>(&'a self, _params: InitializeParams) -> impl std::future::Future<Output = ()> + Send + 'a {
        async move {}
    }

    fn initialized<'a>(&'a self) -> impl std::future::Future<Output = ()> + Send + 'a {
        async move {}
    }

    fn shutdown<'a>(&'a self) -> impl std::future::Future<Output = ()> + Send + 'a {
        async move {}
    }

    fn code_action<'a>(
        &'a self,
        _uri: &'a str,
        _range: Range<usize>,
    ) -> impl std::future::Future<Output = Vec<CodeAction>> + Send + 'a {
        async move { vec![] }
    }

    fn document_highlight<'a>(
        &'a self,
        _uri: &'a str,
        _range: Range<usize>,
    ) -> impl std::future::Future<Output = Vec<DocumentHighlight>> + Send + 'a {
        async move { vec![] }
    }

    fn folding_ranges(&self, _uri: &str) -> impl std::future::Future<Output = Vec<FoldingRange>> + Send + '_ {
        async move { vec![] }
    }

    fn semantic_tokens<'a>(&'a self, uri: &'a str) -> impl std::future::Future<Output = Option<SemanticTokens>> + Send + 'a {
        async move {
            let content = self.get_document(uri).or_else(|| self.get_file_content(uri))?;

            /// 获取符号表用于语义高亮
            let symbol_table = self.get_symbol_table(uri).unwrap_or_default();

            /// 使用 highlighter 模块进行语法高亮
            /// 这里我们返回一个空的实现
            Some(SemanticTokens { data: Vec::new(), result_id: None })
        }
    }

    fn inlay_hint<'a>(
        &'a self,
        uri: &'a str,
        range: Range<usize>,
    ) -> impl std::future::Future<Output = Vec<InlayHint>> + Send + 'a {
        async move {
            let content = match self.get_document(uri).or_else(|| self.get_file_content(uri)) {
                Some(c) => c,
                None => return vec![],
            };

            /// 获取或创建符号表
            let symbol_table = self.get_symbol_table(uri).unwrap_or_default();

            /// 创建内联提示提供者
            let provider = InlayHintProvider::new(symbol_table);

            /// 提供内联提示
            let hints = provider.provide_hints(&content, range);

            /// 转换为 LSP InlayHint 格式
            hints
                .into_iter()
                .map(|hint| {
                    let position = self.offset_to_position(&content, hint.position);
                    InlayHint {
                        position,
                        label: hint.label,
                        kind: Some(match hint.kind {
                            crate::lsp::inlay_hints::InlayHintKind::ParameterName => oak_lsp::types::InlayHintKind::Parameter,
                            _ => oak_lsp::types::InlayHintKind::Type,
                        }),
                        padding_left: Some(hint.kind == crate::lsp::inlay_hints::InlayHintKind::ParameterName),
                        padding_right: Some(false),
                        tooltip: None,
                    }
                })
                .collect()
        }
    }

    fn signature_help<'a>(
        &'a self,
        _uri: &'a str,
        _range: Range<usize>,
    ) -> impl std::future::Future<Output = Option<SignatureHelp>> + Send + 'a {
        async move { None }
    }

    fn rename<'a>(
        &'a self,
        _uri: &'a str,
        _range: Range<usize>,
        _new_name: String,
    ) -> impl std::future::Future<Output = Option<WorkspaceEdit>> + Send + 'a {
        async move { None }
    }

    fn type_definition<'a>(
        &'a self,
        _uri: &'a str,
        _range: Range<usize>,
    ) -> impl std::future::Future<Output = Vec<LocationRange>> + Send + 'a {
        async move { vec![] }
    }

    fn implementation<'a>(
        &'a self,
        _uri: &'a str,
        _range: Range<usize>,
    ) -> impl std::future::Future<Output = Vec<LocationRange>> + Send + 'a {
        async move { vec![] }
    }

    fn workspace_symbols<'a>(&'a self, query: String) -> impl std::future::Future<Output = Vec<WorkspaceSymbol>> + Send + 'a {
        async move { self.workspace.symbols.query(&query).into_iter().map(|s| WorkspaceSymbol::from(s)).collect() }
    }

    fn diagnostics<'a>(&'a self, uri: &'a str) -> impl std::future::Future<Output = Vec<Diagnostic>> + Send + 'a {
        async move {
            let content = match self.get_document(uri).or_else(|| self.get_file_content(uri)) {
                Some(c) => c,
                None => return vec![],
            };

            // 检查缓存
            let content_hash = self.calculate_hash(&content);
            let cache_key = uri.to_string();
            {
                let cache = self.diagnostic_cache.lock().unwrap();
                if let Some(cached) = cache.get(&cache_key) {
                    // 简单的缓存策略，实际项目中可能需要更复杂的缓存失效机制
                    return cached.clone();
                }
            }

            /// 获取或更新符号表
            let symbol_table = self.get_symbol_table(uri).unwrap_or_else(|| {
                self.update_symbol_table(uri, &content);
                self.get_symbol_table(uri).unwrap_or_default()
            });

            /// 获取修改范围
            let change_range = {
                let changes = self.document_changes.read().unwrap();
                changes.get(uri).cloned().unwrap_or(None)
            };

            /// 创建诊断分析器
            let analyzer = DiagnosticAnalyzer::new(symbol_table);

            /// 分析诊断（使用增量诊断）
            let diagnostics = analyzer.analyze_with_range(&content, change_range);

            /// 转换为 LSP Diagnostic 格式
            let result: Vec<Diagnostic> = diagnostics
                .into_iter()
                .map(|d| {
                    let (start_line, start_col) = self.get_line_and_column(&content, d.range.start);
                    let (end_line, end_col) = self.get_line_and_column(&content, d.range.end);

                    Diagnostic {
                        range: d.range,
                        severity: Some(match d.level {
                            DiagnosticLevel::Error => oak_lsp::types::DiagnosticSeverity::Error,
                            DiagnosticLevel::Warning => oak_lsp::types::DiagnosticSeverity::Warning,
                            DiagnosticLevel::Info => oak_lsp::types::DiagnosticSeverity::Information,
                            DiagnosticLevel::Hint => oak_lsp::types::DiagnosticSeverity::Hint,
                        }),
                        code: d.code,
                        source: Some(d.source),
                        message: d.message,
                    }
                })
                .collect();

            // 缓存结果
            let mut cache = self.diagnostic_cache.lock().unwrap();
            cache.insert(cache_key, result.clone());

            result
        }
    }
}

/// Start the LSP server
pub async fn start_server() {
    let service = std::sync::Arc::new(TypeScriptLanguageService::new());
    let server = oak_lsp::LspServer::new(service);
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    if let Err(_e) = server.run(stdin, stdout).await {
        // Server error handling
    }
}
