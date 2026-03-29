use crate::errors::{ConfigError, ConfigResult};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

/// 项目引用
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectReference {
    /// 引用的项目路径
    pub path: PathBuf,
}

/// 编译器选项
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompilerOptions {
    /// 目标 ECMAScript 版本
    pub target: Option<String>,
    /// 模块系统
    pub module: Option<String>,
    /// 严格模式
    pub strict: Option<bool>,
    /// JSX 模式
    pub jsx: Option<String>,
    /// 输出目录
    pub out_dir: Option<PathBuf>,
    /// 根目录
    pub root_dir: Option<PathBuf>,
    /// 是否生成 source map
    pub source_map: Option<bool>,
    /// 是否生成声明文件
    pub declaration: Option<bool>,
    /// 声明文件输出目录
    pub declaration_dir: Option<PathBuf>,
    /// 是否允许 JS 文件
    pub allow_js: Option<bool>,
    /// 是否检查 JS 文件
    pub check_js: Option<bool>,
    /// 模块解析策略
    pub module_resolution: Option<String>,
    /// 基础 URL
    pub base_url: Option<String>,
    /// 路径映射
    pub paths: Option<HashMap<String, Vec<String>>>,
    /// 严格空检查
    pub strict_null_checks: Option<bool>,
    /// 禁止隐式 any
    pub no_implicit_any: Option<bool>,
    /// 跳过库检查
    pub skip_lib_check: Option<bool>,
    /// ES 模块互操作性
    pub es_module_interop: Option<bool>,
    /// 是否生成输出文件
    pub no_emit: Option<bool>,
}

impl Default for CompilerOptions {
    fn default() -> Self {
        Self {
            target: Some("ES2020".to_string()),
            module: Some("ESNext".to_string()),
            strict: Some(true),
            jsx: Some("react".to_string()),
            out_dir: None,
            root_dir: None,
            source_map: Some(true),
            declaration: Some(true),
            declaration_dir: None,
            allow_js: Some(false),
            check_js: Some(false),
            module_resolution: Some("node".to_string()),
            base_url: None,
            paths: None,
            strict_null_checks: Some(true),
            no_implicit_any: Some(true),
            skip_lib_check: Some(true),
            es_module_interop: Some(true),
            no_emit: Some(false),
        }
    }
}

impl CompilerOptions {
    /// 合并编译器选项，当前配置覆盖基础配置
    pub fn merge(&self, base: &CompilerOptions) -> CompilerOptions {
        CompilerOptions {
            target: self.target.clone().or(base.target.clone()),
            module: self.module.clone().or(base.module.clone()),
            strict: self.strict.or(base.strict),
            jsx: self.jsx.clone().or(base.jsx.clone()),
            out_dir: self.out_dir.clone().or(base.out_dir.clone()),
            root_dir: self.root_dir.clone().or(base.root_dir.clone()),
            source_map: self.source_map.or(base.source_map),
            declaration: self.declaration.or(base.declaration),
            declaration_dir: self.declaration_dir.clone().or(base.declaration_dir.clone()),
            allow_js: self.allow_js.or(base.allow_js),
            check_js: self.check_js.or(base.check_js),
            module_resolution: self.module_resolution.clone().or(base.module_resolution.clone()),
            base_url: self.base_url.clone().or(base.base_url.clone()),
            paths: self.paths.clone().or(base.paths.clone()),
            strict_null_checks: self.strict_null_checks.or(base.strict_null_checks),
            no_implicit_any: self.no_implicit_any.or(base.no_implicit_any),
            skip_lib_check: self.skip_lib_check.or(base.skip_lib_check),
            es_module_interop: self.es_module_interop.or(base.es_module_interop),
            no_emit: self.no_emit.or(base.no_emit),
        }
    }
}

