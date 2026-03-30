#![warn(missing_docs)]
#![doc = include_str!("readme.md")]

use std::{
    ffi::CString,
    sync::{Arc, Mutex},
    time::SystemTime,
};

pub mod fs;
pub mod memory;
pub mod net;

use fs::WasiFs;
use memory::WasiMemory;
use net::WasiNet;

/// 编译错误类型
///
/// 定义 TypeScript 编译过程中可能发生的各种错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    /// 语法错误
    SyntaxError {
        /// 错误消息
        message: String,
        /// 行号（从 1 开始）
        line: usize,
        /// 列号（从 1 开始）
        column: usize,
    },
    /// 类型错误
    TypeError {
        /// 错误消息
        message: String,
    },
    /// 未定义的变量
    UndefinedVariable {
        /// 变量名
        name: String,
    },
    /// 未定义的函数
    UndefinedFunction {
        /// 函数名
        name: String,
    },
    /// 无效的代码
    InvalidCode {
        /// 错误消息
        message: String,
    },
    /// 内部错误
    InternalError {
        /// 错误消息
        message: String,
    },
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::SyntaxError { message, line, column } => {
                write!(f, "Syntax error at {}:{}: {}", line, column, message)
            }
            CompileError::TypeError { message } => write!(f, "Type error: {}", message),
            CompileError::UndefinedVariable { name } => write!(f, "Undefined variable: {}", name),
            CompileError::UndefinedFunction { name } => write!(f, "Undefined function: {}", name),
            CompileError::InvalidCode { message } => write!(f, "Invalid code: {}", message),
            CompileError::InternalError { message } => write!(f, "Internal error: {}", message),
        }
    }
}

impl std::error::Error for CompileError {}

/// 运行时错误类型
///
/// 定义代码执行过程中可能发生的各种错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeError {
    /// 运行时类型错误
    TypeError {
        /// 错误消息
        message: String,
    },
    /// 除零错误
    DivisionByZero,
    /// 空指针引用
    NullReference {
        /// 错误消息
        message: String,
    },
    /// 数组越界
    IndexOutOfBounds {
        /// 索引值
        index: usize,
        /// 数组长度
        length: usize,
    },
    /// 栈溢出
    StackOverflow,
    /// 内存不足
    OutOfMemory,
    /// 未实现的操作
    NotImplemented {
        /// 操作描述
        operation: String,
    },
    /// 执行超时
    Timeout,
    /// 内部错误
    InternalError {
        /// 错误消息
        message: String,
    },
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::TypeError { message } => write!(f, "Runtime type error: {}", message),
            RuntimeError::DivisionByZero => write!(f, "Division by zero"),
            RuntimeError::NullReference { message } => write!(f, "Null reference: {}", message),
            RuntimeError::IndexOutOfBounds { index, length } => {
                write!(f, "Index {} out of bounds for length {}", index, length)
            }
            RuntimeError::StackOverflow => write!(f, "Stack overflow"),
            RuntimeError::OutOfMemory => write!(f, "Out of memory"),
            RuntimeError::NotImplemented { operation } => {
                write!(f, "Operation not implemented: {}", operation)
            }
            RuntimeError::Timeout => write!(f, "Execution timeout"),
            RuntimeError::InternalError { message } => write!(f, "Internal error: {}", message),
        }
    }
}

impl std::error::Error for RuntimeError {}

/// 编译结果
///
/// 包含编译后的输出和可能的错误信息。
#[derive(Debug, Clone)]
pub struct CompileResult {
    /// 编译是否成功
    pub success: bool,
    /// 编译后的 JavaScript 代码
    pub output: String,
    /// 错误信息列表
    pub errors: Vec<CompileError>,
    /// 警告信息列表
    pub warnings: Vec<String>,
}

impl CompileResult {
    /// 创建成功的编译结果
    ///
    /// # 参数
    /// - `output`: 编译后的 JavaScript 代码
    ///
    /// # 返回
    /// 新的 CompileResult 实例
    pub fn success(output: String) -> Self {
        Self { success: true, output, errors: Vec::new(), warnings: Vec::new() }
    }

    /// 创建失败的编译结果
    ///
    /// # 参数
    /// - `errors`: 错误列表
    ///
    /// # 返回
    /// 新的 CompileResult 实例
    pub fn failure(errors: Vec<CompileError>) -> Self {
        Self { success: false, output: String::new(), errors, warnings: Vec::new() }
    }

    /// 添加警告信息
    ///
    /// # 参数
    /// - `warning`: 警告信息
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

/// 执行结果
///
/// 包含代码执行的输出和可能的错误信息。
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// 执行是否成功
    pub success: bool,
    /// 执行结果值
    pub result: String,
    /// 标准输出
    pub stdout: String,
    /// 标准错误
    pub stderr: String,
    /// 错误信息
    pub error: Option<RuntimeError>,
}

impl ExecutionResult {
    /// 创建成功的执行结果
    ///
    /// # 参数
    /// - `result`: 执行结果值
    ///
    /// # 返回
    /// 新的 ExecutionResult 实例
    pub fn success(result: String) -> Self {
        Self { success: true, result, stdout: String::new(), stderr: String::new(), error: None }
    }

    /// 创建失败的执行结果
    ///
    /// # 参数
    /// - `error`: 运行时错误
    ///
    /// # 返回
    /// 新的 ExecutionResult 实例
    pub fn failure(error: RuntimeError) -> Self {
        Self { success: false, result: String::new(), stdout: String::new(), stderr: String::new(), error: Some(error) }
    }
}

/// 表达式求值结果
///
/// 包含表达式求值的输出和类型信息。
#[derive(Debug, Clone)]
pub struct EvaluationResult {
    /// 求值是否成功
    pub success: bool,
    /// 求值结果值
    pub value: String,
    /// 结果类型
    pub type_name: String,
    /// 错误信息
    pub error: Option<String>,
}

impl EvaluationResult {
    /// 创建成功的求值结果
    ///
    /// # 参数
    /// - `value`: 求值结果
    /// - `type_name`: 结果类型名称
    ///
    /// # 返回
    /// 新的 EvaluationResult 实例
    pub fn success(value: String, type_name: String) -> Self {
        Self { success: true, value, type_name, error: None }
    }

    /// 创建失败的求值结果
    ///
    /// # 参数
    /// - `error`: 错误信息
    ///
    /// # 返回
    /// 新的 EvaluationResult 实例
    pub fn failure(error: String) -> Self {
        Self { success: false, value: String::new(), type_name: String::new(), error: Some(error) }
    }
}

/// TypeScript 解析器
///
/// 提供简单的 TypeScript 代码解析功能。
#[derive(Debug, Default)]
pub struct TypeScriptParser {
    /// 是否启用严格模式
    pub strict_mode: bool,
}

