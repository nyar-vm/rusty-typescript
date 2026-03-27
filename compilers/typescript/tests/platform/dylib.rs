use typescript::platform::dylib::{build_dylib_name, get_dylib_extension, get_dylib_prefix};

#[test]
fn test_get_dylib_extension() {
    let ext = get_dylib_extension();
    assert!(!ext.is_empty());
}

#[test]
fn test_get_dylib_prefix() {
    let prefix = get_dylib_prefix();
    // Windows 前缀为空，其他平台为 "lib"
    #[cfg(target_os = "windows")]
    assert_eq!(prefix, "");

    #[cfg(not(target_os = "windows"))]
    assert_eq!(prefix, "lib");
}

#[test]
fn test_build_dylib_name() {
    let name = build_dylib_name("test");
    assert!(name.contains("test"));

    #[cfg(target_os = "windows")]
    assert!(name.ends_with(".dll"));

    #[cfg(target_os = "macos")]
    assert!(name.ends_with(".dylib"));

    #[cfg(target_os = "linux")]
    assert!(name.ends_with(".so"));
}
