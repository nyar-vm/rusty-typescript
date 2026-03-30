//! WebIDL 类型检查模块
//!
//! 本模块提供 WebIDL 类型检查、类型兼容性验证、类型推断和错误报告功能。

use crate::types::WebIdlResult;
use oak_idl::ast::{IdlItem, IdlRoot};
use std::collections::HashMap;

/// 类型兼容性结果
///
/// 表示类型兼容性检查的详细结果，包含是否兼容、不兼容原因和建议修复方案。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeCompatibilityResult {
    /// 是否兼容
    pub is_compatible: bool,
    /// 不兼容原因
    pub reason: Option<String>,
    /// 建议修复方案
    pub suggestions: Vec<String>,
    /// 兼容性详情
    pub details: CompatibilityDetails,
}

/// 兼容性详情
///
/// 包含类型兼容性检查的详细信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityDetails {
    /// 源类型
    pub source_type: String,
    /// 目标类型
    pub target_type: String,
    /// 兼容性级别
    pub level: CompatibilityLevel,
    /// 匹配的类型映射
    pub matched_mappings: Vec<(String, String)>,
}

/// 兼容性级别
///
/// 表示类型之间的兼容程度。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompatibilityLevel {
    /// 完全兼容
    Full,
    /// 部分兼容
    Partial,
    /// 不兼容
    Incompatible,
    /// 需要类型转换
    NeedsConversion,
}

impl Default for TypeCompatibilityResult {
    fn default() -> Self {
        Self {
            is_compatible: true,
            reason: None,
            suggestions: Vec::new(),
            details: CompatibilityDetails {
                source_type: String::new(),
                target_type: String::new(),
                level: CompatibilityLevel::Full,
                matched_mappings: Vec::new(),
            },
        }
    }
}

impl TypeCompatibilityResult {
    /// 创建兼容的结果
    ///
    /// # 参数
    ///
    /// * `source_type` - 源类型
    /// * `target_type` - 目标类型
    ///
    /// # 返回值
    ///
    /// 返回表示兼容的结果
    pub fn compatible(source_type: impl Into<String>, target_type: impl Into<String>) -> Self {
        Self {
            is_compatible: true,
            reason: None,
            suggestions: Vec::new(),
            details: CompatibilityDetails {
                source_type: source_type.into(),
                target_type: target_type.into(),
                level: CompatibilityLevel::Full,
                matched_mappings: Vec::new(),
            },
        }
    }

    /// 创建不兼容的结果
    ///
    /// # 参数
    ///
    /// * `source_type` - 源类型
    /// * `target_type` - 目标类型
    /// * `reason` - 不兼容原因
    ///
    /// # 返回值
    ///
    /// 返回表示不兼容的结果
    pub fn incompatible(source_type: impl Into<String>, target_type: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            is_compatible: false,
            reason: Some(reason.into()),
            suggestions: Vec::new(),
            details: CompatibilityDetails {
                source_type: source_type.into(),
                target_type: target_type.into(),
                level: CompatibilityLevel::Incompatible,
                matched_mappings: Vec::new(),
            },
        }
    }

    /// 添加建议修复方案
    ///
    /// # 参数
    ///
    /// * `suggestion` - 建议修复方案
    ///
    /// # 返回值
    ///
    /// 返回修改后的结果
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    /// 设置兼容性级别
    ///
    /// # 参数
    ///
    /// * `level` - 兼容性级别
    ///
    /// # 返回值
    ///
    /// 返回修改后的结果
    pub fn with_level(mut self, level: CompatibilityLevel) -> Self {
        self.details.level = level;
        self
    }

    /// 添加匹配的类型映射
    ///
    /// # 参数
    ///
    /// * `source` - 源类型
    /// * `target` - 目标类型
    ///
    /// # 返回值
    ///
    /// 返回修改后的结果
    pub fn with_mapping(mut self, source: impl Into<String>, target: impl Into<String>) -> Self {
        self.details.matched_mappings.push((source.into(), target.into()));
        self
    }
}

/// 类型错误
///
/// 表示类型检查过程中发现的错误，包含错误信息、位置和建议修复方案。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeError {
    /// 错误信息
    pub message: String,
    /// 错误代码
    pub code: String,
    /// 错误位置（行号，从 1 开始）
    pub line: Option<usize>,
    /// 错误位置（列号，从 1 开始）
    pub column: Option<usize>,
    /// 源文件路径
    pub file_path: Option<String>,
    /// 建议修复方案
    pub suggestions: Vec<String>,
    /// 相关类型
    pub related_types: Vec<String>,
    /// 错误严重程度
    pub severity: TypeErrorSeverity,
}

/// 类型错误严重程度
///
/// 表示类型错误的严重级别。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeErrorSeverity {
    /// 错误
    Error,
    /// 警告
    Warning,
    /// 信息
    Info,
    /// 提示
    Hint,
}

