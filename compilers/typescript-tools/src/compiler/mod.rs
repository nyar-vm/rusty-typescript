/// 编译器模块
///
/// 处理 TypeScript 文件的编译和执行
use std::path::PathBuf;
use typescript::{create_runtime, run_script};

/// 编译选项
pub struct CompileOptions {
    /// 输出目录
    pub out_dir: Option<PathBuf>,
    /// 目标 ECMAScript 版本
    pub target: Option<String>,
    /// 模块系统
    pub module: Option<String>,
    /// 严格模式
    pub strict: bool,
    /// 不生成输出文件
    pub no_emit: bool,
    /// 生成 source map
    pub source_map: bool,
}

/// 编译结果
pub struct CompileResult {
    /// 是否成功
    pub success: bool,
    /// 错误信息
    pub errors: Vec<String>,
    /// 生成的文件
    pub generated_files: Vec<PathBuf>,
}

/// 执行 TypeScript 脚本
pub fn execute_script(script: &str) -> Result<String, String> {
    let mut runtime = create_runtime();
    match run_script(&mut runtime, script) {
        Ok(result) => Ok(result),
        Err(error) => Err(error.to_string()),
    }
}

/// 执行 TypeScript 文件
pub fn execute_file(file: &PathBuf) -> Result<String, String> {
    use std::fs::read_to_string;

    let content = match read_to_string(file) {
        Ok(content) => content,
        Err(e) => return Err(format!("无法读取文件: {}", e)),
    };

    execute_script(&content)
}

/// 编译 TypeScript 文件
pub fn compile_file(file: &PathBuf, _options: &CompileOptions) -> CompileResult {
    use std::fs::read_to_string;

    // 读取文件内容
    let file_content = match read_to_string(file) {
        Ok(content) => content,
        Err(e) => {
            return CompileResult {
                success: false, errors: vec![format!("无法读取文件: {}", e)], generated_files: vec![]
            };
        }
    };

    // 尝试执行脚本（作为编译的一部分）
    match execute_script(&file_content) {
        Ok(_) => {
            // 编译成功
            CompileResult { success: true, errors: vec![], generated_files: vec![] }
        }
        Err(e) => {
            // 编译失败
            CompileResult { success: false, errors: vec![e], generated_files: vec![] }
        }
    }
}

/// 并行编译多个 TypeScript 文件
pub fn compile_files(files: &[PathBuf], options: &CompileOptions) -> Vec<CompileResult> {
    use rayon::prelude::*;

    files.par_iter().map(|file| compile_file(file, options)).collect()
}
