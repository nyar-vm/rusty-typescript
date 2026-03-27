#![feature(new_range_api)]
#![warn(missing_docs)]

//! TypeScript LSP implementation using oak-lsp framework.

pub mod lsp;
pub mod mcp;

pub use lsp::{TypeScriptLanguageService, start_server};
pub use lsp::diagnostics::{Diagnostic, DiagnosticLevel, DiagnosticAnalyzer};
pub use lsp::inlay_hints::{InlayHint, InlayHintKind, InlayHintProvider};
pub use lsp::symbols::{Symbol, SymbolKind, SymbolTable, SymbolCollector};
