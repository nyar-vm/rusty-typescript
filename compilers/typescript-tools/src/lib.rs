#![warn(missing_docs)]

pub mod compiler;
/// Rusty TypeScript CLI 工具集
///
/// 提供编译、检查、格式化、打包等开发工具
// 暂时注释掉 commands 模块，等实现后再添加
// pub mod commands;
pub mod config;
pub mod errors;
pub mod formatter;
pub mod utils;

/// 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 工具集名称
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// 工具集描述
pub const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");

/// 初始化工具集
pub fn init() {
    env_logger::init();
    log::debug!("Initializing {}-{}", NAME, VERSION);
}
