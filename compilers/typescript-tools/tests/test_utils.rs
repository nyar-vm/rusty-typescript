use std::{fs, path::PathBuf};
use typescript_tools::utils::{
    ensure_dir, find_project_root, get_env_separator, get_output_path, get_platform_newline, get_temp_dir, is_javascript_file,
    is_typescript_file, normalize_path_separators, platform_path, read_file, write_file,
};

#[test]
fn test_find_project_root() {
    // 测试在当前目录查找项目根目录
    let current_dir = PathBuf::from(".");
    let result = find_project_root(&current_dir);
    assert!(result.is_some());
}

#[test]
fn test_get_output_path() {
    let input = PathBuf::from("test.ts");
    let out_dir = Some(PathBuf::from("dist"));
    let extension = ".js";

    let output_path = get_output_path(&input, &out_dir, extension);
    assert_eq!(output_path, PathBuf::from("dist/test.js"));

    // 测试没有输出目录的情况
    let output_path = get_output_path(&input, &None, extension);
    assert_eq!(output_path, PathBuf::from("test.js"));
}

#[test]
fn test_ensure_dir() {
    let temp_dir = get_temp_dir();
    let test_dir = temp_dir.join("typescript-tools-test");

    // 确保目录不存在
    if test_dir.exists() {
        fs::remove_dir_all(&test_dir).unwrap();
    }

    // 测试创建目录
    let result = ensure_dir(&test_dir);
    assert!(result.is_ok());
    assert!(test_dir.exists());

    // 测试目录已存在的情况
    let result = ensure_dir(&test_dir);
    assert!(result.is_ok());

    // 清理
    fs::remove_dir_all(&test_dir).unwrap();
}

#[test]
fn test_read_write_file() {
    let temp_dir = get_temp_dir();
    let test_file = temp_dir.join("test.txt");
    let content = "Hello, world!";

    // 测试写入文件
    let write_result = write_file(&test_file, content);
    assert!(write_result.is_ok());

    // 测试读取文件
    let read_result = read_file(&test_file);
    assert!(read_result.is_ok());
    assert_eq!(read_result.unwrap(), content);

    // 清理
    fs::remove_file(&test_file).unwrap();
}

#[test]
fn test_normalize_path_separators() {
    let path_with_slash = PathBuf::from("path/to/file.ts");
    let normalized_path = normalize_path_separators(&path_with_slash);

    #[cfg(windows)]
    assert_eq!(normalized_path, PathBuf::from("path\\to\\file.ts"));

    #[cfg(not(windows))]
    assert_eq!(normalized_path, PathBuf::from("path/to/file.ts"));
}

#[test]
fn test_is_typescript_file() {
    let ts_file = PathBuf::from("test.ts");
    let tsx_file = PathBuf::from("test.tsx");
    let js_file = PathBuf::from("test.js");

    assert!(is_typescript_file(&ts_file));
    assert!(is_typescript_file(&tsx_file));
    assert!(!is_typescript_file(&js_file));
}

#[test]
fn test_is_javascript_file() {
    let js_file = PathBuf::from("test.js");
    let jsx_file = PathBuf::from("test.jsx");
    let ts_file = PathBuf::from("test.ts");

    assert!(is_javascript_file(&js_file));
    assert!(is_javascript_file(&jsx_file));
    assert!(!is_javascript_file(&ts_file));
}

#[test]
fn test_get_platform_newline() {
    let newline = get_platform_newline();

    #[cfg(windows)]
    assert_eq!(newline, "\r\n");

    #[cfg(not(windows))]
    assert_eq!(newline, "\n");
}

#[test]
fn test_platform_path() {
    let path_str = "path/to\\file.ts";
    let result = platform_path(path_str);

    // 验证路径被正确处理，不依赖于输入的分隔符
    assert!(result.to_string_lossy().contains("path"));
    assert!(result.to_string_lossy().contains("to"));
    assert!(result.to_string_lossy().contains("file.ts"));
}

#[test]
fn test_get_env_separator() {
    let separator = get_env_separator();

    #[cfg(windows)]
    assert_eq!(separator, ';');

    #[cfg(not(windows))]
    assert_eq!(separator, ':');
}
