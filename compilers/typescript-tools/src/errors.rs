/// 错误定义模块
///
/// 定义工具链中使用的错误类型
use serde_json;
use std::{error::Error, fmt, io};
use toml::de;

/// 配置错误
#[derive(Debug)]
pub enum ConfigError {
    /// IO 错误
    Io(io::Error),
    /// TOML 解析错误
    TomlParse(de::Error),
    /// JSON 解析错误
    JsonParse(serde_json::Error),
    /// 配置继承循环检测
    CircularExtends(String),
    /// 未找到基础配置文件
    BaseConfigNotFound(String),
    /// 文件模式匹配错误
    PatternMatchError(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(err) => write!(f, "IO error: {}", err),
            ConfigError::TomlParse(err) => write!(f, "TOML parse error: {}", err),
            ConfigError::JsonParse(err) => write!(f, "JSON parse error: {}", err),
            ConfigError::CircularExtends(path) => write!(f, "Circular extends detected: {}", path),
            ConfigError::BaseConfigNotFound(path) => write!(f, "Base config not found: {}", path),
            ConfigError::PatternMatchError(msg) => write!(f, "Pattern match error: {}", msg),
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::Io(err) => Some(err),
            ConfigError::TomlParse(err) => Some(err),
            ConfigError::JsonParse(err) => Some(err),
            ConfigError::CircularExtends(_) => None,
            ConfigError::BaseConfigNotFound(_) => None,
            ConfigError::PatternMatchError(_) => None,
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(err: io::Error) -> Self {
        ConfigError::Io(err)
    }
}

impl From<de::Error> for ConfigError {
    fn from(err: de::Error) -> Self {
        ConfigError::TomlParse(err)
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        ConfigError::JsonParse(err)
    }
}

/// 编译错误
#[derive(Debug)]
pub enum CompileError {
    /// IO 错误
    Io(io::Error),
    /// 配置错误
    Config(ConfigError),
    /// 编译失败
    CompilationFailed(String),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::Io(err) => write!(f, "IO error: {}", err),
            CompileError::Config(err) => write!(f, "Config error: {}", err),
            CompileError::CompilationFailed(msg) => write!(f, "Compilation failed: {}", msg),
        }
    }
}

impl Error for CompileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            CompileError::Io(err) => Some(err),
            CompileError::Config(err) => Some(err),
            CompileError::CompilationFailed(_) => None,
        }
    }
}

impl From<io::Error> for CompileError {
    fn from(err: io::Error) -> Self {
        CompileError::Io(err)
    }
}

impl From<ConfigError> for CompileError {
    fn from(err: ConfigError) -> Self {
        CompileError::Config(err)
    }
}

/// 格式化错误
#[derive(Debug)]
pub enum FormatError {
    /// IO 错误
    Io(io::Error),
    /// 格式化失败
    FormattingFailed(String),
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::Io(err) => write!(f, "IO error: {}", err),
            FormatError::FormattingFailed(msg) => write!(f, "Formatting failed: {}", msg),
        }
    }
}

impl Error for FormatError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            FormatError::Io(err) => Some(err),
            FormatError::FormattingFailed(_) => None,
        }
    }
}

impl From<io::Error> for FormatError {
    fn from(err: io::Error) -> Self {
        FormatError::Io(err)
    }
}

/// 通用结果类型
pub type ConfigResult<T> = Result<T, ConfigError>;
pub type CompileResult<T> = Result<T, CompileError>;
pub type FormatResult<T> = Result<T, FormatError>;