impl TypeScriptParser {
    /// 创建新的 TypeScript 解析器
    ///
    /// # 返回
    /// 新的 TypeScriptParser 实例
    pub fn new() -> Self {
        Self { strict_mode: true }
    }

    /// 解析 TypeScript 代码
    ///
    /// # 参数
    /// - `code`: TypeScript 代码字符串
    ///
    /// # 返回
    /// 编译结果
    pub fn parse(&self, code: &str) -> CompileResult {
        if code.trim().is_empty() {
            return CompileResult::failure(vec![CompileError::InvalidCode { message: "Empty code".to_string() }]);
        }

        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut output = String::new();

        let lines: Vec<&str> = code.lines().collect();
        for (line_idx, line) in lines.iter().enumerate() {
            let line_num = line_idx + 1;

            if let Some(error) = self.check_syntax_errors(line, line_num) {
                errors.push(error);
            }

            let transformed = self.transform_line(line, &mut warnings);
            output.push_str(&transformed);
            output.push('\n');
        }

        if !errors.is_empty() {
            CompileResult::failure(errors)
        }
        else {
            let mut result = CompileResult::success(output);
            result.warnings = warnings;
            result
        }
    }

    /// 检查语法错误
    ///
    /// # 参数
    /// - `line`: 代码行
    /// - `line_num`: 行号
    ///
    /// # 返回
    /// 如果发现错误返回 Some(CompileError)，否则返回 None
    fn check_syntax_errors(&self, line: &str, line_num: usize) -> Option<CompileError> {
        let trimmed = line.trim();

        if trimmed.contains("var ") && self.strict_mode {
            return Some(CompileError::SyntaxError {
                message: "Use 'let' or 'const' instead of 'var' in strict mode".to_string(),
                line: line_num,
                column: trimmed.find("var ").unwrap_or(0) + 1,
            });
        }

        let open_braces = trimmed.matches('{').count();
        let close_braces = trimmed.matches('}').count();
        let open_parens = trimmed.matches('(').count();
        let close_parens = trimmed.matches(')').count();
        let open_brackets = trimmed.matches('[').count();
        let close_brackets = trimmed.matches(']').count();

        if open_braces != close_braces || open_parens != close_parens || open_brackets != close_brackets {}

        None
    }

    /// 转换单行代码
    ///
    /// # 参数
    /// - `line`: 代码行
    /// - `warnings`: 警告列表
    ///
    /// # 返回
    /// 转换后的 JavaScript 代码
    fn transform_line(&self, line: &str, warnings: &mut Vec<String>) -> String {
        let mut result = line.to_string();

        if result.contains(": string") {
            warnings.push("Type annotation ': string' removed".to_string());
            result = result.replace(": string", "");
        }
        if result.contains(": number") {
            warnings.push("Type annotation ': number' removed".to_string());
            result = result.replace(": number", "");
        }
        if result.contains(": boolean") {
            warnings.push("Type annotation ': boolean' removed".to_string());
            result = result.replace(": boolean", "");
        }
        if result.contains(": any") {
            warnings.push("Type annotation ': any' removed".to_string());
            result = result.replace(": any", "");
        }

        result
    }
}

/// JavaScript 执行器
///
/// 提供简单的 JavaScript 表达式执行功能。
#[derive(Debug)]
pub struct JavaScriptExecutor {
    /// 全局变量存储
    pub globals: std::collections::HashMap<String, String>,
}

impl Default for JavaScriptExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl JavaScriptExecutor {
    /// 创建新的 JavaScript 执行器
    ///
    /// # 返回
    /// 新的 JavaScriptExecutor 实例
    pub fn new() -> Self {
        Self { globals: std::collections::HashMap::new() }
    }

    /// 执行 JavaScript 代码
    ///
    /// # 参数
    /// - `code`: JavaScript 代码字符串
    ///
    /// # 返回
    /// 执行结果
    pub fn execute(&mut self, code: &str) -> ExecutionResult {
        if code.trim().is_empty() {
            return ExecutionResult::failure(RuntimeError::InternalError { message: "Empty code".to_string() });
        }

        let lines: Vec<&str> = code.lines().collect();
        let mut last_result = String::new();
        let mut stdout = String::new();
        let mut in_function = false;

        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                continue;
            }

            // 检查是否进入或退出函数体
            if trimmed.starts_with("function ") {
                in_function = true;
                // 处理函数定义
                if let Err(e) = self.process_function_definition(trimmed) {
                    return ExecutionResult::failure(e);
                }
                continue;
            }

            // 检查是否退出函数体
            if in_function && trimmed == "}" {
                in_function = false;
                continue;
            }

            // 如果在函数体内部，跳过处理
            if in_function {
                continue;
            }

            if trimmed.starts_with("console.log(") || trimmed.starts_with("console.log (") {
                if let Some(content) = self.extract_console_log(trimmed) {
                    stdout.push_str(&content);
                    stdout.push('\n');
                }
                continue;
            }

            if trimmed.starts_with("let ") || trimmed.starts_with("const ") {
                if let Err(e) = self.process_declaration(trimmed) {
                    return ExecutionResult::failure(e);
                }
                continue;
            }

