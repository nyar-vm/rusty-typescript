//! 平台检测与兼容性测试模块
//!
//! 测试平台检测函数、内存对齐函数和 PlatformInfo 结构体。

use typescript::platform::{
    Arch, Endianness, OsType, PlatformInfo, PlatformPointer, PlatformSize, align_size, is_aligned, is_freebsd, is_linux,
    is_macos, is_netbsd, is_supported_platform, is_unix, is_windows,
};

/// 测试 Windows 平台检测
///
/// 验证 is_windows 函数能够正确检测当前是否为 Windows 系统
#[test]
fn test_is_windows() {
    let result = is_windows();
    #[cfg(target_os = "windows")]
    assert!(result);
    #[cfg(not(target_os = "windows"))]
    assert!(!result);
}

/// 测试 macOS 平台检测
///
/// 验证 is_macos 函数能够正确检测当前是否为 macOS 系统
#[test]
fn test_is_macos() {
    let result = is_macos();
    #[cfg(target_os = "macos")]
    assert!(result);
    #[cfg(not(target_os = "macos"))]
    assert!(!result);
}

/// 测试 Linux 平台检测
///
/// 验证 is_linux 函数能够正确检测当前是否为 Linux 系统
#[test]
fn test_is_linux() {
    let result = is_linux();
    #[cfg(target_os = "linux")]
    assert!(result);
    #[cfg(not(target_os = "linux"))]
    assert!(!result);
}

/// 测试平台互斥性
///
/// 验证同一时间只能检测到一个主要操作系统
#[test]
fn test_platform_exclusivity() {
    let windows = is_windows();
    let macos = is_macos();
    let linux = is_linux();
    let freebsd = is_freebsd();
    let netbsd = is_netbsd();

    let count = [windows, macos, linux, freebsd, netbsd].iter().filter(|&&x| x).count();
    assert!(count <= 1, "At most one major OS should be detected");
}

/// 测试 FreeBSD 平台检测
///
/// 验证 is_freebsd 函数能够正确检测当前是否为 FreeBSD 系统
#[test]
fn test_is_freebsd() {
    let result = is_freebsd();
    #[cfg(target_os = "freebsd")]
    assert!(result);
    #[cfg(not(target_os = "freebsd"))]
    assert!(!result);
}

/// 测试 NetBSD 平台检测
///
/// 验证 is_netbsd 函数能够正确检测当前是否为 NetBSD 系统
#[test]
fn test_is_netbsd() {
    let result = is_netbsd();
    #[cfg(target_os = "netbsd")]
    assert!(result);
    #[cfg(not(target_os = "netbsd"))]
    assert!(!result);
}

/// 测试类 Unix 平台检测
///
/// 验证 is_unix 函数能够正确检测当前是否为类 Unix 系统
#[test]
fn test_is_unix() {
    let result = is_unix();
    #[cfg(target_os = "windows")]
    assert!(!result);
    #[cfg(not(target_os = "windows"))]
    assert!(result);
}

/// 测试支持的平台检测
///
/// 验证 is_supported_platform 函数能够正确检测当前是否为支持的平台
#[test]
fn test_is_supported_platform() {
    let result = is_supported_platform();
    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    assert!(result);
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    assert!(!result);
}

/// 测试内存对齐计算 - 零大小
///
/// 验证 align_size 函数对零大小的处理
#[test]
fn test_align_size_zero() {
    assert_eq!(align_size(0, 1), 0);
    assert_eq!(align_size(0, 4), 0);
    assert_eq!(align_size(0, 8), 0);
    assert_eq!(align_size(0, 16), 0);
}

/// 测试内存对齐计算 - 已对齐大小
///
/// 验证 align_size 函数对已对齐大小的处理
#[test]
fn test_align_size_already_aligned() {
    assert_eq!(align_size(4, 4), 4);
    assert_eq!(align_size(8, 8), 8);
    assert_eq!(align_size(16, 16), 16);
    assert_eq!(align_size(32, 8), 32);
}

/// 测试内存对齐计算 - 需要对齐的大小
///
/// 验证 align_size 函数对需要向上对齐的大小的处理
#[test]
fn test_align_size_needs_alignment() {
    assert_eq!(align_size(1, 4), 4);
    assert_eq!(align_size(2, 4), 4);
    assert_eq!(align_size(3, 4), 4);
    assert_eq!(align_size(5, 4), 8);
    assert_eq!(align_size(6, 4), 8);
    assert_eq!(align_size(7, 4), 8);
    assert_eq!(align_size(9, 8), 16);
    assert_eq!(align_size(15, 16), 16);
    assert_eq!(align_size(17, 16), 32);
}

