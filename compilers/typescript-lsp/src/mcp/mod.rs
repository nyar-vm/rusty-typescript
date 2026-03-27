#![doc = include_str!("readme.md")]

use crate::lsp::TypeScriptLanguageService;
use std::sync::Arc;

/// MCP service for TypeScript.
pub struct TypeScriptMcpService {
    lsp_service: Arc<TypeScriptLanguageService>,
}

impl TypeScriptMcpService {
    /// Creates a new `TypeScriptMcpService`.
    pub fn new(lsp_service: Arc<TypeScriptLanguageService>) -> Self {
        Self { lsp_service }
    }

    /// Gets the underlying LSP service.
    pub fn lsp_service(&self) -> &TypeScriptLanguageService {
        &self.lsp_service
    }
}

/// Starts the TypeScript MCP service.
pub async fn serve_typescript_mcp() {
    let lsp_service = Arc::new(TypeScriptLanguageService::new());
    let _mcp_service = TypeScriptMcpService::new(lsp_service);

    // MCP service implementation
    // This is a placeholder until the necessary dependencies are available
    unimplemented!("MCP service not yet implemented")
}