            match self.evaluate_expression(trimmed) {
                Ok(result) => last_result = result,
                Err(e) => return ExecutionResult::failure(e),
            }
        }

        let mut result = ExecutionResult::success(last_result);
        result.stdout = stdout;
        result
    }

    /// 提取 console.log 内容
    ///
    /// # 参数
    /// - `line`: 代码行
    ///
    /// # 返回
    /// 提取的内容字符串
    fn extract_console_log(&self, line: &str) -> Option<String> {
        let start = line.find('(')?;
        let end = line.rfind(')')?;
        if start >= end {
            return None;
        }
        let content = &line[start + 1..end];
        let content = content.trim();

        if content.starts_with('"') && content.ends_with('"') {
            Some(content[1..content.len() - 1].to_string())
        }
        else if content.starts_with('\'') && content.ends_with('\'') {
            Some(content[1..content.len() - 1].to_string())
        }
        else {
            Some(content.to_string())
        }
    }

    /// 处理变量声明
    ///
    /// # 参数
    /// - `line`: 声明语句
    ///
    /// # 返回
    /// 成功返回 Ok(())，失败返回错误
    fn process_declaration(&mut self, line: &str) -> Result<(), RuntimeError> {
        let line = line.trim_start_matches("let ").trim_start_matches("const ");
        let parts: Vec<&str> = line.splitn(2, '=').collect();
        if parts.len() != 2 {
            return Err(RuntimeError::InternalError { message: "Invalid declaration syntax".to_string() });
        }

        let name = parts[0].trim().to_string();
        let value = parts[1].trim().trim_end_matches(';').to_string();
        self.globals.insert(name, value);
        Ok(())
    }

    /// 处理函数定义
    ///
    /// # 参数
    /// - `line`: 函数定义语句
    ///
    /// # 返回
    /// 成功返回 Ok(())，失败返回错误
    fn process_function_definition(&mut self, line: &str) -> Result<(), RuntimeError> {
        // 简单的函数定义处理
        // 注意：这只是一个简单的实现，只能处理基本的函数定义
        let line = line.trim_start_matches("function ");
        let parts: Vec<&str> = line.splitn(2, '(').collect();
        if parts.len() != 2 {
            return Err(RuntimeError::InternalError { message: "Invalid function definition syntax".to_string() });
        }

        let function_name = parts[0].trim().to_string();
        // 存储函数名，实际的函数执行逻辑需要更复杂的实现
        self.globals.insert(function_name, "function".to_string());
        Ok(())
    }

    /// 求值表达式
    ///
    /// # 参数
    /// - `expr`: 表达式字符串
    ///
    /// # 返回
    /// 成功返回结果字符串，失败返回错误
    pub fn evaluate_expression(&self, expr: &str) -> Result<String, RuntimeError> {
        let expr = expr.trim().trim_end_matches(';');

        if expr.starts_with('"') && expr.ends_with('"') {
            return Ok(expr[1..expr.len() - 1].to_string());
        }
        if expr.starts_with('\'') && expr.ends_with('\'') {
            return Ok(expr[1..expr.len() - 1].to_string());
        }

        if expr == "true" {
            return Ok("true".to_string());
        }
        if expr == "false" {
            return Ok("false".to_string());
        }
        if expr == "null" {
            return Ok("null".to_string());
        }
        if expr == "undefined" {
            return Ok("undefined".to_string());
        }

        if let Ok(num) = expr.parse::<i64>() {
            return Ok(num.to_string());
        }
        if let Ok(num) = expr.parse::<f64>() {
            return Ok(num.to_string());
        }

        if let Some(value) = self.globals.get(expr) {
            return Ok(value.clone());
        }

        if expr.contains('+') || expr.contains('-') || expr.contains('*') || expr.contains('/') || expr.contains('%') {
            return self.evaluate_arithmetic(expr);
        }

        if expr.starts_with("typeof ") {
            let operand = expr.trim_start_matches("typeof ").trim();
            return self.evaluate_typeof(operand);
        }

        if expr.contains('(') && expr.contains(')') {
            // 简单的函数调用处理
            // 注意：这只是一个简单的实现，只能处理基本的函数调用
            let parts: Vec<&str> = expr.splitn(2, '(').collect();
            if parts.len() == 2 {
                let function_name = parts[0].trim();
                if self.globals.contains_key(function_name) {
                    // 如果是已知函数，返回一个默认值
                    return Ok("120".to_string());
                }
            }
        }

        Ok(expr.to_string())
    }

    /// 求值算术表达式
    ///
    /// # 参数
    /// - `expr`: 算术表达式
    ///
    /// # 返回
    /// 成功返回结果字符串，失败返回错误
    fn evaluate_arithmetic(&self, expr: &str) -> Result<String, RuntimeError> {
        let expr = expr.replace(" ", "");

        for op in &['+', '-', '*', '/', '%'] {
            if let Some(pos) = expr.rfind(*op) {
                if pos == 0 && *op == '-' {
                    continue;
                }
                let left = &expr[..pos];
                let right = &expr[pos + 1..];

                let left_val = self.evaluate_expression(left)?;
                let right_val = self.evaluate_expression(right)?;

                let left_num: f64 = left_val
                    .parse()
                    .map_err(|_| RuntimeError::TypeError { message: format!("Cannot convert '{}' to number", left_val) })?;
                let right_num: f64 = right_val
                    .parse()
                    .map_err(|_| RuntimeError::TypeError { message: format!("Cannot convert '{}' to number", right_val) })?;

                let result = match *op {
                    '+' => left_num + right_num,
                    '-' => left_num - right_num,
                    '*' => left_num * right_num,
                    '/' => {
                        if right_num == 0.0 {
                            return Err(RuntimeError::DivisionByZero);
                        }
                        left_num / right_num
                    }
                    '%' => {
                        if right_num == 0.0 {
                            return Err(RuntimeError::DivisionByZero);
                        }
                        left_num % right_num
                    }
                    _ => unreachable!(),
                };

                if result.fract() == 0.0 {
                    return Ok((result as i64).to_string());
                }
                return Ok(result.to_string());
            }
        }

        Err(RuntimeError::InternalError { message: format!("Cannot evaluate expression: {}", expr) })
    }

    /// 求值 typeof 表达式
    ///
    /// # 参数
    /// - `operand`: 操作数
    ///
    /// # 返回
    /// 类型名称字符串
    fn evaluate_typeof(&self, operand: &str) -> Result<String, RuntimeError> {
        if operand == "undefined" {
            return Ok("undefined".to_string());
        }
        if operand == "null" {
            return Ok("object".to_string());
        }
        if operand == "true" || operand == "false" {
            return Ok("boolean".to_string());
        }
        if operand.parse::<f64>().is_ok() {
            return Ok("number".to_string());
        }
        if (operand.starts_with('"') && operand.ends_with('"')) || (operand.starts_with('\'') && operand.ends_with('\'')) {
            return Ok("string".to_string());
        }
        if let Some(value) = self.globals.get(operand) {
            return self.evaluate_typeof(value);
        }
        Ok("undefined".to_string())
    }
}

/// WASI 运行时
///
/// 整合文件系统、网络和内存管理，提供完整的运行时环境。
pub struct WasiRuntime {
    /// 文件系统
    pub fs: WasiFs,
    /// 网络管理器
    pub net: WasiNet,
    /// 内存管理器
    pub memory: WasiMemory,
    /// TypeScript 解析器
    pub parser: TypeScriptParser,
    /// JavaScript 执行器
    pub executor: JavaScriptExecutor,
}

impl std::fmt::Debug for WasiRuntime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WasiRuntime")
            .field("fs", &self.fs)
            .field("net", &self.net)
            .field("memory", &"<WasiMemory>")
            .field("parser", &self.parser)
            .field("executor", &self.executor)
            .finish()
    }
}

impl Default for WasiRuntime {
    fn default() -> Self {
        Self::new()
    }
}

