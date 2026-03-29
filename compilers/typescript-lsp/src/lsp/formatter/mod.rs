use oak_core::Range;
use oak_lsp::types::TextEdit;

/// 尾随逗号选项
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrailingComma {
    /// 不使用尾随逗号
    None,
    /// ES5 支持的尾随逗号（数组、对象等）
    ES5,
    /// 所有位置都使用尾随逗号（包括函数参数）
    #[default]
    All,
}

/// 箭头函数参数括号选项
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ArrowParens {
    /// 单参数时省略括号
    Avoid,
    /// 总是使用括号
    #[default]
    Always,
}

/// 换行风格选项
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineEnding {
    /// Windows 风格（CRLF）
    CRLF,
    /// Linux/Mac 风格（LF）
    #[default]
    LF,
}

/// 代码格式化选项
#[derive(Debug, Clone)]
#[derive(Eq, Hash, PartialEq)]
pub struct FormatOptions {
    /// 缩进大小（空格数）
    pub indent_size: u32,
    /// 是否使用制表符代替空格
    pub use_tabs: bool,
    /// 最大行宽
    pub line_width: u32,
    /// 左大括号前是否添加空格
    pub space_before_brace: bool,
    /// 逗号后是否添加空格
    pub space_after_comma: bool,
    /// 分号后是否添加空格
    pub space_after_semicolon: bool,
    /// 运算符周围是否添加空格
    pub spaces_around_operators: bool,
    /// 函数括号内是否添加空格
    pub spaces_in_function: bool,
    /// 对象括号内是否添加空格
    pub spaces_in_object: bool,
    /// 是否使用分号
    pub semicolons: bool,
    /// 是否使用单引号
    pub single_quote: bool,
    /// 尾随逗号选项
    pub trailing_comma: TrailingComma,
    /// 对象字面量中是否添加空格
    pub bracket_spacing: bool,
    /// 箭头函数参数括号选项
    pub arrow_parens: ArrowParens,
    /// 换行风格
    pub line_ending: LineEnding,
    /// 是否在块之间添加空行
    pub insert_empty_lines_between_blocks: bool,
    /// 是否在函数之间添加空行
    pub insert_empty_lines_between_functions: bool,
    /// 是否在类成员之间添加空行
    pub insert_empty_lines_between_class_members: bool,
    /// 是否保持注释的格式
    pub preserve_comment_format: bool,
    /// 是否对 JSDoc 注释进行格式化
    pub format_jsdoc: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            indent_size: 4,
            use_tabs: false,
            line_width: 80,
            space_before_brace: true,
            space_after_comma: true,
            space_after_semicolon: false,
            spaces_around_operators: true,
            spaces_in_function: false,
            spaces_in_object: false,
            semicolons: true,
            single_quote: false,
            trailing_comma: TrailingComma::default(),
            bracket_spacing: true,
            arrow_parens: ArrowParens::default(),
            line_ending: LineEnding::default(),
            insert_empty_lines_between_blocks: true,
            insert_empty_lines_between_functions: true,
            insert_empty_lines_between_class_members: false,
            preserve_comment_format: true,
            format_jsdoc: true,
        }
    }
}

/// TypeScript 代码格式化器
pub struct Formatter {
    options: FormatOptions,
}

impl Formatter {
    /// 使用指定选项创建新的格式化器
    pub fn new(options: FormatOptions) -> Self {
        Self { options }
    }

    /// 格式化整个文档
    pub fn format_document(&self, text: &str) -> String {
        self.format_range(text, 0, text.len())
    }

    /// 格式化文档中的指定范围
    pub fn format_range(&self, text: &str, start: usize, end: usize) -> String {
        let range_text = &text[start..end];
        let formatted_range = self.format_text(range_text);

        let mut result = String::new();
        result.push_str(&text[..start]);
        result.push_str(&formatted_range);
        result.push_str(&text[end..]);

        result
    }

