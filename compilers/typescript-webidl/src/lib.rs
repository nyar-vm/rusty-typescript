#![doc = include_str!("readme.md")]
#![warn(missing_docs)]
#![feature(new_range_api)]

use std::fs;

/// WebIDL 到 TypeScript 转换模块
pub mod converter;

/// WebIDL 类型定义模块
pub mod types;

/// WebIDL 类型检查模块
pub mod type_checker;

// 使用 oak-idl 的 AST 结构
use oak_idl::IdlRoot;

/// 解析 WebIDL 字符串
///
/// # 参数
/// - `idl`: 要解析的 WebIDL 字符串
///
/// # 返回值
/// - `Ok(IdlRoot)`: 解析成功，返回 WebIDL AST
/// - `Err(String)`: 解析失败，返回错误信息
pub fn parse(idl: &str) -> Result<IdlRoot, String> {
    oak_idl::parse(idl)
}

/// 从文件读取并解析 WebIDL
///
/// # 参数
/// - `file_path`: WebIDL 文件路径
///
/// # 返回值
/// - `Ok(IdlRoot)`: 读取和解析成功，返回 WebIDL AST
/// - `Err(String)`: 读取或解析失败，返回错误信息
pub fn parse_file(file_path: &str) -> Result<IdlRoot, String> {
    if file_path.is_empty() {
        return Err("Empty file path".to_string());
    }

    match fs::read_to_string(file_path) {
        Ok(content) => {
            if content.trim().is_empty() {
                return Err("Empty WebIDL file".to_string());
            }
            parse(&content)
        }
        Err(error) => {
            let error_msg = format!("Failed to read file '{}': {}", file_path, error);
            Err(error_msg)
        }
    }
}

/// 将 WebIDL AST 转换为 TypeScript 类型定义
///
/// # 参数
/// - `root`: WebIDL AST 根节点
///
/// # 返回值
/// - 生成的 TypeScript 类型定义字符串
pub fn convert_to_typescript(root: &IdlRoot) -> String {
    converter::convert(root)
}
