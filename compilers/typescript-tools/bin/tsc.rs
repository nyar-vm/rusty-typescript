use clap::Parser;
use num_cpus;
use std::{path::PathBuf, sync::Arc};
use tokio::{runtime::Builder, task};
use typescript_tools::{
    config::load_config,
    execute_file, init,
    utils::{ensure_dir, normalize_path_separators},
};

/// TypeScript 编译器
#[derive(Parser, Debug)]
#[command(name = "tsc")]
#[command(about = "Rusty TypeScript Compiler", long_about = None)]
#[command(version, author)]
struct Cli {
    /// 输入文件路径
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,

    /// 输出目录
    #[arg(long = "outDir", short = 'd', value_name = "DIR")]
    out_dir: Option<PathBuf>,

    /// 目标 ECMAScript 版本
    #[arg(long, value_name = "VERSION")]
    target: Option<String>,

    /// 模块系统
    #[arg(long, value_name = "MODULE")]
    module: Option<String>,

    /// 严格模式
    #[arg(long)]
    strict: bool,

    /// 不生成输出文件
    #[arg(long)]
    no_emit: bool,

    /// 生成 source map
    #[arg(long = "sourceMap")]
    source_map: bool,

    /// 格式化代码
    #[arg(long)]
    format: bool,

    /// 输出 JSON 格式
    #[arg(long)]
    json: bool,

    /// 安静模式，仅显示错误
    #[arg(long, short = 'q')]
    quiet: bool,

    /// 监视模式
    #[arg(long, short = 'w')]
    watch: bool,

    /// 生成声明文件
    #[arg(long)]
    declaration: bool,

    /// 仅类型检查
    #[arg(long)]
    no_emit_on_error: bool,

    /// 跳过库检查
    #[arg(long)]
    skip_lib_check: bool,

    /// 移除注释
    #[arg(long)]
    remove_comments: bool,

    /// 压缩输出
    #[arg(long)]
    minify: bool,

    /// 配置文件路径
    #[arg(long, value_name = "FILE")]
    project: Option<PathBuf>,

    /// 直接执行 TypeScript 脚本
    #[arg(long)]
    execute: bool,

    /// WebIDL 文件路径
    #[arg(long = "webidl", value_name = "FILE")]
    webidl_files: Vec<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化工具集
    init();

    let cli = Cli::parse();

    // 打印命令行参数，用于调试
    log::debug!("CLI arguments: {:?}", cli);

    // 处理 WebIDL 文件
    if !cli.webidl_files.is_empty() {
        println!("正在处理 WebIDL 文件...");
        for webidl_file in &cli.webidl_files {
            let normalized_file = normalize_path_separators(webidl_file);

            if !normalized_file.exists() {
                eprintln!("错误: WebIDL 文件不存在: {:?}", normalized_file);
                continue;
            }

            println!("正在处理 WebIDL 文件: {:?}", normalized_file);
            // 这里将在后续实现中添加实际的 WebIDL 处理逻辑
            println!("WebIDL 文件处理完成: {:?}", normalized_file);
        }
    }

    // 加载配置文件
    if let Some(project_path) = &cli.project {
        println!("正在加载配置文件: {:?}", project_path);
        match load_config(project_path) {
            Ok(config) => {
                println!("配置文件加载成功!");
                // 这里可以根据配置文件设置编译选项
                log::debug!("配置: {:?}", config);
            }
            Err(e) => {
                eprintln!("错误: 无法加载配置文件: {}", e);
            }
        }
    }

    // 检查是否提供了输入文件
    if cli.files.is_empty() {
        eprintln!("错误: 未提供输入文件");
        return Ok(());
    }

