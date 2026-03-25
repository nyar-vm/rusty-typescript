use typescript_types::{TsError, TsValue};
/// WebIDL 处理模块
use typescript_webidl::{convert_to_typescript, parse};

/// WebIDL 处理器
pub struct WebIdlProcessor {
    /// 解析后的 WebIDL 类型定义
    pub typescript_definitions: String,
}

impl WebIdlProcessor {
    /// 创建新的 WebIDL 处理器
    ///
    /// # 返回值
    /// - 新的 WebIDL 处理器
    pub fn new() -> Self {
        Self { typescript_definitions: String::new() }
    }

    /// 处理 WebIDL 字符串
    ///
    /// # 参数
    /// - `idl`: WebIDL 字符串
    ///
    /// # 返回值
    /// - `Ok(())` 处理成功，`Err(TsError)` 处理失败
    pub fn process_webidl(&mut self, idl: &str) -> Result<(), TsError> {
        match parse(idl) {
            Ok(root) => {
                self.typescript_definitions = convert_to_typescript(&root);
                Ok(())
            }
            Err(error) => Err(TsError::SyntaxError(format!("WebIDL parse error: {}", error))),
        }
    }

    /// 处理 WebIDL 文件
    ///
    /// # 参数
    /// - `path`: WebIDL 文件路径
    ///
    /// # 返回值
    /// - `Ok(())` 处理成功，`Err(TsError)` 处理失败
    pub fn process_webidl_file(&mut self, path: &std::path::Path) -> Result<(), TsError> {
        use std::fs::read_to_string;

        let idl = read_to_string(path).map_err(|e| TsError::Other(format!("Failed to read WebIDL file: {}", e)))?;
        self.process_webidl(&idl)
    }

    /// 获取生成的 TypeScript 类型定义
    ///
    /// # 返回值
    /// - 生成的 TypeScript 类型定义字符串
    pub fn get_typescript_definitions(&self) -> &str {
        &self.typescript_definitions
    }
}