impl TypeError {
    /// 创建新的类型错误
    ///
    /// # 参数
    ///
    /// * `message` - 错误信息
    /// * `code` - 错误代码
    ///
    /// # 返回值
    ///
    /// 返回新的类型错误实例
    pub fn new(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: code.into(),
            line: None,
            column: None,
            file_path: None,
            suggestions: Vec::new(),
            related_types: Vec::new(),
            severity: TypeErrorSeverity::Error,
        }
    }

    /// 设置错误位置
    ///
    /// # 参数
    ///
    /// * `line` - 行号
    /// * `column` - 列号
    ///
    /// # 返回值
    ///
    /// 返回修改后的错误实例
    pub fn with_location(mut self, line: usize, column: usize) -> Self {
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    /// 设置源文件路径
    ///
    /// # 参数
    ///
    /// * `path` - 文件路径
    ///
    /// # 返回值
    ///
    /// 返回修改后的错误实例
    pub fn with_file(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    /// 添加建议修复方案
    ///
    /// # 参数
    ///
    /// * `suggestion` - 建议修复方案
    ///
    /// # 返回值
    ///
    /// 返回修改后的错误实例
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestions.push(suggestion.into());
        self
    }

    /// 添加相关类型
    ///
    /// # 参数
    ///
    /// * `type_name` - 相关类型名称
    ///
    /// # 返回值
    ///
    /// 返回修改后的错误实例
    pub fn with_related_type(mut self, type_name: impl Into<String>) -> Self {
        self.related_types.push(type_name.into());
        self
    }

    /// 设置错误严重程度
    ///
    /// # 参数
    ///
    /// * `severity` - 严重程度
    ///
    /// # 返回值
    ///
    /// 返回修改后的错误实例
    pub fn with_severity(mut self, severity: TypeErrorSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// 格式化错误信息
    ///
    /// # 返回值
    ///
    /// 返回格式化后的错误信息字符串
    pub fn format(&self) -> String {
        let mut result = String::new();

        let severity_str = match self.severity {
            TypeErrorSeverity::Error => "error",
            TypeErrorSeverity::Warning => "warning",
            TypeErrorSeverity::Info => "info",
            TypeErrorSeverity::Hint => "hint",
        };

        result.push_str(&format!("{}[{}]: {}", severity_str, self.code, self.message));

        if let Some(ref path) = self.file_path {
            result.push_str(&format!("\n  --> {}", path));
            if let (Some(line), Some(col)) = (self.line, self.column) {
                result.push_str(&format!(":{}:{}", line, col));
            }
        }

        if !self.related_types.is_empty() {
            result.push_str("\n  Related types:");
            for type_name in &self.related_types {
                result.push_str(&format!("\n    - {}", type_name));
            }
        }

        if !self.suggestions.is_empty() {
            result.push_str("\n  Suggestions:");
            for suggestion in &self.suggestions {
                result.push_str(&format!("\n    - {}", suggestion));
            }
        }

        result
    }
}

/// 类型错误收集器
///
/// 用于收集和管理类型检查过程中发现的所有错误。
#[derive(Debug, Clone, Default)]
pub struct TypeErrorCollector {
    /// 收集的错误列表
    errors: Vec<TypeError>,
    /// 错误计数
    error_count: usize,
    /// 警告计数
    warning_count: usize,
}

impl TypeErrorCollector {
    /// 创建新的错误收集器
    ///
    /// # 返回值
    ///
    /// 返回新的错误收集器实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 报告错误
    ///
    /// # 参数
    ///
    /// * `error` - 要报告的错误
    pub fn report_error(&mut self, error: TypeError) {
        match error.severity {
            TypeErrorSeverity::Error => self.error_count += 1,
            TypeErrorSeverity::Warning => self.warning_count += 1,
            _ => {}
        }
        self.errors.push(error);
    }

    /// 添加简单错误
    ///
    /// # 参数
    ///
    /// * `message` - 错误信息
    /// * `code` - 错误代码
    pub fn add_error(&mut self, message: impl Into<String>, code: impl Into<String>) {
        self.report_error(TypeError::new(message, code));
    }

    /// 添加带位置的简单错误
    ///
    /// # 参数
    ///
    /// * `message` - 错误信息
    /// * `code` - 错误代码
    /// * `line` - 行号
    /// * `column` - 列号
    pub fn add_error_with_location(&mut self, message: impl Into<String>, code: impl Into<String>, line: usize, column: usize) {
        self.report_error(TypeError::new(message, code).with_location(line, column));
    }

    /// 获取所有错误
    ///
    /// # 返回值
    ///
    /// 返回所有收集的错误列表
    pub fn get_errors(&self) -> &[TypeError] {
        &self.errors
    }

    /// 获取指定严重程度的错误
    ///
    /// # 参数
    ///
    /// * `severity` - 错误严重程度
    ///
    /// # 返回值
    ///
    /// 返回指定严重程度的错误列表
    pub fn get_errors_by_severity(&self, severity: TypeErrorSeverity) -> Vec<&TypeError> {
        self.errors.iter().filter(|e| e.severity == severity).collect()
    }

    /// 获取错误数量
    ///
    /// # 返回值
    ///
    /// 返回错误数量
    pub fn error_count(&self) -> usize {
        self.error_count
    }

    /// 获取警告数量
    ///
    /// # 返回值
    ///
    /// 返回警告数量
    pub fn warning_count(&self) -> usize {
        self.warning_count
    }

    /// 检查是否有错误
    ///
    /// # 返回值
    ///
    /// 如果有错误返回 true，否则返回 false
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    /// 检查是否有警告
    ///
    /// # 返回值
    ///
    /// 如果有警告返回 true，否则返回 false
    pub fn has_warnings(&self) -> bool {
        self.warning_count > 0
    }

    /// 清空所有错误
    pub fn clear(&mut self) {
        self.errors.clear();
        self.error_count = 0;
        self.warning_count = 0;
    }

    /// 格式化所有错误
    ///
    /// # 返回值
    ///
    /// 返回格式化后的错误信息字符串
    pub fn format_errors(&self) -> String {
        if self.errors.is_empty() {
            return String::from("No errors found.");
        }

        let mut result = String::new();
        result.push_str(&format!("Found {} error(s) and {} warning(s):\n\n", self.error_count, self.warning_count));

        for (i, error) in self.errors.iter().enumerate() {
            if i > 0 {
                result.push_str("\n\n");
            }
            result.push_str(&error.format());
        }

        result
    }

    /// 合并另一个错误收集器
    ///
    /// # 参数
    ///
    /// * `other` - 要合并的错误收集器
    pub fn merge(&mut self, other: TypeErrorCollector) {
        self.error_count += other.error_count;
        self.warning_count += other.warning_count;
        self.errors.extend(other.errors);
    }
}

/// 推断的类型信息
///
/// 表示类型推断的结果，包含推断出的类型和置信度。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InferredType {
    /// 推断的类型名称
    pub type_name: String,
    /// 类型置信度（0-100）
    pub confidence: u8,
    /// 类型来源
    pub source: TypeInferenceSource,
    /// 相关类型信息
    pub related_info: Vec<String>,
}

/// 类型推断来源
///
/// 表示类型是如何被推断出来的。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeInferenceSource {
    /// 从字面量推断
    Literal,
    /// 从表达式推断
    Expression,
    /// 从上下文推断
    Context,
    /// 从默认值推断
    DefaultValue,
    /// 从使用方式推断
    Usage,
    /// 从类型注解推断
    Annotation,
}

impl InferredType {
    /// 创建新的推断类型
    ///
    /// # 参数
    ///
    /// * `type_name` - 类型名称
    ///
    /// # 返回值
    ///
    /// 返回新的推断类型实例
    pub fn new(type_name: impl Into<String>) -> Self {
        Self { type_name: type_name.into(), confidence: 100, source: TypeInferenceSource::Literal, related_info: Vec::new() }
    }

    /// 设置置信度
    ///
    /// # 参数
    ///
    /// * `confidence` - 置信度（0-100）
    ///
    /// # 返回值
    ///
    /// 返回修改后的推断类型实例
    pub fn with_confidence(mut self, confidence: u8) -> Self {
        self.confidence = confidence.min(100);
        self
    }

    /// 设置推断来源
    ///
    /// # 参数
    ///
    /// * `source` - 推断来源
    ///
    /// # 返回值
    ///
    /// 返回修改后的推断类型实例
    pub fn with_source(mut self, source: TypeInferenceSource) -> Self {
        self.source = source;
        self
    }

    /// 添加相关信息
    ///
    /// # 参数
    ///
    /// * `info` - 相关信息
    ///
    /// # 返回值
    ///
    /// 返回修改后的推断类型实例
    pub fn with_related_info(mut self, info: impl Into<String>) -> Self {
        self.related_info.push(info.into());
        self
    }
}

/// 类型推断器
///
/// 用于根据值或表达式推断类型。
#[derive(Debug, Clone)]
pub struct TypeInference {
    /// 已知的类型别名映射
    type_aliases: HashMap<String, String>,
    /// 已知的接口定义
    interfaces: HashMap<String, Vec<InterfaceProperty>>,
}

/// 接口属性信息
///
/// 表示接口中的属性定义。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceProperty {
    /// 属性名称
    pub name: String,
    /// 属性类型
    pub type_name: String,
    /// 是否可选
    pub optional: bool,
    /// 是否只读
    pub readonly: bool,
}

impl TypeInference {
    /// 创建新的类型推断器
    ///
    /// # 返回值
    ///
    /// 返回新的类型推断器实例
    pub fn new() -> Self {
        Self { type_aliases: HashMap::new(), interfaces: HashMap::new() }
    }

    /// 从 WebIDL 结果创建类型推断器
    ///
    /// # 参数
    ///
    /// * `result` - WebIDL 解析结果
    ///
    /// # 返回值
    ///
    /// 返回新的类型推断器实例
    pub fn from_webidl_result(result: &WebIdlResult) -> Self {
        let mut inference = Self::new();
        inference.load_webidl_types(result);
        inference
    }

    /// 加载 WebIDL 类型定义
    ///
    /// # 参数
    ///
    /// * `result` - WebIDL 解析结果
    pub fn load_webidl_types(&mut self, result: &WebIdlResult) {
        for item in &result.root.items {
            match item {
                IdlItem::Interface(interface) => {
                    let properties: Vec<InterfaceProperty> = interface
                        .members
                        .iter()
                        .filter_map(|member| match member {
                            oak_idl::ast::IdlMember::Attribute(attr) => Some(InterfaceProperty {
                                name: attr.name.clone(),
                                type_name: attr.type_name.clone(),
                                optional: false,
                                readonly: attr.readonly,
                            }),
                            _ => None,
                        })
                        .collect();
                    self.interfaces.insert(interface.name.clone(), properties);
                }
                IdlItem::Struct(struct_) => {
                    let properties: Vec<InterfaceProperty> = struct_
                        .fields
                        .iter()
                        .map(|field| InterfaceProperty {
                            name: field.name.clone(),
                            type_name: field.type_name.clone(),
                            optional: field.type_name.ends_with('?'),
                            readonly: false,
                        })
                        .collect();
                    self.interfaces.insert(struct_.name.clone(), properties);
                }
                IdlItem::Typedef(typedef) => {
                    self.type_aliases.insert(typedef.name.clone(), typedef.type_name.clone());
                }
                _ => {}
            }
        }
    }

