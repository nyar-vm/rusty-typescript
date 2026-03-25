use tower_lsp::lsp_types::TextEdit;

/// 格式化选项
#[derive(Debug, Default, Clone)]
pub struct FormatOptions {
    /// 缩进大小
    pub indent_size: u32,
    /// 是否使用制表符缩进
    pub use_tabs: bool,
    /// 行宽
    pub line_width: u32,
    /// 是否在大括号前添加空格
    pub space_before_brace: bool,
    /// 是否在逗号后添加空格
    pub space_after_comma: bool,
    /// 是否在分号后添加空格
    pub space_after_semicolon: bool,
    /// 是否在操作符周围添加空格
    pub spaces_around_operators: bool,
    /// 是否在函数括号内添加空格
    pub spaces_in_function: bool,
    /// 是否在对象字面量括号内添加空格
    pub spaces_in_object: bool,
}

/// 格式化器
pub struct Formatter {
    options: FormatOptions,
}

impl Formatter {
    /// 创建新的格式化器
    pub fn new(options: FormatOptions) -> Self {
        Self { options }
    }

    /// 格式化整个文档
    pub fn format_document(&self, text: &str) -> String {
        self.format_range(text, 0, text.len())
    }

    /// 格式化指定范围
    pub fn format_range(&self, text: &str, start: usize, end: usize) -> String {
        let range_text = &text[start..end];
        let formatted_range = self.format_text(range_text);

        // 保持范围外的文本不变
        let mut result = String::new();
        result.push_str(&text[..start]);
        result.push_str(&formatted_range);
        result.push_str(&text[end..]);

        result
    }

