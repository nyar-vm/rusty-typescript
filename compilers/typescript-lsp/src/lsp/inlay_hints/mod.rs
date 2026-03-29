//! 内联提示模块
//!
//! 提供代码中的内联提示功能，如参数名称提示、类型推断提示等。

use crate::lsp::symbols::{SymbolKind, SymbolTable};
use core::range::Range;

/// 内联提示类型
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InlayHintKind {
    /// 参数名称提示
    ParameterName,
    /// 类型推断提示
    TypeInference,
    /// 枚举成员值提示
    EnumMemberValue,
    /// 函数返回类型提示
    ReturnType,
}

/// 内联提示
#[derive(Clone, Debug)]
pub struct InlayHint {
    /// 提示位置（字符偏移量）
    pub position: usize,
    /// 提示标签
    pub label: String,
    /// 提示类型
    pub kind: InlayHintKind,
    /// 工具提示
    pub tooltip: Option<String>,
}

impl InlayHint {
    /// 创建新的内联提示
    pub fn new(position: usize, label: impl Into<String>, kind: InlayHintKind) -> Self {
        Self { position, label: label.into(), kind, tooltip: None }
    }

    /// 设置工具提示
    pub fn with_tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// 创建参数名称提示
    pub fn parameter_name(position: usize, name: impl Into<String>) -> Self {
        Self::new(position, format!("{}:", name.into()), InlayHintKind::ParameterName)
    }

    /// 创建类型推断提示
    pub fn type_inference(position: usize, type_name: impl Into<String>) -> Self {
        Self::new(position, format!(": {}", type_name.into()), InlayHintKind::TypeInference)
    }

    /// 创建枚举成员值提示
    pub fn enum_value(position: usize, value: impl Into<String>) -> Self {
        Self::new(position, format!(" = {}", value.into()), InlayHintKind::EnumMemberValue)
    }
}

/// 内联提示提供者
pub struct InlayHintProvider {
    /// 符号表
    symbol_table: SymbolTable,
}

impl InlayHintProvider {
    /// 创建新的内联提示提供者
    pub fn new(symbol_table: SymbolTable) -> Self {
        Self { symbol_table }
    }

    /// 为指定范围提供内联提示
    pub fn provide_hints(&self, content: &str, range: Range<usize>) -> Vec<InlayHint> {
        let mut hints = Vec::new();
        let range_text = &content[range.start..range.end.min(content.len())];

        /// 参数名称提示
        hints.extend(self.provide_parameter_name_hints(range_text, range.start));

        /// 类型推断提示
        hints.extend(self.provide_type_inference_hints(range_text, range.start));

        /// 枚举成员值提示
        hints.extend(self.provide_enum_member_hints(range_text, range.start));

        hints
    }

