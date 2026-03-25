#![doc = include_str!("readme.md")]

use std::{collections::HashMap, fs::File, io::Read, sync::Arc};
use tower_lsp::{Client, LanguageServer, LspService, Server, jsonrpc::Result, lsp_types::*};

pub mod formatter;

use crate::lsp::formatter::{FormatOptions, format_code_with_options, format_range as format_code_range};

/// TypeScript language service.
#[derive(Clone)]
pub struct TypeScriptLanguageService {
    client: Arc<Client>,
    documents: Arc<std::sync::Mutex<HashMap<String, String>>>,
}

impl TypeScriptLanguageService {
    /// Creates a new `TypeScriptLanguageService`.
    pub fn new(client: Arc<Client>) -> Self {
        Self { client, documents: Arc::new(std::sync::Mutex::new(HashMap::new())) }
    }

    /// Get document content by URI
    async fn get_document(&self, uri: &str) -> Option<String> {
        let documents = self.documents.lock().unwrap();
        documents.get(uri).cloned()
    }

    /// Set document content by URI
    async fn set_document(&self, uri: String, content: String) {
        let mut documents = self.documents.lock().unwrap();
        documents.insert(uri, content);
    }

    /// Get file content from disk if not in memory
    fn get_file_content(&self, uri: &str) -> Option<String> {
        // Convert URI to file path
        let file_path = uri.replace("file://", "");
        let mut file = File::open(file_path).ok()?;
        let mut content = String::new();
        file.read_to_string(&mut content).ok()?;
        Some(content)
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
            }
            else {
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
            }
            else {
                break;
            }
        }

        if symbol.is_empty() { None } else { Some(symbol) }
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
            }
            else {
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
            }
            else {
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
    fn parse_format_options(&self, options: &FormattingOptions) -> FormatOptions {
        FormatOptions {
            indent_size: options.tab_size,
            use_tabs: !options.insert_spaces,
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
        }
        else if expr == "true" || expr == "false" {
            return "boolean".to_string();
        }
        else if expr.starts_with('{') && expr.ends_with('}') {
            return "object".to_string();
        }
        else if expr.starts_with('[') && expr.ends_with(']') {
            return "any[]".to_string();
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
            return "Function".to_string();
        }

        "any".to_string()
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for TypeScriptLanguageService {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::INCREMENTAL)),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec!["/".to_string(), ".".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                workspace_symbol_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                document_range_formatting_provider: Some(OneOf::Left(true)),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                    ..Default::default()
                }),
                type_definition_provider: Some(TypeDefinitionProviderCapability::Simple(true)),
                implementation_provider: Some(ImplementationProviderCapability::Simple(true)),
                rename_provider: Some(OneOf::Left(true)),
                document_highlight_provider: Some(OneOf::Left(true)),
                folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),
                semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(
                    SemanticTokensOptions {
                        legend: SemanticTokensLegend {
                            token_types: vec![
                                SemanticTokenType::VARIABLE,
                                SemanticTokenType::FUNCTION,
                                SemanticTokenType::CLASS,
                                SemanticTokenType::INTERFACE,
                                SemanticTokenType::TYPE,
                                SemanticTokenType::KEYWORD,
                                SemanticTokenType::STRING,
                                SemanticTokenType::NUMBER,
                                SemanticTokenType::COMMENT,
                            ],
                            token_modifiers: vec![],
                        },
                        range: Some(true),
                        full: Some(SemanticTokensFullOptions::Bool(true)),
                        ..Default::default()
                    },
                )),
                inlay_hint_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(tower_lsp::lsp_types::MessageType::INFO, "TypeScript LSP server initialized").await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.set_document(params.text_document.uri.to_string(), params.text_document.text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(content) = params.content_changes.into_iter().last() {
            self.set_document(params.text_document.uri.to_string(), content.text).await;
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let position = params.text_document_position_params.position;

        if let Some(content) = self.get_document(&uri).await.or_else(|| self.get_file_content(&uri)) {
            let offset = self.get_offset(&content, position.line as usize, position.character as usize);
            if let Some(symbol_name) = self.extract_symbol_at_position(&content, offset) {
                // Simple hover implementation
                let hover = Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("**{symbol_name}**\n\nType: any"),
                    }),
                    range: None,
                };
                return Ok(Some(hover));
            }
        }

        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;

        if let Some(content) = self.get_document(&uri).await.or_else(|| self.get_file_content(&uri)) {
            let _offset = self.get_offset(&content, position.line as usize, position.character as usize);

            // Simple completion implementation
            let mut completions = Vec::new();

            // Add keywords
            let keywords = vec![
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

            for keyword in keywords {
                completions.push(CompletionItem {
                    label: keyword.to_string(),
                    kind: Some(tower_lsp::lsp_types::CompletionItemKind::KEYWORD),
                    detail: Some("Keyword".to_string()),
                    documentation: None,
                    insert_text: Some(keyword.to_string()),
                    ..Default::default()
                });
            }

            return Ok(Some(CompletionResponse::Array(completions)));
        }

        Ok(None)
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let position = params.text_document_position_params.position;

        if let Some(content) = self.get_document(&uri).await.or_else(|| self.get_file_content(&uri)) {
            let offset = self.get_offset(&content, position.line as usize, position.character as usize);
            if let Some(symbol_name) = self.extract_symbol_at_position(&content, offset) {
                // Simple definition implementation
                let lines: Vec<&str> = content.lines().collect();
                for (line_idx, line) in lines.iter().enumerate() {
                    if line.contains(&format!(" {} ", symbol_name))
                        || line.starts_with(&format!("{} ", symbol_name))
                        || line.ends_with(&format!(" {}", symbol_name))
                    {
                        let location = Location {
                            uri: params.text_document_position_params.text_document.uri.clone(),
                            range: Range {
                                start: Position { line: line_idx as u32, character: 0 },
                                end: Position { line: line_idx as u32, character: line.len() as u32 },
                            },
                        };
                        return Ok(Some(GotoDefinitionResponse::Scalar(location)));
                    }
                }
            }
        }

        Ok(None)
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;

        if let Some(content) = self.get_document(&uri).await.or_else(|| self.get_file_content(&uri)) {
            let offset = self.get_offset(&content, position.line as usize, position.character as usize);
            if let Some(symbol_name) = self.extract_symbol_at_position(&content, offset) {
                // Simple references implementation
                let mut references = Vec::new();
                let lines: Vec<&str> = content.lines().collect();

                for (line_idx, line) in lines.iter().enumerate() {
                    let mut current_pos = 0;
                    while let Some(start) = line[current_pos..].find(&symbol_name) {
                        let actual_start = current_pos + start;
                        let end = actual_start + symbol_name.len();

                        if self.is_complete_symbol(line, actual_start, end) {
                            let location = Location {
                                uri: params.text_document_position.text_document.uri.clone(),
                                range: Range {
                                    start: Position { line: line_idx as u32, character: actual_start as u32 },
                                    end: Position { line: line_idx as u32, character: end as u32 },
                                },
                            };
                            references.push(location);
                        }

                        current_pos = end;
                    }
                }

                return Ok(Some(references));
            }
        }

        Ok(None)
    }

    async fn document_symbol(&self, params: DocumentSymbolParams) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri.to_string();

        if let Some(content) = self.get_document(&uri).await.or_else(|| self.get_file_content(&uri)) {
            // Simple document symbol implementation
            let mut symbols = Vec::new();
            let lines: Vec<&str> = content.lines().collect();

            for (line_idx, line) in lines.iter().enumerate() {
                if line.starts_with("class ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let class_name = parts[1].trim();
                        symbols.push(DocumentSymbol {
                            name: class_name.to_string(),
                            kind: tower_lsp::lsp_types::SymbolKind::CLASS,
                            range: Range {
                                start: Position { line: line_idx as u32, character: 0 },
                                end: Position { line: line_idx as u32, character: line.len() as u32 },
                            },
                            selection_range: Range {
                                start: Position { line: line_idx as u32, character: 6 },
                                end: Position { line: line_idx as u32, character: (6 + class_name.len()) as u32 },
                            },
                            children: None,
                            detail: None,
                            tags: None,
                            deprecated: None,
                        });
                    }
                }
                else if line.starts_with("function ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let function_name = parts[1].split('(').next().unwrap_or("").trim();
                        symbols.push(DocumentSymbol {
                            name: function_name.to_string(),
                            kind: tower_lsp::lsp_types::SymbolKind::FUNCTION,
                            range: Range {
                                start: Position { line: line_idx as u32, character: 0 },
                                end: Position { line: line_idx as u32, character: line.len() as u32 },
                            },
                            selection_range: Range {
                                start: Position { line: line_idx as u32, character: 9 },
                                end: Position { line: line_idx as u32, character: (9 + function_name.len()) as u32 },
                            },
                            children: None,
                            detail: None,
                            tags: None,
                            deprecated: None,
                        });
                    }
                }
                else if line.starts_with("const ") || line.starts_with("let ") || line.starts_with("var ") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let variable_name = parts[1].split(|c| c == '=' || c == ';' || c == ':').next().unwrap_or("").trim();
                        symbols.push(DocumentSymbol {
                            name: variable_name.to_string(),
                            kind: tower_lsp::lsp_types::SymbolKind::VARIABLE,
                            range: Range {
                                start: Position { line: line_idx as u32, character: 0 },
                                end: Position { line: line_idx as u32, character: line.len() as u32 },
                            },
                            selection_range: Range {
                                start: Position { line: line_idx as u32, character: (parts[0].len() + 1) as u32 },
                                end: Position {
                                    line: line_idx as u32,
                                    character: (parts[0].len() + 1 + variable_name.len()) as u32,
                                },
                            },
                            children: None,
                            detail: None,
                            tags: None,
                            deprecated: None,
                        });
                    }
                }
            }

            return Ok(Some(DocumentSymbolResponse::Nested(symbols)));
        }

        Ok(None)
    }

    async fn code_action(&self, _params: CodeActionParams) -> Result<Option<Vec<CodeActionOrCommand>>> {
        // Simple code action implementation
        Ok(None)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri.to_string();

        if let Some(content) = self.get_document(&uri).await.or_else(|| self.get_file_content(&uri)) {
            let options = self.parse_format_options(&params.options);
            let formatted_text = format_code_with_options(&content, options);

            if formatted_text != content {
                let edit = TextEdit {
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: content.lines().count() as u32, character: 0 },
                    },
                    new_text: formatted_text,
                };
                return Ok(Some(vec![edit]));
            }
        }

        Ok(None)
    }

    async fn range_formatting(&self, params: DocumentRangeFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri.to_string();

        if let Some(content) = self.get_document(&uri).await.or_else(|| self.get_file_content(&uri)) {
            let options = self.parse_format_options(&params.options);
            let start_offset =
                self.get_offset(&content, params.range.start.line as usize, params.range.start.character as usize);
            let end_offset = self.get_offset(&content, params.range.end.line as usize, params.range.end.character as usize);

            let formatted_text = format_code_range(&content, start_offset, end_offset, options);

            if formatted_text != content {
                let edit = TextEdit { range: params.range, new_text: formatted_text[start_offset..end_offset].to_string() };
                return Ok(Some(vec![edit]));
            }
        }

        Ok(None)
    }

    async fn signature_help(&self, _params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        // Simple signature help implementation
        Ok(None)
    }

    async fn goto_type_definition(&self, _params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
        // Simple type definition implementation
        Ok(None)
    }

    async fn document_highlight(&self, _params: DocumentHighlightParams) -> Result<Option<Vec<DocumentHighlight>>> {
        // Simple document highlight implementation
        Ok(None)
    }

    async fn folding_range(&self, _params: FoldingRangeParams) -> Result<Option<Vec<FoldingRange>>> {
        // Simple folding range implementation
        Ok(None)
    }

    async fn semantic_tokens_full(&self, _params: SemanticTokensParams) -> Result<Option<SemanticTokensResult>> {
        // Simple semantic tokens implementation
        Ok(None)
    }

    async fn inlay_hint(&self, _params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        // Simple inlay hint implementation
        Ok(None)
    }
}

/// Start the LSP server
pub async fn start_server() {
    let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());
    let (service, socket) = LspService::new(|client| TypeScriptLanguageService::new(client.into()));
    Server::new(stdin, stdout, socket).serve(service).await;
}