    /// 内部方法：格式化文本内容
    fn format_text(&self, text: &str) -> String {
        // 预分配足够的空间，减少内存分配
        let mut formatted = String::with_capacity(text.len() * 2);
        let mut indent_level = 0;
        let mut in_string = false;
        let mut string_char = '\0';
        let mut in_template = false;
        let mut in_comment = false;
        let mut in_multiline_comment = false;
        let mut prev_char = '\0';
        let mut line_start = true;
        let mut in_object = false;
        let mut object_depth = 0;
        let mut in_array = false;
        let mut array_depth = 0;
        let mut in_arrow_function = false;
        let mut arrow_param_count = 0;
        let mut paren_depth = 0;
        let mut pending_arrow_check = false;
        let mut in_type_annotation = false;
        let mut in_generic = false;
        let mut generic_depth = 0;
        let mut in_interface = false;
        let mut in_decorator = false;
        let mut chars = text.chars().peekable();

        while let Some(c) = chars.next() {
            let next_char = chars.peek().copied().unwrap_or('\0');

            match c {
                '"' if prev_char != '\\' && !in_comment && !in_multiline_comment && !in_template => {
                    if !in_string {
                        in_string = true;
                        string_char = '"';
                        let quote = if self.options.single_quote { '\'' } else { '"' };
                        self.append_char(&mut formatted, quote, indent_level, line_start);
                    }
                    else if string_char == '"' {
                        in_string = false;
                        string_char = '\0';
                        let quote = if self.options.single_quote { '\'' } else { '"' };
                        self.append_char(&mut formatted, quote, indent_level, line_start);
                    }
                    else {
                        self.append_char(&mut formatted, c, indent_level, line_start);
                    }
                    line_start = false;
                }
                '\'' if prev_char != '\\' && !in_comment && !in_multiline_comment && !in_template => {
                    if !in_string {
                        in_string = true;
                        string_char = '\'';
                        let quote = if self.options.single_quote { '\'' } else { '"' };
                        self.append_char(&mut formatted, quote, indent_level, line_start);
                    }
                    else if string_char == '\'' {
                        in_string = false;
                        string_char = '\0';
                        let quote = if self.options.single_quote { '\'' } else { '"' };
                        self.append_char(&mut formatted, quote, indent_level, line_start);
                    }
                    else {
                        self.append_char(&mut formatted, c, indent_level, line_start);
                    }
                    line_start = false;
                }

                '`' if prev_char != '\\' && !in_comment && !in_multiline_comment => {
                    in_template = !in_template;
                    self.append_char(&mut formatted, c, indent_level, line_start);
                    line_start = false;
                }

                '/' if prev_char == '/' && !in_string && !in_multiline_comment => {
                    in_comment = true;
                    self.append_char(&mut formatted, c, indent_level, line_start);
                    line_start = false;
                }

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

                '\n' => {
                    in_comment = false;
                    in_decorator = false;
                    match self.options.line_ending {
                        LineEnding::CRLF => formatted.push_str("\r\n"),
                        LineEnding::LF => formatted.push('\n'),
                    }
                    line_start = true;
                }

                '\r' => {
                    // 忽略单独的 CR，由 line_ending 选项控制换行风格
                    continue;
                }

                '@' if !in_string && !in_comment && !in_multiline_comment && !in_template && line_start => {
                    in_decorator = true;
                    self.append_char(&mut formatted, c, indent_level, line_start);
                    line_start = false;
                }

                '{' if !in_string && !in_comment && !in_multiline_comment && !in_template => {
                    if self.options.space_before_brace && !formatted.ends_with(' ') && !line_start {
                        formatted.push(' ');
                    }
                    self.append_char(&mut formatted, '{', indent_level, line_start);

                    if self.options.bracket_spacing && !in_array {
                        formatted.push(' ');
                    }

                    formatted.push('\n');
                    indent_level += 1;
                    line_start = true;

                    if !in_object && !in_array {
                        in_object = true;
                    }
                    object_depth += 1;
                }
                '}' if !in_string && !in_comment && !in_multiline_comment && !in_template => {
                    formatted.push('\n');
                    indent_level = indent_level.saturating_sub(1);
                    self.append_indent(&mut formatted, indent_level);

                    if self.options.bracket_spacing && object_depth > 0 {
                        formatted.push(' ');
                    }

                    formatted.push('}');
                    line_start = false;

                    if object_depth > 0 {
                        object_depth -= 1;
                        if object_depth == 0 {
                            in_object = false;
                        }
                    }
                }

                '[' if !in_string && !in_comment && !in_multiline_comment && !in_template => {
                    self.append_char(&mut formatted, '[', indent_level, line_start);
                    if self.options.bracket_spacing {
                        formatted.push(' ');
                    }
                    line_start = false;
                    in_array = true;
                    array_depth += 1;
                }
                ']' if !in_string && !in_comment && !in_multiline_comment && !in_template => {
                    if self.options.bracket_spacing && array_depth > 0 {
                        formatted.push(' ');
                    }
                    formatted.push(']');
                    line_start = false;

                    if array_depth > 0 {
                        array_depth -= 1;
                        if array_depth == 0 {
                            in_array = false;
                        }
                    }
                }

                '<' if !in_string && !in_comment && !in_multiline_comment && !in_template => {
                    if prev_char.is_alphabetic() || prev_char == '>' || prev_char == '=' {
                        // 可能是泛型或类型参数
                        in_generic = true;
                        generic_depth += 1;
                        self.append_char(&mut formatted, '<', indent_level, line_start);
                        if self.options.spaces_around_operators {
                            formatted.push(' ');
                        }
                    }
                    else if !self.options.spaces_around_operators {
                        self.append_char(&mut formatted, '<', indent_level, line_start);
                    }
                    else {
                        self.handle_operator(&mut formatted, c, indent_level, line_start);
                    }
                    line_start = false;
                }
                '>' if !in_string && !in_comment && !in_multiline_comment && !in_template && in_generic => {
                    if self.options.spaces_around_operators {
                        formatted.push(' ');
                    }
                    formatted.push('>');
                    line_start = false;
                    if generic_depth > 0 {
                        generic_depth -= 1;
                        if generic_depth == 0 {
                            in_generic = false;
                        }
                    }
                }
                '>' if !in_string && !in_comment && !in_multiline_comment && !in_template && !in_generic => {
                    self.handle_operator(&mut formatted, c, indent_level, line_start);
                    line_start = false;
                }

                '(' if !in_string && !in_comment && !in_multiline_comment && !in_template => {
                    self.append_char(&mut formatted, '(', indent_level, line_start);
                    if self.options.spaces_in_function {
                        formatted.push(' ');
                    }
                    line_start = false;
                    paren_depth += 1;
                }
                ')' if !in_string && !in_comment && !in_multiline_comment && !in_template => {
                    if self.options.spaces_in_function {
                        formatted.push(' ');
                    }
                    formatted.push(')');
                    line_start = false;

                    if paren_depth > 0 {
                        paren_depth -= 1;
                    }

                    if pending_arrow_check && paren_depth == 0 {
                        pending_arrow_check = false;
                        if next_char == '=' && chars.peek().map(|&c| c == '>').unwrap_or(false) {
                            in_arrow_function = true;
                        }
                    }
                }

                ':' if !in_string && !in_comment && !in_multiline_comment && !in_template => {
                    if !in_type_annotation && !in_object && !in_generic {
                        in_type_annotation = true;
                    }
                    self.append_char(&mut formatted, ':', indent_level, line_start);
                    if self.options.spaces_around_operators {
                        formatted.push(' ');
                    }
                    line_start = false;
                }

                '=' if !in_string && !in_comment && !in_multiline_comment && !in_template => {
                    if next_char == '>' && prev_char == ')' {
                        // 检查是否是单参数箭头函数，根据选项省略括号
                        if matches!(self.options.arrow_parens, ArrowParens::Avoid) {
                            // 检查括号内是否只有一个参数
                            let mut paren_count = 0;
                            let mut param_count = 0;
                            let mut found_left_paren = false;
                            let mut params_str = String::new();

                            // 从后向前查找最近的左括号
                            for (i, ch) in formatted.char_indices().rev() {
                                match ch {
                                    ')' => {
                                        paren_count += 1;
                                    }
                                    '(' => {
                                        paren_count -= 1;
                                        if paren_count == 0 {
                                            // 找到匹配的左括号，检查参数数量
                                            params_str = formatted[i + 1..formatted.len() - 1].to_string();
                                            found_left_paren = true;
                                            break;
                                        }
                                    }
                                    _ => {}
                                }
                            }

                            if found_left_paren {
                                let params_trimmed = params_str.trim();
                                param_count = params_trimmed.split(',').filter(|p| !p.trim().is_empty()).count();

                                if param_count == 1 {
                                    // 单参数箭头函数，省略括号
                                    // 移除括号
                                    let last_paren_pos = formatted.rfind('(').unwrap_or(0);
                                    let param = formatted[last_paren_pos + 1..formatted.len() - 1].trim().to_string();
                                    formatted.truncate(last_paren_pos);
                                    formatted.push_str(&param);
                                }
                            }
                        }

                        formatted.push_str("=>");
                        chars.next();
                        line_start = false;
                        continue;
                    }
                    in_type_annotation = false;
                    self.handle_operator(&mut formatted, c, indent_level, line_start);
                    line_start = false;
                }

                ';' if !in_string && !in_comment && !in_multiline_comment && !in_template => {
                    in_type_annotation = false;
                    if self.options.semicolons {
                        formatted.push(';');
                        if self.options.space_after_semicolon {
                            formatted.push(' ');
                        }
                        formatted.push('\n');
                        line_start = true;
                    }
                }

                ',' if !in_string && !in_comment && !in_multiline_comment && !in_template => {
                    formatted.push(',');

                    if self.should_add_trailing_comma(in_object, in_array, next_char) {
                        formatted.push(',');
                    }

                    if self.options.space_after_comma {
                        formatted.push(' ');
                    }
                    line_start = false;

                    if in_arrow_function {
                        arrow_param_count += 1;
                    }
                }

                '+' | '-' | '*' | '/' | '%' | '>' | '<' | '!' | '&' | '|' | '^' | '~' | '?'
                    if !in_string && !in_comment && !in_multiline_comment && !in_template =>
                {
                    if c == '/' && (prev_char == '*' || prev_char == '/') {
                        self.append_char(&mut formatted, c, indent_level, line_start);
                    }
                    else {
                        self.handle_operator(&mut formatted, c, indent_level, line_start);
                    }
                    line_start = false;
                }

                ' ' if line_start || (prev_char == ' ' && !in_string && !in_comment && !in_multiline_comment) => {}

                _ => {
                    self.append_char(&mut formatted, c, indent_level, line_start);
                    line_start = false;
                }
            }
            prev_char = c;
        }

        self.normalize_semicolons(formatted)
    }

