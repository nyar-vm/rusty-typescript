/// WebIDL 类型定义模块

/// WebIDL 解析结果
pub struct WebIdlResult {
    /// WebIDL AST 根节点
    pub root: oak_idl::ast::IdlRoot,
    /// 生成的 TypeScript 类型定义
    pub typescript: String,
}

/// WebIDL 错误
pub enum WebIdlError {
    /// 解析错误
    ParseError(String),
    /// 转换错误
    ConvertError(String),
}

impl std::fmt::Display for WebIdlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WebIdlError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            WebIdlError::ConvertError(msg) => write!(f, "Convert error: {}", msg),
        }
    }
}

impl std::fmt::Debug for WebIdlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

impl std::error::Error for WebIdlError {}