/// 测试内存对齐计算 - 不同对齐边界
///
/// 验证 align_size 函数对不同对齐边界的处理
#[test]
fn test_align_size_various_alignments() {
    assert_eq!(align_size(0, 1), 0);
    assert_eq!(align_size(1, 1), 1);
    assert_eq!(align_size(100, 1), 100);

    assert_eq!(align_size(0, 2), 0);
    assert_eq!(align_size(1, 2), 2);
    assert_eq!(align_size(2, 2), 2);
    assert_eq!(align_size(3, 2), 4);

    assert_eq!(align_size(0, 32), 0);
    assert_eq!(align_size(1, 32), 32);
    assert_eq!(align_size(31, 32), 32);
    assert_eq!(align_size(32, 32), 32);
    assert_eq!(align_size(33, 32), 64);
}

/// 测试指针对齐检查 - 1 字节对齐
///
/// 验证 is_aligned 函数对 1 字节对齐的处理（所有指针都应该满足）
#[test]
fn test_is_aligned_one_byte() {
    let data: [u8; 16] = [0; 16];
    let ptr = data.as_ptr();
    assert!(is_aligned(ptr, 1));

    for i in 0..16 {
        let offset_ptr = unsafe { ptr.add(i) };
        assert!(is_aligned(offset_ptr, 1));
    }
}

/// 测试指针对齐检查 - 数组对齐
///
/// 验证 is_aligned 函数对数组元素对齐的处理
#[test]
fn test_is_aligned_array() {
    let data: [u64; 8] = [0; 8];
    let ptr = data.as_ptr();

    assert!(is_aligned(ptr as *const u8, std::mem::align_of::<u64>()));

    for i in 0..8 {
        let elem_ptr = unsafe { ptr.add(i) };
        assert!(is_aligned(elem_ptr as *const u8, std::mem::align_of::<u64>()));
    }
}

/// 测试指针对齐检查 - 非对齐指针
///
/// 验证 is_aligned 函数能够正确检测非对齐指针
#[test]
fn test_is_aligned_unaligned() {
    let data: [u8; 32] = [0; 32];
    let ptr = data.as_ptr();

    if std::mem::align_of_val(&data) >= 16 {
        if is_aligned(ptr, 16) {
            let offset_ptr = unsafe { ptr.add(1) };
            assert!(!is_aligned(offset_ptr, 16));
            assert!(!is_aligned(offset_ptr, 8));
            assert!(!is_aligned(offset_ptr, 4));
            assert!(!is_aligned(offset_ptr, 2));
        }
    }
}

/// 测试 PlatformInfo 创建
///
/// 验证 PlatformInfo::current 能够正确创建平台信息
#[test]
fn test_platform_info_creation() {
    let info = PlatformInfo::current();
    assert!(!info.name.is_empty());
}

/// 测试 PlatformInfo 默认值
///
/// 验证 PlatformInfo::default 返回当前平台信息
#[test]
fn test_platform_info_default() {
    let info = PlatformInfo::default();
    let current = PlatformInfo::current();

    assert_eq!(info.name, current.name);
    assert_eq!(info.pointer_size, current.pointer_size);
    assert_eq!(info.endianness, current.endianness);
    assert_eq!(info.os_type, current.os_type);
    assert_eq!(info.arch, current.arch);
}

/// 测试 PlatformInfo 指针大小
///
/// 验证 PlatformInfo 的指针大小字段正确反映系统架构
#[test]
fn test_platform_info_pointer_size() {
    let info = PlatformInfo::current();

    #[cfg(target_pointer_width = "64")]
    assert_eq!(info.pointer_size, 8);

    #[cfg(target_pointer_width = "32")]
    assert_eq!(info.pointer_size, 4);
}

/// 测试 PlatformInfo 64 位检测
///
/// 验证 is_64bit 方法能够正确检测系统架构
#[test]
fn test_platform_info_is_64bit() {
    let info = PlatformInfo::current();

    #[cfg(target_pointer_width = "64")]
    assert!(info.is_64bit());

    #[cfg(target_pointer_width = "32")]
    assert!(!info.is_64bit());
}

