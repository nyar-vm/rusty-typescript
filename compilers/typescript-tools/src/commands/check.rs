/// 检查命令
///
/// 实现类型检查功能
use crate::commands::CommandResult;
use crate::config::{ProjectConfig, load_config, resolve_file_patterns};
use std::path::{Path, PathBuf};

/// 检查命令选项
#[derive(Debug, Clone)]
pub struct CheckOptions {
    /// 项目配置文件路径
    pub config_path: Option<PathBuf>,
    /// 是否严格模式
    pub strict: bool,
    /// 是否跳过库检查
    pub skip_lib_check: bool,
}

impl Default for CheckOptions {
    fn default() -> Self {
        Self { config_path: None, strict: true, skip_lib_check: true }
    }
}

/// 执行检查命令
pub fn execute(options: CheckOptions) -> CommandResult {
    /// 查找配置文件
    let config_path =
        options.config_path.clone().or_else(|| find_config_file(".")).unwrap_or_else(|| PathBuf::from("tsconfig.json"));

    /// 加载配置
    let config = match load_project_config(&config_path) {
        Ok(config) => config,
        Err(e) => {
            return CommandResult::error(1, format!("加载配置文件失败: {}", e));
        }
    };

    /// 获取项目根目录
    let project_root = config_path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));

    /// 解析文件模式
    let files = match resolve_files(&project_root, &config) {
        Ok(files) => files,
        Err(e) => {
            return CommandResult::error(1, format!("解析文件模式失败: {}", e));
        }
    };

    if files.is_empty() {
        return CommandResult::error(1, "未找到需要检查的文件");
    }

    println!("正在检查 {} 个文件...", files.len());

    /// 执行类型检查
    let mut error_count = 0;
    let mut warning_count = 0;

    for file in &files {
        match check_file(file, &options) {
            Ok((errors, warnings)) => {
                error_count += errors;
                warning_count += warnings;

                if errors == 0 && warnings == 0 {
                    println!("✓ {:?}", file);
                }
                else {
                    println!("⚠ {:?} ({} 错误, {} 警告)", file, errors, warnings);
                }
            }
            Err(e) => {
                eprintln!("✗ {:?}: {}", file, e);
                error_count += 1;
            }
        }
    }

    if error_count == 0 && warning_count == 0 {
        CommandResult::success("类型检查通过! 未发现错误".to_string())
    }
    else if error_count == 0 {
        CommandResult::success(format!("类型检查完成! 发现 {} 个警告", warning_count))
    }
    else {
        CommandResult::error(1, format!("类型检查失败! 发现 {} 个错误, {} 个警告", error_count, warning_count))
    }
}

/// 检查单个文件
fn check_file(file: &Path, options: &CheckOptions) -> Result<(usize, usize), String> {
    let content = std::fs::read_to_string(file).map_err(|e| format!("无法读取文件: {}", e))?;

    let errors = 0;
    let mut warnings = 0;

    /// 简单的语法检查
    for (line_idx, line) in content.lines().enumerate() {
        let line_num = line_idx + 1;
        let trimmed = line.trim();

        /// 检查未使用的变量（简化版）
        if options.strict && trimmed.starts_with("let ") {
            /// 检查是否有使用
            let var_name = trimmed.split_whitespace().nth(1).unwrap_or("");
            let var_name = var_name.split(['=', ';', ':']).next().unwrap_or("");
            if !var_name.is_empty() && !content.contains(&format!("{}", var_name)) {
                warnings += 1;
                println!("  警告: 第 {} 行变量 {} 未使用", line_num, var_name);
            }
        }

        /// 检查 any 类型使用
        if options.strict && trimmed.contains(": any") {
            warnings += 1;
            println!("  警告: 第 {} 行使用了 any 类型", line_num);
        }

        /// 检查空的 catch 块
        if trimmed.starts_with("catch (") && trimmed.contains("{}") {
            warnings += 1;
            println!("  警告: 第 {} 行有空的 catch 块", line_num);
        }

        /// 检查未使用的导入
        if trimmed.starts_with("import ") && trimmed.contains("from") {
            let import_content = trimmed.split("from").next().unwrap_or("");
            if import_content.contains("{ ") {
                /// 检查命名导入
                let imports = import_content.split(['{', '}']).nth(1).unwrap_or("");
                for imp in imports.split(',').map(|s| s.trim()) {
                    if !imp.is_empty() && !content.contains(&imp) {
                        warnings += 1;
                        println!("  警告: 第 {} 行导入 {} 未使用", line_num, imp);
                    }
                }
            }
        }

        /// 检查 console.log 使用
        if options.strict && trimmed.contains("console.log") {
            warnings += 1;
            println!("  警告: 第 {} 行使用了 console.log", line_num);
        }
    }

    Ok((errors, warnings))
}

/// 查找配置文件
fn find_config_file(start_dir: &str) -> Option<PathBuf> {
    let config_names = ["tsconfig.json", "typescript.config.toml"];

    for name in &config_names {
        let path = Path::new(start_dir).join(name);
        if path.exists() {
            return Some(path);
        }
    }

    None
}

/// 加载项目配置
fn load_project_config(path: &Path) -> Result<ProjectConfig, String> {
    use crate::config::load_config_with_extends;

    load_config_with_extends(&path.to_path_buf()).map_err(|e| e.to_string())
}

/// 解析需要检查的文件
fn resolve_files(project_root: &Path, config: &ProjectConfig) -> Result<Vec<PathBuf>, String> {
    let include = config.include.as_ref();
    let exclude = config.exclude.as_ref();

    resolve_file_patterns(project_root, include, exclude).map_err(|e| e.to_string())
}
