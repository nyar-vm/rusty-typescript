//! 诊断模块
//!
//! 提供语义诊断功能，检测语法错误、类型错误和未使用符号。

pub mod syntax_checker;
pub mod type_checker;

use crate::lsp::symbols::{SymbolKind, SymbolTable};
use core::range::Range;

/// 诊断等级
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticLevel {
    /// 错误
    Error,
    /// 警告
    Warning,
    /// 信息
    Info,
    /// 提示
    Hint,
}

/// 诊断信息
#[derive(Clone, Debug)]
pub struct Diagnostic {
    /// 诊断等级
    pub level: DiagnosticLevel,
    /// 诊断消息
    pub message: String,
    /// 代码范围
    pub range: Range<usize>,
    /// 错误代码
    pub code: Option<String>,
    /// 诊断来源
    pub source: String,
    /// 修复建议
    pub fix: Option<String>,
}

impl Diagnostic {
    /// 创建新的诊断信息
    pub fn new(level: DiagnosticLevel, message: impl Into<String>, range: Range<usize>) -> Self {
        Self { level, message: message.into(), range, code: None, source: "rusty-typescript".to_string(), fix: None }
    }

    /// 设置错误代码
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }

    /// 设置修复建议
    pub fn with_fix(mut self, fix: impl Into<String>) -> Self {
        self.fix = Some(fix.into());
        self
    }

    /// 创建错误级别的诊断
    pub fn error(message: impl Into<String>, range: Range<usize>) -> Self {
        Self::new(DiagnosticLevel::Error, message, range)
    }

    /// 创建警告级别的诊断
    pub fn warning(message: impl Into<String>, range: Range<usize>) -> Self {
        Self::new(DiagnosticLevel::Warning, message, range)
    }

    /// 创建信息级别的诊断
    pub fn info(message: impl Into<String>, range: Range<usize>) -> Self {
        Self::new(DiagnosticLevel::Info, message, range)
    }

    /// 创建提示级别的诊断
    pub fn hint(message: impl Into<String>, range: Range<usize>) -> Self {
        Self::new(DiagnosticLevel::Hint, message, range)
    }
}

/// 诊断分析器
pub struct DiagnosticAnalyzer {
    /// 符号表
    symbol_table: SymbolTable,
}

impl DiagnosticAnalyzer {
    /// 创建新的诊断分析器
    pub fn new(symbol_table: SymbolTable) -> Self {
        Self { symbol_table }
    }

    /// 分析代码中的所有诊断问题
    pub fn analyze(&self, content: &str) -> Vec<Diagnostic> {
        self.analyze_with_range(content, None)
    }

    /// 分析代码中的诊断问题（支持增量诊断）
    pub fn analyze_with_range(&self, content: &str, change_range: Option<Range<usize>>) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        /// 语法错误检测
        diagnostics.extend(self.analyze_syntax_errors(content, change_range));

        /// 类型错误检测
        diagnostics.extend(self.analyze_type_errors(content, change_range));

        /// 未使用符号检测
        diagnostics.extend(self.analyze_unused_symbols(content, change_range));

