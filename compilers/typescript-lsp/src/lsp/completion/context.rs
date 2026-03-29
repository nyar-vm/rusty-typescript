//! 补全上下文模块
//!
//! 用于识别不同的语法上下文，为代码补全提供上下文感知能力。

use super::*;

/// 补全上下文类型
#[derive(Debug, PartialEq, Eq)]
pub enum CompletionContextType {
    /// 默认上下文
    Default,
    /// 成员访问上下文 (e.g., obj.|)
    MemberAccess,
    /// 对象字面量上下文 (e.g., { | })
    ObjectLiteral,
    /// 类型注解上下文 (e.g., let x: |)
    TypeAnnotation,
    /// 函数参数上下文 (e.g., function f(|))
    FunctionParameter,
}

/// 补全上下文
pub struct CompletionContext {
    pub context_type: CompletionContextType,
    pub prefix: String,
    pub object_name: Option<String>,
}

impl CompletionContext {
    /// 创建新的补全上下文
    pub fn new() -> Self {
        Self { context_type: CompletionContextType::Default, prefix: String::new(), object_name: None }
    }
}

/// 上下文检测器
pub struct ContextDetector;

impl ContextDetector {
    /// 检测补全上下文
    pub fn detect(text: &str, offset: usize) -> CompletionContext {
        let mut context = CompletionContext::new();

        // 提取前缀
        context.prefix = Self::extract_prefix(text, offset);

        // 检测上下文类型
        if Self::is_member_access_context(text, offset) {
            context.context_type = CompletionContextType::MemberAccess;
            context.object_name = Self::parse_member_access(text, offset);
        }
        else if Self::is_object_literal_context(text, offset) {
            context.context_type = CompletionContextType::ObjectLiteral;
        }
        else if Self::is_type_annotation_context(text, offset) {
            context.context_type = CompletionContextType::TypeAnnotation;
        }
        else if Self::is_function_parameter_context(text, offset) {
            context.context_type = CompletionContextType::FunctionParameter;
        }

        context
    }

    /// 检查是否是成员访问上下文
    fn is_member_access_context(text: &str, offset: usize) -> bool {
        if offset == 0 {
            return false;
        }

        let before_cursor = &text[..offset];
        let trimmed = before_cursor.trim_end();
        trimmed.ends_with('.')
    }

    /// 检查是否是对象字面量上下文
    fn is_object_literal_context(text: &str, offset: usize) -> bool {
        let before_cursor = &text[..offset];

        // Check if there's an opening brace without a closing brace
        let open_braces = before_cursor.chars().filter(|&c| c == '{').count();
        let close_braces = before_cursor.chars().filter(|&c| c == '}').count();

        if open_braces <= close_braces {
            return false;
        }

        // Check if we're after a comma or opening brace
        let trimmed = before_cursor.trim_end();
        trimmed.ends_with('{') || trimmed.ends_with(',')
    }

    /// 检查是否是类型注解上下文
    fn is_type_annotation_context(text: &str, offset: usize) -> bool {
        let before_cursor = &text[..offset];

        // Look for a colon before the cursor
        if let Some(colon_pos) = before_cursor.rfind(':') {
            // Check if there's no semicolon or assignment after the colon
            let after_colon = &before_cursor[colon_pos + 1..];
            let trimmed = after_colon.trim();

            // Check if we're not in a string or comment
            !Self::is_in_string_or_comment(text, offset) && trimmed.is_empty()
        }
        else {
            false
        }
    }

    /// 检查是否是函数参数上下文
    fn is_function_parameter_context(text: &str, offset: usize) -> bool {
        let before_cursor = &text[..offset];

        // Look for an opening parenthesis without a closing parenthesis
        let open_parens = before_cursor.chars().filter(|&c| c == '(').count();
        let close_parens = before_cursor.chars().filter(|&c| c == ')').count();

        if open_parens <= close_parens {
            return false;
        }

        // Check if the last opening parenthesis is part of a function definition
        let mut paren_count = 0;
        for (i, c) in before_cursor.char_indices().rev() {
            match c {
                ')' => paren_count += 1,
                '(' => {
                    paren_count -= 1;
                    if paren_count < 0 {
                        // Found the matching opening parenthesis
                        let before_paren = &before_cursor[..i];
                        let trimmed = before_paren.trim_end();
                        // Check if it's part of a function definition
                        return trimmed.ends_with("function")
                            || trimmed.ends_with("=>")
                            || trimmed.ends_with("=")
                            || trimmed.ends_with(",");
                    }
                }
                _ => {}
            }
        }

        false
    }

    /// 检查位置是否在字符串或注释中
    fn is_in_string_or_comment(text: &str, offset: usize) -> bool {
        let mut in_string = false;
        let mut in_comment = false;
        let mut string_delimiter = '"';

        for (i, c) in text.char_indices() {
            if i >= offset {
                break;
            }

            if !in_comment && !in_string && c == '/' {
                // Check for comment
                if let Some(next) = text.chars().nth(i + 1) {
                    if next == '/' {
                        in_comment = true;
                    }
                    else if next == '*' {
                        in_comment = true;
                    }
                }
            }
            else if in_comment && c == '*' {
                // Check for end of multi-line comment
                if let Some(next) = text.chars().nth(i + 1) {
                    if next == '/' {
                        in_comment = false;
                    }
                }
            }
            else if in_comment && c == '\n' {
                // End of single-line comment
                in_comment = false;
            }
            else if !in_comment && (c == '"' || c == '\'') {
                // Check for string
                if !in_string {
                    in_string = true;
                    string_delimiter = c;
                }
                else if c == string_delimiter {
                    // Check if not escaped
                    let mut escaped = false;
                    let mut j = i - 1;
                    while j >= 0 {
                        if let Some(ch) = text.chars().nth(j) {
                            if ch == '\\' {
                                escaped = !escaped;
                            }
                            else {
                                break;
                            }
                        }
                        j -= 1;
                    }
                    if !escaped {
                        in_string = false;
                    }
                }
            }
        }

        in_string || in_comment
    }

    /// 解析成员访问表达式
    fn parse_member_access(text: &str, offset: usize) -> Option<String> {
        let before_cursor = &text[..offset];

        if let Some(dot_pos) = before_cursor.rfind('.') {
            let before_dot = &before_cursor[..dot_pos];
            Self::extract_identifier_before(before_dot)
        }
        else {
            None
        }
    }

    /// 提取位置前的标识符
    fn extract_identifier_before(text: &str) -> Option<String> {
        let mut end = text.len();

        while end > 0 {
            let ch = text.chars().nth(end - 1)?;
            if ch.is_whitespace() {
                end -= 1;
            }
            else {
                break;
            }
        }

        if end == 0 {
            return None;
        }

        let mut start = end;
        while start > 0 {
            let ch = text.chars().nth(start - 1)?;
            if ch.is_alphanumeric() || ch == '_' || ch == '$' {
                start -= 1;
            }
            else {
                break;
            }
        }

        if start < end { Some(text[start..end].to_string()) } else { None }
    }

    /// 提取补全前缀
    fn extract_prefix(text: &str, offset: usize) -> String {
        let before_cursor = &text[..offset];
        let mut prefix = String::new();

        for c in before_cursor.chars().rev() {
            if c.is_alphanumeric() || c == '_' || c == '$' {
                prefix.insert(0, c);
            }
            else {
                break;
            }
        }

        prefix
    }
}