    /// 处理运算符周围的空格
    fn handle_operator(&self, formatted: &mut String, op: char, indent_level: u32, line_start: bool) {
        if self.options.spaces_around_operators && !line_start {
            formatted.push(' ');
        }
        self.append_char(formatted, op, indent_level, line_start);
        if self.options.spaces_around_operators {
            formatted.push(' ');
        }
    }

    /// 判断是否应该添加尾随逗号
    fn should_add_trailing_comma(&self, in_object: bool, in_array: bool, next_char: char) -> bool {
        if !matches!(self.options.trailing_comma, TrailingComma::None) {
            let is_multiline_context = next_char == '\n' || next_char == '\r';
            if is_multiline_context {
                if in_object || in_array {
                    return true;
                }
                if matches!(self.options.trailing_comma, TrailingComma::All) {
                    return true;
                }
            }
        }
        false
    }

    /// 规范化引号
    fn normalize_quotes(&self, text: &str) -> String {
        if !self.options.single_quote {
            return text.to_string();
        }

        let mut result = String::new();
        let mut in_string = false;
        let mut string_char = '\0';
        let mut prev_char = '\0';

        for c in text.chars() {
            match c {
                '"' | '\'' if prev_char != '\\' && !in_string => {
                    in_string = true;
                    string_char = c;
                    result.push('\'');
                }
                '"' | '\'' if prev_char != '\\' && in_string && c == string_char => {
                    in_string = false;
                    string_char = '\0';
                    result.push('\'');
                }
                _ => {
                    result.push(c);
                }
            }
            prev_char = c;
        }

        result
    }