        diagnostics
    }

    /// 分析语法错误
    fn analyze_syntax_errors(&self, content: &str, change_range: Option<Range<usize>>) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        /// 检查括号匹配
        diagnostics.extend(syntax_checker::check_bracket_matching(content));

        /// 检查引号匹配
        diagnostics.extend(syntax_checker::check_quote_matching(content));

        /// 检查基本语法结构
        diagnostics.extend(syntax_checker::check_basic_syntax(content));

        // 如果有修改范围，过滤出只在修改范围内的诊断
        if let Some(range) = change_range {
            diagnostics.retain(|d| d.range.start >= range.start && d.range.end <= range.end);
        }

        diagnostics
    }

    /// 分析类型错误
    fn analyze_type_errors(&self, content: &str, change_range: Option<Range<usize>>) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        /// 检查变量类型一致性
        diagnostics.extend(type_checker::check_variable_types(content, &self.symbol_table));

        /// 检查函数调用参数
        diagnostics.extend(type_checker::check_function_calls(content, &self.symbol_table));

        // 如果有修改范围，过滤出只在修改范围内的诊断
        if let Some(range) = change_range {
            diagnostics.retain(|d| d.range.start >= range.start && d.range.end <= range.end);
        }

        diagnostics
    }

    /// 分析未使用的符号
    fn analyze_unused_symbols(&self, content: &str, change_range: Option<Range<usize>>) -> Vec<Diagnostic> {
        let mut diagnostics = Vec::new();

        for symbol in self.symbol_table.all_symbols() {
            /// 跳过公共导出符号
            if symbol.is_exported {
                continue;
            }

            /// 检查符号是否被使用
            if !self.is_symbol_used(content, &symbol.name) {
                // 如果有修改范围，只检查修改范围内的符号
                if let Some(range) = change_range {
                    if symbol.range.start >= range.start && symbol.range.end <= range.end {
                        let message = format!("'{}' 已声明但未使用", symbol.name);
                        diagnostics.push(Diagnostic::warning(message, symbol.range.clone()));
                    }
                }
                else {
                    let message = format!("'{}' 已声明但未使用", symbol.name);
                    diagnostics.push(Diagnostic::warning(message, symbol.range.clone()));
                }
            }
        }

        diagnostics
    }

    /// 检查符号是否被使用
    fn is_symbol_used(&self, content: &str, symbol_name: &str) -> bool {
        let declaration_pattern =
            format!("(const|let|var|function|class|interface|type|enum)\\s+{}", regex_escape(symbol_name));

        let mut found_declaration = false;
        let mut usage_count = 0;

        for (line_idx, line) in content.lines().enumerate() {
            /// 简单检测：统计符号出现次数
            if line.contains(symbol_name) {
                let line_offset = self.get_line_offset(content, line_idx);

                /// 检查是否是声明行
                if !found_declaration && self.is_declaration_line(line, symbol_name) {
                    found_declaration = true;
                    continue;
                }

                /// 检查是否是实际使用（不是注释、字符串中）
                if self.is_actual_usage(line, symbol_name) {
                    usage_count += 1;
                }
            }
        }

        usage_count > 0
    }

    /// 检查是否是声明行
    fn is_declaration_line(&self, line: &str, symbol_name: &str) -> bool {
        let trimmed = line.trim();
        let patterns = [
            format!("const {}", symbol_name),
            format!("let {}", symbol_name),
            format!("var {}", symbol_name),
            format!("function {}", symbol_name),
            format!("class {}", symbol_name),
            format!("interface {}", symbol_name),
            format!("type {}", symbol_name),
            format!("enum {}", symbol_name),
        ];

        patterns.iter().any(|p| trimmed.starts_with(p))
    }

    /// 检查是否是实际使用
    fn is_actual_usage(&self, line: &str, symbol_name: &str) -> bool {
        let trimmed = line.trim();

        /// 跳过注释行
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
            return false;
        }

        /// 跳过纯字符串行
        if (trimmed.starts_with('"') || trimmed.starts_with('\'')) && (trimmed.ends_with('"') || trimmed.ends_with('\'')) {
            return false;
        }

        true
    }

    /// 获取行偏移量
    fn get_line_offset(&self, content: &str, line_idx: usize) -> usize {
        let mut offset = 0;
        for (i, line) in content.lines().enumerate() {
            if i == line_idx {
                return offset;
            }
            offset += line.len() + 1; // +1 for newline
        }
        content.len()
    }
}

/// 转义正则表达式特殊字符
fn regex_escape(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            '\\' | '.' | '+' | '*' | '?' | '(' | ')' | '|' | '[' | ']' | '{' | '}' | '^' | '$' | '#' => format!("\\{}", c),
            _ => c.to_string(),
        })
        .collect()
}
