use std::{fs, path::PathBuf};
use typescript_tools::compiler::{CompileOptions, compile_file, compile_files};

#[test]
fn test_compile_file() {
    // 创建测试文件
    let test_file = PathBuf::from("test_compile.ts");
    let content = "function test() { console.log('Hello'); }";
    fs::write(&test_file, content).unwrap();

    // 测试编译文件
    let options =
        CompileOptions { out_dir: None, target: None, module: None, strict: false, no_emit: false, source_map: false };

    let result = compile_file(&test_file, &options);
    assert!(result.success);
    assert!(result.errors.is_empty());

    // 清理
    fs::remove_file(&test_file).unwrap();
}

#[test]
fn test_compile_files() {
    // 创建测试文件
    let test_file1 = PathBuf::from("test_compile1.ts");
    let test_file2 = PathBuf::from("test_compile2.ts");
    let content = "function test() { console.log('Hello'); }";

    fs::write(&test_file1, content).unwrap();
    fs::write(&test_file2, content).unwrap();

    // 测试并行编译多个文件
    let options =
        CompileOptions { out_dir: None, target: None, module: None, strict: false, no_emit: false, source_map: false };

    let files = vec![test_file1.clone(), test_file2.clone()];
    let results = compile_files(&files, &options);

    assert_eq!(results.len(), 2);
    for result in results {
        assert!(result.success);
        assert!(result.errors.is_empty());
    }

    // 清理
    fs::remove_file(&test_file1).unwrap();
    fs::remove_file(&test_file2).unwrap();
}
