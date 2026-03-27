/// 工具模块
///
/// 提供各种通用工具函数
use std::path::PathBuf;

/// 查找项目根目录
pub fn find_project_root(start: &PathBuf) -> Option<PathBuf> {
    let mut current = start.canonicalize().ok()?;

    loop {
        if current.join("tsconfig.json").exists() || current.join(".rts.toml").exists() {
            return Some(current);
        }

        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        }
        else {
            return None;
        }
    }
}

/// 获取输出文件路径
pub fn get_output_path(input: &PathBuf, out_dir: &Option<PathBuf>, extension: &str) -> PathBuf {
    if let Some(out_dir) = out_dir {
        let file_name = input.file_stem().unwrap_or_default();
        out_dir.join(format!("{}{}", file_name.to_str().unwrap_or_default(), extension))
    }
    else {
        let mut output = input.clone();
        // 移除扩展名前缀的点号
        let extension_without_dot = extension.trim_start_matches('.');
        output.set_extension(extension_without_dot);
        output
    }
}

/// 确保目录存在
pub fn ensure_dir(path: &PathBuf) -> Result<(), std::io::Error> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

/// 读取文件内容
pub fn read_file(path: &PathBuf) -> Result<String, std::io::Error> {
    use std::fs::read_to_string;
    read_to_string(path)
}

/// 写入文件内容
pub fn write_file(path: &PathBuf, content: &str) -> Result<(), std::io::Error> {
    use std::fs::write;
    write(path, content)
}

/// 规范化路径分隔符
///
/// 在 Windows 上，将正斜杠转换为反斜杠
/// 在 Unix 系统上，将反斜杠转换为正斜杠
pub fn normalize_path_separators(path: &PathBuf) -> PathBuf {
    let path_str = path.to_string_lossy();
    #[cfg(windows)]
    let normalized = path_str.replace('/', "\\");
    #[cfg(not(windows))]
    let normalized = path_str.replace('\\', "/");
    PathBuf::from(normalized)
}

/// 获取平台特定的临时目录
pub fn get_temp_dir() -> PathBuf {
    std::env::temp_dir()
}

/// 获取平台特定的换行符
pub fn get_platform_newline() -> &'static str {
    #[cfg(windows)]
    return "\r\n";
    #[cfg(not(windows))]
    return "\n";
}

/// 检查文件是否为 TypeScript 文件
pub fn is_typescript_file(path: &PathBuf) -> bool {
    matches!(path.extension().and_then(|ext| ext.to_str()), Some("ts") | Some("tsx"))
}

/// 检查文件是否为 JavaScript 文件
pub fn is_javascript_file(path: &PathBuf) -> bool {
    matches!(path.extension().and_then(|ext| ext.to_str()), Some("js") | Some("jsx"))
}

/// 平台特定的路径处理
pub fn platform_path(path: &str) -> PathBuf {
    let mut result = PathBuf::new();
    for component in path.split(['/', '\\']) {
        if !component.is_empty() {
            result.push(component);
        }
    }
    result
}

/// 获取平台特定的环境变量分隔符
pub fn get_env_separator() -> char {
    #[cfg(windows)]
    return ';';
    #[cfg(not(windows))]
    return ':';
}