    /// 根据值推断类型
    ///
    /// # 参数
    ///
    /// * `value` - 要推断类型的值
    ///
    /// # 返回值
    ///
    /// 返回推断出的类型信息
    pub fn infer_type(&self, value: &str) -> InferredType {
        let trimmed = value.trim();

        if trimmed.is_empty() {
            return InferredType::new("undefined").with_source(TypeInferenceSource::Literal).with_confidence(100);
        }

        if trimmed == "null" {
            return InferredType::new("null").with_source(TypeInferenceSource::Literal).with_confidence(100);
        }

        if trimmed == "undefined" {
            return InferredType::new("undefined").with_source(TypeInferenceSource::Literal).with_confidence(100);
        }

        if trimmed == "true" || trimmed == "false" {
            return InferredType::new("boolean").with_source(TypeInferenceSource::Literal).with_confidence(100);
        }

        if Self::is_number_literal(trimmed) {
            return InferredType::new("number").with_source(TypeInferenceSource::Literal).with_confidence(100);
        }

        if Self::is_string_literal(trimmed) {
            return InferredType::new("string").with_source(TypeInferenceSource::Literal).with_confidence(100);
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            return self.infer_array_type(trimmed);
        }

        if trimmed.starts_with('{') && trimmed.ends_with('}') {
            return self.infer_object_type(trimmed);
        }

        if trimmed.starts_with("new ") {
            return self.infer_constructor_type(trimmed);
        }

        if trimmed.contains('(') && trimmed.contains(')') {
            return self.infer_function_expression_type(trimmed);
        }

        InferredType::new("any").with_source(TypeInferenceSource::Context).with_confidence(50)
    }

    /// 检查是否为数字字面量
    ///
    /// # 参数
    ///
    /// * `s` - 要检查的字符串
    ///
    /// # 返回值
    ///
    /// 如果是数字字面量返回 true
    fn is_number_literal(s: &str) -> bool {
        if s.is_empty() {
            return false;
        }

        let s = s.trim();

        if s.starts_with("0x") || s.starts_with("0X") {
            return s[2..].chars().all(|c| c.is_ascii_hexdigit());
        }

        if s.starts_with("0b") || s.starts_with("0B") {
            return s[2..].chars().all(|c| c == '0' || c == '1');
        }

        if s.starts_with("0o") || s.starts_with("0O") {
            return s[2..].chars().all(|c| c >= '0' && c <= '7');
        }

        let mut has_dot = false;
        let mut has_exp = false;
        let chars: Vec<char> = s.chars().collect();

        for (i, &c) in chars.iter().enumerate() {
            if i == 0 && (c == '+' || c == '-') {
                continue;
            }

            if c == '.' && !has_dot && !has_exp {
                has_dot = true;
                continue;
            }

            if (c == 'e' || c == 'E') && !has_exp {
                has_exp = true;
                continue;
            }

            if (c == '+' || c == '-') && i > 0 && (chars[i - 1] == 'e' || chars[i - 1] == 'E') {
                continue;
            }

            if !c.is_ascii_digit() {
                return false;
            }
        }

        !s.is_empty() && s != "." && s != "+" && s != "-"
    }

    /// 检查是否为字符串字面量
    ///
    /// # 参数
    ///
    /// * `s` - 要检查的字符串
    ///
    /// # 返回值
    ///
    /// 如果是字符串字面量返回 true
    fn is_string_literal(s: &str) -> bool {
        if s.len() < 2 {
            return false;
        }

        let first = s.chars().next().unwrap();
        let last = s.chars().last().unwrap();

        (first == '"' && last == '"') || (first == '\'' && last == '\'') || (first == '`' && last == '`')
    }

    /// 推断数组类型
    ///
    /// # 参数
    ///
    /// * `value` - 数组字面量字符串
    ///
    /// # 返回值
    ///
    /// 返回推断出的数组类型信息
    fn infer_array_type(&self, value: &str) -> InferredType {
        let inner = &value[1..value.len() - 1].trim();

        if inner.is_empty() {
            return InferredType::new("any[]")
                .with_source(TypeInferenceSource::Literal)
                .with_confidence(100)
                .with_related_info("Empty array");
        }

        let elements: Vec<&str> = self.split_array_elements(inner);
        let mut element_types: Vec<String> = Vec::new();

        for element in elements {
            let inferred = self.infer_type(element);
            element_types.push(inferred.type_name);
        }

        if element_types.is_empty() {
            return InferredType::new("any[]").with_source(TypeInferenceSource::Literal).with_confidence(100);
        }

        let unique_types: Vec<&String> = element_types.iter().collect::<std::collections::HashSet<_>>().into_iter().collect();

        if unique_types.len() == 1 {
            InferredType::new(format!("{}[]", unique_types[0])).with_source(TypeInferenceSource::Literal).with_confidence(95)
        }
        else {
            let union_type = element_types.join(" | ");
            InferredType::new(format!("({})[]", union_type)).with_source(TypeInferenceSource::Literal).with_confidence(90)
        }
    }