/// 编译器配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CompilerConfig {
    /// 目标 ECMAScript 版本
    pub target: Option<String>,
    /// 模块系统
    pub module: Option<String>,
    /// 严格模式
    pub strict: Option<bool>,
    /// JSX 模式
    pub jsx: Option<String>,
}

/// 构建配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BuildConfig {
    /// 输出目录
    pub out_dir: Option<PathBuf>,
    /// 是否生成 source map
    pub source_map: Option<bool>,
    /// 是否生成声明文件
    pub declaration: Option<bool>,
    /// 是否压缩
    pub minify: Option<bool>,
}

/// 检查配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CheckConfig {
    /// 严格空检查
    pub strict_null_checks: Option<bool>,
    /// 禁止隐式 any
    pub no_implicit_any: Option<bool>,
    /// 跳过库检查
    pub skip_lib_check: Option<bool>,
}

/// 格式化配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FmtConfig {
    /// 缩进大小
    pub indent_size: Option<usize>,
    /// 行宽
    pub line_width: Option<usize>,
    /// 是否使用分号
    pub semicolons: Option<bool>,
    /// 是否使用单引号
    pub single_quote: Option<bool>,
    /// 尾随逗号
    pub trailing_comma: Option<String>,
}

/// 测试配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TestConfig {
    /// 是否生成覆盖率报告
    pub coverage: Option<bool>,
    /// 是否并行运行
    pub parallel: Option<bool>,
    /// 超时时间（毫秒）
    pub timeout: Option<u64>,
}

/// 项目配置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProjectConfig {
    /// 继承的配置文件路径
    pub extends: Option<String>,
    /// 包含的文件模式
    pub include: Option<Vec<String>>,
    /// 排除的文件模式
    pub exclude: Option<Vec<String>>,
    /// 明确指定的文件列表
    pub files: Option<Vec<PathBuf>>,
    /// 编译器选项
    pub compiler_options: Option<CompilerOptions>,
    /// 项目引用
    pub references: Option<Vec<ProjectReference>>,
    /// 编译器配置（向后兼容）
    pub compiler: Option<CompilerConfig>,
    /// 构建配置
    pub build: Option<BuildConfig>,
    /// 检查配置
    pub check: Option<CheckConfig>,
    /// 格式化配置
    pub fmt: Option<FmtConfig>,
    /// 测试配置
    pub test: Option<TestConfig>,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            extends: None,
            include: Some(vec!["src/**/*".to_string()]),
            exclude: Some(vec!["node_modules".to_string(), "dist".to_string()]),
            files: None,
            compiler_options: Some(CompilerOptions::default()),
            references: None,
            compiler: None,
            build: None,
            check: None,
            fmt: None,
            test: None,
        }
    }
}

impl ProjectConfig {
    /// 合并项目配置，当前配置覆盖基础配置
    pub fn merge(&self, base: &ProjectConfig) -> ProjectConfig {
        let merged_compiler_options = match (&self.compiler_options, &base.compiler_options) {
            (Some(current), Some(base_opts)) => Some(current.merge(base_opts)),
            (Some(current), None) => Some(current.clone()),
            (None, Some(base_opts)) => Some(base_opts.clone()),
            (None, None) => None,
        };

        ProjectConfig {
            extends: self.extends.clone().or(base.extends.clone()),
            include: self.include.clone().or(base.include.clone()),
            exclude: self.exclude.clone().or(base.exclude.clone()),
            files: self.files.clone().or(base.files.clone()),
            compiler_options: merged_compiler_options,
            references: self.references.clone().or(base.references.clone()),
            compiler: self.compiler.clone().or(base.compiler.clone()),
            build: self.build.clone().or(base.build.clone()),
            check: self.check.clone().or(base.check.clone()),
            fmt: self.fmt.clone().or(base.fmt.clone()),
            test: self.test.clone().or(base.test.clone()),
        }
    }
}

/// 解析 JSON 格式的配置文件
fn parse_json_config(content: &str) -> ConfigResult<ProjectConfig> {
    serde_json::from_str(content).map_err(ConfigError::from)
}