    /// 规范化分号
    fn normalize_semicolons(&self, text: String) -> String {
        if self.options.semicolons {
            text
        }
        else {
            let mut result = String::new();
            let mut chars = text.chars().peekable();
            let mut in_string = false;
            let mut string_char = '\0';
            let mut prev_char = '\0';

            while let Some(c) = chars.next() {
                match c {
                    '"' | '\'' if prev_char != '\\' && !in_string => {
                        in_string = true;
                        string_char = c;
                        result.push(c);
                    }
                    '"' | '\'' if prev_char != '\\' && in_string && c == string_char => {
                        in_string = false;
                        string_char = '\0';
                        result.push(c);
                    }
                    ';' if !in_string => {
                        let next_char = chars.peek().copied().unwrap_or('\0');
                        if next_char == '\n' || next_char == '\r' || next_char == '}' || next_char == '\0' {
                        }
                        else {
                            result.push(c);
                        }
                    }
                    _ => {
                        result.push(c);
                    }
                }
                prev_char = c;
            }

            result
        }
    }

    /// 处理尾随逗号
    fn handle_trailing_comma(&self, text: &str) -> String {
        match self.options.trailing_comma {
            TrailingComma::None => self.remove_trailing_commas(text),
            TrailingComma::ES5 | TrailingComma::All => self.add_trailing_commas(text),
        }
    }

