/// WebIDL 类型检查模块
use crate::types::WebIdlResult;
use oak_idl::ast::{IdlItem, IdlRoot};

/// WebIDL 类型检查器
pub struct WebIdlTypeChecker {
    /// WebIDL 解析结果
    pub webidl_result: WebIdlResult,
}

impl WebIdlTypeChecker {
    /// 创建新的 WebIDL 类型检查器
    ///
    /// # 参数
    /// - `webidl_result`: WebIDL 解析结果
    ///
    /// # 返回值
    /// - 新的 WebIDL 类型检查器
    pub fn new(webidl_result: WebIdlResult) -> Self {
        Self { webidl_result }
    }

    /// 检查 TypeScript 类型是否与 WebIDL 类型兼容
    ///
    /// # 参数
    /// - `ts_type`: TypeScript 类型名称
    /// - `webidl_type`: WebIDL 类型名称
    ///
    /// # 返回值
    /// - `true` 如果类型兼容，`false` 否则
    pub fn check_type_compatibility(&self, ts_type: &str, webidl_type: &str) -> bool {
        // 基本类型映射
        let type_map = vec![
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
        ];

        // 检查直接映射
        for (ts, webidl) in type_map {
            if ts == ts_type && webidl == webidl_type {
                return true;
            }
        }

        // 检查复杂类型兼容性
        if self.check_complex_type_compatibility(ts_type, webidl_type) {
            return true;
        }

        // 检查用户定义的类型
        self.check_user_defined_types(ts_type, webidl_type)
    }

    /// 检查复杂类型兼容性
    ///
    /// # 参数
    /// - `ts_type`: TypeScript 类型名称
    /// - `webidl_type`: WebIDL 类型名称
    ///
    /// # 返回值
    /// - `true` 如果类型兼容，`false` 否则
    fn check_complex_type_compatibility(&self, ts_type: &str, webidl_type: &str) -> bool {
        // 检查数组类型
        if ts_type.starts_with("Array<") && webidl_type.starts_with("sequence<") {
            let ts_inner = &ts_type[6..ts_type.len() - 1];
            let webidl_inner = &webidl_type[9..webidl_type.len() - 1];
            return self.check_type_compatibility(ts_inner, webidl_inner);
        }

        // 检查 Promise 类型
        if ts_type.starts_with("Promise<") && webidl_type.starts_with("Promise<") {
            let ts_inner = &ts_type[8..ts_type.len() - 1];
            let webidl_inner = &webidl_type[8..webidl_type.len() - 1];
            return self.check_type_compatibility(ts_inner, webidl_inner);
        }

        // 检查联合类型
        if ts_type.contains("|") && webidl_type.contains("or") {
            let ts_types: Vec<&str> = ts_type.split("|").map(|t| t.trim()).collect();
            let webidl_types: Vec<&str> = webidl_type.split("or").map(|t| t.trim()).collect();

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

        // 检查映射类型
        if ts_type.starts_with("{") && ts_type.contains("[key:") && webidl_type.starts_with("record<") {
            return true;
        }

        false
    }

    /// 检查用户定义的类型是否兼容
    ///
    /// # 参数
    /// - `ts_type`: TypeScript 类型名称
    /// - `webidl_type`: WebIDL 类型名称
    ///
    /// # 返回值
    /// - `true` 如果类型兼容，`false` 否则
    fn check_user_defined_types(&self, ts_type: &str, webidl_type: &str) -> bool {
        // 简单的名称匹配
        if ts_type == webidl_type {
            return true;
        }

        // 检查类型别名
        self.check_type_alias_compatibility(ts_type, webidl_type)
    }

    /// 检查类型别名兼容性
    ///
    /// # 参数
    /// - `ts_type`: TypeScript 类型名称
    /// - `webidl_type`: WebIDL 类型名称
    ///
    /// # 返回值
    /// - `true` 如果类型兼容，`false` 否则
    fn check_type_alias_compatibility(&self, ts_type: &str, webidl_type: &str) -> bool {
        // 遍历所有类型定义，检查是否存在类型别名
        for item in &self.webidl_result.root.items {
            if let IdlItem::Typedef(typedef) = item {
                if typedef.name == webidl_type {
                    // 检查类型别名的底层类型
                    return self.check_type_compatibility(ts_type, &typedef.type_name);
                }
            }
        }

        false
    }

    /// 验证 WebIDL 类型定义的完整性
    ///
    /// # 返回值
    /// - `true` 如果类型定义完整，`false` 否则
    pub fn validate_webidl_types(&self) -> bool {
        // 检查是否至少有一个类型定义
        if self.webidl_result.root.items.is_empty() {
            return false;
        }

        // 检查所有类型定义的有效性
        self.validate_items(&self.webidl_result.root)
    }

    /// 验证 WebIDL 项目的有效性
    ///
    /// # 参数
    /// - `root`: WebIDL AST 根节点
    ///
    /// # 返回值
    /// - `true` 如果所有项目有效，`false` 否则
    fn validate_items(&self, root: &IdlRoot) -> bool {
        for item in &root.items {
            match item {
                IdlItem::Interface(interface) => {
                    // 检查接口名称是否为空
                    if interface.name.is_empty() {
                        return false;
                    }
                }
                IdlItem::Struct(struct_) => {
                    // 检查结构体名称是否为空
                    if struct_.name.is_empty() {
                        return false;
                    }
                }
                IdlItem::Enum(enum_) => {
                    // 检查枚举名称是否为空
                    if enum_.name.is_empty() {
                        return false;
                    }
                    // 检查枚举是否至少有一个变体
                    if enum_.variants.is_empty() {
                        return false;
                    }
                }
                IdlItem::Typedef(typedef) => {
                    // 检查类型别名名称是否为空
                    if typedef.name.is_empty() {
                        return false;
                    }
                }
                IdlItem::Const(const_) => {
                    // 检查常量名称是否为空
                    if const_.name.is_empty() {
                        return false;
                    }
                }
                IdlItem::Module(module) => {
                    // 检查模块名称是否为空
                    if module.name.is_empty() {
                        return false;
                    }
                    // 递归检查模块内的项目
                    if !self.validate_items(&IdlRoot { items: module.items.clone() }) {
                        return false;
                    }
                }
                _ => {
                    // 忽略未知类型
                }
            }
        }

        true
    }
}
