//! 平台检测与兼容性模块
//!
//! 提供跨平台的类型定义、内存对齐工具和平台信息查询功能。

/// 跨平台动态库加载模块
pub mod dylib;

pub use dylib::{DylibError, DynamicLibrary, Symbol, build_dylib_name, get_dylib_extension, get_dylib_prefix};

/// 检测是否为 Windows 平台
///
/// 返回 `true` 表示当前运行在 Windows 系统上
#[cfg(target_os = "windows")]
pub const fn is_windows() -> bool {
    true
}

/// 检测是否为 Windows 平台
///
/// 返回 `true` 表示当前运行在 Windows 系统上
#[cfg(not(target_os = "windows"))]
pub const fn is_windows() -> bool {
    false
}

/// 检测是否为 macOS 平台
///
/// 返回 `true` 表示当前运行在 macOS 系统上
#[cfg(target_os = "macos")]
pub const fn is_macos() -> bool {
    true
}

/// 检测是否为 macOS 平台
///
/// 返回 `true` 表示当前运行在 macOS 系统上
#[cfg(not(target_os = "macos"))]
pub const fn is_macos() -> bool {
    false
}

/// 检测是否为 Linux 平台
///
/// 返回 `true` 表示当前运行在 Linux 系统上
#[cfg(target_os = "linux")]
pub const fn is_linux() -> bool {
    true
}

/// 检测是否为 Linux 平台
///
/// 返回 `true` 表示当前运行在 Linux 系统上
#[cfg(not(target_os = "linux"))]
pub const fn is_linux() -> bool {
    false
}

/// 检测是否为 FreeBSD 平台
///
/// 返回 `true` 表示当前运行在 FreeBSD 系统上
#[cfg(target_os = "freebsd")]
pub const fn is_freebsd() -> bool {
    true
}

/// 检测是否为 FreeBSD 平台
///
/// 返回 `true` 表示当前运行在 FreeBSD 系统上
#[cfg(not(target_os = "freebsd"))]
pub const fn is_freebsd() -> bool {
    false
}

/// 检测是否为 NetBSD 平台
///
/// 返回 `true` 表示当前运行在 NetBSD 系统上
#[cfg(target_os = "netbsd")]
pub const fn is_netbsd() -> bool {
    true
}

/// 检测是否为 NetBSD 平台
///
/// 返回 `true` 表示当前运行在 NetBSD 系统上
#[cfg(not(target_os = "netbsd"))]
pub const fn is_netbsd() -> bool {
    false
}

/// 检测是否为类 Unix 平台
///
/// 返回 `true` 表示当前运行在类 Unix 系统上（Linux、macOS、FreeBSD、NetBSD 等）
pub const fn is_unix() -> bool {
    !is_windows()
}

/// 检测是否为主要支持的平台
///
/// 返回 `true` 表示当前运行在我们主要支持的平台上（Windows、Linux、macOS）
pub const fn is_supported_platform() -> bool {
    is_windows() || is_linux() || is_macos()
}

/// 平台相关指针类型别名
///
/// 根据目标平台的指针大小定义相应的类型
#[cfg(target_pointer_width = "64")]
pub type PlatformPointer = u64;

/// 平台相关指针类型别名
///
/// 根据目标平台的指针大小定义相应的类型
#[cfg(target_pointer_width = "32")]
pub type PlatformPointer = u32;

/// 平台相关 size_t 类型别名
///
/// 根据目标平台定义 size_t 类型，用于表示对象大小
#[cfg(target_pointer_width = "64")]
pub type PlatformSize = u64;

/// 平台相关 size_t 类型别名
///
/// 根据目标平台定义 size_t 类型，用于表示对象大小
#[cfg(target_pointer_width = "32")]
pub type PlatformSize = u32;

/// 计算对齐后的大小
///
/// 将给定的大小向上对齐到指定的对齐边界
///
/// # 参数
/// - `size`: 原始大小
/// - `alignment`: 对齐边界，必须是 2 的幂
///
/// # 返回值
/// 对齐后的大小
///
/// # 示例
/// ```
/// use typescript::platform::align_size;
/// assert_eq!(align_size(5, 4), 8);
/// assert_eq!(align_size(8, 4), 8);
/// ```
pub const fn align_size(size: usize, alignment: usize) -> usize {
    let mask = alignment - 1;
    (size + mask) & !mask
}

/// 检查指针是否对齐
///
/// 判断给定指针是否按指定的对齐边界对齐
///
/// # 参数
/// - `ptr`: 要检查的指针
/// - `alignment`: 对齐边界，必须是 2 的幂
///
/// # 返回值
/// 如果指针对齐则返回 `true`，否则返回 `false`
///
/// # 示例
/// ```
/// use typescript::platform::is_aligned;
/// let data: [u8; 16] = [0; 16];
/// let ptr = data.as_ptr();
/// assert!(is_aligned(ptr, 1));
/// ```
pub fn is_aligned(ptr: *const u8, alignment: usize) -> bool {
    let addr = ptr as usize;
    addr % alignment == 0
}

/// 字节序枚举
///
/// 表示系统的字节序类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endianness {
    /// 小端序，低字节存储在低地址
    Little,
    /// 大端序，高字节存储在低地址
    Big,
}

/// 平台信息结构体
///
/// 包含当前运行环境的平台相关信息
#[derive(Debug, Clone)]
pub struct PlatformInfo {
    /// 平台名称
    pub name: String,
    /// 指针大小（字节）
    pub pointer_size: usize,
    /// 字节序
    pub endianness: Endianness,
    /// 操作系统类型
    pub os_type: OsType,
    /// CPU 架构
    pub arch: Arch,
}

