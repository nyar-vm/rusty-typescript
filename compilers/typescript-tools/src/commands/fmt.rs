/// 格式化命令
///
/// 实现代码格式化功能
use crate::commands::CommandResult;
use std::path::{Path, PathBuf};

/// 格式化命令选项
#[derive(Debug, Clone)]
pub struct FmtOptions {
    /// 文件路径列表
    pub files: Vec<PathBuf>,
    /// 是否检查格式（不写入）
    pub check: bool,
    /// 是否写入文件
    pub write: bool,
    /// 缩进大小
    pub indent_size: usize,
    /// 是否使用分号
    pub semicolons: bool,
    /// 是否使用单引号
    pub single_quote: bool,
}

impl Default for FmtOptions {
    fn default() -> Self {
        Self { files: Vec::new(), check: false, write: false, indent_size: 2, semicolons: true, single_quote: false }
    }
}

/// 执行格式化命令
pub fn execute(options: FmtOptions) -> CommandResult {
    if options.files.is_empty() {
        return CommandResult::error(1, "未指定需要格式化的文件");
    }

    let mut formatted_count = 0;
    let mut unchanged_count = 0;
    let mut error_count = 0;

    for file in &options.files {
        match format_file(file, &options) {
            Ok(changed) => {
                if changed {
                    formatted_count += 1;
                    if options.check {
                        println!("需要格式化: {:?}", file);
                    }
                    else if options.write {
                        println!("已格式化: {:?}", file);
                    }
                    else {
                        println!("将格式化: {:?} (使用 --write 写入)", file);
                    }
                }
                else {
                    unchanged_count += 1;
                    if !options.check {
                        println!("无需格式化: {:?}", file);
                    }
                }
            }
            Err(e) => {
                error_count += 1;
                eprintln!("格式化失败 {:?}: {}", file, e);
            }
        }
    }

    if error_count > 0 {
        CommandResult::error(1, format!("格式化完成，但有 {} 个文件失败", error_count))
    }
    else if options.check && formatted_count > 0 {
        CommandResult::error(1, format!("发现 {} 个文件需要格式化", formatted_count))
    }
    else {
        CommandResult::success(format!("格式化完成! {} 个文件已格式化, {} 个文件无需更改", formatted_count, unchanged_count))
    }
}

/// 格式化单个文件
fn format_file(file: &Path, options: &FmtOptions) -> Result<bool, String> {
    let content = std::fs::read_to_string(file).map_err(|e| format!("无法读取文件: {}", e))?;

    /// 执行格式化
    let formatted = format_code(&content, options);

    let changed = content != formatted;

    if changed {
        if options.write {
            /// 写入文件
            std::fs::write(file, formatted).map_err(|e| format!("无法写入文件: {}", e))?;
        }
        else if !options.check {
            /// 输出到标准输出
            println!("--- {:?} ---", file);
            println!("{}", formatted);
        }
    }

    Ok(changed)
}

/// 格式化代码
fn format_code(content: &str, options: &FmtOptions) -> String {
    let mut formatted = String::new();
    let mut indent_level: usize = 0;
    let mut in_string = false;
    let mut string_char = '\0';
    let mut escape_next = false;
    let mut in_comment = false;
    let mut in_multi_comment = false;

    for line in content.lines() {
        let trimmed = line.trim();

        /// 跳过空行
        if trimmed.is_empty() {
            formatted.push('\n');
            continue;
        }

        // 减少缩进（遇到闭合括号）
        if !in_comment && !in_multi_comment {
            if trimmed.starts_with('}') || trimmed.starts_with(']') || trimmed.starts_with(')') {
                indent_level = indent_level.saturating_sub(1);
            }
        }

        // 添加缩进
        let indent = " ".repeat(indent_level * options.indent_size);
        formatted.push_str(&indent);

        // 处理行内容
        let mut formatted_line = String::new();
        let mut i = 0;
        let chars: Vec<char> = trimmed.chars().collect();

        while i < chars.len() {
            let ch = chars[i];

            if escape_next {
                formatted_line.push(ch);
                escape_next = false;
                i += 1;
                continue;
            }

            if ch == '\\' {
                formatted_line.push(ch);
                escape_next = true;
                i += 1;
                continue;
            }

            if in_string {
                if ch == string_char {
                    in_string = false;
                }
                formatted_line.push(ch);
                i += 1;
                continue;
            }

            if in_multi_comment {
                formatted_line.push(ch);
                if ch == '*' && i + 1 < chars.len() && chars[i + 1] == '/' {
                    formatted_line.push('/');
                    in_multi_comment = false;
                    i += 2;
                }
                else {
                    i += 1;
                }
                continue;
            }

            if !in_comment && ch == '/' && i + 1 < chars.len() {
                if chars[i + 1] == '/' {
                    in_comment = true;
                    formatted_line.push('/');
                    formatted_line.push('/');
                    i += 2;
                    continue;
                }
                else if chars[i + 1] == '*' {
                    in_multi_comment = true;
                    formatted_line.push('/');
                    formatted_line.push('*');
                    i += 2;
                    continue;
                }
            }

            match ch {
                '"' | '\'' => {
                    in_string = true;
                    string_char = ch;
                    if options.single_quote && ch == '"' {
                        formatted_line.push('\'');
                    }
                    else {
                        formatted_line.push(ch);
                    }
                    i += 1;
                }
                '{' | '[' | '(' => {
                    formatted_line.push(ch);
                    if !in_comment && !in_multi_comment {
                        indent_level += 1;
                    }
                    i += 1;
                }
                '}' | ']' | ')' => {
                    if !in_comment && !in_multi_comment {
                        indent_level = indent_level.saturating_sub(1);
                    }
                    formatted_line.push(ch);
                    i += 1;
                }
                ';' => {
                    if options.semicolons {
                        formatted_line.push(ch);
                    }
                    i += 1;
                }
                ' ' | '\t' => {
                    /// 处理空格，确保只有一个空格
                    if formatted_line.ends_with(' ') {
                        i += 1;
                        continue;
                    }
                    formatted_line.push(' ');
                    i += 1;
                }
                _ => {
                    formatted_line.push(ch);
                    i += 1;
                }
            }
        }

        formatted.push_str(&formatted_line);
        formatted.push('\n');

        // 重置单行注释状态
        in_comment = false;
    }

    formatted
}
