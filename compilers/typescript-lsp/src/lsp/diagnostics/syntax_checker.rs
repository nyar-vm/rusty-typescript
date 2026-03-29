//! 语法检查器模块
//!
//! 检测语法错误，如括号不匹配、引号不匹配等。

use super::Diagnostic;
use core::range::Range;

/// 括号类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BracketType {
    /// 圆括号
    Parenthesis,
    /// 方括号
    Square,
    /// 花括号
    Curly,
}

impl std::fmt::Display for BracketType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            BracketType::Parenthesis => write!(f, "("),
            BracketType::Square => write!(f, "["),
            BracketType::Curly => write!(f, "{{"),
        }
    }
}

/// 括号信息
#[derive(Clone, Copy, Debug)]
struct BracketInfo {
    /// 括号类型
    bracket_type: BracketType,
    /// 是否为开括号
    is_open: bool,
    /// 位置
    position: usize,
}

/// 检查括号匹配
pub fn check_bracket_matching(content: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut stack: Vec<BracketInfo> = Vec::new();

    for (idx, ch) in content.char_indices() {
        match ch {
            '(' => stack.push(BracketInfo { bracket_type: BracketType::Parenthesis, is_open: true, position: idx }),
            '[' => stack.push(BracketInfo { bracket_type: BracketType::Square, is_open: true, position: idx }),
            '{' => stack.push(BracketInfo { bracket_type: BracketType::Curly, is_open: true, position: idx }),
            ')' => {
                if let Some(top) = stack.pop() {
                    if top.bracket_type != BracketType::Parenthesis {
                        diagnostics.push(
                            Diagnostic::error("括号不匹配：期望 ')'", Range::from(top.position..top.position + 1))
                                .with_fix(format!("将 '{}' 改为 '('", top.bracket_type)),
                        );
                    }
                }
                else {
                    diagnostics.push(Diagnostic::error("多余的闭合括号 ')'", Range::from(idx..idx + 1)));
                }
            }
            ']' => {
                if let Some(top) = stack.pop() {
                    if top.bracket_type != BracketType::Square {
                        diagnostics.push(
                            Diagnostic::error("括号不匹配：期望 ']'", Range::from(top.position..top.position + 1))
                                .with_fix(format!("将 '{}' 改为 '['", top.bracket_type)),
                        );
                    }
                }
                else {
                    diagnostics
                        .push(Diagnostic::error("多余的闭合括号 ']'", Range::from(idx..idx + 1)).with_fix("删除多余的 ']'"));
                }
            }
            '}' => {
                if let Some(top) = stack.pop() {
                    if top.bracket_type != BracketType::Curly {
                        diagnostics.push(
                            Diagnostic::error("括号不匹配：期望 '}'", Range::from(top.position..top.position + 1))
                                .with_fix(format!("将 '{}' 改为 '{{'", top.bracket_type)),
                        );
                    }
                }
                else {
                    diagnostics
                        .push(Diagnostic::error("多余的闭合括号 '}'", Range::from(idx..idx + 1)).with_fix("删除多余的 '}'"));
                }
            }
            _ => {}
        }
    }

    // 检查未闭合的括号
    for bracket in stack {
        let message = match bracket.bracket_type {
            BracketType::Parenthesis => "未闭合的 '('",
            BracketType::Square => "未闭合的 '['",
            BracketType::Curly => "未闭合的 '{'",
        };
        let fix = match bracket.bracket_type {
            BracketType::Parenthesis => "添加 ')' 以闭合括号",
            BracketType::Square => "添加 ']' 以闭合括号",
            BracketType::Curly => "添加 '}' 以闭合括号",
        };
        diagnostics.push(Diagnostic::error(message, Range::from(bracket.position..bracket.position + 1)).with_fix(fix));
    }

    diagnostics
}