    /// 分割数组元素
    ///
    /// # 参数
    ///
    /// * `s` - 数组内容字符串
    ///
    /// # 返回值
    ///
    /// 返回分割后的元素列表
    fn split_array_elements<'a>(&self, s: &'a str) -> Vec<&'a str> {
        let mut elements = Vec::new();
        let mut depth = 0;
        let mut start = 0;
        let mut in_string = false;
        let mut string_char = '\0';

        for (i, c) in s.char_indices() {
            if in_string {
                if c == string_char && s.as_bytes().get(i.wrapping_sub(1)).copied() != Some(b'\\') {
                    in_string = false;
                }
            }
            else {
                match c {
                    '"' | '\'' | '`' => {
                        in_string = true;
                        string_char = c;
                    }
                    '[' | '{' | '(' => depth += 1,
                    ']' | '}' | ')' => depth -= 1,
                    ',' if depth == 0 => {
                        elements.push(s[start..i].trim());
                        start = i + 1;
                    }
                    _ => {}
                }
            }
        }

        if start < s.len() {
            elements.push(s[start..].trim());
        }

        elements
    }

    /// 推断对象类型
    ///
    /// # 参数
    ///
    /// * `value` - 对象字面量字符串
    ///
    /// # 返回值
    ///
    /// 返回推断出的对象类型信息
    fn infer_object_type(&self, value: &str) -> InferredType {
        let inner = &value[1..value.len() - 1].trim();

        if inner.is_empty() {
            return InferredType::new("object")
                .with_source(TypeInferenceSource::Literal)
                .with_confidence(100)
                .with_related_info("Empty object");
        }

        let properties = self.parse_object_properties(inner);
        let mut type_parts: Vec<String> = Vec::new();

        for (key, prop_value) in &properties {
            let inferred = self.infer_type(prop_value);
            type_parts.push(format!("{}: {}", key, inferred.type_name));
        }

        if type_parts.is_empty() {
            InferredType::new("object").with_source(TypeInferenceSource::Literal).with_confidence(100)
        }
        else {
            InferredType::new(format!("{{ {} }}", type_parts.join("; ")))
                .with_source(TypeInferenceSource::Literal)
                .with_confidence(90)
        }
    }

    /// 解析对象属性
    ///
    /// # 参数
    ///
    /// * `s` - 对象内容字符串
    ///
    /// # 返回值
    ///
    /// 返回解析后的属性列表
    fn parse_object_properties(&self, s: &str) -> Vec<(String, String)> {
        let mut properties = Vec::new();
        let mut depth = 0;
        let mut start = 0;
        let mut in_string = false;
        let mut string_char = '\0';

        for (i, c) in s.char_indices() {
            if in_string {
                if c == string_char && s.as_bytes().get(i.wrapping_sub(1)).copied() != Some(b'\\') {
                    in_string = false;
                }
            }
            else {
                match c {
                    '"' | '\'' | '`' => {
                        in_string = true;
                        string_char = c;
                    }
                    '[' | '{' | '(' => depth += 1,
                    ']' | '}' | ')' => depth -= 1,
                    ',' if depth == 0 => {
                        if let Some(prop) = self.parse_single_property(s[start..i].trim()) {
                            properties.push(prop);
                        }
                        start = i + 1;
                    }
                    _ => {}
                }
            }
        }

        if start < s.len() {
            if let Some(prop) = self.parse_single_property(s[start..].trim()) {
                properties.push(prop);
            }
        }

        properties
    }

    /// 解析单个属性
    ///
    /// # 参数
    ///
    /// * `s` - 属性字符串
    ///
    /// # 返回值
    ///
    /// 返回解析后的属性键值对
    fn parse_single_property(&self, s: &str) -> Option<(String, String)> {
        let colon_pos = s.find(':')?;
        let key = s[..colon_pos].trim().trim_matches('"').trim_matches('\'').to_string();
        let value = s[colon_pos + 1..].trim().to_string();
        Some((key, value))
    }

    /// 推断构造函数类型
    ///
    /// # 参数
    ///
    /// * `value` - 构造函数调用字符串
    ///
    /// # 返回值
    ///
    /// 返回推断出的类型信息
    fn infer_constructor_type(&self, value: &str) -> InferredType {
        let without_new = value[4..].trim();

        let type_name = if without_new.contains('(') {
            &without_new[..without_new.find('(').unwrap_or(without_new.len())]
        }
        else {
            without_new
        };

        InferredType::new(type_name.trim()).with_source(TypeInferenceSource::Expression).with_confidence(95)
    }

    /// 推断函数表达式类型
    ///
    /// # 参数
    ///
    /// * `value` - 函数表达式字符串
    ///
    /// # 返回值
    ///
    /// 返回推断出的类型信息
    fn infer_function_expression_type(&self, value: &str) -> InferredType {
        if value.starts_with("function") || value.starts_with("async function") {
            return InferredType::new("Function").with_source(TypeInferenceSource::Expression).with_confidence(90);
        }

        if value.starts_with('(') || value.starts_with("async (") {
            return InferredType::new("(...args: any[]) => any")
                .with_source(TypeInferenceSource::Expression)
                .with_confidence(85);
        }

        InferredType::new("any").with_source(TypeInferenceSource::Expression).with_confidence(50)
    }

    /// 推断表达式类型
    ///
    /// # 参数
    ///
    /// * `expression` - 表达式字符串
    ///
    /// # 返回值
    ///
    /// 返回推断出的类型信息
    pub fn infer_expression_type(&self, expression: &str) -> InferredType {
        let trimmed = expression.trim();

        if trimmed.is_empty() {
            return InferredType::new("undefined").with_source(TypeInferenceSource::Expression).with_confidence(100);
        }

        if let Some(idx) = trimmed.find(" + ") {
            let left = &trimmed[..idx];
            let right = &trimmed[idx + 3..];
            return self.infer_binary_expression_type(left, right, "+");
        }

        if let Some(idx) = trimmed.find(" - ") {
            let left = &trimmed[..idx];
            let right = &trimmed[idx + 3..];
            return self.infer_binary_expression_type(left, right, "-");
        }

        if let Some(idx) = trimmed.find(" * ") {
            let left = &trimmed[..idx];
            let right = &trimmed[idx + 3..];
            return self.infer_binary_expression_type(left, right, "*");
        }

        if let Some(idx) = trimmed.find(" / ") {
            let left = &trimmed[..idx];
            let right = &trimmed[idx + 3..];
            return self.infer_binary_expression_type(left, right, "/");
        }

        if let Some(idx) = trimmed.find(" && ") {
            let left = &trimmed[..idx];
            let right = &trimmed[idx + 4..];
            return self.infer_logical_expression_type(left, right, "&&");
        }

        if let Some(idx) = trimmed.find(" || ") {
            let left = &trimmed[..idx];
            let right = &trimmed[idx + 4..];
            return self.infer_logical_expression_type(left, right, "||");
        }

        if trimmed.starts_with('!') {
            return InferredType::new("boolean").with_source(TypeInferenceSource::Expression).with_confidence(100);
        }

        if trimmed.starts_with("typeof ") {
            return InferredType::new("string").with_source(TypeInferenceSource::Expression).with_confidence(100);
        }

        if trimmed.starts_with("instanceof ") {
            return InferredType::new("boolean").with_source(TypeInferenceSource::Expression).with_confidence(100);
        }

        if trimmed.contains('?') && trimmed.contains(':') {
            return self.infer_ternary_expression_type(trimmed);
        }

        if trimmed.contains('.') && !Self::is_number_literal(trimmed) {
            return self.infer_property_access_type(trimmed);
        }

        if trimmed.ends_with(']') && trimmed.contains('[') {
            return self.infer_index_access_type(trimmed);
        }

        self.infer_type(trimmed)
    }

    /// 推断二元表达式类型
    ///
    /// # 参数
    ///
    /// * `left` - 左操作数
    /// * `right` - 右操作数
    /// * `operator` - 运算符
    ///
    /// # 返回值
    ///
    /// 返回推断出的类型信息
    fn infer_binary_expression_type(&self, left: &str, right: &str, operator: &str) -> InferredType {
        let left_type = self.infer_type(left.trim());
        let right_type = self.infer_type(right.trim());

        match operator {
            "+" => {
                if left_type.type_name == "string" || right_type.type_name == "string" {
                    InferredType::new("string").with_source(TypeInferenceSource::Expression).with_confidence(95)
                }
                else {
                    InferredType::new("number").with_source(TypeInferenceSource::Expression).with_confidence(90)
                }
            }
            "-" | "*" | "/" | "%" => {
                InferredType::new("number").with_source(TypeInferenceSource::Expression).with_confidence(95)
            }
            _ => self.infer_type("any"),
        }
    }

    /// 推断逻辑表达式类型
    ///
    /// # 参数
    ///
    /// * `left` - 左操作数
    /// * `right` - 右操作数
    /// * `operator` - 运算符
    ///
    /// # 返回值
    ///
    /// 返回推断出的类型信息
    fn infer_logical_expression_type(&self, left: &str, right: &str, operator: &str) -> InferredType {
        let left_type = self.infer_type(left.trim());
        let right_type = self.infer_type(right.trim());

        match operator {
            "&&" => InferredType::new(right_type.type_name).with_source(TypeInferenceSource::Expression).with_confidence(80),
            "||" => InferredType::new(format!("{} | {}", left_type.type_name, right_type.type_name))
                .with_source(TypeInferenceSource::Expression)
                .with_confidence(80),
            _ => self.infer_type("any"),
        }
    }

    /// 推断三元表达式类型
    ///
    /// # 参数
    ///
    /// * `expression` - 三元表达式字符串
    ///
    /// # 返回值
    ///
    /// 返回推断出的类型信息
    fn infer_ternary_expression_type(&self, expression: &str) -> InferredType {
        let depth = Self::find_ternary_colon(expression);
        if let Some(colon_pos) = depth {
            let question_pos = expression.find('?').unwrap();
            let true_branch = expression[question_pos + 1..colon_pos].trim();
            let false_branch = expression[colon_pos + 1..].trim();

            let true_type = self.infer_type(true_branch);
            let false_type = self.infer_type(false_branch);

            if true_type.type_name == false_type.type_name {
                InferredType::new(true_type.type_name).with_source(TypeInferenceSource::Expression).with_confidence(95)
            }
            else {
                InferredType::new(format!("{} | {}", true_type.type_name, false_type.type_name))
                    .with_source(TypeInferenceSource::Expression)
                    .with_confidence(90)
            }
        }
        else {
            InferredType::new("any").with_source(TypeInferenceSource::Expression).with_confidence(50)
        }
    }

    /// 查找三元表达式的冒号位置
    ///
    /// # 参数
    ///
    /// * `expression` - 表达式字符串
    ///
    /// # 返回值
    ///
    /// 返回冒号位置
    fn find_ternary_colon(expression: &str) -> Option<usize> {
        let mut depth = 0;
        let mut after_question = false;

        for (i, c) in expression.char_indices() {
            match c {
                '?' => {
                    depth += 1;
                    after_question = true;
                }
                ':' if after_question && depth == 1 => {
                    return Some(i);
                }
                ':' if depth > 1 => {
                    depth -= 1;
                }
                _ => {}
            }
        }

        None
    }

    /// 推断属性访问类型
    ///
    /// # 参数
    ///
    /// * `expression` - 属性访问表达式字符串
    ///
    /// # 返回值
    ///
    /// 返回推断出的类型信息
    fn infer_property_access_type(&self, expression: &str) -> InferredType {
        let parts: Vec<&str> = expression.split('.').collect();
        if parts.len() < 2 {
            return InferredType::new("any").with_source(TypeInferenceSource::Expression).with_confidence(50);
        }

        let object_name = parts[0].trim();
        let property_name = parts[1].trim();

        if let Some(properties) = self.interfaces.get(object_name) {
            for prop in properties {
                if prop.name == property_name {
                    return InferredType::new(&prop.type_name)
                        .with_source(TypeInferenceSource::Expression)
                        .with_confidence(95)
                        .with_related_info(format!("Property of {}", object_name));
                }
            }
        }

        InferredType::new("any").with_source(TypeInferenceSource::Expression).with_confidence(60)
    }

    /// 推断索引访问类型
    ///
    /// # 参数
    ///
    /// * `expression` - 索引访问表达式字符串
    ///
    /// # 返回值
    ///
    /// 返回推断出的类型信息
    fn infer_index_access_type(&self, expression: &str) -> InferredType {
        let Some(open_bracket) = expression.find('[')
        else {
            return InferredType::new("any");
        };
        let Some(close_bracket) = expression.rfind(']')
        else {
            return InferredType::new("any");
        };

        let object_name = &expression[..open_bracket];
        let index_str = &expression[open_bracket + 1..close_bracket];

        let object_type = self.infer_type(object_name.trim());

        if object_type.type_name.ends_with("[]") {
            let element_type = &object_type.type_name[..object_type.type_name.len() - 2];
            return InferredType::new(element_type).with_source(TypeInferenceSource::Expression).with_confidence(90);
        }

        if object_type.type_name.starts_with("Array<") && object_type.type_name.ends_with('>') {
            let element_type = &object_type.type_name[6..object_type.type_name.len() - 1];
            return InferredType::new(element_type).with_source(TypeInferenceSource::Expression).with_confidence(90);
        }

        if Self::is_number_literal(index_str.trim()) {
            return InferredType::new("any")
                .with_source(TypeInferenceSource::Expression)
                .with_confidence(70)
                .with_related_info("Array element access");
        }

        InferredType::new("any").with_source(TypeInferenceSource::Expression).with_confidence(60)
    }

    /// 添加类型别名
    ///
    /// # 参数
    ///
    /// * `name` - 别名名称
    /// * `target` - 目标类型
    pub fn add_type_alias(&mut self, name: impl Into<String>, target: impl Into<String>) {
        self.type_aliases.insert(name.into(), target.into());
    }

    /// 添加接口定义
    ///
    /// # 参数
    ///
    /// * `name` - 接口名称
    /// * `properties` - 属性列表
    pub fn add_interface(&mut self, name: impl Into<String>, properties: Vec<InterfaceProperty>) {
        self.interfaces.insert(name.into(), properties);
    }
}