/// 解析 TOML 格式的配置文件
fn parse_toml_config(content: &str) -> ConfigResult<ProjectConfig> {
    toml::from_str(content).map_err(ConfigError::from)
}

/// 加载配置文件
pub fn load_config(path: &PathBuf) -> ConfigResult<ProjectConfig> {
    use std::{fs::File, io::Read};

    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or("");

    if file_name.ends_with(".json") { parse_json_config(&content) } else { parse_toml_config(&content) }
}

/// 加载配置文件并处理继承
pub fn load_config_with_extends(path: &PathBuf) -> ConfigResult<ProjectConfig> {
    load_config_with_extends_internal(path, &mut Vec::new())
}

/// 加载配置文件并处理继承（内部实现，包含循环检测）
fn load_config_with_extends_internal(path: &PathBuf, visited: &mut Vec<PathBuf>) -> ConfigResult<ProjectConfig> {
    let canonical_path =
        path.canonicalize().map_err(|_| ConfigError::BaseConfigNotFound(path.to_string_lossy().to_string()))?;

    if visited.contains(&canonical_path) {
        return Err(ConfigError::CircularExtends(canonical_path.to_string_lossy().to_string()));
    }

    visited.push(canonical_path.clone());

    let config = load_config(path)?;

    if let Some(extends) = &config.extends {
        let base_path = resolve_extends_path(extends, path)?;
        let base_config = load_config_with_extends_internal(&base_path, visited)?;
        Ok(config.merge(&base_config))
    }
    else {
        Ok(config)
    }
}

/// 解析 extends 路径
fn resolve_extends_path(extends: &str, current_path: &Path) -> ConfigResult<PathBuf> {
    let current_dir = current_path
        .parent()
        .ok_or_else(|| ConfigError::BaseConfigNotFound("Cannot determine parent directory".to_string()))?;

    if extends.starts_with('.') || extends.starts_with("..") {
        let resolved = current_dir.join(extends);
        resolve_config_file_path(&resolved)
    }
    else {
        let resolved = current_dir.join("node_modules").join(extends);
        resolve_config_file_path(&resolved)
    }
}