/// 测试 PlatformInfo 字节序检测
///
/// 验证 is_little_endian 方法能够正确检测字节序
#[test]
fn test_platform_info_is_little_endian() {
    let info = PlatformInfo::current();

    #[cfg(target_endian = "little")]
    assert!(info.is_little_endian());

    #[cfg(target_endian = "big")]
    assert!(!info.is_little_endian());
}

/// 测试 PlatformInfo 平台名称
///
/// 验证 PlatformInfo 的 name 字段正确反映操作系统
#[test]
fn test_platform_info_name() {
    let info = PlatformInfo::current();

    #[cfg(target_os = "windows")]
    assert_eq!(info.name, "Windows");

    #[cfg(target_os = "macos")]
    assert_eq!(info.name, "macOS");

    #[cfg(target_os = "linux")]
    assert_eq!(info.name, "Linux");
}

/// 测试 PlatformInfo 操作系统类型
///
/// 验证 PlatformInfo 的 os_type 字段正确反映操作系统
#[test]
fn test_platform_info_os_type() {
    let info = PlatformInfo::current();

    #[cfg(target_os = "windows")]
    assert_eq!(info.os_type, OsType::Windows);

    #[cfg(target_os = "macos")]
    assert_eq!(info.os_type, OsType::Macos);

    #[cfg(target_os = "linux")]
    assert_eq!(info.os_type, OsType::Linux);
}

/// 测试 PlatformInfo CPU 架构
///
/// 验证 PlatformInfo 的 arch 字段正确反映 CPU 架构
#[test]
fn test_platform_info_arch() {
    let info = PlatformInfo::current();

    #[cfg(target_arch = "x86_64")]
    assert_eq!(info.arch, Arch::X86_64);

    #[cfg(target_arch = "x86")]
    assert_eq!(info.arch, Arch::X86);

    #[cfg(target_arch = "aarch64")]
    assert_eq!(info.arch, Arch::Aarch64);

    #[cfg(target_arch = "arm")]
    assert_eq!(info.arch, Arch::Arm);
}

/// 测试 Endianness 枚举
///
/// 验证 Endianness 枚举的 PartialEq 和 Clone 实现
#[test]
fn test_endianness_enum() {
    assert_eq!(Endianness::Little, Endianness::Little);
    assert_eq!(Endianness::Big, Endianness::Big);
    assert_ne!(Endianness::Little, Endianness::Big);

    let little = Endianness::Little;
    let cloned = little.clone();
    assert_eq!(little, cloned);
}

/// 测试 Endianness Debug 实现
///
/// 验证 Endianness 的 Debug trait 实现
#[test]
fn test_endianness_debug() {
    assert_eq!(format!("{:?}", Endianness::Little), "Little");
    assert_eq!(format!("{:?}", Endianness::Big), "Big");
}

/// 测试 OsType 枚举
///
/// 验证 OsType 枚举的 PartialEq 和 Clone 实现
#[test]
fn test_os_type_enum() {
    assert_eq!(OsType::Windows, OsType::Windows);
    assert_eq!(OsType::Macos, OsType::Macos);
    assert_eq!(OsType::Linux, OsType::Linux);
    assert_eq!(OsType::Unknown, OsType::Unknown);

    assert_ne!(OsType::Windows, OsType::Linux);
    assert_ne!(OsType::Macos, OsType::Windows);

    let windows = OsType::Windows;
    let cloned = windows.clone();
    assert_eq!(windows, cloned);
}

/// 测试 OsType Debug 实现
///
/// 验证 OsType 的 Debug trait 实现
#[test]
fn test_os_type_debug() {
    assert_eq!(format!("{:?}", OsType::Windows), "Windows");
    assert_eq!(format!("{:?}", OsType::Macos), "Macos");
    assert_eq!(format!("{:?}", OsType::Linux), "Linux");
    assert_eq!(format!("{:?}", OsType::Unknown), "Unknown");
}

/// 测试 Arch 枚举
///
/// 验证 Arch 枚举的 PartialEq 和 Clone 实现
#[test]
fn test_arch_enum() {
    assert_eq!(Arch::X86_64, Arch::X86_64);
    assert_eq!(Arch::X86, Arch::X86);
    assert_eq!(Arch::Aarch64, Arch::Aarch64);
    assert_eq!(Arch::Arm, Arch::Arm);
    assert_eq!(Arch::Unknown, Arch::Unknown);

    assert_ne!(Arch::X86_64, Arch::X86);
    assert_ne!(Arch::Aarch64, Arch::Arm);

    let x86_64 = Arch::X86_64;
    let cloned = x86_64.clone();
    assert_eq!(x86_64, cloned);
}

