/// 构建命令
///
/// 实现项目构建功能
use crate::commands::CommandResult;
use crate::{
    compiler::{CompileOptions, CompileResult, compile_file, compile_files},
    config::{ProjectConfig, load_config, resolve_file_patterns},
};
use std::path::{Path, PathBuf};

/// 构建命令选项
#[derive(Debug, Clone)]
pub struct BuildOptions {
    /// 项目配置文件路径
    pub config_path: Option<PathBuf>,
    /// 输出目录
    pub out_dir: Option<PathBuf>,
    /// 是否生成 source map
    pub source_map: bool,
    /// 是否生成声明文件
    pub declaration: bool,
    /// 是否压缩
    pub minify: bool,
    /// 是否监视模式
    pub watch: bool,
    /// 是否清理输出目录
    pub clean: bool,
    /// 目标 ECMAScript 版本
    pub target: Option<String>,
    /// 模块系统
    pub module: Option<String>,
    /// 是否启用严格模式
    pub strict: bool,
    /// 自定义编译选项
    pub compiler_options: Option<serde_json::Value>,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            config_path: None,
            out_dir: None,
            source_map: true,
            declaration: true,
            minify: false,
            watch: false,
            clean: false,
            target: None,
            module: None,
            strict: false,
            compiler_options: None,
        }
    }
}

/// 执行构建命令
pub fn execute(options: BuildOptions) -> CommandResult {
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
        return CommandResult::error(1, "未找到需要编译的文件");
    }

    println!("找到 {} 个文件需要编译", files.len());

    /// 清理输出目录
    if options.clean {
        if let Some(ref out_dir) = options.out_dir {
            if out_dir.exists() {
                println!("清理输出目录: {:?}", out_dir);
                if let Err(e) = std::fs::remove_dir_all(out_dir) {
                    return CommandResult::error(1, format!("清理输出目录失败: {}", e));
                }
            }
        }
    }

    /// 准备编译选项
    let compile_options = create_compile_options(&config, &options);

    /// 执行编译
    let results = compile_files(&files, &compile_options);

    /// 统计结果
    let success_count = results.iter().filter(|r| r.success).count();
    let error_count = results.len() - success_count;

    /// 输出结果
    for (file, result) in files.iter().zip(results.iter()) {
        if result.success {
            println!("✓ {:?}", file);
        }
        else {
            eprintln!("✗ {:?}", file);
            for error in &result.errors {
                eprintln!("  错误: {}", error);
            }
        }
    }

    if error_count == 0 {
        CommandResult::success(format!("构建成功! 编译了 {} 个文件", success_count))
    }
    else {
        CommandResult::error(1, format!("构建失败! {} 个文件编译成功, {} 个文件编译失败", success_count, error_count))
    }
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

/// 解析需要编译的文件
fn resolve_files(project_root: &Path, config: &ProjectConfig) -> Result<Vec<PathBuf>, String> {
    let include = config.include.as_ref();
    let exclude = config.exclude.as_ref();

    resolve_file_patterns(project_root, include, exclude).map_err(|e| e.to_string())
}

/// 创建编译选项
fn create_compile_options(config: &ProjectConfig, build_options: &BuildOptions) -> CompileOptions {
    let compiler_options = config.compiler_options.as_ref();

    CompileOptions {
        out_dir: build_options.out_dir.clone().or_else(|| compiler_options.and_then(|o| o.out_dir.clone())),
        target: build_options.target.clone().or_else(|| compiler_options.and_then(|o| o.target.clone())),
        module: build_options.module.clone().or_else(|| compiler_options.and_then(|o| o.module.clone())),
        strict: build_options.strict || compiler_options.and_then(|o| o.strict).unwrap_or(false),
        no_emit: false,
        source_map: build_options.source_map || compiler_options.and_then(|o| o.source_map).unwrap_or(true),
        declaration: build_options.declaration || compiler_options.and_then(|o| o.declaration).unwrap_or(true),
        minify: build_options.minify,
    }
}