    /// 为函数调用提供参数名称提示
    fn provide_parameter_name_hints(&self, content: &str, base_offset: usize) -> Vec<InlayHint> {
        let mut hints = Vec::new();

        for (line_idx, line) in content.lines().enumerate() {
            let line_offset = self.get_line_offset(content, line_idx);

            /// 查找函数调用模式
            if let Some(paren_idx) = line.find('(') {
                let before_paren = &line[..paren_idx];
                let func_name = before_paren.trim().split_whitespace().last().unwrap_or("");

                if !func_name.is_empty() {
                    /// 尝试从符号表获取函数参数信息
                    if let Some(symbol) = self.symbol_table.find_by_name(func_name) {
                        if symbol.kind == SymbolKind::Function {
                            if let Some(ref type_annotation) = symbol.type_annotation {
                                let params = self.extract_parameter_names(type_annotation);
                                let arg_positions = self.find_argument_positions(line, paren_idx);

                                for (i, (arg_pos, param_name)) in arg_positions.iter().zip(params.iter()).enumerate() {
                                    hints.push(InlayHint::parameter_name(
                                        base_offset + line_offset + arg_pos,
                                        param_name.clone(),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        hints
    }

    /// 为变量声明提供类型推断提示
    fn provide_type_inference_hints(&self, content: &str, base_offset: usize) -> Vec<InlayHint> {
        let mut hints = Vec::new();

        for (line_idx, line) in content.lines().enumerate() {
            let line_offset = self.get_line_offset(content, line_idx);

            /// 检测使用类型推断的变量声明（没有显式类型注解）
            if (line.trim().starts_with("const ") || line.trim().starts_with("let ") || line.trim().starts_with("var "))
                && !line.contains(":")
            {
                /// 提取变量名
                if let Some(var_name) = self.extract_variable_name(line) {
                    /// 查找符号表中的类型信息
                    if let Some(symbol) = self.symbol_table.find_by_name(&var_name) {
                        if let Some(ref type_annotation) = symbol.type_annotation {
                            /// 在变量名后添加类型提示
                            if let Some(var_end) = line.find(&var_name) {
                                let var_end_pos = var_end + var_name.len();
                                hints.push(InlayHint::type_inference(
                                    base_offset + line_offset + var_end_pos,
                                    type_annotation.clone(),
                                ));
                            }
                        }
                    }
                }
            }
        }

        hints
    }

    /// 为枚举成员提供值提示
    fn provide_enum_member_hints(&self, content: &str, base_offset: usize) -> Vec<InlayHint> {
        let mut hints = Vec::new();
        let mut in_enum = false;
        let mut enum_value = 0;

        for (line_idx, line) in content.lines().enumerate() {
            let line_offset = self.get_line_offset(content, line_idx);
            let trimmed = line.trim();

            /// 检测枚举开始
            if trimmed.starts_with("enum ") {
                in_enum = true;
                enum_value = 0;
                continue;
            }

            /// 检测枚举结束
            if in_enum && trimmed.starts_with('}') {
                in_enum = false;
                continue;
            }

            /// 在枚举内部检测成员
            if in_enum && !trimmed.is_empty() && !trimmed.starts_with("//") {
                /// 检查是否已经有显式值
                if !trimmed.contains('=') {
                    /// 提取成员名
                    if let Some(member_name) = trimmed.split(|c: char| c == ',' || c == ' ' || c == '\t').next() {
                        if !member_name.is_empty() && !member_name.starts_with("//") {
                            /// 在成员名后添加值提示
                            if let Some(name_end) = line.find(member_name) {
                                let name_end_pos = name_end + member_name.len();
                                hints.push(InlayHint::enum_value(
                                    base_offset + line_offset + name_end_pos,
                                    enum_value.to_string(),
                                ));
                                enum_value += 1;
                            }
                        }
                    }
                }
                else {
                    /// 如果有显式值，解析并更新 enum_value
                    if let Some(eq_pos) = trimmed.find('=') {
                        let value_str = &trimmed[eq_pos + 1..].trim();
                        if let Ok(val) = value_str.trim_matches(|c| c == ',' || c == ' ').parse::<i32>() {
                            enum_value = val + 1;
                        }
                    }
                }
            }
        }

        hints
    }

    /// 从函数签名中提取参数名称
    fn extract_parameter_names(&self, type_annotation: &str) -> Vec<String> {
        let mut params = Vec::new();

        /// 查找括号内的参数列表
        if let Some(start) = type_annotation.find('(') {
            if let Some(end) = type_annotation.find(')') {
                let params_str = &type_annotation[start + 1..end];

                /// 分割参数
                for param in params_str.split(',') {
                    let param = param.trim();
                    if !param.is_empty() {
                        /// 提取参数名（冒号前的部分）
                        if let Some(colon_pos) = param.find(':') {
                            let param_name = param[..colon_pos].trim();
                            params.push(param_name.to_string());
                        }
                        else {
                            /// 没有类型注解，整个作为参数名
                            params.push(param.to_string());
                        }
                    }
                }
            }
        }

        params
    }

    /// 查找函数调用中参数的位置
    fn find_argument_positions(&self, line: &str, paren_start: usize) -> Vec<usize> {
        let mut positions = Vec::new();
        let after_paren = &line[paren_start + 1..];

        let mut depth = 0;
        let mut in_string = false;
        let mut string_char = '\0';
        let mut escape_next = false;
        let mut arg_start = 0;
        let mut found_first = false;

        for (idx, ch) in after_paren.char_indices() {
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
                    if depth == 0 {
                        /// 记录最后一个参数
                        if found_first {
                            positions.push(paren_start + 1 + arg_start);
                        }
                        break;
                    }
                    depth -= 1;
                }
                ']' | '}' => depth -= 1,
                ',' if depth == 0 => {
                    positions.push(paren_start + 1 + arg_start);
                    arg_start = idx + 1;
                    found_first = true;
                }
                c if !c.is_whitespace() && !found_first => {
                    found_first = true;
                    arg_start = idx;
                }
                '"' | '\'' => {
                    in_string = true;
                    string_char = ch;
                }
                _ => {}
            }
        }

        positions
    }

    /// 从变量声明行提取变量名
    fn extract_variable_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();

        /// 移除 const/let/var
        let after_keyword = if trimmed.starts_with("const ") {
            &trimmed[6..]
        }
        else if trimmed.starts_with("let ") {
            &trimmed[4..]
        }
        else if trimmed.starts_with("var ") {
            &trimmed[4..]
        }
        else {
            return None;
        };

        /// 提取变量名（直到空格、等号、冒号等）
        let name: String = after_keyword.chars().take_while(|&c| c.is_alphanumeric() || c == '_' || c == '$').collect();

        if name.is_empty() { None } else { Some(name) }
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


