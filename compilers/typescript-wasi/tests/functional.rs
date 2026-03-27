//! WebAssembly 运行时功能测试
//!
//! 测试 WebAssembly 运行时的核心功能，验证其正常运行。

use typescript_wasi::{fs::WasiFs, memory::WasiMemory, WasiRuntime};

/// 测试 WASI 运行时基本功能
#[test]
fn test_wasi_runtime_basic() {
    let mut runtime = WasiRuntime::new();

    let code = r#"1 + 2"#;
    let result = runtime.execute(code);
    assert!(result.success, "执行简单 JavaScript 代码应该成功");
    assert_eq!(result.result, "3".to_string(), "执行结果应该是 3");
}

/// 测试 WASI 运行时编译功能
#[test]
fn test_wasi_runtime_compile() {
    let mut runtime = WasiRuntime::new();

    let code = r#"
        function add(a: number, b: number): number {
            return a + b;
        }
        add(1, 2);
    "#;
    let result = runtime.compile(code);
    assert!(result.success, "编译 TypeScript 代码应该成功");
}

/// 测试内存管理器基本功能
#[test]
fn test_memory_manager_basic() {
    let mut memory = WasiMemory::new(1024 * 1024);

    let size = 1024;
    let ptr = memory.allocate(size, false);
    assert!(ptr.is_some(), "内存分配应该成功");

    let ptr = ptr.unwrap();
    memory.deallocate(ptr, false);

    let ptr2 = memory.allocate(size, false);
    assert!(ptr2.is_some(), "内存释放后应该可以再次分配");
}

/// 测试内存管理器碎片整理
#[test]
fn test_memory_defragmentation() {
    let mut memory = WasiMemory::new(1024 * 1024);

    let mut allocations = vec![];
    for _ in 0..1000 {
        if let Some(ptr) = memory.allocate(16, false) {
            allocations.push(ptr);
        }
    }

    for i in 0..allocations.len() / 2 {
        memory.deallocate(allocations[i], false);
    }

    let fragmentation_rate = memory.defragment();
    assert!(fragmentation_rate >= 0.0 && fragmentation_rate <= 1.0, "碎片率应该在 0-1 之间");
}

/// 测试 WASI 运行时执行复杂 JavaScript
#[test]
fn test_wasi_runtime_complex_js() {
    let mut runtime = WasiRuntime::new();

    let code = r#"
        function factorial(n) {
            if (n <= 1) return 1;
            return n * factorial(n - 1);
        }
        factorial(5);
    "#;
    let result = runtime.execute(code);
    assert!(result.success, "执行复杂 JavaScript 代码应该成功");
    assert_eq!(result.result, "120".to_string(), "执行结果应该是 120");
}

/// 测试 WASI 运行时执行 TypeScript
#[test]
fn test_wasi_runtime_typescript() {
    let mut runtime = WasiRuntime::new();

    let code = r#"
        interface Person {
            name: string;
            age: number;
        }
        
        function greet(person: Person): string {
            return `Hello, ${person.name}! You are ${person.age} years old.`;
        }
        
        greet({ name: "John", age: 30 });
    "#;
    let result = runtime.compile(code);
    assert!(result.success, "执行 TypeScript 代码应该成功");
}

/// 测试 WASI 文件系统基本操作
#[test]
fn test_wasi_fs_basic_operations() {
    use typescript_wasi::fs::OpenMode;

    let mut fs = WasiFs::new();

    let fd = fs.fd_open("/test.txt", OpenMode::Write).unwrap();
    let written = fs.fd_write(fd, b"Hello, World!").unwrap();
    assert_eq!(written, 13);

    fs.fd_close(fd).unwrap();

    let fd = fs.fd_open("/test.txt", OpenMode::Read).unwrap();
    let mut buf = [0u8; 13];
    let read = fs.fd_read(fd, &mut buf).unwrap();
    assert_eq!(read, 13);
    assert_eq!(&buf, b"Hello, World!");
}
