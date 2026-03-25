/// 错误定义模块
///
/// 定义工具链中使用的错误类型
use std::error::Error;
use std::{fmt, io};
use toml::de;

/// 配置错误
#[derive(Debug)]
pub enum ConfigError {
    /// IO 错误
    Io(io::Error),
    /// 解析错误
    Parse(de::Error),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(err) => write!(f, "IO error: {}", err),
            ConfigError::Parse(err) => write!(f, "Parse error: {}", err),
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::Io(err) => Some(err),
            ConfigError::Parse(err) => Some(err),
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
        ConfigError::Parse(err)
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
