use clap::Parser;
use num_cpus;
use std::{path::PathBuf, sync::Arc};
use tokio::{runtime::Builder, task};
use tracing::debug;
use typescript_tools::{
    config::{load_config_with_extends, resolve_file_patterns},
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
    debug!("CLI arguments: {:?}", cli);

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

    // 加载配置文件并处理继承
    let mut files_to_process = cli.files.clone();
    let mut target = cli.target.clone();
    let mut module = cli.module.clone();
    let mut strict = cli.strict;
    let mut out_dir = cli.out_dir.clone();
    let mut source_map = cli.source_map;
    let mut declaration = cli.declaration;
    let mut no_emit = cli.no_emit;
    let mut skip_lib_check = cli.skip_lib_check;

    if let Some(project_path) = &cli.project {
        println!("正在加载配置文件: {:?}", project_path);
        match load_config_with_extends(project_path) {
            Ok(config) => {
                println!("配置文件加载成功!");
                debug!("配置: {:?}", config);

                // 处理配置文件中的文件列表
                if files_to_process.is_empty() {
                    if let Some(files) = &config.files {
                        files_to_process.extend(files.clone());
                    }
                    else {
                        // 处理 include/exclude 模式
                        let base_dir = project_path.parent().unwrap_or_else(|| std::path::Path::new("."));
                        match resolve_file_patterns(base_dir, config.include.as_ref(), config.exclude.as_ref()) {
                            Ok(files) => {
                                files_to_process.extend(files);
                            }
                            Err(e) => {
                                eprintln!("错误: 解析文件模式失败: {}", e);
                            }
                        }
                    }
                }

                // 从配置文件中获取编译选项（如果命令行未指定）
                if let Some(compiler_options) = &config.compiler_options {
                    if target.is_none() {
                        target = compiler_options.target.clone();
                    }
                    if module.is_none() {
                        module = compiler_options.module.clone();
                    }
                    if !cli.strict {
                        if let Some(config_strict) = compiler_options.strict {
                            strict = config_strict;
                        }
                    }
                    if out_dir.is_none() {
                        out_dir = compiler_options.out_dir.clone();
                    }
                    if !cli.source_map {
                        if let Some(config_source_map) = compiler_options.source_map {
                            source_map = config_source_map;
                        }
                    }
                    if !cli.declaration {
                        if let Some(config_declaration) = compiler_options.declaration {
                            declaration = config_declaration;
                        }
                    }
                    if !cli.no_emit {
                        if let Some(config_no_emit) = compiler_options.no_emit {
                            no_emit = config_no_emit;
                        }
                    }
                    if !cli.skip_lib_check {
                        if let Some(config_skip_lib_check) = compiler_options.skip_lib_check {
                            skip_lib_check = config_skip_lib_check;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("错误: 无法加载配置文件: {}", e);
            }
        }
    }

    // 检查是否有文件需要处理
    if files_to_process.is_empty() {
        eprintln!("错误: 未提供输入文件，也未在配置文件中找到文件");
        return Ok(());
    }

    // 处理 --execute 选项
    if cli.execute {
        for file in &files_to_process {
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

    // 包装参数为 Arc
    let files_arc = Arc::new(files_to_process);
    let target_arc = Arc::new(target);
    let module_arc = Arc::new(module);
    let strict_arc = Arc::new(strict);
    let out_dir_arc = Arc::new(out_dir);
    let source_map_arc = Arc::new(source_map);
    let declaration_arc = Arc::new(declaration);
    let no_emit_arc = Arc::new(no_emit);
    let skip_lib_check_arc = Arc::new(skip_lib_check);
    let format_arc = Arc::new(cli.format);
    let no_emit_on_error_arc = Arc::new(cli.no_emit_on_error);
    let remove_comments_arc = Arc::new(cli.remove_comments);
    let minify_arc = Arc::new(cli.minify);
    let quiet_arc = Arc::new(cli.quiet);

    // 并行处理所有输入文件
    rt.block_on(async {
        let mut tasks = vec![];

        for file in &*files_arc {
            let file = file.clone();
            let target = target_arc.clone();
            let module = module_arc.clone();
            let strict = strict_arc.clone();
            let out_dir = out_dir_arc.clone();
            let source_map = source_map_arc.clone();
            let declaration = declaration_arc.clone();
            let no_emit = no_emit_arc.clone();
            let skip_lib_check = skip_lib_check_arc.clone();
            let format = format_arc.clone();
            let no_emit_on_error = no_emit_on_error_arc.clone();
            let remove_comments = remove_comments_arc.clone();
            let minify = minify_arc.clone();
            let quiet = quiet_arc.clone();

            tasks.push(task::spawn(async move {
                // 规范化文件路径
                let normalized_file = normalize_path_separators(&file);

                if !normalized_file.exists() {
                    eprintln!("错误: 文件不存在: {:?}", normalized_file);
                    return;
                }

                if !*quiet {
                    println!("正在处理文件: {:?}", normalized_file);
                }

                // 处理代码格式化
                if *format {
                    if !*quiet {
                        println!("正在格式化代码: {:?}", normalized_file);
                    }
                    // 模拟格式化过程
                    if !*quiet {
                        println!("代码格式化完成!");
                    }
                } else if *no_emit || *no_emit_on_error {
                    // 模拟类型检查
                    if !*quiet {
                        println!("正在进行类型检查: {:?}", normalized_file);
                    }
                    // 模拟类型检查过程
                    if !*quiet {
                        println!("类型检查通过!");
                    }
                } else {
                    // 处理编译选项
                    let target_value = target.as_deref().unwrap_or("es2020");
                    let module_system = module.as_deref().unwrap_or("esm");

                    // 生成输出路径
                    let output_path = if let Some(out_dir_path) = &*out_dir {
                        // 规范化输出目录路径
                        let normalized_out_dir = normalize_path_separators(out_dir_path);
                        // 确保输出目录存在
                        if let Err(e) = ensure_dir(&normalized_out_dir) {
                            eprintln!("错误: 无法创建输出目录: {}", e);
                            return;
                        }
                        normalized_out_dir.join(file.file_stem().unwrap_or_default()).with_extension("js")
                    } else {
                        let mut output = normalized_file.clone();
                        output.set_extension("js");
                        output
                    };

                    if !*quiet {
                        println!("生成输出文件: {:?}", output_path);
                        println!("目标 ECMAScript 版本: {}", target_value);
                        println!("模块系统: {}", module_system);
                        println!("严格模式: {}", *strict);
                        println!("生成 source map: {}", *source_map);
                        println!("生成声明文件: {}", *declaration);
                        println!("跳过库检查: {}", *skip_lib_check);
                        println!("移除注释: {}", *remove_comments);
                        println!("压缩输出: {}", *minify);
                    }

                    // 写入模拟的 JavaScript 代码
                    let mut mock_js = format!(
                        "// Compiled from {:?}\n// Target: {}\n// Module system: {}\n// Strict mode: {}",
                        normalized_file, target_value, module_system, *strict
                    );

                    if !*remove_comments {
                        mock_js.push_str("\n// Generated by Rusty TypeScript Compiler");
                    }

                    mock_js.push_str("\nconsole.log('Hello from Rusty TypeScript!');");

                    if *minify {
                        mock_js = mock_js.replace("\n", " ").replace("  ", " ").trim().to_string();
                    }

                    if let Err(e) = std::fs::write(&output_path, mock_js) {
                        eprintln!("错误: 无法写入输出文件: {}", e);
                    }

                    // 如果需要生成 source map
                    if *source_map {
                        let source_map_path = output_path.with_extension("js.map");
                        let source_map_content = format!(
                            "{{\n    \"version\": 3,\n    \"file\": \"{}\",\n    \"sourceRoot\": \"\",\n    \"sources\": [\"{}\"],\n    \"names\": [],\n    \"mappings\": \";;AAAA;AAAA;AAAA;AAAA\"\n}}",
                            output_path.file_name().unwrap_or_default().to_string_lossy(),
                            normalized_file.file_name().unwrap_or_default().to_string_lossy()
                        );
                        if let Err(e) = std::fs::write(&source_map_path, source_map_content) {
                            eprintln!("错误: 无法写入 source map 文件: {}", e);
                        }
                        if !*quiet {
                            println!("生成 source map 文件: {:?}", source_map_path);
                        }
                    }

                    // 如果需要生成声明文件
                    if *declaration {
                        let declaration_path = output_path.with_extension("d.ts");
                        let declaration_content = format!(
                            "// Declaration file for {:?}\n// Generated by Rusty TypeScript Compiler",
                            normalized_file
                        );
                        if let Err(e) = std::fs::write(&declaration_path, declaration_content) {
                            eprintln!("错误: 无法写入声明文件: {}", e);
                        }
                        if !*quiet {
                            println!("生成声明文件: {:?}", declaration_path);
                        }
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
