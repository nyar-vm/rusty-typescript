//! 类型检查器模块
//!
//! 检测类型错误，如类型不匹配、函数参数错误等。

use super::Diagnostic;
use crate::lsp::symbols::{SymbolKind, SymbolTable};
use core::range::Range;

/// 检查变量类型
pub fn check_variable_types(content: &str, symbol_table: &SymbolTable) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for symbol in symbol_table.all_symbols() {
        if symbol.kind == SymbolKind::Variable {
            /// 检查变量是否被正确初始化
            if let Some(ref type_annotation) = symbol.type_annotation {
                /// 检查类型注解是否有效
                if !is_valid_type_annotation(type_annotation) {
                    diagnostics.push(Diagnostic::error(format!("无效的类型注解: {}", type_annotation), symbol.range.clone()));
                }
            }

            /// 检查变量类型与初始化值是否匹配
            if let Some(_) = symbol.type_annotation {
                // 这里需要实现类型推断逻辑
                // 暂时注释掉，因为需要完整的类型推断系统
            }

            /// 检查未使用的变量
            if !symbol.is_exported && !is_variable_used(content, &symbol.name) {
                diagnostics.push(Diagnostic::warning(format!("变量 '{}' 已声明但未使用", symbol.name), symbol.range.clone()));
            }
        }
    }

    /// 检查变量阴影
    diagnostics.extend(check_variable_shadowing(symbol_table));

    diagnostics
}