impl Default for TypeInference {
    fn default() -> Self {
        Self::new()
    }
}

/// WebIDL 类型检查器
///
/// 用于检查 TypeScript 类型与 WebIDL 类型之间的兼容性。
pub struct WebIdlTypeChecker {
    /// WebIDL 解析结果
    pub webidl_result: WebIdlResult,
    /// 类型错误收集器
    pub error_collector: TypeErrorCollector,
    /// 类型推断器
    pub type_inference: TypeInference,
}

impl WebIdlTypeChecker {
    /// 创建新的 WebIDL 类型检查器
    ///
    /// # 参数
    ///
    /// * `webidl_result` - WebIDL 解析结果
    ///
    /// # 返回值
    ///
    /// 返回新的 WebIDL 类型检查器实例
    pub fn new(webidl_result: WebIdlResult) -> Self {
        let type_inference = TypeInference::from_webidl_result(&webidl_result);
        Self { webidl_result, error_collector: TypeErrorCollector::new(), type_inference }
    }

    /// 检查 TypeScript 类型是否与 WebIDL 类型兼容
    ///
    /// # 参数
    ///
    /// * `ts_type` - TypeScript 类型名称
    /// * `webidl_type` - WebIDL 类型名称
    ///
    /// # 返回值
    ///
    /// 返回类型兼容性结果
    pub fn check_type_compatibility_detailed(&self, ts_type: &str, webidl_type: &str) -> TypeCompatibilityResult {
        let basic_map = self.get_basic_type_mappings();

        for (ts, webidl) in &basic_map {
            if ts_type == *ts && webidl_type == *webidl {
                return TypeCompatibilityResult::compatible(ts_type, webidl_type).with_mapping(ts_type, webidl_type);
            }
        }

        if let result @ TypeCompatibilityResult { is_compatible: true, .. } =
            self.check_function_type_compatibility(ts_type, webidl_type)
        {
            return result;
        }

        if let result @ TypeCompatibilityResult { is_compatible: true, .. } =
            self.check_object_type_compatibility(ts_type, webidl_type)
        {
            return result;
        }

        if let result @ TypeCompatibilityResult { is_compatible: true, .. } =
            self.check_array_type_compatibility(ts_type, webidl_type)
        {
            return result;
        }

        if let result @ TypeCompatibilityResult { is_compatible: true, .. } =
            self.check_complex_type_compatibility_detailed(ts_type, webidl_type)
        {
            return result;
        }

        if let result @ TypeCompatibilityResult { is_compatible: true, .. } =
            self.check_user_defined_types_detailed(ts_type, webidl_type)
        {
            return result;
        }

        TypeCompatibilityResult::incompatible(
            ts_type,
            webidl_type,
            format!("Type '{}' is not compatible with WebIDL type '{}'", ts_type, webidl_type),
        )
        .with_suggestion(format!("Consider using a compatible type for '{}'", webidl_type))
    }

    /// 获取基本类型映射
    ///
    /// # 返回值
    ///
    /// 返回基本类型映射列表
    fn get_basic_type_mappings(&self) -> Vec<(&'static str, &'static str)> {
        vec![
            ("string", "DOMString"),
            ("string", "ByteString"),
            ("string", "USVString"),
            ("string", "string"),
            ("number", "byte"),
            ("number", "octet"),
            ("number", "short"),
            ("number", "unsigned short"),
            ("number", "long"),
            ("number", "unsigned long"),
            ("number", "long long"),
            ("number", "unsigned long long"),
            ("number", "float"),
            ("number", "unrestricted float"),
            ("number", "double"),
            ("number", "unrestricted double"),
            ("number", "integer"),
            ("number", "unsigned integer"),
            ("boolean", "boolean"),
            ("any", "any"),
            ("undefined", "undefined"),
            ("null", "null"),
            ("void", "void"),
        ]
    }

