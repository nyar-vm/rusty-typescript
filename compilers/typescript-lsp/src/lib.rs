#![feature(new_range_api)]
#![warn(missing_docs)]

//! TypeScript LSP implementation using oak-lsp framework.

pub mod lsp;
pub mod mcp;

pub use lsp::{TypeScriptLanguageService, start_server};