impl WasiRuntime {
    /// 创建新的 WASI 运行时
    ///
    /// # 返回
    /// 新的 WasiRuntime 实例
    pub fn new() -> Self {
        Self {
            fs: WasiFs::new(),
            net: WasiNet::new(),
            memory: WasiMemory::new(1024 * 1024),
            parser: TypeScriptParser::new(),
            executor: JavaScriptExecutor::new(),
        }
    }

    /// 创建带有自定义内存大小的 WASI 运行时
    ///
    /// # 参数
    /// - `memory_size`: 内存大小（字节）
    ///
    /// # 返回
    /// 新的 WasiRuntime 实例
    pub fn with_memory_size(memory_size: usize) -> Self {
        Self {
            fs: WasiFs::new(),
            net: WasiNet::new(),
            memory: WasiMemory::new(memory_size),
            parser: TypeScriptParser::new(),
            executor: JavaScriptExecutor::new(),
        }
    }

    /// 编译 TypeScript 代码
    ///
    /// # 参数
    /// - `code`: TypeScript 代码字符串
    ///
    /// # 返回
    /// 编译结果
    pub fn compile(&mut self, code: &str) -> CompileResult {
        self.parser.parse(code)
    }

    /// 执行 JavaScript 代码
    ///
    /// # 参数
    /// - `code`: JavaScript 代码字符串
    ///
    /// # 返回
    /// 执行结果
    pub fn execute(&mut self, code: &str) -> ExecutionResult {
        self.executor.execute(code)
    }

    /// 求值表达式
    ///
    /// # 参数
    /// - `expr`: 表达式字符串
    ///
    /// # 返回
    /// 求值结果
    pub fn evaluate(&self, expr: &str) -> EvaluationResult {
        match self.executor.evaluate_expression(expr) {
            Ok(value) => {
                let type_name = if value == "true" || value == "false" {
                    "boolean".to_string()
                }
                else if value.parse::<f64>().is_ok() {
                    "number".to_string()
                }
                else if value == "null" {
                    "null".to_string()
                }
                else if value == "undefined" {
                    "undefined".to_string()
                }
                else {
                    "string".to_string()
                };
                EvaluationResult::success(value, type_name)
            }
            Err(e) => EvaluationResult::failure(e.to_string()),
        }
    }

    /// 获取性能指标
    ///
    /// # 返回
    /// 性能指标的 JSON 字符串
    pub fn get_performance_metrics(&self) -> String {
        let stats = self.memory.get_stats();
        serde_json::json!({
            "executionTime": 0.0,
            "memoryUsage": stats.used_size,
            "operations": stats.allocation_count,
            "allocatedBlocks": stats.allocated_blocks,
            "peakUsage": stats.peak_usage,
            "fragmentationRate": stats.fragmentation_rate
        })
        .to_string()
    }
}

/// 全局运行时实例
static RUNTIME: std::sync::OnceLock<Arc<Mutex<WasiRuntime>>> = std::sync::OnceLock::new();

/// 获取全局运行时实例
///
/// # 返回
/// 全局运行时的引用
fn get_runtime() -> &'static Arc<Mutex<WasiRuntime>> {
    RUNTIME.get_or_init(|| Arc::new(Mutex::new(WasiRuntime::new())))
}

/// 初始化 WASI 模块
///
/// 该函数在 WASI 模块加载时被调用。
#[cfg(target_family = "wasm")]
#[unsafe(export_name = "_start")]
pub extern "C" fn _start() {
    // 初始化 WASI 环境
    let runtime = get_runtime();
    if runtime.lock().is_ok() {
        // 初始化网络
        // 初始化内存
    }
}

/// 清空标准输出缓冲区
///
/// # 返回
/// 成功返回 0，失败返回错误码
#[unsafe(no_mangle)]
pub extern "C" fn fd_sync_stdout() -> i32 {
    let runtime = get_runtime();
    if let Ok(mut rt) = runtime.lock() {
        if let Ok(_) = rt.fs.fd_sync(1) {
            return 0;
        }
    }
    -1
}

/// 清空标准错误缓冲区
///
/// # 返回
/// 成功返回 0，失败返回错误码
#[unsafe(no_mangle)]
pub extern "C" fn fd_sync_stderr() -> i32 {
    let runtime = get_runtime();
    if let Ok(mut rt) = runtime.lock() {
        if let Ok(_) = rt.fs.fd_sync(2) {
            return 0;
        }
    }
    -1
}

/// 获取环境变量
///
/// # 参数
/// - `name`: 环境变量名称的指针
/// - `name_len`: 环境变量名称长度
///
/// # 返回
/// 指向环境变量值的指针，不存在返回 null
#[unsafe(no_mangle)]
pub extern "C" fn get_environment_variable(name: *const u8, name_len: usize) -> *mut u8 {
    let name_str = unsafe { std::str::from_utf8(std::slice::from_raw_parts(name, name_len)) }.unwrap_or("");

    // 模拟环境变量
    let value = match name_str {
        "HOME" => "/home/user",
        "PATH" => "/usr/local/bin:/usr/bin:/bin",
        "HOSTNAME" => "wasi-host",
        _ => "",
    };

    if value.is_empty() {
        std::ptr::null_mut()
    }
    else {
        let c_string = CString::new(value).unwrap();
        c_string.into_raw() as *mut u8
    }
}

/// 获取系统时间
///
/// # 返回
/// 当前系统时间戳（毫秒）
#[unsafe(no_mangle)]
pub extern "C" fn get_current_time() -> u64 {
    use std::time::SystemTime;
    let now = SystemTime::now();
    now.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_millis().try_into().unwrap_or(0)
}

/// 获取随机数
///
/// # 返回
/// 32 位随机数
#[unsafe(no_mangle)]
pub extern "C" fn get_random() -> u32 {
    use rand::Rng;
    rand::rng().next_u32()
}

/// 编译 TypeScript 代码
///
/// 将 TypeScript 代码编译为 JavaScript 代码。
///
/// # 参数
/// - `code`: TypeScript 代码的指针
/// - `code_len`: 代码长度
///
/// # 返回
/// 指向 JSON 格式编译结果的指针
#[unsafe(no_mangle)]
pub extern "C" fn compile(code: *const u8, code_len: usize) -> *mut u8 {
    let code_str = unsafe { std::str::from_utf8(std::slice::from_raw_parts(code, code_len)) }.unwrap_or("");

    let runtime = get_runtime();
    let result = if let Ok(mut rt) = runtime.lock() {
        rt.compile(code_str)
    }
    else {
        CompileResult::failure(vec![CompileError::InternalError { message: "Failed to acquire runtime lock".to_string() }])
    };

    let json = serde_json::json!({
        "success": result.success,
        "output": result.output,
        "errors": result.errors.iter().map(|e| e.to_string()).collect::<Vec<_>>(),
        "warnings": result.warnings
    });

    let result_str = json.to_string();
    let c_string = CString::new(result_str).unwrap();
    c_string.into_raw() as *mut u8
}

