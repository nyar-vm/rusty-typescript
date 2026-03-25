//! exception 模块
//!
pub struct ExceptionHandler {
    /// 处理的起始指令位置
    pub start_ip: usize,
    /// 处理的结束指令位置
    pub end_ip: usize,
    /// 处理器的指令位置
    pub handler_ip: usize,
    /// 异常变量名
    pub exception_var: Option<String>,
    /// 是否有 finally 块
    pub has_finally: bool,
    /// finally 块的指令位置
    pub finally_ip: Option<usize>,
}

/// 模块实例
#[derive(Debug, Clone)]