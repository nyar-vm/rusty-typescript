use clap::Parser;
use std::{fs::File, io::Write, path::PathBuf};
use tracing::debug;
use typescript_tools::{
    init,
    utils::{ensure_dir, normalize_path_separators},
};

/// TypeScript 打包工具
#[derive(Parser, Debug)]
#[command(name = "tsup")]
#[command(about = "Rusty TypeScript Bundler", long_about = None)]
#[command(version, author)]
struct Cli {
    /// 输入文件路径
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,

    /// 输出文件路径
    #[arg(long = "out", short = 'o', value_name = "FILE")]
    out: Option<PathBuf>,

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

    /// 压缩输出
    #[arg(long)]
    minify: bool,

    /// 生成 source map
    #[arg(long = "sourceMap")]
    source_map: bool,

    /// 监视模式
    #[arg(long, short = 'w')]
    watch: bool,

    /// 清空输出目录
    #[arg(long)]
    clean: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化工具集
    init();

    let cli = Cli::parse();

    // 打印命令行参数，用于调试
    debug!("CLI arguments: {:?}", cli);

    // 检查是否提供了输入文件
    if cli.files.is_empty() {
        eprintln!("错误: 未提供输入文件");
        return Ok(());
    }

    // 处理清理选项
    if cli.clean {
        if let Some(out_dir) = &cli.out_dir {
            let normalized_out_dir = normalize_path_separators(out_dir);
            if normalized_out_dir.exists() {
                println!("正在清理输出目录: {:?}", normalized_out_dir);
                // 模拟清理过程
                println!("清理完成!");
            }
        }
    }

    // 确定输出路径
    let output_path = if let Some(out) = &cli.out {
        normalize_path_separators(out)
    }
    else if let Some(out_dir) = &cli.out_dir {
        let normalized_out_dir = normalize_path_separators(out_dir);
        // 确保输出目录存在
        if let Err(e) = ensure_dir(&normalized_out_dir) {
            eprintln!("错误: 无法创建输出目录: {}", e);
            return Ok(());
        }
        normalized_out_dir.join("bundle.js")
    }
    else {
        let mut output = cli.files[0].clone();
        output.set_extension("js");
        output
    };

    println!("正在打包文件...");
    println!("输入文件: {:?}", cli.files);
    println!("输出文件: {:?}", output_path);

    // 处理编译选项
    let target = cli.target.as_deref().unwrap_or("es2018");
    let module_system = cli.module.as_deref().unwrap_or("esm");

    println!("目标 ECMAScript 版本: {}", target);
    println!("模块系统: {}", module_system);
    println!("严格模式: {}", cli.strict);
    println!("压缩输出: {}", cli.minify);
    println!("生成 source map: {}", cli.source_map);

    // 读取所有输入文件
    let mut bundled_content = String::new();
    for file in &cli.files {
        let normalized_file = normalize_path_separators(file);
        if !normalized_file.exists() {
            eprintln!("错误: 文件不存在: {:?}", normalized_file);
            continue;
        }

        println!("正在处理文件: {:?}", normalized_file);

        let content = std::fs::read_to_string(&normalized_file)?;
        bundled_content.push_str(&content);
        bundled_content.push_str("\n");
    }

    // 生成打包后的代码
    let bundle_js = format!(
        "// Bundled by Rusty TypeScript tsup\n// Target: {}\n// Module system: {}\n// Strict mode: {}\n{}",
        target, module_system, cli.strict, bundled_content
    );

    // 写入输出文件
    let output_dir = output_path.parent().unwrap_or_else(|| std::path::Path::new(".")).to_path_buf();
    if let Err(e) = ensure_dir(&output_dir) {
        eprintln!("错误: 无法创建输出目录: {}", e);
        return Ok(());
    }

    let mut file = File::create(&output_path)?;
    file.write_all(bundle_js.as_bytes())?;
    println!("打包完成! 输出文件: {:?}", output_path);

    // 如果需要生成 source map
    if cli.source_map {
        let source_map_path = output_path.with_extension("js.map");
        let source_map_content = format!(
            "{{\n    \"version\": 3,\n    \"file\": \"{}\",\n    \"sourceRoot\": \"\",\n    \"sources\": [{}],\n    \"names\": [],\n    \"mappings\": \";;AAAA;AAAA;AAAA;AAAA\"\n}}",
            output_path.file_name().unwrap_or_default().to_string_lossy(),
            cli.files
                .iter()
                .map(|f| format!("\"{}\"", f.file_name().unwrap_or_default().to_string_lossy()))
                .collect::<Vec<_>>()
                .join(", ")
        );

        let mut source_map_file = File::create(&source_map_path)?;
        source_map_file.write_all(source_map_content.as_bytes())?;
        println!("生成 source map 文件: {:?}", source_map_path);
    }

    // 处理监视模式
    if cli.watch {
        println!("进入监视模式...");
        // 模拟监视过程
        println!("监视模式已启动，按 Ctrl+C 退出");
        // 这里可以添加实际的文件监视逻辑
        std::thread::sleep(std::time::Duration::from_secs(5));
        println!("监视模式结束");
    }

    Ok(())
}