/// 解析配置文件路径，处理可能的文件名变体
fn resolve_config_file_path(base: &Path) -> ConfigResult<PathBuf> {
    let candidates = vec![
        base.to_path_buf(),
        base.join("tsconfig.json"),
        base.with_extension("json"),
        base.with_file_name(format!("{}.json", base.file_name().and_then(|n| n.to_str()).unwrap_or("tsconfig"))),
    ];

    for candidate in candidates {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(ConfigError::BaseConfigNotFound(base.to_string_lossy().to_string()))
}

/// 解析文件模式匹配，返回匹配的文件列表
pub fn resolve_file_patterns(
    base_dir: &Path,
    include: Option<&Vec<String>>,
    exclude: Option<&Vec<String>>,
) -> ConfigResult<Vec<PathBuf>> {
    let default_include = vec!["**/*.ts".to_string(), "**/*.tsx".to_string()];
    let default_exclude = vec!["node_modules/**".to_string()];
    let include_patterns = include.unwrap_or(&default_include);
    let exclude_patterns = exclude.unwrap_or(&default_exclude);

    let mut files = Vec::new();

    for pattern in include_patterns {
        let matched = glob_match(base_dir, pattern)?;
        files.extend(matched);
    }

    let mut result = Vec::new();
    for file in files {
        let relative = file.strip_prefix(base_dir).map_err(|e| ConfigError::PatternMatchError(e.to_string()))?;
        let relative_str = relative.to_string_lossy().replace('\\', "/");

        let should_exclude = exclude_patterns.iter().any(|exclude_pattern| matches_pattern(&relative_str, exclude_pattern));

        if !should_exclude {
            result.push(file);
        }
    }

    result.sort();
    result.dedup();

    Ok(result)
}

/// 简单的 glob 模式匹配实现
fn glob_match(base_dir: &Path, pattern: &str) -> ConfigResult<Vec<PathBuf>> {
    use std::fs;

    let mut result = Vec::new();
    let normalized_pattern = pattern.replace('\\', "/");

    if normalized_pattern.contains("**") {
        let parts: Vec<&str> = normalized_pattern.split("**").collect();
        if parts.len() == 2 {
            let prefix = parts[0].trim_end_matches('/');
            let suffix = parts[1].trim_start_matches('/');

            visit_dir_recursive(base_dir, base_dir, prefix, suffix, &mut result)?;
        }
    }
    else if normalized_pattern.contains('*') {
        let parts: Vec<&str> = normalized_pattern.split('*').collect();
        if parts.len() == 2 {
            let prefix = parts[0];
            let suffix = parts[1];
            let search_dir = if prefix.is_empty() { base_dir.to_path_buf() } else { base_dir.join(prefix) };

            if search_dir.exists() && search_dir.is_dir() {
                let entries = fs::read_dir(&search_dir).map_err(|e| ConfigError::PatternMatchError(e.to_string()))?;

                for entry in entries {
                    let entry = entry.map_err(|e| ConfigError::PatternMatchError(e.to_string()))?;
                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_string_lossy();

                    if suffix.is_empty() || file_name_str.ends_with(suffix) {
                        result.push(entry.path());
                    }
                }
            }
        }
    }
    else {
        let full_path = base_dir.join(pattern);
        if full_path.exists() {
            result.push(full_path);
        }
    }

    Ok(result)
}

/// 递归访问目录
fn visit_dir_recursive(
    base_dir: &Path,
    current_dir: &Path,
    prefix: &str,
    suffix: &str,
    result: &mut Vec<PathBuf>,
) -> ConfigResult<()> {
    use std::fs;

    if !current_dir.exists() {
        return Ok(());
    }

    let entries = fs::read_dir(current_dir).map_err(|e| ConfigError::PatternMatchError(e.to_string()))?;

    for entry in entries {
        let entry = entry.map_err(|e| ConfigError::PatternMatchError(e.to_string()))?;
        let path = entry.path();

        if path.is_dir() {
            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            if dir_name != "node_modules" && dir_name != ".git" {
                visit_dir_recursive(base_dir, &path, prefix, suffix, result)?;
            }
        }
        else if path.is_file() {
            let relative = path.strip_prefix(base_dir).map_err(|e| ConfigError::PatternMatchError(e.to_string()))?;
            let relative_str = relative.to_string_lossy().replace('\\', "/");

            if matches_pattern(&relative_str, &format!("{}**{}", prefix, suffix)) {
                result.push(path);
            }
        }
    }

    Ok(())
}

/// 检查字符串是否匹配模式
fn matches_pattern(text: &str, pattern: &str) -> bool {
    let normalized_text = text.replace('\\', "/");
    let normalized_pattern = pattern.replace('\\', "/");

    if normalized_pattern.contains("**") {
        let parts: Vec<&str> = normalized_pattern.split("**").collect();
        if parts.len() == 2 {
            let prefix = parts[0].trim_end_matches('/');
            let suffix = parts[1].trim_start_matches('/');

            let starts_with = prefix.is_empty() || normalized_text.starts_with(prefix);
            let ends_with = suffix.is_empty() || normalized_text.ends_with(suffix);

            return starts_with && ends_with;
        }
    }
    else if normalized_pattern.contains('*') {
        let parts: Vec<&str> = normalized_pattern.split('*').collect();
        if parts.len() == 2 {
            let prefix = parts[0];
            let suffix = parts[1];

            return normalized_text.starts_with(prefix) && normalized_text.ends_with(suffix);
        }
    }

    normalized_text == normalized_pattern
}