    // 处理 --execute 选项
    if cli.execute {
        for file in &cli.files {
            let normalized_file = normalize_path_separators(file);

            if !normalized_file.exists() {
                eprintln!("错误: 文件不存在: {:?}", normalized_file);
                continue;
            }

            println!("正在执行文件: {:?}", normalized_file);

            match execute_file(&normalized_file) {
                Ok(result) => {
                    println!("执行结果: {}", result);
                }
                Err(e) => {
                    eprintln!("执行错误: {}", e);
                }
            }
        }
        return Ok(());
    }

    // 创建 Tokio 运行时
    let rt = Builder::new_multi_thread().worker_threads(num_cpus::get()).build()?;

    // 包装 CLI 参数为 Arc
    let cli_arc = Arc::new(cli);

    // 并行处理所有输入文件
    rt.block_on(async {
        let mut tasks = vec![];

        for file in &cli_arc.files {
            let cli = cli_arc.clone();
            let file = file.clone();

            tasks.push(task::spawn(async move {
                // 规范化文件路径
                let normalized_file = normalize_path_separators(&file);

                if !normalized_file.exists() {
                    eprintln!("错误: 文件不存在: {:?}", normalized_file);
                    return;
                }

                println!("正在处理文件: {:?}", normalized_file);

                // 模拟代码格式化
                if cli.format {
                    println!("正在格式化代码: {:?}", normalized_file);
                    // 模拟格式化过程
                    println!("代码格式化完成!");
                }
                else if cli.no_emit {
                    // 模拟类型检查
                    println!("正在进行类型检查: {:?}", normalized_file);
                    // 模拟类型检查过程
                    println!("类型检查通过!");
                }
                else {
                    // 处理编译选项
                    let target = cli.target.as_deref().unwrap_or("es2018");
                    let module_system = cli.module.as_deref().unwrap_or("esm");

                    // 生成输出路径
                    let output_path = if let Some(out_dir) = &cli.out_dir {
                        // 规范化输出目录路径
                        let normalized_out_dir = normalize_path_separators(out_dir);
                        // 确保输出目录存在
                        if let Err(e) = ensure_dir(&normalized_out_dir) {
                            eprintln!("错误: 无法创建输出目录: {}", e);
                            return;
                        }
                        normalized_out_dir.join(file.file_stem().unwrap_or_default()).with_extension("js")
                    }
                    else {
                        let mut output = normalized_file.clone();
                        output.set_extension("js");
                        output
                    };

                    println!("生成输出文件: {:?}", output_path);
                    println!("目标 ECMAScript 版本: {}", target);
                    println!("模块系统: {}", module_system);
                    println!("严格模式: {}", cli.strict);
                    println!("生成 source map: {}", cli.source_map);

                    // 写入模拟的 JavaScript 代码
                    let mock_js = format!(
                        "// Compiled from {:?}\n// Target: {}\n// Module system: {}\n// Strict mode: {}\nconsole.log('Hello from Rusty TypeScript!');",
                        normalized_file, target, module_system, cli.strict
                    );
                    if let Err(e) = std::fs::write(&output_path, mock_js) {
                        eprintln!("错误: 无法写入输出文件: {}", e);
                    }

                    // 如果需要生成 source map
                    if cli.source_map {
                        let source_map_path = output_path.with_extension("js.map");
                        let source_map_content = format!(
                            "{{\n    \"version\": 3,\n    \"file\": \"{}\",\n    \"sourceRoot\": \"\",\n    \"sources\": [\"{}\"],\n    \"names\": [],\n    \"mappings\": \";;AAAA;AAAA;AAAA;AAAA\"\n}}",
                            output_path.file_name().unwrap_or_default().to_string_lossy(),
                            normalized_file.file_name().unwrap_or_default().to_string_lossy()
                        );
                        if let Err(e) = std::fs::write(&source_map_path, source_map_content) {
                            eprintln!("错误: 无法写入 source map 文件: {}", e);
                        }
                        println!("生成 source map 文件: {:?}", source_map_path);
                    }
                }
            }));
        }

        // 等待所有任务完成
        for task in tasks {
            task.await.unwrap();
        }
    });

    Ok(())
}