/// 检查函数调用
pub fn check_function_calls(content: &str, symbol_table: &SymbolTable) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    /// 遍历代码查找函数调用
    for (line_idx, line) in content.lines().enumerate() {
        let line_offset = get_line_offset(content, line_idx);

        /// 简单检测函数调用模式：函数名后跟括号
        if let Some(call_idx) = line.find('(') {
            if call_idx > 0 {
                let before_paren = &line[..call_idx];
                let func_name = before_paren.trim().split_whitespace().last().unwrap_or("");

                if !func_name.is_empty() && is_valid_identifier(func_name) {
                    /// 查找函数定义
                    if let Some(symbol) = symbol_table.find_by_name(func_name) {
                        if symbol.kind == SymbolKind::Function {
                            /// 检查参数数量
                            let call_args = extract_argument_count(&line[call_idx..]);
                            let expected_params = extract_param_count(&symbol.type_annotation);

                            if let (Some(actual), Some(expected)) = (call_args, expected_params) {
                                if actual != expected {
                                    diagnostics.push(Diagnostic::error(
                                        format!("函数 '{}' 期望 {} 个参数，但提供了 {} 个", func_name, expected, actual),
                                        Range::from(line_offset + call_idx..line_offset + call_idx + 1),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    diagnostics
}

/// 检查类型注解是否有效
fn is_valid_type_annotation(type_annotation: &str) -> bool {
    let valid_types = [
        "string",
        "number",
        "boolean",
        "any",
        "unknown",
        "never",
        "void",
        "null",
        "undefined",
        "object",
        "symbol",
        "bigint",
        "true",
        "false",
    ];

    let trimmed = type_annotation.trim();

    /// 检查是否是基本类型
    if valid_types.contains(&trimmed) {
        return true;
    }

    /// 检查是否是数组类型
    if trimmed.ends_with("[]") {
        let element_type = &trimmed[..trimmed.len() - 2];
        return is_valid_type_annotation(element_type);
    }

    /// 检查是否是联合类型
    if trimmed.contains(" | ") {
        return trimmed.split(" | ").all(|t| is_valid_type_annotation(t.trim()));
    }

    /// 检查是否是交叉类型
    if trimmed.contains(" & ") {
        return trimmed.split(" & ").all(|t| is_valid_type_annotation(t.trim()));
    }

    // 检查是否是泛型类型
    if trimmed.contains('<') && trimmed.ends_with('>') {
        return true; // 简化处理
    }

    /// 检查是否是有效的标识符（自定义类型）
    is_valid_identifier(trimmed)
}

/// 检查类型是否兼容
fn types_compatible(expected: &str, actual: &str) -> bool {
    let expected = expected.trim();
    let actual = actual.trim();

    /// 相同类型兼容
    if expected == actual {
        return true;
    }

    /// any 类型兼容所有类型
    if expected == "any" || actual == "any" {
        return true;
    }

    /// unknown 可以赋值给任何类型
    if actual == "unknown" {
        return true;
    }

    /// never 可以赋值给任何类型
    if actual == "never" {
        return true;
    }

    /// 检查联合类型
    if expected.contains(" | ") {
        return expected.split(" | ").any(|t| types_compatible(t.trim(), actual));
    }

    /// 检查数组类型
    if expected.ends_with("[]") && actual.ends_with("[]") {
        let expected_elem = &expected[..expected.len() - 2];
        let actual_elem = &actual[..actual.len() - 2];
        return types_compatible(expected_elem, actual_elem);
    }

    /// 数字类型兼容
    if expected == "number" && (actual == "number" || actual.parse::<f64>().is_ok()) {
        return true;
    }

    /// 字符串类型兼容
    if expected == "string" && (actual == "string" || actual.starts_with('"') || actual.starts_with('\'')) {
        return true;
    }

    /// 布尔类型兼容
    if expected == "boolean" && (actual == "boolean" || actual == "true" || actual == "false") {
        return true;
    }

    false
}

/// 检查是否是有效的标识符
fn is_valid_identifier(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    let mut chars = s.chars();
    let first = chars.next().unwrap();

    /// 首字符必须是字母、下划线或美元符号
    if !first.is_alphabetic() && first != '_' && first != '$' {
        return false;
    }

    /// 其余字符必须是字母、数字、下划线或美元符号
    chars.all(|c| c.is_alphanumeric() || c == '_' || c == '$')
}

/// 提取函数调用参数数量
fn extract_argument_count(call_expr: &str) -> Option<usize> {
    /// 找到匹配的右括号
    let mut depth = 0;
    let mut in_string = false;
    let mut string_char = '\0';
    let mut escape_next = false;
    let mut comma_count = 0;

    for ch in call_expr.chars() {
        if escape_next {
            escape_next = false;
            continue;
        }

        if ch == '\\' {
            escape_next = true;
            continue;
        }

        if in_string {
            if ch == string_char {
                in_string = false;
            }
            continue;
        }

        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' => {
                if depth == 1 {
                    /// 找到匹配的右括号
                    return if comma_count == 0 && call_expr.trim() == "()" { Some(0) } else { Some(comma_count + 1) };
                }
                depth -= 1;
            }
            ']' | '}' => depth -= 1,
            ',' if depth == 1 => comma_count += 1,
            '"' | '\'' => {
                in_string = true;
                string_char = ch;
            }
            _ => {}
        }
    }

    None
}

/// 从函数签名提取参数数量
fn extract_param_count(type_annotation: &Option<String>) -> Option<usize> {
    let type_str = type_annotation.as_ref()?;

    /// 查找函数签名中的参数列表
    if let Some(start) = type_str.find('(') {
        if let Some(end) = type_str.find(')') {
            let params = &type_str[start + 1..end];
            let trimmed = params.trim();

            if trimmed.is_empty() {
                return Some(0);
            }

            /// 计算逗号数量 + 1
            let comma_count = trimmed.chars().filter(|&c| c == ',').count();
            return Some(comma_count + 1);
        }
    }

    None
}

/// 获取行偏移量
fn get_line_offset(content: &str, line_idx: usize) -> usize {
    let mut offset = 0;
    for (i, line) in content.lines().enumerate() {
        if i == line_idx {
            return offset;
        }
        offset += line.len() + 1; // +1 for newline
    }
    content.len()
}

/// 检查变量是否被使用
fn is_variable_used(content: &str, variable_name: &str) -> bool {
    let mut usage_count = 0;
    let mut in_declaration = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // 检查是否是声明行
        let is_declaration = trimmed.starts_with("const ") || trimmed.starts_with("let ") || trimmed.starts_with("var ");
        if is_declaration && trimmed.contains(variable_name) {
            in_declaration = true;
            continue;
        }

        // 检查是否是实际使用
        if !in_declaration && trimmed.contains(variable_name) {
            // 确保是完整的变量名
            let chars = trimmed.chars();
            let mut in_identifier = false;
            let mut current_identifier = String::new();

            for ch in chars {
                if ch.is_alphanumeric() || ch == '_' || ch == '$' {
                    in_identifier = true;
                    current_identifier.push(ch);
                }
                else {
                    if in_identifier {
                        if current_identifier == variable_name {
                            usage_count += 1;
                        }
                        in_identifier = false;
                        current_identifier.clear();
                    }
                }
            }

            // 检查最后一个标识符
            if in_identifier && current_identifier == variable_name {
                usage_count += 1;
            }
        }

        in_declaration = false;
    }

    usage_count > 0
}

/// 检查变量阴影
fn check_variable_shadowing(symbol_table: &SymbolTable) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut symbol_names: std::collections::HashMap<String, core::range::Range<usize>> = std::collections::HashMap::new();

    // 按作用域顺序遍历符号（从外到内）
    for symbol in symbol_table.all_symbols() {
        if symbol.kind == SymbolKind::Variable {
            // 检查是否在当前作用域或外部作用域中已存在同名变量
            if let Some(existing_symbol) = symbol_names.get(&symbol.name) {
                // 检查是否在嵌套作用域中
                if is_nested_scope(symbol.range.start, existing_symbol.start, existing_symbol.end) {
                    diagnostics.push(
                        Diagnostic::warning(
                            format!("变量 '{}' 阴影了外部作用域中的同名变量", symbol.name),
                            symbol.range.clone(),
                        )
                        .with_fix(format!("重命名变量 '{}' 以避免阴影", symbol.name)),
                    );
                }
            }

            // 更新符号表，记录最新的变量定义
            symbol_names.insert(symbol.name.clone(), symbol.range.clone());
        }
    }

    diagnostics
}

/// 检查是否在嵌套作用域中
fn is_nested_scope(new_pos: usize, outer_start: usize, outer_end: usize) -> bool {
    new_pos > outer_start && new_pos < outer_end
}
