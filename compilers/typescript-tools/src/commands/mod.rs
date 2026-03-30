/// 命令模块
///
/// 包含各种 CLI 命令的实现
/// 构建命令
/// 编译 TypeScript 代码
pub mod build;

/// 检查命令
/// 检查 TypeScript 代码的语法和类型错误
pub mod check;

/// 格式化命令
/// 格式化 TypeScript 代码
pub mod fmt;

/// 运行命令
/// 执行 TypeScript 脚本
pub mod run;

/// 命令执行结果
#[derive(Debug)]
pub struct CommandResult {
    /// 是否成功
    pub success: bool,
    /// 退出代码
    pub exit_code: i32,
    /// 输出消息
    pub message: String,
}

impl CommandResult {
    /// 创建成功的结果
    pub fn success(message: impl Into<String>) -> Self {
        Self { success: true, exit_code: 0, message: message.into() }
    }

    /// 创建失败的结果
    pub fn error(exit_code: i32, message: impl Into<String>) -> Self {
        Self { success: false, exit_code, message: message.into() }
    }
}