    /// 检查函数类型兼容性
    ///
    /// # 参数
    ///
    /// * `ts_type` - TypeScript 函数类型
    /// * `webidl_type` - WebIDL 函数类型
    ///
    /// # 返回值
    ///
    /// 返回类型兼容性结果
    pub fn check_function_type_compatibility(&self, ts_type: &str, webidl_type: &str) -> TypeCompatibilityResult {
        let ts_params = self.parse_function_params(ts_type);
        let webidl_params = self.parse_function_params(webidl_type);

        if ts_params.is_none() && webidl_params.is_none() {
            return TypeCompatibilityResult::incompatible(ts_type, webidl_type, "Not function types");
        }

        let ts_params = ts_params.unwrap_or_default();
        let webidl_params = webidl_params.unwrap_or_default();

        let ts_return = self.extract_return_type(ts_type);
        let webidl_return = self.extract_return_type(webidl_type);

        if ts_params.len() != webidl_params.len() {
            return TypeCompatibilityResult::incompatible(
                ts_type,
                webidl_type,
                format!("Parameter count mismatch: {} vs {}", ts_params.len(), webidl_params.len()),
            )
            .with_suggestion("Ensure function signatures have the same number of parameters");
        }

        let mut all_compatible = true;
        let mut details = TypeCompatibilityResult::compatible(ts_type, webidl_type);

        for (i, (ts_param, webidl_param)) in ts_params.iter().zip(webidl_params.iter()).enumerate() {
            let param_result = self.check_type_compatibility(ts_param, webidl_param);
            if !param_result {
                all_compatible = false;
                details = details.with_suggestion(format!(
                    "Parameter {} type '{}' should be compatible with '{}'",
                    i + 1,
                    ts_param,
                    webidl_param
                ));
            }
        }

        if let (Some(ts_ret), Some(webidl_ret)) = (&ts_return, &webidl_return) {
            if !self.check_type_compatibility(ts_ret, webidl_ret) {
                all_compatible = false;
                details =
                    details.with_suggestion(format!("Return type '{}' should be compatible with '{}'", ts_ret, webidl_ret));
            }
        }

        if all_compatible {
            details
        }
        else {
            TypeCompatibilityResult::incompatible(ts_type, webidl_type, "Function signature mismatch")
                .with_level(CompatibilityLevel::Partial)
        }
    }

    /// 解析函数参数类型
    ///
    /// # 参数
    ///
    /// * `func_type` - 函数类型字符串
    ///
    /// # 返回值
    ///
    /// 返回参数类型列表
    fn parse_function_params(&self, func_type: &str) -> Option<Vec<String>> {
        let start = func_type.find('(')?;
        let end = func_type.rfind(')')?;

        if start >= end {
            return Some(Vec::new());
        }

        let params_str = &func_type[start + 1..end];
        if params_str.trim().is_empty() {
            return Some(Vec::new());
        }

        let params: Vec<String> = params_str
            .split(',')
            .filter_map(|p| {
                let p = p.trim();
                if p.is_empty() {
                    return None;
                }

                if p.contains(':') {
                    let colon_pos = p.find(':')?;
                    Some(p[colon_pos + 1..].trim().to_string())
                }
                else {
                    Some(p.to_string())
                }
            })
            .collect();

        Some(params)
    }

    /// 提取返回类型
    ///
    /// # 参数
    ///
    /// * `func_type` - 函数类型字符串
    ///
    /// # 返回值
    ///
    /// 返回返回类型
    fn extract_return_type(&self, func_type: &str) -> Option<String> {
        let arrow_pos = func_type.find("=>")?;
        let return_type = func_type[arrow_pos + 2..].trim();

        if return_type.is_empty() { Some("void".to_string()) } else { Some(return_type.to_string()) }
    }

    /// 检查对象类型兼容性（结构化类型）
    ///
    /// # 参数
    ///
    /// * `ts_type` - TypeScript 对象类型
    /// * `webidl_type` - WebIDL 字典类型
    ///
    /// # 返回值
    ///
    /// 返回类型兼容性结果
    pub fn check_object_type_compatibility(&self, ts_type: &str, webidl_type: &str) -> TypeCompatibilityResult {
        let ts_props = self.parse_object_properties_for_compatibility(ts_type);
        let webidl_props = self.get_webidl_dictionary_properties(webidl_type);

        if ts_props.is_empty() && webidl_props.is_empty() {
            return TypeCompatibilityResult::incompatible(ts_type, webidl_type, "Not object types");
        }

        let mut missing_props: Vec<String> = Vec::new();
        let mut incompatible_props: Vec<(String, String, String)> = Vec::new();
        let mut matched_props: Vec<(String, String)> = Vec::new();

        for (prop_name, prop_type) in &webidl_props {
            if let Some(ts_prop_type) = ts_props.get(prop_name) {
                if self.check_type_compatibility(ts_prop_type, prop_type) {
                    matched_props.push((prop_name.clone(), ts_prop_type.clone()));
                }
                else {
                    incompatible_props.push((prop_name.clone(), ts_prop_type.clone(), prop_type.clone()));
                }
            }
            else {
                missing_props.push(prop_name.clone());
            }
        }

        if missing_props.is_empty() && incompatible_props.is_empty() {
            let mut result = TypeCompatibilityResult::compatible(ts_type, webidl_type);
            for (name, ts_type) in matched_props {
                result = result.with_mapping(name, ts_type);
            }
            result
        }
        else {
            let mut reason = String::new();
            if !missing_props.is_empty() {
                reason.push_str(&format!("Missing properties: {} ", missing_props.join(", ")));
            }
            if !incompatible_props.is_empty() {
                reason.push_str("Incompatible property types: ");
                for (name, ts, webidl) in &incompatible_props {
                    reason.push_str(&format!("{} ({} vs {}) ", name, ts, webidl));
                }
            }

            TypeCompatibilityResult::incompatible(ts_type, webidl_type, reason.trim()).with_level(CompatibilityLevel::Partial)
        }
    }

    /// 解析对象属性用于兼容性检查
    ///
    /// # 参数
    ///
    /// * `obj_type` - 对象类型字符串
    ///
    /// # 返回值
    ///
    /// 返回属性名称到类型的映射
    fn parse_object_properties_for_compatibility(&self, obj_type: &str) -> HashMap<String, String> {
        let mut props = HashMap::new();

        if !obj_type.starts_with('{') || !obj_type.ends_with('}') {
            return props;
        }

        let inner = &obj_type[1..obj_type.len() - 1];
        let parts: Vec<&str> = inner.split(';').chain(inner.split(',')).collect();

        for part in parts {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            if let Some(colon_pos) = part.find(':') {
                let name = part[..colon_pos].trim().to_string();
                let type_name = part[colon_pos + 1..].trim().to_string();

                let name = name.trim_end_matches('?').trim().to_string();
                props.insert(name, type_name);
            }
        }

        props
    }

    /// 获取 WebIDL 字典属性
    ///
    /// # 参数
    ///
    /// * `dict_name` - 字典名称
    ///
    /// # 返回值
    ///
    /// 返回属性名称到类型的映射
    fn get_webidl_dictionary_properties(&self, dict_name: &str) -> HashMap<String, String> {
        let mut props = HashMap::new();

        for item in &self.webidl_result.root.items {
            if let IdlItem::Struct(struct_) = item {
                if struct_.name == dict_name {
                    for field in &struct_.fields {
                        props.insert(field.name.clone(), field.type_name.clone());
                    }
                    break;
                }
            }

            if let IdlItem::Interface(interface) = item {
                if interface.name == dict_name {
                    for member in &interface.members {
                        if let oak_idl::ast::IdlMember::Attribute(attr) = member {
                            props.insert(attr.name.clone(), attr.type_name.clone());
                        }
                    }
                    break;
                }
            }
        }

        props
    }

    /// 检查数组类型兼容性
    ///
    /// # 参数
    ///
    /// * `ts_type` - TypeScript 数组类型
    /// * `webidl_type` - WebIDL 序列类型
    ///
    /// # 返回值
    ///
    /// 返回类型兼容性结果
    pub fn check_array_type_compatibility(&self, ts_type: &str, webidl_type: &str) -> TypeCompatibilityResult {
        let ts_element = self.extract_array_element_type(ts_type);
        let webidl_element = self.extract_sequence_element_type(webidl_type);

        match (ts_element, webidl_element) {
            (Some(ts_elem), Some(webidl_elem)) => {
                if self.check_type_compatibility(&ts_elem, &webidl_elem) {
                    TypeCompatibilityResult::compatible(ts_type, webidl_type).with_mapping(&ts_elem, &webidl_elem)
                }
                else {
                    TypeCompatibilityResult::incompatible(
                        ts_type,
                        webidl_type,
                        format!("Array element type '{}' is not compatible with '{}'", ts_elem, webidl_elem),
                    )
                    .with_suggestion(format!("Use array with element type compatible with '{}'", webidl_elem))
                }
            }
            _ => TypeCompatibilityResult::incompatible(ts_type, webidl_type, "Not array types"),
        }
    }

