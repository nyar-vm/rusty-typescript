#![warn(missing_docs)]

use tracing::debug;
use tracing_subscriber::{
    filter::EnvFilter,
    fmt::{Layer, format::FmtSpan},
    prelude::*,
};

pub mod commands;
pub mod compiler;
pub mod config;
pub mod errors;
pub mod formatter;
pub mod utils;

pub use compiler::incremental::{CompileCache, DependencyGraph, IncrementalCompiler};

/// 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 工具集名称
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// 工具集描述
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// 初始化工具集
pub fn init() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("info"));
    let fmt_layer = Layer::new().with_span_events(FmtSpan::NONE);
    tracing_subscriber::registry().with(env_filter).with(fmt_layer).init();
    debug!("Initializing {}-{}", NAME, VERSION);
}

/// 执行 TypeScript 脚本
///
/// # 参数
/// - `script`: 要执行的 TypeScript 脚本字符串
///
/// # 返回
/// 返回脚本执行结果的字符串表示，或错误信息
pub fn execute_script(script: &str) -> Result<String, String> {
    compiler::execute_script(script)
}

/// 执行 TypeScript 文件
///
/// # 参数
/// - `file`: 要执行的 TypeScript 文件路径
///
/// # 返回
/// 返回脚本执行结果的字符串表示，或错误信息
pub fn execute_file(file: &std::path::PathBuf) -> Result<String, String> {
    compiler::execute_file(file)
}
