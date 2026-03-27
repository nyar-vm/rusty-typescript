#![doc = include_str!("readme.md")]

use core::range::Range;
use oak_core::language::Language;
use oak_lsp::{
    LanguageService, WorkspaceManager,
    types::{
        CodeAction, CompletionItem, Diagnostic, DocumentHighlight, FoldingRange, Hover,
        InitializeParams, InlayHint, LocationRange, SemanticTokens, SignatureHelp,
        StructureItem, TextEdit, WorkspaceEdit, WorkspaceSymbol,
    },
};
use oak_vfs::{MemoryVfs, Vfs, WritableVfs};
use std::{collections::HashMap, sync::Arc};
use typescript::TypeScriptLanguage;

pub mod formatter;

use formatter::{FormatOptions, format_code_with_options, format_range as format_code_range};

/// TypeScript language service implementing oak-lsp's LanguageService trait.
pub struct TypeScriptLanguageService {
    vfs: MemoryVfs,
    workspace: WorkspaceManager,
    documents: Arc<std::sync::Mutex<HashMap<String, String>>>,
}

impl TypeScriptLanguageService {
    /// Creates a new `TypeScriptLanguageService`.
    pub fn new() -> Self {
        Self {
            vfs: MemoryVfs::new(),
            workspace: WorkspaceManager::new(),
            documents: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    /// Get document content by URI
    async fn get_document(&self, uri: &str) -> Option<String> {
        let documents = self.documents.lock().unwrap();
        documents.get(uri).cloned()
    }

    /// Set document content by URI
    async fn set_document(&self, uri: String, content: String) {
        let mut documents = self.documents.lock().unwrap();
        documents.insert(uri.clone(), content.clone());
        // Also update VFS
        self.vfs.write_file(&uri, content.into_bytes());
    }

    /// Get file content from disk if not in memory
    fn get_file_content(&self, uri: &str) -> Option<String> {
        self.vfs.read_file_to_string(uri)
    }

    /// Extract symbol at position
    fn extract_symbol_at_position(&self, text: &str, offset: usize) -> Option<String> {
        let mut symbol = String::new();
        let mut current_pos = offset;

        // Search backwards
        while current_pos > 0 {
            let ch = text.chars().nth(current_pos - 1).unwrap_or(' ');
            if ch.is_alphanumeric() || ch == '_' || ch == '$' {
                symbol.insert(0, ch);
                current_pos -= 1;
            } else {
                break;
            }
        }

        // Search forwards
        current_pos = offset;
        while current_pos < text.len() {
            let ch = text.chars().nth(current_pos).unwrap_or(' ');
            if ch.is_alphanumeric() || ch == '_' || ch == '$' {
                symbol.push(ch);
                current_pos += 1;
            } else {
                break;
            }
        }

        if symbol.is_empty() {
            None
        } else {
            Some(symbol)
        }
    }

    /// Get offset from line and column
    fn get_offset(&self, text: &str, line: usize, column: usize) -> usize {
        let mut offset = 0;
        let mut current_line = 0;

        for (i, c) in text.char_indices() {
            if current_line == line && offset >= column {
                return i;
            }

            if c == '\n' {
                current_line += 1;
                offset = 0;
            } else {
                offset += 1;
            }
        }

        text.len()
    }

    /// Get line and column from offset
    fn get_line_and_column(&self, text: &str, offset: usize) -> (usize, usize) {
        let mut line = 0;
        let mut col = 0;

        for (i, c) in text.char_indices() {
            if i >= offset {
                break;
            }
            if c == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }

        (line, col)
    }

    /// Check if it's a complete symbol
    fn is_complete_symbol(&self, line: &str, start: usize, end: usize) -> bool {
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
    fn parse_format_options(&self, tab_size: u32, insert_spaces: bool) -> FormatOptions {
        FormatOptions {
            indent_size: tab_size,
            use_tabs: !insert_spaces,
            line_width: 80,                // Default
            space_before_brace: true,      // Default
            space_after_comma: true,       // Default
            space_after_semicolon: true,   // Default
            spaces_around_operators: true, // Default
            spaces_in_function: false,     // Default
            spaces_in_object: true,        // Default
        }
    }

    /// Infer variable type
    fn infer_variable_type(&self, init_expr: &str) -> String {
        let expr = init_expr.trim();

        if expr.starts_with('"') || expr.starts_with('\'') {
            return "string".to_string();
        } else if expr == "true" || expr == "false" {
            return "boolean".to_string();
        } else if expr.starts_with('{') && expr.ends_with('}') {
            return "object".to_string();
        } else if expr.starts_with('[') && expr.ends_with(']') {
            return "any[]".to_string();
        } else if expr.parse::<i64>().is_ok() || expr.parse::<f64>().is_ok() {
            return "number".to_string();
        } else if expr == "null" {
            return "null".to_string();
        } else if expr == "undefined" {
            return "undefined".to_string();
        } else if expr.starts_with("function") || expr.contains("=>") {
            return "Function".to_string();
        }

        "any".to_string()
    }
}

impl LanguageService for TypeScriptLanguageService {
    type Lang = TypeScriptLanguage;
    type Vfs = MemoryVfs;

    fn vfs(&self) -> &Self::Vfs {
        &self.vfs
    }

    fn workspace(&self) -> &WorkspaceManager {
        &self.workspace
    }

    async fn hover(&self, uri: &str, range: Range<usize>) -> Option<Hover> {
        let content = self.get_document(uri).await.or_else(|| self.get_file_content(uri))?;

        if let Some(symbol_name) = self.extract_symbol_at_position(&content, range.start) {
            Some(Hover {
                contents: format!("**{}**\n\nType: any", symbol_name),
                range: Some(range),
            })
        } else {
            None
        }
    }

    async fn completion(&self, uri: &str, offset: usize) -> Vec<CompletionItem> {
        let content = match self.get_document(uri).await.or_else(|| self.get_file_content(uri)) {
            Some(c) => c,
            None => return vec![],
        };

        let _ = offset;
        let mut completions = Vec::new();

        // Add keywords
        let keywords = vec![
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

        completions
    }

    async fn definition(&self, uri: &str, range: Range<usize>) -> Vec<LocationRange> {
        let content = match self.get_document(uri).await.or_else(|| self.get_file_content(uri)) {
            Some(c) => c,
            None => return vec![],
        };

        let symbol_name = match self.extract_symbol_at_position(&content, range.start) {
            Some(s) => s,
            None => return vec![],
        };

        let mut locations = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            if line.contains(&format!(" {} ", symbol_name))
                || line.starts_with(&format!("{} ", symbol_name))
                || line.ends_with(&format!(" {}", symbol_name))
            {
                let line_offset = self.get_offset(&content, line_idx, 0);
                let line_end = self.get_offset(&content, line_idx, line.len());
                locations.push(LocationRange {
                    uri: Arc::from(uri),
                    range: Range {
                        start: line_offset,
                        end: line_end,
                    },
                });
                break; // Return first definition
            }
        }

        locations
    }

    async fn references(&self, uri: &str, range: Range<usize>) -> Vec<LocationRange> {
        let content = match self.get_document(uri).await.or_else(|| self.get_file_content(uri)) {
            Some(c) => c,
            None => return vec![],
        };

        let symbol_name = match self.extract_symbol_at_position(&content, range.start) {
            Some(s) => s,
            None => return vec![],
        };

        let mut references = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let mut current_pos = 0;
            while let Some(start) = line[current_pos..].find(&symbol_name) {
                let actual_start = current_pos + start;
                let end = actual_start + symbol_name.len();

                if self.is_complete_symbol(line, actual_start, end) {
                    let start_offset = self.get_offset(&content, line_idx, actual_start);
                    let end_offset = self.get_offset(&content, line_idx, end);
                    references.push(LocationRange {
                        uri: Arc::from(uri),
                        range: Range {
                            start: start_offset,
                            end: end_offset,
                        },
                    });
                }

                current_pos = end;
            }
        }

        references
    }

    async fn document_symbols(&self, uri: &str) -> Vec<StructureItem> {
        let content = match self.get_document(uri).await.or_else(|| self.get_file_content(uri)) {
            Some(c) => c,
            None => return vec![],
        };

        let mut symbols = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            let line_offset = self.get_offset(&content, line_idx, 0);
            let line_end = self.get_offset(&content, line_idx, line.len());
            let range = Range {
                start: line_offset,
                end: line_end,
            };

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
                        selection_range: Range {
                            start: name_offset,
                            end: name_end,
                        },
                        deprecated: false,
                        children: vec![],
                    });
                }
            } else if line.starts_with("function ") {
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
                        selection_range: Range {
                            start: name_offset,
                            end: name_end,
                        },
                        deprecated: false,
                        children: vec![],
                    });
                }
            } else if line.starts_with("const ") || line.starts_with("let ") || line.starts_with("var ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let variable_name = parts[1]
                        .split(|c| c == '=' || c == ';' || c == ':')
                        .next()
                        .unwrap_or("")
                        .trim();
                    let keyword_len = parts[0].len();
                    let name_offset = self.get_offset(&content, line_idx, keyword_len + 1);
                    let name_end = self.get_offset(&content, line_idx, keyword_len + 1 + variable_name.len());
                    symbols.push(StructureItem {
                        name: variable_name.to_string(),
                        detail: None,
                        role: oak_core::language::UniversalElementRole::Binding,
                        kind: oak_lsp::types::SymbolKind::Variable,
                        range,
                        selection_range: Range {
                            start: name_offset,
                            end: name_end,
                        },
                        deprecated: false,
                        children: vec![],
                    });
                }
            }
        }

        symbols
    }

    async fn formatting(&self, uri: &str) -> Vec<TextEdit> {
        let content = match self.get_document(uri).await.or_else(|| self.get_file_content(uri)) {
            Some(c) => c,
            None => return vec![],
        };

        let options = FormatOptions::default();
        let formatted_text = format_code_with_options(&content, options);

        if formatted_text != content {
            let end_offset = content.len();
            vec![TextEdit {
                range: Range { start: 0, end: end_offset },
                new_text: formatted_text,
            }]
        } else {
            vec![]
        }
    }

    async fn range_formatting(&self, uri: &str, range: Range<usize>) -> Vec<TextEdit> {
        let content = match self.get_document(uri).await.or_else(|| self.get_file_content(uri)) {
            Some(c) => c,
            None => return vec![],
        };

        let options = FormatOptions::default();
        let formatted_text = format_code_range(&content, range.start, range.end, options);

        if formatted_text != &content[range.start..range.end] {
            vec![TextEdit {
                range,
                new_text: formatted_text[range.start..range.end].to_string(),
            }]
        } else {
            vec![]
        }
    }

    async fn initialize(&self, _params: InitializeParams) {}

    async fn initialized(&self) {}

    async fn shutdown(&self) {}

    async fn did_open(&self, uri: &str, content: String) {
        self.set_document(uri.to_string(), content).await;
    }

    async fn did_change(&self, uri: &str, content: String) {
        self.set_document(uri.to_string(), content).await;
    }

    async fn did_close(&self, uri: &str) {
        let mut documents = self.documents.lock().unwrap();
        documents.remove(uri);
    }

    async fn code_action(&self, _uri: &str, _range: Range<usize>) -> Vec<CodeAction> {
        vec![]
    }

    async fn document_highlight(&self, _uri: &str, _range: Range<usize>) -> Vec<DocumentHighlight> {
        vec![]
    }

    async fn folding_ranges(&self, _uri: &str) -> Vec<FoldingRange> {
        vec![]
    }

    async fn semantic_tokens(&self, _uri: &str) -> Option<SemanticTokens> {
        None
    }

    async fn inlay_hint(&self, _uri: &str, _range: Range<usize>) -> Vec<InlayHint> {
        vec![]
    }

    async fn signature_help(&self, _uri: &str, _range: Range<usize>) -> Option<SignatureHelp> {
        None
    }

    async fn rename(&self, _uri: &str, _range: Range<usize>, _new_name: String) -> Option<WorkspaceEdit> {
        None
    }

    async fn type_definition(&self, _uri: &str, _range: Range<usize>) -> Vec<LocationRange> {
        vec![]
    }

    async fn implementation(&self, _uri: &str, _range: Range<usize>) -> Vec<LocationRange> {
        vec![]
    }

    async fn workspace_symbols(&self, query: String) -> Vec<WorkspaceSymbol> {
        self.workspace
            .symbols
            .query(&query)
            .into_iter()
            .map(|s| WorkspaceSymbol::from(s))
            .collect()
    }

    async fn diagnostics(&self, _uri: &str) -> Vec<Diagnostic> {
        vec![]
    }
}

/// Start the LSP server
pub async fn start_server() {
    let service = Arc::new(TypeScriptLanguageService::new());
    let server = oak_lsp::LspServer::new(service);
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    if let Err(e) = server.run(stdin, stdout).await {
        log::error!("LSP server error: {}", e);
    }
}

use std::sync::Arc;