/// 测试 Arch Debug 实现
///
/// 验证 Arch 的 Debug trait 实现
#[test]
fn test_arch_debug() {
    assert_eq!(format!("{:?}", Arch::X86_64), "X86_64");
    assert_eq!(format!("{:?}", Arch::X86), "X86");
    assert_eq!(format!("{:?}", Arch::Aarch64), "Aarch64");
    assert_eq!(format!("{:?}", Arch::Arm), "Arm");
    assert_eq!(format!("{:?}", Arch::Unknown), "Unknown");
}

/// 测试 PlatformInfo Debug 实现
///
/// 验证 PlatformInfo 的 Debug trait 实现
#[test]
fn test_platform_info_debug() {
    let info = PlatformInfo::current();
    let debug_str = format!("{:?}", info);

    assert!(debug_str.contains("PlatformInfo"));
    assert!(debug_str.contains("name"));
    assert!(debug_str.contains("pointer_size"));
    assert!(debug_str.contains("endianness"));
    assert!(debug_str.contains("os_type"));
    assert!(debug_str.contains("arch"));
}

/// 测试 PlatformInfo Clone 实现
///
/// 验证 PlatformInfo 的 Clone trait 实现
#[test]
fn test_platform_info_clone() {
    let info = PlatformInfo::current();
    let cloned = info.clone();

    assert_eq!(info.name, cloned.name);
    assert_eq!(info.pointer_size, cloned.pointer_size);
    assert_eq!(info.endianness, cloned.endianness);
    assert_eq!(info.os_type, cloned.os_type);
    assert_eq!(info.arch, cloned.arch);
}

/// 测试 PlatformPointer 类型别名
///
/// 验证 PlatformPointer 类型正确反映平台指针大小
#[test]
fn test_platform_pointer_type() {
    #[cfg(target_pointer_width = "64")]
    {
        let _: PlatformPointer = 0u64;
    }

    #[cfg(target_pointer_width = "32")]
    {
        let _: PlatformPointer = 0u32;
    }
}

/// 测试 PlatformSize 类型别名
///
/// 验证 PlatformSize 类型正确反映平台 size_t 大小
#[test]
fn test_platform_size_type() {
    #[cfg(target_pointer_width = "64")]
    {
        let _: PlatformSize = 0u64;
    }

    #[cfg(target_pointer_width = "32")]
    {
        let _: PlatformSize = 0u32;
    }
}

/// 测试内存对齐与指针大小关系
///
/// 验证对齐计算与平台指针大小的关系
#[test]
fn test_align_size_with_pointer_size() {
    let ptr_size = std::mem::size_of::<*const u8>();
    let aligned = align_size(1, ptr_size);
    assert_eq!(aligned, ptr_size);
}

/// 测试结构体对齐
///
/// 验证常见结构体的对齐情况
#[test]
fn test_struct_alignment() {
    assert!(std::mem::align_of::<u8>() >= 1);
    assert!(std::mem::align_of::<u16>() >= 2);
    assert!(std::mem::align_of::<u32>() >= 4);
    assert!(std::mem::align_of::<u64>() >= 8);

    let u8_array: [u8; 16] = [0; 16];
    let ptr = u8_array.as_ptr();
    assert!(is_aligned(ptr, std::mem::align_of::<u8>()));
}

/// 测试大数值对齐
///
/// 验证大数值的对齐计算
#[test]
fn test_align_size_large_values() {
    assert_eq!(align_size(1024, 1024), 1024);
    assert_eq!(align_size(1025, 1024), 2048);
    assert_eq!(align_size(2047, 1024), 2048);
    assert_eq!(align_size(2048, 1024), 2048);
    assert_eq!(align_size(2049, 1024), 3072);
}

/// 测试 PlatformInfo 所有字段非空
///
/// 验证 PlatformInfo 的所有字段都有有效值
#[test]
fn test_platform_info_all_fields_valid() {
    let info = PlatformInfo::current();

    assert!(!info.name.is_empty());
    assert!(info.pointer_size == 4 || info.pointer_size == 8);

    assert!(matches!(info.endianness, Endianness::Little | Endianness::Big));

    assert!(matches!(
        info.os_type,
        OsType::Windows | OsType::Macos | OsType::Linux | OsType::Freebsd | OsType::Netbsd | OsType::Unknown
    ));

    assert!(matches!(
        info.arch,
        Arch::X86_64 | Arch::X86 | Arch::Aarch64 | Arch::Arm | Arch::Riscv64 | Arch::Riscv32 | Arch::Unknown
    ));
}

