use oak_core::Range;
use oak_lsp::types::TextEdit;

/// Format options for code formatting.
#[derive(Debug, Default, Clone)]
pub struct FormatOptions {
    /// Indent size in spaces.
    pub indent_size: u32,
    /// Use tabs instead of spaces.
    pub use_tabs: bool,
    /// Maximum line width.
    pub line_width: u32,
    /// Add space before opening braces.
    pub space_before_brace: bool,
    /// Add space after commas.
    pub space_after_comma: bool,
    /// Add space after semicolons.
    pub space_after_semicolon: bool,
    /// Add spaces around operators.
    pub spaces_around_operators: bool,
    /// Add spaces inside function parentheses.
    pub spaces_in_function: bool,
    /// Add spaces inside object brackets.
    pub spaces_in_object: bool,
}

/// Formatter for TypeScript code.
pub struct Formatter {
    options: FormatOptions,
}

impl Formatter {
    /// Creates a new Formatter with the given options.
    pub fn new(options: FormatOptions) -> Self {
        Self { options }
    }

    /// Formats the entire document.
    pub fn format_document(&self, text: &str) -> String {
        self.format_range(text, 0, text.len())
    }

    /// Formats a specific range within the document.
    pub fn format_range(&self, text: &str, start: usize, end: usize) -> String {
        let range_text = &text[start..end];
        let formatted_range = self.format_text(range_text);

        // Keep text outside the range unchanged
        let mut result = String::new();
        result.push_str(&text[..start]);
        result.push_str(&formatted_range);
        result.push_str(&text[end..]);

        result
    }

    /// Internal method to format text content.
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
                // String handling
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

                // Single-line comment
                '/' if prev_char == '/' && !in_string && !in_multiline_comment => {
                    in_comment = true;
                    self.append_char(&mut formatted, c, indent_level, line_start);
                    line_start = false;
                }

                // Multi-line comment
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

                // Newline handling
                '\n' => {
                    in_comment = false;
                    formatted.push('\n');
                    line_start = true;
                }

                // Brace handling
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

                // Semicolon handling
                ';' if !in_string && !in_comment && !in_multiline_comment => {
                    formatted.push(';');
                    if self.options.space_after_semicolon {
                        formatted.push(' ');
                    }
                    formatted.push('\n');
                    line_start = true;
                }

                // Comma handling
                ',' if !in_string && !in_comment && !in_multiline_comment => {
                    formatted.push(',');
                    if self.options.space_after_comma {
                        formatted.push(' ');
                    }
                    line_start = false;
                }

                // Operator handling
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

                // Two-character operators
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

                // Parentheses handling
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

                // Whitespace handling
                ' ' if line_start || (prev_char == ' ' && !in_string && !in_comment && !in_multiline_comment) => {
                    // Skip leading and consecutive spaces
                }

                // Other characters
                _ => {
                    self.append_char(&mut formatted, c, indent_level, line_start);
                    line_start = false;
                }
            }
            prev_char = c;
        }

        formatted
    }

    /// Appends a character with proper indentation.
    fn append_char(&self, formatted: &mut String, c: char, indent_level: u32, line_start: bool) {
        if line_start {
            self.append_indent(formatted, indent_level);
        }
        formatted.push(c);
    }

    /// Appends indentation based on current level.
    fn append_indent(&self, formatted: &mut String, indent_level: u32) {
        if self.options.use_tabs {
            formatted.push_str(&"\t".repeat(indent_level as usize));
        }
        else {
            formatted.push_str(&" ".repeat((indent_level * self.options.indent_size) as usize));
        }
    }
}

/// Formats code with default options.
pub fn format_code(text: &str) -> String {
    let options = FormatOptions::default();
    let formatter = Formatter::new(options);
    formatter.format_document(text)
}

/// Formats code with custom options.
pub fn format_code_with_options(text: &str, options: FormatOptions) -> String {
    let formatter = Formatter::new(options);
    formatter.format_document(text)
}

/// Formats a specific range within the code.
pub fn format_range(text: &str, start: usize, end: usize, options: FormatOptions) -> String {
    let formatter = Formatter::new(options);
    formatter.format_range(text, start, end)
}

/// Creates a TextEdit for formatting the entire document.
pub fn create_formatting_edit(text: &str, options: FormatOptions) -> TextEdit {
    let formatted = format_code_with_options(text, options);
    TextEdit { range: Range { start: 0, end: text.len() }, new_text: formatted }
}