    /// 提取数组元素类型
    ///
    /// # 参数
    ///
    /// * `array_type` - 数组类型字符串
    ///
    /// # 返回值
    ///
    /// 返回元素类型
    fn extract_array_element_type(&self, array_type: &str) -> Option<String> {
        if array_type.ends_with("[]") {
            return Some(array_type[..array_type.len() - 2].to_string());
        }

        if array_type.starts_with("Array<") && array_type.ends_with('>') {
            return Some(array_type[6..array_type.len() - 1].to_string());
        }

        if array_type.starts_with("ReadonlyArray<") && array_type.ends_with('>') {
            return Some(array_type[14..array_type.len() - 1].to_string());
        }

        None
    }

    /// 提取序列元素类型
    ///
    /// # 参数
    ///
    /// * `sequence_type` - 序列类型字符串
    ///
    /// # 返回值
    ///
    /// 返回元素类型
    fn extract_sequence_element_type(&self, sequence_type: &str) -> Option<String> {
        if sequence_type.starts_with("sequence<") && sequence_type.ends_with('>') {
            return Some(sequence_type[9..sequence_type.len() - 1].to_string());
        }

        if sequence_type.starts_with("FrozenArray<") && sequence_type.ends_with('>') {
            return Some(sequence_type[12..sequence_type.len() - 1].to_string());
        }

        if sequence_type.ends_with("[]") {
            return Some(sequence_type[..sequence_type.len() - 2].to_string());
        }

        None
    }

    /// 检查复杂类型兼容性（详细版本）
    ///
    /// # 参数
    ///
    /// * `ts_type` - TypeScript 类型名称
    /// * `webidl_type` - WebIDL 类型名称
    ///
    /// # 返回值
    ///
    /// 返回类型兼容性结果
    fn check_complex_type_compatibility_detailed(&self, ts_type: &str, webidl_type: &str) -> TypeCompatibilityResult {
        if ts_type.starts_with("Array<") && webidl_type.starts_with("sequence<") {
            let ts_inner = &ts_type[6..ts_type.len() - 1];
            let webidl_inner = &webidl_type[9..webidl_type.len() - 1];

            if self.check_type_compatibility(ts_inner, webidl_inner) {
                return TypeCompatibilityResult::compatible(ts_type, webidl_type).with_mapping(ts_inner, webidl_inner);
            }
        }

        if ts_type.starts_with("Promise<") && webidl_type.starts_with("Promise<") {
            let ts_inner = &ts_type[8..ts_type.len() - 1];
            let webidl_inner = &webidl_type[8..webidl_type.len() - 1];

            if self.check_type_compatibility(ts_inner, webidl_inner) {
                return TypeCompatibilityResult::compatible(ts_type, webidl_type).with_mapping(ts_inner, webidl_inner);
            }
        }

        if ts_type.contains('|') && webidl_type.contains(" or ") {
            return self.check_union_type_compatibility(ts_type, webidl_type);
        }

        if ts_type.starts_with('{') && ts_type.contains("[key:") && webidl_type.starts_with("record<") {
            return TypeCompatibilityResult::compatible(ts_type, webidl_type).with_level(CompatibilityLevel::NeedsConversion);
        }

        TypeCompatibilityResult::incompatible(ts_type, webidl_type, "Complex types not compatible")
    }

    /// 检查联合类型兼容性
    ///
    /// # 参数
    ///
    /// * `ts_type` - TypeScript 联合类型
    /// * `webidl_type` - WebIDL 联合类型
    ///
    /// # 返回值
    ///
    /// 返回类型兼容性结果
    fn check_union_type_compatibility(&self, ts_type: &str, webidl_type: &str) -> TypeCompatibilityResult {
        let ts_types: Vec<&str> = ts_type.split('|').map(|t| t.trim()).collect();
        let webidl_types: Vec<&str> = webidl_type.split(" or ").map(|t| t.trim()).collect();

        if ts_types.len() != webidl_types.len() {
            return TypeCompatibilityResult::incompatible(
                ts_type,
                webidl_type,
                format!("Union type member count mismatch: {} vs {}", ts_types.len(), webidl_types.len()),
            )
            .with_suggestion("Ensure union types have the same number of members");
        }

        let mut all_compatible = true;
        let mut result = TypeCompatibilityResult::compatible(ts_type, webidl_type);

        for (ts, webidl) in ts_types.iter().zip(webidl_types.iter()) {
            if self.check_type_compatibility(ts, webidl) {
                result = result.with_mapping(*ts, *webidl);
            }
            else {
                all_compatible = false;
            }
        }

        if all_compatible {
            result
        }
        else {
            TypeCompatibilityResult::incompatible(ts_type, webidl_type, "Union type members not compatible")
        }
    }

    /// 检查用户定义的类型是否兼容（详细版本）
    ///
    /// # 参数
    ///
    /// * `ts_type` - TypeScript 类型名称
    /// * `webidl_type` - WebIDL 类型名称
    ///
    /// # 返回值
    ///
    /// 返回类型兼容性结果
    fn check_user_defined_types_detailed(&self, ts_type: &str, webidl_type: &str) -> TypeCompatibilityResult {
        if ts_type == webidl_type {
            return TypeCompatibilityResult::compatible(ts_type, webidl_type).with_mapping(ts_type, webidl_type);
        }

        self.check_type_alias_compatibility_detailed(ts_type, webidl_type)
    }

    /// 检查类型别名兼容性（详细版本）
    ///
    /// # 参数
    ///
    /// * `ts_type` - TypeScript 类型名称
    /// * `webidl_type` - WebIDL 类型名称
    ///
    /// # 返回值
    ///
    /// 返回类型兼容性结果
    fn check_type_alias_compatibility_detailed(&self, ts_type: &str, webidl_type: &str) -> TypeCompatibilityResult {
        for item in &self.webidl_result.root.items {
            if let IdlItem::Typedef(typedef) = item {
                if typedef.name == webidl_type {
                    if self.check_type_compatibility(ts_type, &typedef.type_name) {
                        return TypeCompatibilityResult::compatible(ts_type, webidl_type)
                            .with_mapping(ts_type, &typedef.type_name)
                            .with_level(CompatibilityLevel::NeedsConversion);
                    }
                }
            }
        }

        TypeCompatibilityResult::incompatible(ts_type, webidl_type, "No matching type alias found")
    }

    /// 检查 TypeScript 类型是否与 WebIDL 类型兼容
    ///
    /// # 参数
    ///
    /// * `ts_type` - TypeScript 类型名称
    /// * `webidl_type` - WebIDL 类型名称
    ///
    /// # 返回值
    ///
    /// 如果类型兼容返回 true，否则返回 false
    pub fn check_type_compatibility(&self, ts_type: &str, webidl_type: &str) -> bool {
        let type_map = self.get_basic_type_mappings();

        for (ts, webidl) in type_map {
            if ts == ts_type && webidl == webidl_type {
                return true;
            }
        }

        if self.check_complex_type_compatibility(ts_type, webidl_type) {
            return true;
        }

        self.check_user_defined_types(ts_type, webidl_type)
    }

    /// 检查复杂类型兼容性
    ///
    /// # 参数
    ///
    /// * `ts_type` - TypeScript 类型名称
    /// * `webidl_type` - WebIDL 类型名称
    ///
    /// # 返回值
    ///
    /// 如果类型兼容返回 true，否则返回 false
    fn check_complex_type_compatibility(&self, ts_type: &str, webidl_type: &str) -> bool {
        if ts_type.starts_with("Array<") && webidl_type.starts_with("sequence<") {
            let ts_inner = &ts_type[6..ts_type.len() - 1];
            let webidl_inner = &webidl_type[9..webidl_type.len() - 1];
            return self.check_type_compatibility(ts_inner, webidl_inner);
        }

        if ts_type.starts_with("Promise<") && webidl_type.starts_with("Promise<") {
            let ts_inner = &ts_type[8..ts_type.len() - 1];
            let webidl_inner = &webidl_type[8..webidl_type.len() - 1];
            return self.check_type_compatibility(ts_inner, webidl_inner);
        }

        if ts_type.contains('|') && webidl_type.contains(" or ") {
            let ts_types: Vec<&str> = ts_type.split('|').map(|t| t.trim()).collect();
            let webidl_types: Vec<&str> = webidl_type.split(" or ").map(|t| t.trim()).collect();

            if ts_types.len() != webidl_types.len() {
                return false;
            }

            for (ts, webidl) in ts_types.iter().zip(webidl_types.iter()) {
                if !self.check_type_compatibility(ts, webidl) {
                    return false;
                }
            }

            return true;
        }

        if ts_type.starts_with('{') && ts_type.contains("[key:") && webidl_type.starts_with("record<") {
            return true;
        }

        false
    }

