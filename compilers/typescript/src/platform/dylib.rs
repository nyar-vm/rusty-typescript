//! 跨平台动态库加载模块
//!
//! 提供统一的跨平台动态库加载接口，支持 Windows、Linux 和 macOS。

use std::ffi::{c_char, c_void, CStr, CString, OsStr};
use std::path::Path;

/// 动态库错误类型
#[derive(Debug, Clone)]
pub enum DylibError {
    /// 打开动态库失败
    OpenError(String),
    /// 查找符号失败
    SymbolError(String),
    /// 关闭动态库失败
    CloseError(String),
}

impl std::fmt::Display for DylibError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DylibError::OpenError(msg) => write!(f, "Failed to open dynamic library: {}", msg),
            DylibError::SymbolError(msg) => write!(f, "Failed to find symbol: {}", msg),
            DylibError::CloseError(msg) => write!(f, "Failed to close dynamic library: {}", msg),
        }
    }
}

impl std::error::Error for DylibError {}

/// 动态库句柄
pub struct DynamicLibrary {
    /// 平台特定的句柄
    handle: *mut c_void,
}

/// 动态库符号
pub struct Symbol<T> {
    /// 符号指针
    ptr: *mut c_void,
    /// 类型标记
    _marker: std::marker::PhantomData<T>,
}

impl<T> Symbol<T> {
    /// 获取符号指针
    pub fn as_ptr(&self) -> *mut c_void {
        self.ptr
    }
}

impl<T> std::ops::Deref for Symbol<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.ptr as *const T) }
    }
}

impl DynamicLibrary {
    /// 打开动态库
    ///
    /// # 参数
    /// - `path`: 动态库路径
    ///
    /// # 返回
    /// - 成功时返回动态库句柄
    /// - 失败时返回错误
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, DylibError> {
        let path_str = path.as_ref().to_string_lossy();

        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::HMODULE;
            use windows_sys::Win32::System::LibraryLoader::{LoadLibraryA, GetLastError};

            let c_path = CString::new(path_str.as_bytes())
                .map_err(|e| DylibError::OpenError(e.to_string()))?;

            let handle = unsafe { LoadLibraryA(c_path.as_ptr() as *const u8) };

            if handle == 0 {
                let error_code = unsafe { GetLastError() };
                return Err(DylibError::OpenError(format!("Windows error code: {}", error_code)));
            }

            Ok(Self { handle: handle as *mut c_void })
        }

        #[cfg(unix)]
        {
            use libc::{dlopen, RTLD_NOW, RTLD_LOCAL};

            let c_path = CString::new(path_str.as_bytes())
                .map_err(|e| DylibError::OpenError(e.to_string()))?;

            let handle = unsafe { dlopen(c_path.as_ptr(), RTLD_NOW | RTLD_LOCAL) };

            if handle.is_null() {
                let error = unsafe {
                    let err_ptr = libc::dlerror();
                    if err_ptr.is_null() {
                        "Unknown error".to_string()
                    } else {
                        CStr::from_ptr(err_ptr).to_string_lossy().to_string()
                    }
                };
                return Err(DylibError::OpenError(error));
            }

            Ok(Self { handle })
        }
    }

    /// 获取动态库中的符号
    ///
    /// # 参数
    /// - `name`: 符号名称
    ///
    /// # 返回
    /// - 成功时返回符号
    /// - 失败时返回错误
    pub fn get<T>(&self, name: &str) -> Result<Symbol<T>, DylibError> {
        #[cfg(windows)]
        {
            use windows_sys::Win32::Foundation::HMODULE;
            use windows_sys::Win32::System::LibraryLoader::GetProcAddress;

            let c_name = CString::new(name)
                .map_err(|e| DylibError::SymbolError(e.to_string()))?;

            let ptr = unsafe { GetProcAddress(self.handle as HMODULE, c_name.as_ptr() as *const u8) };

            if ptr.is_none() {
                return Err(DylibError::SymbolError(format!("Symbol '{}' not found", name)));
            }

            Ok(Symbol {
                ptr: ptr.unwrap() as *mut c_void,
                _marker: std::marker::PhantomData,
            })
        }

        #[cfg(unix)]
        {
            use libc::dlsym;

            let c_name = CString::new(name)
                .map_err(|e| DylibError::SymbolError(e.to_string()))?;

            let ptr = unsafe { dlsym(self.handle, c_name.as_ptr()) };

            if ptr.is_null() {
                let error = unsafe {
                    let err_ptr = libc::dlerror();
                    if err_ptr.is_null() {
                        format!("Symbol '{}' not found", name)
                    } else {
                        format!("Symbol '{}' not found: {}", name, CStr::from_ptr(err_ptr).to_string_lossy())
                    }
                };
                return Err(DylibError::SymbolError(error));
            }

            Ok(Symbol {
                ptr,
                _marker: std::marker::PhantomData,
            })
        }
    }

    /// 获取动态库句柄
    pub fn as_ptr(&self) -> *mut c_void {
        self.handle
    }
}

impl Drop for DynamicLibrary {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            #[cfg(windows)]
            {
                use windows_sys::Win32::Foundation::HMODULE;
                use windows_sys::Win32::System::LibraryLoader::FreeLibrary;

                unsafe {
                    FreeLibrary(self.handle as HMODULE);
                }
            }

            #[cfg(unix)]
            {
                use libc::dlclose;

                unsafe {
                    dlclose(self.handle);
                }
            }
        }
    }
}

unsafe impl Send for DynamicLibrary {}
unsafe impl Sync for DynamicLibrary {}

/// 获取平台特定的动态库扩展名
pub fn get_dylib_extension() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "dll"
    }

    #[cfg(target_os = "macos")]
    {
        "dylib"
    }

    #[cfg(target_os = "linux")]
    {
        "so"
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        "so"
    }
}

/// 获取平台特定的动态库前缀
pub fn get_dylib_prefix() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        ""
    }

    #[cfg(not(target_os = "windows"))]
    {
        "lib"
    }
}

/// 构建完整的动态库文件名
///
/// # 参数
/// - `name`: 库名称（不含前缀和扩展名）
///
/// # 返回
/// 完整的动态库文件名
///
/// # 示例
/// ```
/// use typescript::platform::dylib::build_dylib_name;
///
/// // Windows: "mylib.dll"
/// // Linux: "libmylib.so"
/// // macOS: "libmylib.dylib"
/// let name = build_dylib_name("mylib");
/// ```
pub fn build_dylib_name(name: &str) -> String {
    format!("{}{}.{}", get_dylib_prefix(), name, get_dylib_extension())
}
