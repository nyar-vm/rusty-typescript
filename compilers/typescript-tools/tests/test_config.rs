//! 配置模块测试
//!
//! 测试配置解析和合并功能。

use std::{collections::HashMap, path::PathBuf};
use typescript_tools::config::CompilerOptions;

#[test]
fn test_compiler_options_merge() {
    let base = CompilerOptions {
        target: Some("ES2015".to_string()),
        module: Some("CommonJS".to_string()),
        strict: Some(false),
        jsx: None,
        out_dir: Some(PathBuf::from("dist")),
        root_dir: None,
        source_map: Some(false),
        declaration: None,
        declaration_dir: None,
        allow_js: None,
        check_js: None,
        module_resolution: None,
        base_url: None,
        paths: None,
        strict_null_checks: None,
        no_implicit_any: None,
        skip_lib_check: None,
        es_module_interop: None,
        no_emit: None,
    };

    let current = CompilerOptions {
        target: Some("ES2020".to_string()),
        module: None,
        strict: None,
        jsx: Some("react".to_string()),
        out_dir: None,
        root_dir: Some(PathBuf::from("src")),
        source_map: Some(true),
        declaration: None,
        declaration_dir: None,
        allow_js: None,
        check_js: None,
        module_resolution: None,
        base_url: None,
        paths: None,
        strict_null_checks: None,
        no_implicit_any: None,
        skip_lib_check: None,
        es_module_interop: None,
        no_emit: None,
    };

    let merged = current.merge(&base);

    assert_eq!(merged.target, Some("ES2020".to_string()));
    assert_eq!(merged.module, Some("CommonJS".to_string()));
    assert_eq!(merged.strict, Some(false));
    assert_eq!(merged.jsx, Some("react".to_string()));
    assert_eq!(merged.out_dir, Some(PathBuf::from("dist")));
    assert_eq!(merged.root_dir, Some(PathBuf::from("src")));
    assert_eq!(merged.source_map, Some(true));
}