    /// 检查用户定义的类型是否兼容
    ///
    /// # 参数
    ///
    /// * `ts_type` - TypeScript 类型名称
    /// * `webidl_type` - WebIDL 类型名称
    ///
    /// # 返回值
    ///
    /// 如果类型兼容返回 true，否则返回 false
    fn check_user_defined_types(&self, ts_type: &str, webidl_type: &str) -> bool {
        if ts_type == webidl_type {
            return true;
        }

        self.check_type_alias_compatibility(ts_type, webidl_type)
    }

    /// 检查类型别名兼容性
    ///
    /// # 参数
    ///
    /// * `ts_type` - TypeScript 类型名称
    /// * `webidl_type` - WebIDL 类型名称
    ///
    /// # 返回值
    ///
    /// 如果类型兼容返回 true，否则返回 false
    fn check_type_alias_compatibility(&self, ts_type: &str, webidl_type: &str) -> bool {
        for item in &self.webidl_result.root.items {
            if let IdlItem::Typedef(typedef) = item {
                if typedef.name == webidl_type {
                    return self.check_type_compatibility(ts_type, &typedef.type_name);
                }
            }
        }

        false
    }

    /// 验证 WebIDL 类型定义的完整性
    ///
    /// # 返回值
    ///
    /// 如果类型定义完整返回 true，否则返回 false
    pub fn validate_webidl_types(&self) -> bool {
        if self.webidl_result.root.items.is_empty() {
            return false;
        }

        self.validate_items(&self.webidl_result.root)
    }

    /// 验证 WebIDL 项目的有效性
    ///
    /// # 参数
    ///
    /// * `root` - WebIDL AST 根节点
    ///
    /// # 返回值
    ///
    /// 如果所有项目有效返回 true，否则返回 false
    fn validate_items(&self, root: &IdlRoot) -> bool {
        for item in &root.items {
            match item {
                IdlItem::Interface(interface) => {
                    if interface.name.is_empty() {
                        return false;
                    }
                }
                IdlItem::Struct(struct_) => {
                    if struct_.name.is_empty() {
                        return false;
                    }
                }
                IdlItem::Enum(enum_) => {
                    if enum_.name.is_empty() {
                        return false;
                    }
                    if enum_.variants.is_empty() {
                        return false;
                    }
                }
                IdlItem::Typedef(typedef) => {
                    if typedef.name.is_empty() {
                        return false;
                    }
                }
                IdlItem::Const(const_) => {
                    if const_.name.is_empty() {
                        return false;
                    }
                }
                IdlItem::Module(module) => {
                    if module.name.is_empty() {
                        return false;
                    }
                    if !self.validate_items(&IdlRoot { items: module.items.clone() }) {
                        return false;
                    }
                }
            }
        }

        true
    }

    /// 验证并收集错误
    ///
    /// # 返回值
    ///
    /// 返回可变的错误收集器引用
    pub fn validate_and_collect_errors(&mut self) -> &TypeErrorCollector {
        self.validate_items_with_errors(&self.webidl_result.root.clone(), None);
        &self.error_collector
    }

    /// 验证项目并收集错误
    ///
    /// # 参数
    ///
    /// * `root` - WebIDL AST 根节点
    /// * `file_path` - 文件路径（可选）
    fn validate_items_with_errors(&mut self, root: &IdlRoot, file_path: Option<&str>) {
        for item in &root.items {
            match item {
                IdlItem::Interface(interface) => {
                    if interface.name.is_empty() {
                        self.error_collector.report_error(
                            TypeError::new("Interface name cannot be empty", "E001")
                                .with_file(file_path.unwrap_or("unknown"))
                                .with_severity(TypeErrorSeverity::Error),
                        );
                    }

                    for member in &interface.members {
                        if let oak_idl::ast::IdlMember::Attribute(attr) = member {
                            if attr.name.is_empty() {
                                self.error_collector.report_error(
                                    TypeError::new("Attribute name cannot be empty", "E002")
                                        .with_file(file_path.unwrap_or("unknown"))
                                        .with_severity(TypeErrorSeverity::Error),
                                );
                            }
                            if attr.type_name.is_empty() {
                                self.error_collector.report_error(
                                    TypeError::new(format!("Attribute '{}' has no type specified", attr.name), "E003")
                                        .with_file(file_path.unwrap_or("unknown"))
                                        .with_severity(TypeErrorSeverity::Warning),
                                );
                            }
                        }
                    }
                }
                IdlItem::Struct(struct_) => {
                    if struct_.name.is_empty() {
                        self.error_collector.report_error(
                            TypeError::new("Struct name cannot be empty", "E004")
                                .with_file(file_path.unwrap_or("unknown"))
                                .with_severity(TypeErrorSeverity::Error),
                        );
                    }

                    for field in &struct_.fields {
                        if field.name.is_empty() {
                            self.error_collector.report_error(
                                TypeError::new("Field name cannot be empty", "E005")
                                    .with_file(file_path.unwrap_or("unknown"))
                                    .with_severity(TypeErrorSeverity::Error),
                            );
                        }
                    }
                }
                IdlItem::Enum(enum_) => {
                    if enum_.name.is_empty() {
                        self.error_collector.report_error(
                            TypeError::new("Enum name cannot be empty", "E006")
                                .with_file(file_path.unwrap_or("unknown"))
                                .with_severity(TypeErrorSeverity::Error),
                        );
                    }
                    if enum_.variants.is_empty() {
                        self.error_collector.report_error(
                            TypeError::new(format!("Enum '{}' has no variants", enum_.name), "E007")
                                .with_file(file_path.unwrap_or("unknown"))
                                .with_severity(TypeErrorSeverity::Warning),
                        );
                    }
                }
                IdlItem::Typedef(typedef) => {
                    if typedef.name.is_empty() {
                        self.error_collector.report_error(
                            TypeError::new("Typedef name cannot be empty", "E008")
                                .with_file(file_path.unwrap_or("unknown"))
                                .with_severity(TypeErrorSeverity::Error),
                        );
                    }
                    if typedef.type_name.is_empty() {
                        self.error_collector.report_error(
                            TypeError::new(format!("Typedef '{}' has no target type", typedef.name), "E009")
                                .with_file(file_path.unwrap_or("unknown"))
                                .with_severity(TypeErrorSeverity::Error),
                        );
                    }
                }
                IdlItem::Const(const_) => {
                    if const_.name.is_empty() {
                        self.error_collector.report_error(
                            TypeError::new("Const name cannot be empty", "E010")
                                .with_file(file_path.unwrap_or("unknown"))
                                .with_severity(TypeErrorSeverity::Error),
                        );
                    }
                    if const_.value.is_empty() {
                        self.error_collector.report_error(
                            TypeError::new(format!("Const '{}' has no value", const_.name), "E011")
                                .with_file(file_path.unwrap_or("unknown"))
                                .with_severity(TypeErrorSeverity::Warning),
                        );
                    }
                }
                IdlItem::Module(module) => {
                    if module.name.is_empty() {
                        self.error_collector.report_error(
                            TypeError::new("Module name cannot be empty", "E012")
                                .with_file(file_path.unwrap_or("unknown"))
                                .with_severity(TypeErrorSeverity::Error),
                        );
                    }
                    self.validate_items_with_errors(&IdlRoot { items: module.items.clone() }, file_path);
                }
            }
        }
    }
}
