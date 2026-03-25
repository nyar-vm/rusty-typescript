#![feature(new_range_api)]
#![warn(missing_docs)]

/// TypeScript LSP implementation
pub mod lsp;
pub use lsp::{TypeScriptLanguageService, start_server};
pub mod mcp;