    /// 移除尾随逗号
    fn remove_trailing_commas(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len());
        let mut chars = text.chars().peekable();
        let mut in_string = false;
        let mut string_char = '\0';
        let mut prev_char = '\0';

        while let Some(c) = chars.next() {
            match c {
                '"' | '\'' if prev_char != '\\' && !in_string => {
                    in_string = true;
                    string_char = c;
                    result.push(c);
                }
                '"' | '\'' if prev_char != '\\' && in_string && c == string_char => {
                    in_string = false;
                    string_char = '\0';
                    result.push(c);
                }
                ',' if !in_string => {
                    let next_char = chars.peek().copied().unwrap_or('\0');
                    if next_char == '\n' || next_char == '\r' || next_char == '}' || next_char == ']' || next_char == ')' {
                        continue;
                    }
                    result.push(c);
                }
                _ => {
                    result.push(c);
                }
            }
            prev_char = c;
        }

        result
    }

    /// 添加尾随逗号
    fn add_trailing_commas(&self, text: &str) -> String {
        let mut result = String::with_capacity(text.len() * 2);
        let mut chars = text.chars().peekable();
        let mut in_string = false;
        let mut string_char = '\0';
        let mut prev_char = '\0';
        let mut in_array = false;
        let mut in_object = false;
        let mut in_function = false;

        while let Some(c) = chars.next() {
            match c {
                '"' | '\'' if prev_char != '\\' && !in_string => {
                    in_string = true;
                    string_char = c;
                    result.push(c);
                }
                '"' | '\'' if prev_char != '\\' && in_string && c == string_char => {
                    in_string = false;
                    string_char = '\0';
                    result.push(c);
                }
                '[' if !in_string => {
                    in_array = true;
                    result.push(c);
                }
                ']' if !in_string && in_array => {
                    in_array = false;
                    // 检查是否需要添加尾随逗号
                    let mut needs_trailing_comma = false;
                    let mut temp_chars = result.chars().rev();
                    let mut seen_non_whitespace = false;

                    while let Some(ch) = temp_chars.next() {
                        if ch.is_whitespace() {
                            continue;
                        }
                        else if ch == '[' {
                            break;
                        }
                        else if ch != ',' {
                            needs_trailing_comma = true;
                            break;
                        }
                        else {
                            break;
                        }
                    }

                    if needs_trailing_comma {
                        result.push(',');
                    }
                    result.push(c);
                }
                '{' if !in_string => {
                    in_object = true;
                    result.push(c);
                }
                '}' if !in_string && in_object => {
                    in_object = false;
                    // 检查是否需要添加尾随逗号
                    let mut needs_trailing_comma = false;
                    let mut temp_chars = result.chars().rev();
                    let mut seen_non_whitespace = false;

                    while let Some(ch) = temp_chars.next() {
                        if ch.is_whitespace() {
                            continue;
                        }
                        else if ch == '{' {
                            break;
                        }
                        else if ch != ',' {
                            needs_trailing_comma = true;
                            break;
                        }
                        else {
                            break;
                        }
                    }

                    if needs_trailing_comma {
                        result.push(',');
                    }
                    result.push(c);
                }
                '(' if !in_string && prev_char.is_alphabetic() => {
                    in_function = true;
                    result.push(c);
                }
                ')' if !in_string && in_function && matches!(self.options.trailing_comma, TrailingComma::All) => {
                    in_function = false;
                    // 检查是否需要添加尾随逗号
                    let mut needs_trailing_comma = false;
                    let mut temp_chars = result.chars().rev();
                    let mut seen_non_whitespace = false;

                    while let Some(ch) = temp_chars.next() {
                        if ch.is_whitespace() {
                            continue;
                        }
                        else if ch == '(' {
                            break;
                        }
                        else if ch != ',' {
                            needs_trailing_comma = true;
                            break;
                        }
                        else {
                            break;
                        }
                    }

                    if needs_trailing_comma {
                        result.push(',');
                    }
                    result.push(c);
                }
                _ => {
                    result.push(c);
                }
            }
            prev_char = c;
        }

        result
    }

    /// 追加字符并处理缩进
    fn append_char(&self, formatted: &mut String, c: char, indent_level: u32, line_start: bool) {
        if line_start {
            self.append_indent(formatted, indent_level);
        }
        formatted.push(c);
    }

    /// 根据当前缩进级别追加缩进
    fn append_indent(&self, formatted: &mut String, indent_level: u32) {
        if self.options.use_tabs {
            formatted.push_str(&"\t".repeat(indent_level as usize));
        }
        else {
            formatted.push_str(&" ".repeat((indent_level * self.options.indent_size) as usize));
        }
    }
}

/// 使用默认选项格式化代码
pub fn format_code(text: &str) -> String {
    let options = FormatOptions::default();
    let formatter = Formatter::new(options);
    formatter.format_document(text)
}

/// 使用自定义选项格式化代码
pub fn format_code_with_options(text: &str, options: FormatOptions) -> String {
    let formatter = Formatter::new(options);
    formatter.format_document(text)
}

/// 格式化代码中的指定范围
pub fn format_range(text: &str, start: usize, end: usize, options: FormatOptions) -> String {
    let formatter = Formatter::new(options);
    formatter.format_range(text, start, end)
}

/// 创建用于格式化整个文档的 TextEdit
pub fn create_formatting_edit(text: &str, options: FormatOptions) -> TextEdit {
    let formatted = format_code_with_options(text, options);
    TextEdit { range: Range { start: 0, end: text.len() }, new_text: formatted }
}