/// 检查引号匹配
pub fn check_quote_matching(content: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut in_template_string = false;
    let mut escape_next = false;
    let mut quote_start: Option<usize> = None;

    for (idx, ch) in content.char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }

        if ch == '\\' {
            escape_next = true;
            continue;
        }

        match ch {
            '\'' if !in_double_quote && !in_template_string => {
                if in_single_quote {
                    in_single_quote = false;
                    quote_start = None;
                }
                else {
                    in_single_quote = true;
                    quote_start = Some(idx);
                }
            }
            '"' if !in_single_quote && !in_template_string => {
                if in_double_quote {
                    in_double_quote = false;
                    quote_start = None;
                }
                else {
                    in_double_quote = true;
                    quote_start = Some(idx);
                }
            }
            '`' if !in_single_quote && !in_double_quote => {
                if in_template_string {
                    in_template_string = false;
                    quote_start = None;
                }
                else {
                    in_template_string = true;
                    quote_start = Some(idx);
                }
            }
            '\n' if in_single_quote || in_double_quote => {
                /// 字符串跨行（非模板字符串）
                if let Some(start) = quote_start {
                    diagnostics.push(
                        Diagnostic::error("字符串不能包含未转义的换行符", Range::from(start..idx + 1))
                            .with_fix("使用模板字符串 (`) 或转义换行符"),
                    );
                }
                in_single_quote = false;
                in_double_quote = false;
                quote_start = None;
            }
            _ => {}
        }
    }

    /// 检查未闭合的引号
    if in_single_quote {
        if let Some(start) = quote_start {
            diagnostics.push(
                Diagnostic::error("未闭合的单引号字符串", Range::from(start..content.len())).with_fix("添加 ' 以闭合字符串"),
            );
        }
    }
    if in_double_quote {
        if let Some(start) = quote_start {
            diagnostics.push(
                Diagnostic::error("未闭合的双引号字符串", Range::from(start..content.len())).with_fix("添加 \" 以闭合字符串"),
            );
        }
    }
    if in_template_string {
        if let Some(start) = quote_start {
            diagnostics.push(
                Diagnostic::error("未闭合的模板字符串", Range::from(start..content.len())).with_fix("添加 ` 以闭合字符串"),
            );
        }
    }

    diagnostics
}

/// 检查基本语法结构
pub fn check_basic_syntax(content: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    /// 检查分号相关的问题
    diagnostics.extend(check_semicolon_issues(content));

    /// 检查关键字使用
    diagnostics.extend(check_keyword_usage(content));

    /// 检查 switch 语句
    diagnostics.extend(check_switch_statements(content));

    /// 检查 try-catch 语句
    diagnostics.extend(check_try_catch_statements(content));

    /// 检查未使用的导入
    diagnostics.extend(check_unused_imports(content));

    diagnostics
}

/// 检查分号相关问题
fn check_semicolon_issues(content: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut offset = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        /// 检查 return 语句后是否有换行
        if trimmed.starts_with("return ") || trimmed == "return" {
            let after_return = trimmed.strip_prefix("return").unwrap_or("").trim();
            if after_return.is_empty() && !line.ends_with(';') {
                /// 检查下一行是否是表达式
                /// 这是一个潜在的自动分号插入问题
                diagnostics.push(Diagnostic::warning(
                    "潜在的自动分号插入问题: return 后直接换行",
                    Range::from(offset..offset + line.len()),
                ));
            }
        }

        /// 检查 continue/break 后是否有表达式
        if (trimmed.starts_with("continue ") || trimmed.starts_with("break ")) && !trimmed.ends_with(';') {
            /// continue 和 break 不应该有标签以外的表达式
            let after_keyword = trimmed.strip_prefix("continue").or_else(|| trimmed.strip_prefix("break")).unwrap_or("").trim();
            if !after_keyword.is_empty() && !after_keyword.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$') {
                diagnostics.push(Diagnostic::error("continue/break 后只能跟标签名", Range::from(offset..offset + line.len())));
            }
        }

        // 检查语句是否以分号结尾（除了块语句）
        if !trimmed.is_empty()
            && !trimmed.ends_with(';')
            && !trimmed.ends_with('{')
            && !trimmed.ends_with('}')
            && !trimmed.starts_with("if")
            && !trimmed.starts_with("for")
            && !trimmed.starts_with("while")
            && !trimmed.starts_with("function")
            && !trimmed.starts_with("class")
            && !trimmed.starts_with("interface")
            && !trimmed.starts_with("type")
            && !trimmed.starts_with("enum")
            && !trimmed.starts_with("export")
            && !trimmed.starts_with("import")
            && !trimmed.starts_with("return")
            && !trimmed.starts_with("continue")
            && !trimmed.starts_with("break")
            && !trimmed.starts_with("throw")
            && !trimmed.starts_with("try")
            && !trimmed.starts_with("catch")
            && !trimmed.starts_with("finally")
            && !trimmed.starts_with("case")
            && !trimmed.starts_with("default")
            && !trimmed.ends_with(':')
        {
            diagnostics.push(Diagnostic::warning("语句可能缺少分号", Range::from(offset..offset + line.len())));
        }

        offset += line.len() + 1;
    }

    diagnostics
}