impl PlatformInfo {
    /// 获取当前平台信息
    ///
    /// 返回包含当前运行环境详细信息的 `PlatformInfo` 实例
    pub fn current() -> Self {
        Self {
            name: Self::get_platform_name(),
            pointer_size: std::mem::size_of::<*const u8>(),
            endianness: Self::get_endianness(),
            os_type: Self::get_os_type(),
            arch: Self::get_arch(),
        }
    }

    /// 获取平台名称
    fn get_platform_name() -> String {
        #[cfg(target_os = "windows")]
        {
            "Windows".to_string()
        }
        #[cfg(target_os = "macos")]
        {
            "macOS".to_string()
        }
        #[cfg(target_os = "linux")]
        {
            "Linux".to_string()
        }
        #[cfg(target_os = "freebsd")]
        {
            "FreeBSD".to_string()
        }
        #[cfg(target_os = "netbsd")]
        {
            "NetBSD".to_string()
        }
        #[cfg(not(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "linux",
            target_os = "freebsd",
            target_os = "netbsd"
        )))]
        {
            "Unknown".to_string()
        }
    }

    /// 获取字节序
    fn get_endianness() -> Endianness {
        #[cfg(target_endian = "little")]
        {
            Endianness::Little
        }
        #[cfg(target_endian = "big")]
        {
            Endianness::Big
        }
    }

    /// 获取操作系统类型
    fn get_os_type() -> OsType {
        #[cfg(target_os = "windows")]
        {
            OsType::Windows
        }
        #[cfg(target_os = "macos")]
        {
            OsType::Macos
        }
        #[cfg(target_os = "linux")]
        {
            OsType::Linux
        }
        #[cfg(target_os = "freebsd")]
        {
            OsType::Freebsd
        }
        #[cfg(target_os = "netbsd")]
        {
            OsType::Netbsd
        }
        #[cfg(not(any(
            target_os = "windows",
            target_os = "macos",
            target_os = "linux",
            target_os = "freebsd",
            target_os = "netbsd"
        )))]
        {
            OsType::Unknown
        }
    }

    /// 获取 CPU 架构
    fn get_arch() -> Arch {
        #[cfg(target_arch = "x86_64")]
        {
            Arch::X86_64
        }
        #[cfg(target_arch = "x86")]
        {
            Arch::X86
        }
        #[cfg(target_arch = "aarch64")]
        {
            Arch::Aarch64
        }
        #[cfg(target_arch = "arm")]
        {
            Arch::Arm
        }
        #[cfg(target_arch = "riscv64")]
        {
            Arch::Riscv64
        }
        #[cfg(target_arch = "riscv32")]
        {
            Arch::Riscv32
        }
        #[cfg(not(any(
            target_arch = "x86_64",
            target_arch = "x86",
            target_arch = "aarch64",
            target_arch = "arm",
            target_arch = "riscv64",
            target_arch = "riscv32"
        )))]
        {
            Arch::Unknown
        }
    }

    /// 检查是否为 64 位系统
    pub fn is_64bit(&self) -> bool {
        self.pointer_size == 8
    }

    /// 检查是否为小端序
    pub fn is_little_endian(&self) -> bool {
        self.endianness == Endianness::Little
    }

    /// 检查是否为大端序
    pub fn is_big_endian(&self) -> bool {
        self.endianness == Endianness::Big
    }

    /// 检查是否为 x86_64 架构
    pub fn is_x86_64(&self) -> bool {
        self.arch == Arch::X86_64
    }

    /// 检查是否为 x86 架构
    pub fn is_x86(&self) -> bool {
        self.arch == Arch::X86
    }

    /// 检查是否为 ARM64 架构
    pub fn is_aarch64(&self) -> bool {
        self.arch == Arch::Aarch64
    }

    /// 检查是否为 ARM 架构
    pub fn is_arm(&self) -> bool {
        self.arch == Arch::Arm
    }

    /// 检查是否为 RISC-V 架构
    pub fn is_riscv(&self) -> bool {
        self.arch == Arch::Riscv64 || self.arch == Arch::Riscv32
    }

    /// 检查是否为 32 位系统
    pub fn is_32bit(&self) -> bool {
        self.pointer_size == 4
    }

    /// 获取平台详细信息字符串
    pub fn to_string(&self) -> String {
        format!("{} ({:?}, {}bit, {:?})", self.name, self.arch, if self.is_64bit() { 64 } else { 32 }, self.endianness)
    }
}

impl Default for PlatformInfo {
    fn default() -> Self {
        Self::current()
    }
}

/// 操作系统类型枚举
///
/// 表示支持的操作系统类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsType {
    /// Windows 操作系统
    Windows,
    /// macOS 操作系统
    Macos,
    /// Linux 操作系统
    Linux,
    /// FreeBSD 操作系统
    Freebsd,
    /// NetBSD 操作系统
    Netbsd,
    /// 未知操作系统
    Unknown,
}

/// CPU 架构枚举
///
/// 表示支持的 CPU 架构类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arch {
    /// x86_64 架构（64 位）
    X86_64,
    /// x86 架构（32 位）
    X86,
    /// ARM64 架构
    Aarch64,
    /// ARM 架构（32 位）
    Arm,
    /// RISC-V 64 位架构
    Riscv64,
    /// RISC-V 32 位架构
    Riscv32,
    /// 未知架构
    Unknown,
}