/// 测试 PlatformInfo 新方法
///
/// 验证 PlatformInfo 的新方法能够正确工作
#[test]
fn test_platform_info_new_methods() {
    let info = PlatformInfo::current();

    // 测试字节序方法
    assert!(info.is_little_endian() || info.is_big_endian());
    assert_ne!(info.is_little_endian(), info.is_big_endian());

    // 测试架构方法
    assert!(info.is_x86_64() || info.is_x86() || info.is_aarch64() || info.is_arm() || info.is_riscv());

    // 测试位宽方法
    assert!(info.is_32bit() || info.is_64bit());
    assert_ne!(info.is_32bit(), info.is_64bit());

    // 测试 to_string 方法
    let info_str = info.to_string();
    assert!(!info_str.is_empty());
    assert!(info_str.contains(&info.name));
}

/// 测试 OsType 新变体
///
/// 验证 OsType 的新变体能够正确工作
#[test]
fn test_os_type_new_variants() {
    assert_eq!(format!("{:?}", OsType::Freebsd), "Freebsd");
    assert_eq!(format!("{:?}", OsType::Netbsd), "Netbsd");

    let freebsd = OsType::Freebsd;
    let cloned = freebsd.clone();
    assert_eq!(freebsd, cloned);

    let netbsd = OsType::Netbsd;
    let cloned = netbsd.clone();
    assert_eq!(netbsd, cloned);
}

/// 测试 Arch 新变体
///
/// 验证 Arch 的新变体能够正确工作
#[test]
fn test_arch_new_variants() {
    assert_eq!(format!("{:?}", Arch::Riscv64), "Riscv64");
    assert_eq!(format!("{:?}", Arch::Riscv32), "Riscv32");

    let riscv64 = Arch::Riscv64;
    let cloned = riscv64.clone();
    assert_eq!(riscv64, cloned);

    let riscv32 = Arch::Riscv32;
    let cloned = riscv32.clone();
    assert_eq!(riscv32, cloned);
}

/// 测试 PlatformInfo 平台名称扩展
///
/// 验证 PlatformInfo 的 name 字段能够正确反映扩展的操作系统
#[test]
fn test_platform_info_name_extended() {
    let info = PlatformInfo::current();

    #[cfg(target_os = "windows")]
    assert_eq!(info.name, "Windows");

    #[cfg(target_os = "macos")]
    assert_eq!(info.name, "macOS");

    #[cfg(target_os = "linux")]
    assert_eq!(info.name, "Linux");

    #[cfg(target_os = "freebsd")]
    assert_eq!(info.name, "FreeBSD");

    #[cfg(target_os = "netbsd")]
    assert_eq!(info.name, "NetBSD");
}

/// 测试 PlatformInfo 操作系统类型扩展
///
/// 验证 PlatformInfo 的 os_type 字段能够正确反映扩展的操作系统
#[test]
fn test_platform_info_os_type_extended() {
    let info = PlatformInfo::current();

    #[cfg(target_os = "windows")]
    assert_eq!(info.os_type, OsType::Windows);

    #[cfg(target_os = "macos")]
    assert_eq!(info.os_type, OsType::Macos);

    #[cfg(target_os = "linux")]
    assert_eq!(info.os_type, OsType::Linux);

    #[cfg(target_os = "freebsd")]
    assert_eq!(info.os_type, OsType::Freebsd);

    #[cfg(target_os = "netbsd")]
    assert_eq!(info.os_type, OsType::Netbsd);
}

/// 测试 PlatformInfo CPU 架构扩展
///
/// 验证 PlatformInfo 的 arch 字段能够正确反映扩展的 CPU 架构
#[test]
fn test_platform_info_arch_extended() {
    let info = PlatformInfo::current();

    #[cfg(target_arch = "x86_64")]
    assert_eq!(info.arch, Arch::X86_64);

    #[cfg(target_arch = "x86")]
    assert_eq!(info.arch, Arch::X86);

    #[cfg(target_arch = "aarch64")]
    assert_eq!(info.arch, Arch::Aarch64);

    #[cfg(target_arch = "arm")]
    assert_eq!(info.arch, Arch::Arm);

    #[cfg(target_arch = "riscv64")]
    assert_eq!(info.arch, Arch::Riscv64);

    #[cfg(target_arch = "riscv32")]
    assert_eq!(info.arch, Arch::Riscv32);
}