/// 检查关键字使用
fn check_keyword_usage(content: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut offset = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        /// 检查 class 关键字使用
        if trimmed.starts_with("class ") {
            let after_class = trimmed.strip_prefix("class").unwrap_or("").trim();
            if after_class.is_empty() {
                diagnostics.push(Diagnostic::error("class 关键字后需要类名", Range::from(offset..offset + line.len())));
            }
            else if after_class.starts_with("extends ") || after_class.starts_with("implements ") {
                /// 检查是否有类名
                let parts: Vec<&str> = after_class.split_whitespace().collect();
                if parts.len() < 2 {
                    diagnostics.push(Diagnostic::error("class 关键字后需要类名", Range::from(offset..offset + line.len())));
                }
            }
        }

        /// 检查 function 关键字使用
        if trimmed.starts_with("function ") {
            let after_function = trimmed.strip_prefix("function").unwrap_or("").trim();
            if after_function.is_empty() {
                diagnostics.push(Diagnostic::error("function 关键字后需要函数名", Range::from(offset..offset + line.len())));
            }
            else if !after_function.chars().next().map(|c| c.is_alphabetic() || c == '_' || c == '$').unwrap_or(false) {
                diagnostics.push(Diagnostic::error(
                    "函数名必须以字母、下划线或美元符号开头",
                    Range::from(offset..offset + line.len()),
                ));
            }
        }

        /// 检查 interface 关键字使用
        if trimmed.starts_with("interface ") {
            let after_interface = trimmed.strip_prefix("interface").unwrap_or("").trim();
            if after_interface.is_empty() {
                diagnostics.push(Diagnostic::error("interface 关键字后需要接口名", Range::from(offset..offset + line.len())));
            }
        }

        offset += line.len() + 1;
    }

    diagnostics
}

/// 检查 switch 语句
fn check_switch_statements(content: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut in_switch = false;
    let mut has_default = false;
    let mut case_count = 0;
    let mut switch_start = 0;
    let mut offset = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("switch (") {
            in_switch = true;
            has_default = false;
            case_count = 0;
            switch_start = offset;
        }
        else if in_switch {
            if trimmed.starts_with("case ") {
                case_count += 1;
                // 检查 case 语句是否以冒号结尾
                if !trimmed.ends_with(':') {
                    diagnostics.push(Diagnostic::error("case 语句必须以冒号结尾", Range::from(offset..offset + line.len())));
                }
            }
            else if trimmed.starts_with("default:") {
                has_default = true;
            }
            else if trimmed == "}" {
                in_switch = false;
                // 检查 switch 语句是否有至少一个 case
                if case_count == 0 && !has_default {
                    diagnostics.push(Diagnostic::warning(
                        "switch 语句至少需要一个 case 或 default",
                        Range::from(switch_start..offset + line.len()),
                    ));
                }
            }
        }

        offset += line.len() + 1;
    }

    diagnostics
}

/// 检查 try-catch 语句
fn check_try_catch_statements(content: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut in_try = false;
    let mut in_catch = false;
    let mut try_start = 0;
    let mut offset = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        if trimmed == "try {" {
            in_try = true;
            try_start = offset;
        }
        else if in_try {
            if trimmed == "}" {
                in_try = false;
                // 检查是否有对应的 catch 或 finally
                let next_lines = content.lines().skip(offset / (line.len() + 1) + 1).take(5);
                let mut has_catch_or_finally = false;
                for next_line in next_lines {
                    let next_trimmed = next_line.trim();
                    if next_trimmed.starts_with("catch (") || next_trimmed == "finally {" {
                        has_catch_or_finally = true;
                        break;
                    }
                }
                if !has_catch_or_finally {
                    diagnostics.push(Diagnostic::error(
                        "try 块必须有对应的 catch 或 finally 块",
                        Range::from(try_start..offset + line.len()),
                    ));
                }
            }
        }
        else if trimmed.starts_with("catch (") {
            in_catch = true;
            // 检查 catch 语句的格式
            if !trimmed.ends_with(") {") {
                diagnostics.push(Diagnostic::error("catch 语句格式错误", Range::from(offset..offset + line.len())));
            }
        }
        else if in_catch {
            if trimmed == "}" {
                in_catch = false;
            }
        }

        offset += line.len() + 1;
    }

    diagnostics
}

