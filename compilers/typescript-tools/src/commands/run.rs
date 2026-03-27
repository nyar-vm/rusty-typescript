/// 运行命令
///
/// 实现直接执行 TypeScript 脚本的功能
use crate::commands::CommandResult;
use crate::compiler::{CompileOptions, compile_file};
use std::path::PathBuf;

/// 运行命令选项
#[derive(Debug, Clone)]
pub struct RunOptions {
    /// 脚本文件路径
    pub file: PathBuf,
    /// 传递给脚本的参数
    pub args: Vec<String>,
    /// 是否监视模式
    pub watch: bool,
    /// 目标 ECMAScript 版本
    pub target: Option<String>,
    /// 模块系统
    pub module: Option<String>,
    /// 是否启用严格模式
    pub strict: bool,
    /// 是否生成 source map
    pub source_map: bool,
}

/// 执行运行命令
pub fn execute(options: RunOptions) -> CommandResult {
    // 检查文件是否存在
    if !options.file.exists() {
        return CommandResult::error(1, format!("文件不存在: {:?}", options.file));
    }

    // 检查文件扩展名
    if let Some(ext) = options.file.extension() {
        if ext != "ts" && ext != "js" {
            return CommandResult::error(1, format!("不支持的文件类型: {:?}", ext));
        }
    }

    if options.watch { run_with_watch(&options) } else { run_once(&options) }
}

/// 执行一次
fn run_once(options: &RunOptions) -> CommandResult {
    println!("正在执行: {:?}", options.file);

    // 编译 TypeScript 文件
    let compile_options = CompileOptions {
        out_dir: None,
        target: options.target.clone().or_else(|| Some("ES2020".to_string())),
        module: options.module.clone().or_else(|| Some("CommonJS".to_string())),
        strict: options.strict,
        no_emit: false,
        source_map: options.source_map,
        declaration: false,
        minify: false,
    };

    let result = compile_file(&options.file, &compile_options);
    if result.success {
        println!("编译成功!");

        // 执行编译后的 JavaScript
        match execute_script(&options.file, &options.args) {
            Ok(output) => {
                println!("{}", output);
                CommandResult::success("脚本执行完成".to_string())
            }
            Err(e) => CommandResult::error(1, format!("脚本执行失败: {}", e)),
        }
    }
    else {
        let errors = result.errors.join("\n");
        CommandResult::error(1, format!("编译失败:\n{}", errors))
    }
}

/// 监视模式运行
fn run_with_watch(options: &RunOptions) -> CommandResult {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    println!("进入监视模式，按 Ctrl+C 退出...");

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    /// 处理 Ctrl+C
    match ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        println!("\n正在退出监视模式...");
    }) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("警告: 无法设置 Ctrl+C 处理器: {}", e);
            eprintln!("请手动终止进程退出监视模式");
        }
    }

    /// 获取文件修改时间
    let mut last_modified =
        std::fs::metadata(&options.file).and_then(|m| m.modified()).unwrap_or_else(|_| std::time::SystemTime::UNIX_EPOCH);

    /// 首次执行
    run_once(options);

    /// 监视循环
    while running.load(Ordering::SeqCst) {
        std::thread::sleep(std::time::Duration::from_millis(500));

        /// 检查文件是否修改
        if let Ok(metadata) = std::fs::metadata(&options.file) {
            if let Ok(modified) = metadata.modified() {
                if modified > last_modified {
                    println!("\n检测到文件变化，重新执行...");
                    last_modified = modified;
                    run_once(options);
                    println!("\n继续监视...");
                }
            }
        }
    }

    CommandResult::success("监视模式已退出".to_string())
}

/// 执行脚本
fn execute_script(file: &PathBuf, args: &[String]) -> Result<String, String> {
    use std::process::{Command, Stdio};

    // 根据平台选择合适的命令执行器
    let (executor, executor_args) = get_script_executor();

    // 构建命令
    let mut command = Command::new(executor);
    command.args(executor_args);
    command.arg(file);
    command.args(args);
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    // 执行命令
    match command.output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);

            if output.status.success() { Ok(stdout.to_string()) } else { Err(format!("执行失败: {}\n{}", stderr, stdout)) }
        }
        Err(e) => Err(format!("无法执行命令: {}", e)),
    }
}

/// 获取平台特定的脚本执行器
fn get_script_executor() -> (&'static str, Vec<&'static str>) {
    #[cfg(windows)]
    {
        ("node.exe", vec![])
    }
    #[cfg(not(windows))]
    {
        ("node", vec![])
    }
}
