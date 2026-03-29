#![feature(new_range_api)]
#![warn(missing_docs)]

//! TypeScript LSP implementation using oak-lsp framework.

pub mod lsp;
pub mod mcp;

pub use lsp::{
    TypeScriptLanguageService,
    diagnostics::{Diagnostic, DiagnosticAnalyzer, DiagnosticLevel},
    inlay_hints::{InlayHint, InlayHintKind, InlayHintProvider},
    start_server,
    symbols::{Symbol, SymbolCollector, SymbolKind, SymbolTable},
};