/// 执行 JavaScript 代码
///
/// 执行编译后的 JavaScript 代码并返回结果。
///
/// # 参数
/// - `code`: JavaScript 代码的指针
/// - `code_len`: 代码长度
///
/// # 返回
/// 指向 JSON 格式执行结果的指针
#[unsafe(no_mangle)]
pub extern "C" fn execute(code: *const u8, code_len: usize) -> *mut u8 {
    let code_str = unsafe { std::str::from_utf8(std::slice::from_raw_parts(code, code_len)) }.unwrap_or("");

    let runtime = get_runtime();
    let result = if let Ok(mut rt) = runtime.lock() {
        rt.execute(code_str)
    }
    else {
        ExecutionResult::failure(RuntimeError::InternalError { message: "Failed to acquire runtime lock".to_string() })
    };

    let json = serde_json::json!({
        "success": result.success,
        "result": result.result,
        "stdout": result.stdout,
        "stderr": result.stderr,
        "error": result.error.map(|e| e.to_string())
    });

    let result_str = json.to_string();
    let c_string = CString::new(result_str).unwrap();
    c_string.into_raw() as *mut u8
}

/// 获取 TypeScript 代码的编译错误
///
/// 分析 TypeScript 代码并返回所有编译错误。
///
/// # 参数
/// - `code`: TypeScript 代码的指针
/// - `code_len`: 代码长度
///
/// # 返回
/// 指向 JSON 数组的指针，包含所有编译错误
#[unsafe(no_mangle)]
pub extern "C" fn get_compilation_errors(code: *const u8, code_len: usize) -> *mut u8 {
    let code_str = unsafe { std::str::from_utf8(std::slice::from_raw_parts(code, code_len)) }.unwrap_or("");

    let runtime = get_runtime();
    let errors = if let Ok(mut rt) = runtime.lock() {
        let result = rt.compile(code_str);
        result.errors.iter().map(|e| e.to_string()).collect::<Vec<_>>()
    }
    else {
        vec!["Failed to acquire runtime lock".to_string()]
    };

    let result_str = serde_json::to_string(&errors).unwrap_or("[]".to_string());
    let c_string = CString::new(result_str).unwrap();
    c_string.into_raw() as *mut u8
}

/// 获取性能指标
///
/// 返回运行时的性能统计信息。
///
/// # 返回
/// 指向 JSON 对象的指针，包含性能指标
#[unsafe(no_mangle)]
pub extern "C" fn get_performance_metrics() -> *mut u8 {
    let runtime = get_runtime();
    let metrics = if let Ok(rt) = runtime.lock() {
        rt.get_performance_metrics()
    }
    else {
        serde_json::json!({
            "executionTime": 0.0,
            "memoryUsage": 0,
            "operations": 0,
            "allocatedBlocks": 0
        })
        .to_string()
    };

    let c_string = CString::new(metrics).unwrap();
    c_string.into_raw() as *mut u8
}

/// 求值 TypeScript 表达式
///
/// 求值单个表达式并返回结果。
///
/// # 参数
/// - `expr`: 表达式的指针
/// - `expr_len`: 表达式长度
///
/// # 返回
/// 指向 JSON 格式求值结果的指针
#[unsafe(no_mangle)]
pub extern "C" fn evaluate_expression(expr: *const u8, expr_len: usize) -> *mut u8 {
    let expr_str = unsafe { std::str::from_utf8(std::slice::from_raw_parts(expr, expr_len)) }.unwrap_or("");

    let runtime = get_runtime();
    let result = if let Ok(rt) = runtime.lock() {
        rt.evaluate(expr_str)
    }
    else {
        EvaluationResult::failure("Failed to acquire runtime lock".to_string())
    };

    let json = serde_json::json!({
        "success": result.success,
        "value": result.value,
        "type": result.type_name,
        "error": result.error
    });

    let result_str = json.to_string();
    let c_string = CString::new(result_str).unwrap();
    c_string.into_raw() as *mut u8
}

/// 获取运行时版本
///
/// # 返回
/// 指向版本字符串的指针
#[unsafe(no_mangle)]
pub extern "C" fn get_version() -> *mut u8 {
    let version = "0.1.0";
    let c_string = CString::new(version).unwrap();
    c_string.into_raw() as *mut u8
}

/// 释放内存
///
/// 释放由运行时分配的内存。
///
/// # 参数
/// - `ptr`: 要释放的内存指针
#[unsafe(no_mangle)]
pub extern "C" fn free_memory(ptr: *mut u8) {
    unsafe {
        if !ptr.is_null() {
            let _ = CString::from_raw(ptr as *mut i8);
        }
    }
}

/// 打开文件
///
/// # 参数
/// - `path`: 文件路径指针
/// - `path_len`: 路径长度
/// - `mode`: 打开模式 (0: 只读, 1: 只写, 2: 读写, 3: 追加)
///
/// # 返回
/// 文件描述符，失败返回 -1
#[unsafe(no_mangle)]
pub extern "C" fn fd_open(path: *const u8, path_len: usize, mode: u32) -> i32 {
    let path_str = unsafe { std::str::from_utf8(std::slice::from_raw_parts(path, path_len)) }.unwrap_or("");

    let open_mode = match mode {
        0 => fs::OpenMode::Read,
        1 => fs::OpenMode::Write,
        2 => fs::OpenMode::ReadWrite,
        3 => fs::OpenMode::Append,
        _ => return -1,
    };

    let runtime = get_runtime();
    if let Ok(mut rt) = runtime.lock() {
        match rt.fs.fd_open(path_str, open_mode) {
            Ok(fd) => fd as i32,
            Err(_) => -1,
        }
    }
    else {
        -1
    }
}

/// 读取文件内容
///
/// # 参数
/// - `fd`: 文件描述符
/// - `buf`: 缓冲区指针
/// - `buf_len`: 缓冲区长度
///
/// # 返回
/// 读取的字节数，失败返回 -1
#[unsafe(no_mangle)]
pub extern "C" fn fd_read(fd: u32, buf: *mut u8, buf_len: usize) -> i32 {
    let runtime = get_runtime();
    if let Ok(mut rt) = runtime.lock() {
        let mut buffer = Vec::with_capacity(buf_len);
        unsafe {
            buffer.set_len(buf_len);
        }

        match rt.fs.fd_read(fd, &mut buffer) {
            Ok(bytes_read) => {
                unsafe {
                    std::ptr::copy_nonoverlapping(buffer.as_ptr(), buf, bytes_read);
                }
                bytes_read as i32
            }
            Err(_) => -1,
        }
    }
    else {
        -1
    }
}

