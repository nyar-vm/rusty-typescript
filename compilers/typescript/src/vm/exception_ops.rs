//! 异常处理模块
//!
//! 提供 try-catch-finally 异常处理机制。

use typescript_types::{TsError, TsValue};

use super::{exception::ExceptionHandler, frame::CallFrame};

/// 异常操作执行器
pub struct ExceptionOperations;

impl ExceptionOperations {
    /// 开始 try 块
    pub fn try_start(
        current_ip: usize,
        handler_ip: usize,
        finally_ip: Option<usize>,
        exception_var: Option<String>,
        exception_handlers: &mut Vec<ExceptionHandler>,
    ) -> Result<(), TsError> {
        let handler = ExceptionHandler {
            start_ip: current_ip,
            end_ip: handler_ip,
            handler_ip,
            exception_var,
            has_finally: finally_ip.is_some(),
            finally_ip,
        };
        exception_handlers.push(handler);
        Ok(())
    }

    /// 结束 try 块
    pub fn try_end(exception_handlers: &mut Vec<ExceptionHandler>) -> Result<(), TsError> {
        exception_handlers.pop();
        Ok(())
    }

    /// 抛出异常
    pub fn throw_exception(stack: &mut Vec<TsValue>) -> Result<(), TsError> {
        if let Some(value) = stack.pop() {
            Err(TsError::Other(format!("Exception: {}", value.to_string())))
        }
        else {
            Err(TsError::TypeError("Stack underflow".to_string()))
        }
    }

    /// 查找异常处理器
    pub fn find_exception_handler(ip: usize, exception_handlers: &[ExceptionHandler]) -> Option<ExceptionHandler> {
        for handler in exception_handlers {
            if handler.start_ip <= ip && ip < handler.end_ip {
                return Some(handler.clone());
            }
        }
        None
    }

    /// 处理异常
    pub fn handle_exception(
        exception: TsError,
        handler: &ExceptionHandler,
        call_stack: &mut Vec<CallFrame>,
    ) -> Result<usize, TsError> {
        if let Some(var_name) = &handler.exception_var {
            let error_value = TsValue::Error(format!("{}", exception));
            if let Some(frame) = call_stack.last_mut() {
                frame.set_local(var_name.as_str(), error_value);
            }
        }

        Ok(handler.handler_ip)
    }
}
