use crate::errors::ConfigResult;
/// 配置模块
///
/// 处理 tsconfig.json 配置文件
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 编译器配置
#[derive(Debug, Deserialize, Serialize)]
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
#[derive(Debug, Deserialize, Serialize)]
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
#[derive(Debug, Deserialize, Serialize)]
pub struct CheckConfig {
    /// 严格空检查
    pub strict_null_checks: Option<bool>,
    /// 禁止隐式 any
    pub no_implicit_any: Option<bool>,
    /// 跳过库检查
    pub skip_lib_check: Option<bool>,
}

/// 格式化配置
#[derive(Debug, Deserialize, Serialize)]
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
#[derive(Debug, Deserialize, Serialize)]
pub struct TestConfig {
    /// 是否生成覆盖率报告
    pub coverage: Option<bool>,
    /// 是否并行运行
    pub parallel: Option<bool>,
    /// 超时时间（毫秒）
    pub timeout: Option<u64>,
}

/// 项目配置
#[derive(Debug, Deserialize, Serialize)]
pub struct ProjectConfig {
    /// 编译器配置
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

/// 加载配置文件
pub fn load_config(path: &PathBuf) -> ConfigResult<ProjectConfig> {
    use std::{fs::File, io::Read};

    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let config: ProjectConfig = toml::from_str(&content)?;
    Ok(config)
}