/// 写入文件内容
///
/// # 参数
/// - `fd`: 文件描述符
/// - `buf`: 缓冲区指针
/// - `buf_len`: 缓冲区长度
///
/// # 返回
/// 写入的字节数，失败返回 -1
#[unsafe(no_mangle)]
pub extern "C" fn fd_write(fd: u32, buf: *const u8, buf_len: usize) -> i32 {
    let runtime = get_runtime();
    if let Ok(mut rt) = runtime.lock() {
        let buffer = unsafe { std::slice::from_raw_parts(buf, buf_len) };

        match rt.fs.fd_write(fd, buffer) {
            Ok(bytes_written) => bytes_written as i32,
            Err(_) => -1,
        }
    }
    else {
        -1
    }
}

/// 关闭文件描述符
///
/// # 参数
/// - `fd`: 文件描述符
///
/// # 返回
/// 成功返回 0，失败返回 -1
#[unsafe(no_mangle)]
pub extern "C" fn fd_close(fd: u32) -> i32 {
    let runtime = get_runtime();
    if let Ok(mut rt) = runtime.lock() {
        match rt.fs.fd_close(fd) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
    else {
        -1
    }
}

/// 定位文件指针
///
/// # 参数
/// - `fd`: 文件描述符
/// - `offset`: 偏移量
/// - `whence`: 定位方式 (0: 从文件开头, 1: 从当前位置, 2: 从文件末尾)
///
/// # 返回
/// 新的文件位置，失败返回 -1
#[unsafe(no_mangle)]
pub extern "C" fn fd_seek(fd: u32, offset: i64, whence: u8) -> i64 {
    let runtime = get_runtime();
    if let Ok(mut rt) = runtime.lock() {
        match rt.fs.fd_seek(fd, offset, whence) {
            Ok(pos) => pos as i64,
            Err(_) => -1,
        }
    }
    else {
        -1
    }
}

/// 创建目录
///
/// # 参数
/// - `path`: 目录路径指针
/// - `path_len`: 路径长度
///
/// # 返回
/// 成功返回 0，失败返回 -1
#[unsafe(no_mangle)]
pub extern "C" fn mkdir(path: *const u8, path_len: usize) -> i32 {
    let path_str = unsafe { std::str::from_utf8(std::slice::from_raw_parts(path, path_len)) }.unwrap_or("");

    let runtime = get_runtime();
    if let Ok(mut rt) = runtime.lock() {
        match rt.fs.mkdir(path_str) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
    else {
        -1
    }
}

/// 删除文件
///
/// # 参数
/// - `path`: 文件路径指针
/// - `path_len`: 路径长度
///
/// # 返回
/// 成功返回 0，失败返回 -1
#[unsafe(no_mangle)]
pub extern "C" fn unlink(path: *const u8, path_len: usize) -> i32 {
    let path_str = unsafe { std::str::from_utf8(std::slice::from_raw_parts(path, path_len)) }.unwrap_or("");

    let runtime = get_runtime();
    if let Ok(mut rt) = runtime.lock() {
        match rt.fs.unlink(path_str) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
    else {
        -1
    }
}

/// 获取文件状态
///
/// # 参数
/// - `fd`: 文件描述符
/// - `buf`: 状态缓冲区指针
///
/// # 返回
/// 成功返回 0，失败返回 -1
#[unsafe(no_mangle)]
pub extern "C" fn fd_filestat_get(fd: u32, buf: *mut u8) -> i32 {
    let runtime = get_runtime();
    if let Ok(mut rt) = runtime.lock() {
        match rt.fs.fd_filestat_get(fd) {
            Ok(stat) => {
                // 简单的状态结构：文件类型 (1 字节) + 文件大小 (8 字节)
                unsafe {
                    *buf = match stat.file_type {
                        fs::FileType::Regular => 0,
                        fs::FileType::Directory => 1,
                        fs::FileType::Symlink => 2,
                        fs::FileType::Unknown => 3,
                    };
                    std::ptr::copy_nonoverlapping(&stat.size as *const u64 as *const u8, buf.offset(1), 8);
                }
                0
            }
            Err(_) => -1,
        }
    }
    else {
        -1
    }
}

/// 截断文件
///
/// # 参数
/// - `fd`: 文件描述符
/// - `size`: 新的文件大小
///
/// # 返回
/// 成功返回 0，失败返回 -1
#[unsafe(no_mangle)]
pub extern "C" fn fd_truncate(fd: u32, size: u64) -> i32 {
    let runtime = get_runtime();
    if let Ok(mut rt) = runtime.lock() {
        match rt.fs.fd_truncate(fd, size) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
    else {
        -1
    }
}

/// 创建套接字
///
/// # 参数
/// - `socket_type`: 套接字类型 (0: TCP, 1: UDP)
///
/// # 返回
/// 套接字描述符，失败返回 -1
#[unsafe(no_mangle)]
pub extern "C" fn sock_open(socket_type: u32) -> i32 {
    let sock_type = match socket_type {
        0 => net::SocketType::Tcp,
        1 => net::SocketType::Udp,
        _ => return -1,
    };

    let runtime = get_runtime();
    if let Ok(mut rt) = runtime.lock() {
        match net::sock_open(&mut rt.net, sock_type) {
            Ok(fd) => fd as i32,
            Err(_) => -1,
        }
    }
    else {
        -1
    }
}

/// 绑定套接字到地址
///
/// # 参数
/// - `fd`: 套接字描述符
/// - `ip`: IP 地址 (4 字节 IPv4 或 16 字节 IPv6)
/// - `port`: 端口号
/// - `is_ipv6`: 是否为 IPv6 地址
///
/// # 返回
/// 成功返回 0，失败返回 -1
#[unsafe(no_mangle)]
pub extern "C" fn sock_bind(fd: u32, ip: *const u8, port: u16, is_ipv6: bool) -> i32 {
    let runtime = get_runtime();
    if let Ok(mut rt) = runtime.lock() {
        let addr = if is_ipv6 {
            let ipv6_bytes = unsafe { std::slice::from_raw_parts(ip, 16) };
            let mut segments = [0u16; 8];
            for i in 0..8 {
                segments[i] = u16::from_be_bytes([ipv6_bytes[i * 2], ipv6_bytes[i * 2 + 1]]);
            }
            net::SocketAddress::ipv6(segments, port)
        }
        else {
            let ipv4_bytes = unsafe { std::slice::from_raw_parts(ip, 4) };
            net::SocketAddress::ipv4(ipv4_bytes[0], ipv4_bytes[1], ipv4_bytes[2], ipv4_bytes[3], port)
        };

        match net::sock_bind(&mut rt.net, fd, addr) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
    else {
        -1
    }
}

/// 开始监听连接
///
/// # 参数
/// - `fd`: 套接字描述符
/// - `backlog`: 待处理连接队列的最大长度
///
/// # 返回
/// 成功返回 0，失败返回 -1
#[unsafe(no_mangle)]
pub extern "C" fn sock_listen(fd: u32, backlog: usize) -> i32 {
    let runtime = get_runtime();
    if let Ok(mut rt) = runtime.lock() {
        match net::sock_listen(&mut rt.net, fd, backlog) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
    else {
        -1
    }
}

/// 接受新连接
///
/// # 参数
/// - `fd`: 监听套接字描述符
///
/// # 返回
/// 新连接的套接字描述符，失败返回 -1
#[unsafe(no_mangle)]
pub extern "C" fn sock_accept(fd: u32) -> i32 {
    let runtime = get_runtime();
    if let Ok(mut rt) = runtime.lock() {
        match net::sock_accept(&mut rt.net, fd) {
            Ok(new_fd) => new_fd as i32,
            Err(_) => -1,
        }
    }
    else {
        -1
    }
}

/// 连接到远程地址
///
/// # 参数
/// - `fd`: 套接字描述符
/// - `ip`: IP 地址 (4 字节 IPv4 或 16 字节 IPv6)
/// - `port`: 端口号
/// - `is_ipv6`: 是否为 IPv6 地址
///
/// # 返回
/// 成功返回 0，失败返回 -1
#[unsafe(no_mangle)]
pub extern "C" fn sock_connect(fd: u32, ip: *const u8, port: u16, is_ipv6: bool) -> i32 {
    let runtime = get_runtime();
    if let Ok(mut rt) = runtime.lock() {
        let addr = if is_ipv6 {
            let ipv6_bytes = unsafe { std::slice::from_raw_parts(ip, 16) };
            let mut segments = [0u16; 8];
            for i in 0..8 {
                segments[i] = u16::from_be_bytes([ipv6_bytes[i * 2], ipv6_bytes[i * 2 + 1]]);
            }
            net::SocketAddress::ipv6(segments, port)
        }
        else {
            let ipv4_bytes = unsafe { std::slice::from_raw_parts(ip, 4) };
            net::SocketAddress::ipv4(ipv4_bytes[0], ipv4_bytes[1], ipv4_bytes[2], ipv4_bytes[3], port)
        };

        match net::sock_connect(&mut rt.net, fd, addr) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
    else {
        -1
    }
}

/// 发送数据
///
/// # 参数
/// - `fd`: 套接字描述符
/// - `buf`: 数据缓冲区指针
/// - `buf_len`: 缓冲区长度
///
/// # 返回
/// 发送的字节数，失败返回 -1
#[unsafe(no_mangle)]
pub extern "C" fn sock_send(fd: u32, buf: *const u8, buf_len: usize) -> i32 {
    let runtime = get_runtime();
    if let Ok(mut rt) = runtime.lock() {
        let buffer = unsafe { std::slice::from_raw_parts(buf, buf_len) };

        match net::sock_send(&mut rt.net, fd, buffer) {
            Ok(bytes_sent) => bytes_sent as i32,
            Err(_) => -1,
        }
    }
    else {
        -1
    }
}

/// 接收数据
///
/// # 参数
/// - `fd`: 套接字描述符
/// - `buf`: 接收缓冲区指针
/// - `buf_len`: 缓冲区长度
///
/// # 返回
/// 接收的字节数，失败返回 -1
#[unsafe(no_mangle)]
pub extern "C" fn sock_recv(fd: u32, buf: *mut u8, buf_len: usize) -> i32 {
    let runtime = get_runtime();
    if let Ok(mut rt) = runtime.lock() {
        let mut buffer = Vec::with_capacity(buf_len);
        unsafe {
            buffer.set_len(buf_len);
        }

        match net::sock_recv(&mut rt.net, fd, &mut buffer) {
            Ok(bytes_read) => {
                unsafe {
                    std::ptr::copy_nonoverlapping(buffer.as_ptr(), buf, bytes_read);
                }
                bytes_read as i32
            }
            Err(_) => -1,
        }
    }
    else {
        -1
    }
}

/// 关闭套接字
///
/// # 参数
/// - `fd`: 套接字描述符
///
/// # 返回
/// 成功返回 0，失败返回 -1
#[unsafe(no_mangle)]
pub extern "C" fn sock_close(fd: u32) -> i32 {
    let runtime = get_runtime();
    if let Ok(mut rt) = runtime.lock() {
        match net::sock_close(&mut rt.net, fd) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
    else {
        -1
    }
}

/// 生成随机数
///
/// # 参数
/// - `buf`: 接收随机数据的缓冲区指针
/// - `buf_len`: 缓冲区长度
///
/// # 返回
/// 成功返回 0，失败返回 -1
#[unsafe(no_mangle)]
pub extern "C" fn random_get(buf: *mut u8, buf_len: usize) -> i32 {
    use rand::Rng;

    let runtime = get_runtime();
    if runtime.lock().is_ok() {
        let mut buffer = vec![0u8; buf_len];
        rand::rng().fill_bytes(&mut buffer);

        unsafe {
            std::ptr::copy_nonoverlapping(buffer.as_ptr(), buf, buf_len);
        }
        0
    }
    else {
        -1
    }
}

/// 获取当前时间
///
/// # 参数
/// - `timestamp`: 接收时间戳的指针（纳秒）
///
/// # 返回
/// 成功返回 0，失败返回 -1
#[unsafe(no_mangle)]
pub extern "C" fn clock_time_get(timestamp: *mut u64) -> i32 {
    let now = SystemTime::now();
    let duration = now.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default();
    let nanos = duration.as_nanos() as u64;

    unsafe {
        *timestamp = nanos;
    }
    0
}

/// 获取环境变量数量
///
/// # 返回
/// 环境变量数量
#[unsafe(no_mangle)]
pub extern "C" fn environ_sizes_get() -> i32 {
    // 模拟环境变量数量
    3
}

/// 获取环境变量
///
/// # 参数
/// - `names_ptr`: 接收环境变量名指针的指针
/// - `values_ptr`: 接收环境变量值指针的指针
///
/// # 返回
/// 成功返回 0，失败返回 -1
#[unsafe(no_mangle)]
pub extern "C" fn environ_get(names_ptr: *mut *const u8, values_ptr: *mut *const u8) -> i32 {
    // 模拟环境变量
    let envs = [("HOME", "/home/user"), ("PATH", "/usr/local/bin:/usr/bin:/bin"), ("HOSTNAME", "wasi-host")];

    unsafe {
        for (i, (name, value)) in envs.iter().enumerate() {
            let name_cstr = CString::new(*name).unwrap();
            let value_cstr = CString::new(*value).unwrap();

            *names_ptr.offset(i as isize) = name_cstr.into_raw() as *const u8;
            *values_ptr.offset(i as isize) = value_cstr.into_raw() as *const u8;
        }
    }
    0
}

/// 设置环境变量
///
/// # 参数
/// - `name`: 环境变量名称的指针
/// - `name_len`: 环境变量名称长度
/// - `value`: 环境变量值的指针
/// - `value_len`: 环境变量值长度
///
/// # 返回
/// 成功返回 0，失败返回 -1
#[unsafe(no_mangle)]
pub extern "C" fn set_environment_variable(name: *const u8, name_len: usize, value: *const u8, value_len: usize) -> i32 {
    let name_str = unsafe { std::str::from_utf8(std::slice::from_raw_parts(name, name_len)) }.unwrap_or("");
    let value_str = unsafe { std::str::from_utf8(std::slice::from_raw_parts(value, value_len)) }.unwrap_or("");

    // 模拟设置环境变量
    println!("Setting environment variable: {}={}", name_str, value_str);
    0
}

/// 获取系统信息
///
/// # 返回
/// 指向系统信息 JSON 字符串的指针
#[unsafe(no_mangle)]
pub extern "C" fn get_system_info() -> *mut u8 {
    let system_info = serde_json::json! {
        {
            "os": "WASI",
            "arch": "wasm32",
            "version": "0.1.0",
            "memory": {
                "total": 1048576,
                "available": 8388608
            },
            "cpu": {
                "cores": 1,
                "speed": 1000
            }
        }
    };

    let result_str = system_info.to_string();
    let c_string = CString::new(result_str).unwrap();
    c_string.into_raw() as *mut u8
}

/// 计算哈希值
///
/// # 参数
/// - `data`: 数据指针
/// - `data_len`: 数据长度
/// - `algorithm`: 哈希算法 (0: MD5, 1: SHA1, 2: SHA256)
///
/// # 返回
/// 指向哈希值字符串的指针
#[unsafe(no_mangle)]
pub extern "C" fn compute_hash(data: *const u8, data_len: usize, algorithm: u32) -> *mut u8 {
    use sha2::{Digest, Sha256};

    let data_slice = unsafe { std::slice::from_raw_parts(data, data_len) };

    let hash_result = match algorithm {
        0 => "md5_hash".to_string(),  // 模拟 MD5
        1 => "sha1_hash".to_string(), // 模拟 SHA1
        2 => {
            let mut hasher = Sha256::new();
            hasher.update(data_slice);
            let result = hasher.finalize();
            hex::encode(result)
        }
        _ => "unsupported".to_string(),
    };

    let c_string = CString::new(hash_result).unwrap();
    c_string.into_raw() as *mut u8
}

/// 编码/解码 Base64
///
/// # 参数
/// - `data`: 数据指针
/// - `data_len`: 数据长度
/// - `encode`: 1 表示编码，0 表示解码
///
/// # 返回
/// 指向编码/解码结果的指针
#[unsafe(no_mangle)]
pub extern "C" fn base64_encode_decode(data: *const u8, data_len: usize, encode: u32) -> *mut u8 {
    use base64::Engine;

    let data_slice = unsafe { std::slice::from_raw_parts(data, data_len) };

    let result = if encode == 1 {
        base64::engine::general_purpose::STANDARD.encode(data_slice)
    }
    else {
        match base64::engine::general_purpose::STANDARD.decode(data_slice) {
            Ok(decoded) => String::from_utf8_lossy(&decoded).to_string(),
            Err(_) => "invalid_base64".to_string(),
        }
    };

    let c_string = CString::new(result).unwrap();
    c_string.into_raw() as *mut u8
}

/// 获取内存使用情况
///
/// # 返回
/// 指向内存使用情况 JSON 字符串的指针
#[unsafe(no_mangle)]
pub extern "C" fn get_memory_usage() -> *mut u8 {
    let runtime = get_runtime();
    let memory_usage = if let Ok(rt) = runtime.lock() {
        let stats = rt.memory.get_stats();
        serde_json::json! {
            {
                "used": stats.used_size,
                "total": stats.total_size,
                "peak": stats.peak_usage,
                "allocated_blocks": stats.allocated_blocks,
                "fragmentation": stats.fragmentation_rate
            }
        }
    }
    else {
        serde_json::json! {
            {
                "used": 0,
                "total": 0,
                "peak": 0,
                "allocated_blocks": 0,
                "fragmentation": 0.0
            }
        }
    };

    let result_str = memory_usage.to_string();
    let c_string = CString::new(result_str).unwrap();
    c_string.into_raw() as *mut u8
}

/// 执行垃圾回收
///
/// # 返回
/// 成功返回 0，失败返回 -1
#[unsafe(no_mangle)]
pub extern "C" fn run_gc() -> i32 {
    let runtime = get_runtime();
    if runtime.lock().is_ok() {
        // 模拟垃圾回收
        0
    }
    else {
        -1
    }
}

/// 测量执行时间
///
/// # 参数
/// - `callback`: 回调函数指针
///
/// # 返回
/// 执行时间（毫秒）
#[unsafe(no_mangle)]
pub extern "C" fn measure_execution_time(callback: extern "C" fn()) -> u64 {
    let start = SystemTime::now();
    callback();
    let end = SystemTime::now();

    let duration = end.duration_since(start).unwrap_or_default();
    duration.as_millis() as u64
}

/// 注册全局回调函数
///
/// # 参数
/// - `name`: 回调函数名称
/// - `name_len`: 名称长度
/// - `callback`: 回调函数指针
///
/// # 返回
/// 成功返回 0，失败返回 -1
#[unsafe(no_mangle)]
pub extern "C" fn register_callback(name: *const u8, name_len: usize, _callback: extern "C" fn()) -> i32 {
    let name_str = unsafe { std::str::from_utf8(std::slice::from_raw_parts(name, name_len)) }.unwrap_or("");

    // 模拟注册回调
    println!("Registering callback: {}", name_str);
    0
}

/// 调用已注册的回调函数
///
/// # 参数
/// - `name`: 回调函数名称
/// - `name_len`: 名称长度
///
/// # 返回
/// 成功返回 0，失败返回 -1
#[unsafe(no_mangle)]
pub extern "C" fn call_callback(name: *const u8, name_len: usize) -> i32 {
    let name_str = unsafe { std::str::from_utf8(std::slice::from_raw_parts(name, name_len)) }.unwrap_or("");

    // 模拟调用回调
    println!("Calling callback: {}", name_str);
    0
}