    /// 格式化文本
    fn format_text(&self, text: &str) -> String {
        let mut formatted = String::new();
        let mut indent_level = 0;
        let mut in_string = false;
        let mut in_comment = false;
        let mut in_multiline_comment = false;
        let mut prev_char = '\0';
        let mut line_start = true;

        for c in text.chars() {
            match c {
                // 字符串处理
                '"' if prev_char != '\\' && !in_comment && !in_multiline_comment => {
                    in_string = !in_string;
                    self.append_char(&mut formatted, c, indent_level, line_start);
                    line_start = false;
                }
                '\'' if prev_char != '\\' && !in_string && !in_comment && !in_multiline_comment => {
                    in_string = !in_string;
                    self.append_char(&mut formatted, c, indent_level, line_start);
                    line_start = false;
                }

                // 单行注释
                '/' if prev_char == '/' && !in_string && !in_multiline_comment => {
                    in_comment = true;
                    self.append_char(&mut formatted, c, indent_level, line_start);
                    line_start = false;
                }

                // 多行注释
                '*' if prev_char == '/' && !in_string && !in_comment => {
                    in_multiline_comment = true;
                    self.append_char(&mut formatted, c, indent_level, line_start);
                    line_start = false;
                }
                '/' if prev_char == '*' && in_multiline_comment => {
                    in_multiline_comment = false;
                    self.append_char(&mut formatted, c, indent_level, line_start);
                    line_start = false;
                }

                // 换行处理
                '\n' => {
                    in_comment = false;
                    formatted.push('\n');
                    line_start = true;
                }

                // 大括号处理
                '{' if !in_string && !in_comment && !in_multiline_comment => {
                    if self.options.space_before_brace {
                        self.append_char(&mut formatted, ' ', indent_level, line_start);
                    }
                    self.append_char(&mut formatted, '{', indent_level, line_start);
                    formatted.push('\n');
                    indent_level += 1;
                    line_start = true;
                }
                '}' if !in_string && !in_comment && !in_multiline_comment => {
                    formatted.push('\n');
                    indent_level = indent_level.saturating_sub(1);
                    self.append_indent(&mut formatted, indent_level);
                    formatted.push('}');
                    line_start = false;
                }

                // 分号处理
                ';' if !in_string && !in_comment && !in_multiline_comment => {
                    formatted.push(';');
                    if self.options.space_after_semicolon {
                        formatted.push(' ');
                    }
                    formatted.push('\n');
                    line_start = true;
                }

                // 逗号处理
                ',' if !in_string && !in_comment && !in_multiline_comment => {
                    formatted.push(',');
                    if self.options.space_after_comma {
                        formatted.push(' ');
                    }
                    line_start = false;
                }

                // 操作符处理
                '+' | '-' | '*' | '/' | '%' | '=' | '>' | '<' if !in_string && !in_comment && !in_multiline_comment => {
                    if self.options.spaces_around_operators && !line_start {
                        formatted.push(' ');
                    }
                    formatted.push(c);
                    if self.options.spaces_around_operators {
                        formatted.push(' ');
                    }
                    line_start = false;
                }

                // 处理双字符操作符
                _ if !in_string && !in_comment && !in_multiline_comment => {
                    let mut op = String::new();
                    op.push(prev_char);
                    op.push(c);

                    match op.as_str() {
                        "==" | "!=" | ">=" | "<=" | "&&" | "||" => {
                            if self.options.spaces_around_operators && !line_start {
                                formatted.push(' ');
                            }
                            formatted.push_str(&op);
                            if self.options.spaces_around_operators {
                                formatted.push(' ');
                            }
                            line_start = false;
                        }
                        _ => {
                            self.append_char(&mut formatted, c, indent_level, line_start);
                            line_start = false;
                        }
                    }
                }

                // 括号处理
                '(' if !in_string && !in_comment && !in_multiline_comment => {
                    formatted.push('(');
                    if self.options.spaces_in_function {
                        formatted.push(' ');
                    }
                    line_start = false;
                }
                ')' if !in_string && !in_comment && !in_multiline_comment => {
                    if self.options.spaces_in_function {
                        formatted.push(' ');
                    }
                    formatted.push(')');
                    line_start = false;
                }
                '[' if !in_string && !in_comment && !in_multiline_comment => {
                    formatted.push('[');
                    if self.options.spaces_in_object {
                        formatted.push(' ');
                    }
                    line_start = false;
                }
                ']' if !in_string && !in_comment && !in_multiline_comment => {
                    if self.options.spaces_in_object {
                        formatted.push(' ');
                    }
                    formatted.push(']');
                    line_start = false;
                }

                // 空格处理
                ' ' if line_start || (prev_char == ' ' && !in_string && !in_comment && !in_multiline_comment) => {
                    // 跳过行首和连续的空格
                }

                // 其他字符
                _ => {
                    self.append_char(&mut formatted, c, indent_level, line_start);
                    line_start = false;
                }
            }
            prev_char = c;
        }

        formatted
    }

    /// 追加字符，处理缩进
    fn append_char(&self, formatted: &mut String, c: char, indent_level: u32, line_start: bool) {
        if line_start {
            self.append_indent(formatted, indent_level);
        }
        formatted.push(c);
    }

    /// 追加缩进
    fn append_indent(&self, formatted: &mut String, indent_level: u32) {
        if self.options.use_tabs {
            formatted.push_str(&"\t".repeat(indent_level as usize));
        }
        else {
            formatted.push_str(&" ".repeat((indent_level * self.options.indent_size) as usize));
        }
    }
}

/// 格式化代码
pub fn format_code(text: &str) -> String {
    let options = FormatOptions::default();
    let formatter = Formatter::new(options);
    formatter.format_document(text)
}

/// 带选项的格式化代码
pub fn format_code_with_options(text: &str, options: FormatOptions) -> String {
    let formatter = Formatter::new(options);
    formatter.format_document(text)
}

/// 范围格式化
pub fn format_range(text: &str, start: usize, end: usize, options: FormatOptions) -> String {
    let formatter = Formatter::new(options);
    formatter.format_range(text, start, end)
}