/// 检查未使用的导入
fn check_unused_imports(content: &str) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut imports = Vec::new();
    let mut offset = 0;

    for line in content.lines() {
        let trimmed = line.trim();

        // 检查 import 语句
        if trimmed.starts_with("import ") {
            // 解析导入的名称
            if let Some(imports_info) = parse_import_statement(trimmed, offset) {
                imports.extend(imports_info);
            }
        }

        offset += line.len() + 1;
    }

    // 检查每个导入是否被使用
    for import in imports {
        if !is_import_used(content, &import.name) {
            diagnostics.push(
                Diagnostic::warning(format!("导入 '{}' 未使用", import.name), import.range)
                    .with_fix(format!("删除未使用的导入 '{}'", import.name)),
            );
        }
    }

    diagnostics
}

/// 导入信息
struct ImportInfo {
    name: String,
    range: Range<usize>,
}

/// 解析导入语句
fn parse_import_statement(line: &str, line_offset: usize) -> Option<Vec<ImportInfo>> {
    // 简单解析 import 语句
    // 支持的格式：
    // import { a, b } from 'module';
    // import defaultExport from 'module';
    // import * as name from 'module';

    let mut imports = Vec::new();

    if line.starts_with("import { ") {
        // 解析命名导入
        let start = line.find('{')? + 1;
        let end = line.find('}')?;
        let import_list = &line[start..end].trim();

        for item in import_list.split(',') {
            let item = item.trim();
            if !item.is_empty() {
                // 提取导入名称（忽略别名）
                let name = item.split(' ').next()?;
                let name_start = line_offset + line.find(name)?;
                let name_end = name_start + name.len();
                imports.push(ImportInfo { name: name.to_string(), range: Range::from(name_start..name_end) });
            }
        }
    }
    else if line.starts_with("import ") && !line.contains("*") && !line.contains("{") {
        // 解析默认导入
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let name = parts[1];
            if !name.starts_with("from") {
                let name_start = line_offset + line.find(name)?;
                let name_end = name_start + name.len();
                imports.push(ImportInfo { name: name.to_string(), range: Range::from(name_start..name_end) });
            }
        }
    }
    else if line.starts_with("import * as ") {
        // 解析命名空间导入
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let name = parts[2];
            let name_start = line_offset + line.find(name)?;
            let name_end = name_start + name.len();
            imports.push(ImportInfo { name: name.to_string(), range: Range::from(name_start..name_end) });
        }
    }

    if imports.is_empty() { None } else { Some(imports) }
}

/// 检查导入是否被使用
fn is_import_used(content: &str, import_name: &str) -> bool {
    let mut usage_count = 0;
    let mut in_import = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // 跳过导入语句本身
        if trimmed.starts_with("import ") {
            in_import = true;
            continue;
        }

        // 检查是否是实际使用
        if !in_import && trimmed.contains(import_name) {
            // 确保是完整的标识符
            let mut chars = trimmed.chars();
            let mut in_identifier = false;
            let mut current_identifier = String::new();

            for ch in chars {
                if ch.is_alphanumeric() || ch == '_' || ch == '$' {
                    in_identifier = true;
                    current_identifier.push(ch);
                }
                else {
                    if in_identifier {
                        if current_identifier == import_name {
                            usage_count += 1;
                        }
                        in_identifier = false;
                        current_identifier.clear();
                    }
                }
            }

            // 检查最后一个标识符
            if in_identifier && current_identifier == import_name {
                usage_count += 1;
            }
        }

        in_import = false;
    }

    usage_count > 0
}
