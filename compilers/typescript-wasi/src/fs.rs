//! WASI 文件系统接口实现
//!
//! 该模块提供了 WASI (WebAssembly System Interface) 文件系统的虚拟实现，
//! 支持内存中的文件操作，包括文件的打开、读取、写入、关闭和定位等功能。

use std::{
    collections::HashMap,
    io::{self, Cursor, Read, Seek, SeekFrom, Write},
    path::PathBuf,
    sync::{Arc, Mutex},
};

/// 文件描述符类型
///
/// 用于唯一标识一个打开的文件描述符。
/// 在 WASI 中，文件描述符是一个非负整数。
pub type Fd = u32;

/// 预定义的文件描述符
///
/// 这些是 WASI 规范中预定义的标准文件描述符。
pub mod preopen {
    use super::Fd;

    /// 标准输入文件描述符
    pub const STDIN: Fd = 0;

    /// 标准输出文件描述符
    pub const STDOUT: Fd = 1;

    /// 标准错误文件描述符
    pub const STDERR: Fd = 2;

    /// 第一个可用的文件描述符
    pub const FIRST_AVAILABLE: Fd = 3;
}

/// 文件打开模式
///
/// 定义文件的打开方式，包括读取、写入和追加等模式。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenMode {
    /// 只读模式
    Read,

    /// 只写模式
    Write,

    /// 读写模式
    ReadWrite,

    /// 追加模式
    Append,
}

/// 文件类型
///
/// 定义文件的类型，包括常规文件、目录等。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    /// 常规文件
    Regular,

    /// 目录
    Directory,

    /// 符号链接
    Symlink,

    /// 未知类型
    Unknown,
}

/// 文件描述符信息
///
/// 包含文件描述符的所有相关信息，如路径、模式、当前位置和文件内容等。
#[derive(Debug, Clone)]
pub struct FileDescriptor {
    /// 文件路径
    pub path: PathBuf,

    /// 打开模式
    pub mode: OpenMode,

    /// 文件类型
    pub file_type: FileType,

    /// 当前读写位置
    pub position: u64,

    /// 文件内容（内存中存储）
    pub content: Arc<Mutex<Cursor<Vec<u8>>>>,

    /// 文件大小
    pub size: u64,
}

impl FileDescriptor {
    /// 创建新的文件描述符
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `mode`: 打开模式
    /// - `content`: 文件初始内容
    ///
    /// # 返回
    /// 返回新创建的文件描述符实例
    pub fn new(path: PathBuf, mode: OpenMode, content: Vec<u8>) -> Self {
        let size = content.len() as u64;
        Self {
            path,
            mode,
            file_type: FileType::Regular,
            position: 0,
            content: Arc::new(Mutex::new(Cursor::new(content))),
            size,
        }
    }

    /// 创建目录类型的文件描述符
    ///
    /// # 参数
    /// - `path`: 目录路径
    ///
    /// # 返回
    /// 返回新创建的目录文件描述符实例
    pub fn new_directory(path: PathBuf) -> Self {
        Self {
            path,
            mode: OpenMode::Read,
            file_type: FileType::Directory,
            position: 0,
            content: Arc::new(Mutex::new(Cursor::new(Vec::new()))),
            size: 0,
        }
    }

    /// 读取文件内容
    ///
    /// # 参数
    /// - `buf`: 用于存储读取数据的缓冲区
    ///
    /// # 返回
    /// 成功时返回实际读取的字节数，失败时返回错误
    pub fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.mode == OpenMode::Write {
            return Err(io::Error::new(io::ErrorKind::PermissionDenied, "File not opened for reading"));
        }

        let mut content =
            self.content.lock().map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to lock file content"))?;
        content.set_position(self.position);
        let bytes_read = content.read(buf)?;
        self.position = content.position();
        Ok(bytes_read)
    }

    /// 写入文件内容
    ///
    /// # 参数
    /// - `buf`: 要写入的数据
    ///
    /// # 返回
    /// 成功时返回实际写入的字节数，失败时返回错误
    pub fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.mode {
            OpenMode::Read => {
                return Err(io::Error::new(io::ErrorKind::PermissionDenied, "File not opened for writing"));
            }
            OpenMode::Append => {
                let mut content =
                    self.content.lock().map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to lock file content"))?;
                content.set_position(self.size);
                let bytes_written = content.write(buf)?;
                self.size += bytes_written as u64;
                self.position = self.size;
                Ok(bytes_written)
            }
            _ => {
                let mut content =
                    self.content.lock().map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to lock file content"))?;
                content.set_position(self.position);
                let bytes_written = content.write(buf)?;
                self.position = content.position();
                if self.position > self.size {
                    self.size = self.position;
                }
                Ok(bytes_written)
            }
        }
    }

    /// 定位文件指针
    ///
    /// # 参数
    /// - `offset`: 偏移量
    /// - `whence`: 定位方式
    ///
    /// # 返回
    /// 成功时返回新的文件位置，失败时返回错误
    pub fn seek(&mut self, offset: i64, whence: u8) -> io::Result<u64> {
        let seek_from = match whence {
            0 => SeekFrom::Start(offset as u64),
            1 => SeekFrom::Current(offset),
            2 => SeekFrom::End(offset),
            _ => return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid whence value")),
        };

        let mut content =
            self.content.lock().map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to lock file content"))?;
        content.seek(seek_from)?;
        self.position = content.position();
        Ok(self.position)
    }
}

/// WASI 错误类型
///
/// 定义 WASI 文件系统操作中可能发生的各种错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WasiError {
    /// 文件未找到
    FileNotFound,

    /// 无效的文件描述符
    BadFd,

    /// 权限被拒绝
    PermissionDenied,

    /// 文件已存在
    FileExists,

    /// 文件系统已满
    NoSpace,

    /// 无效的参数
    InvalidArg,

    /// 不支持的操作
    NotSupported,

    /// IO 错误
    IoError(String),

    /// 文件描述符已用尽
    FdOverflow,

    /// 文件已关闭
    FileClosed,
}

impl std::fmt::Display for WasiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WasiError::FileNotFound => write!(f, "File not found"),
            WasiError::BadFd => write!(f, "Bad file descriptor"),
            WasiError::PermissionDenied => write!(f, "Permission denied"),
            WasiError::FileExists => write!(f, "File already exists"),
            WasiError::NoSpace => write!(f, "No space left on device"),
            WasiError::InvalidArg => write!(f, "Invalid argument"),
            WasiError::NotSupported => write!(f, "Operation not supported"),
            WasiError::IoError(msg) => write!(f, "IO error: {}", msg),
            WasiError::FdOverflow => write!(f, "File descriptor overflow"),
            WasiError::FileClosed => write!(f, "File already closed"),
        }
    }
}

impl std::error::Error for WasiError {}

impl From<io::Error> for WasiError {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            io::ErrorKind::NotFound => WasiError::FileNotFound,
            io::ErrorKind::PermissionDenied => WasiError::PermissionDenied,
            io::ErrorKind::AlreadyExists => WasiError::FileExists,
            io::ErrorKind::InvalidInput => WasiError::InvalidArg,
            _ => WasiError::IoError(err.to_string()),
        }
    }
}

/// WASI 文件系统
///
/// 管理所有文件描述符，提供虚拟文件系统的核心功能。
/// 支持内存中的文件操作，包括文件的创建、打开、读取、写入和删除等。
#[derive(Debug)]
pub struct WasiFs {
    /// 文件描述符映射表
    descriptors: HashMap<Fd, Option<FileDescriptor>>,

    /// 下一个可用的文件描述符
    next_fd: Fd,

    /// 虚拟文件系统中的文件存储
    files: HashMap<PathBuf, Arc<Mutex<Cursor<Vec<u8>>>>>,
}

impl Default for WasiFs {
    fn default() -> Self {
        Self::new()
    }
}

impl WasiFs {
    /// 创建新的 WASI 文件系统实例
    ///
    /// 初始化时会自动创建标准输入、输出和错误文件描述符。
    ///
    /// # 返回
    /// 返回新创建的 WASI 文件系统实例
    pub fn new() -> Self {
        let mut fs = Self { descriptors: HashMap::new(), next_fd: preopen::FIRST_AVAILABLE, files: HashMap::new() };

        fs.init_standard_fds();
        fs
    }

    /// 初始化标准文件描述符
    fn init_standard_fds(&mut self) {
        let stdin = FileDescriptor::new(PathBuf::from("/dev/stdin"), OpenMode::Read, Vec::new());
        let stdout = FileDescriptor::new(PathBuf::from("/dev/stdout"), OpenMode::Write, Vec::new());
        let stderr = FileDescriptor::new(PathBuf::from("/dev/stderr"), OpenMode::Write, Vec::new());

        self.descriptors.insert(preopen::STDIN, Some(stdin));
        self.descriptors.insert(preopen::STDOUT, Some(stdout));
        self.descriptors.insert(preopen::STDERR, Some(stderr));
    }

    /// 分配新的文件描述符
    ///
    /// # 返回
    /// 成功时返回新分配的文件描述符，失败时返回错误
    fn allocate_fd(&mut self) -> Result<Fd, WasiError> {
        let fd = self.next_fd;
        if fd == u32::MAX {
            return Err(WasiError::FdOverflow);
        }
        self.next_fd += 1;
        Ok(fd)
    }

    /// 打开文件
    ///
    /// # 参数
    /// - `path`: 文件路径
    /// - `mode`: 打开模式
    ///
    /// # 返回
    /// 成功时返回文件描述符，失败时返回错误
    pub fn fd_open(&mut self, path: &str, mode: OpenMode) -> Result<Fd, WasiError> {
        let path_buf = PathBuf::from(path);

        let content = if self.files.contains_key(&path_buf) {
            self.files.get(&path_buf).cloned().unwrap()
        }
        else if mode == OpenMode::Read {
            return Err(WasiError::FileNotFound);
        }
        else {
            let content = Arc::new(Mutex::new(Cursor::new(Vec::new())));
            self.files.insert(path_buf.clone(), Arc::clone(&content));
            content
        };

        let content_guard = content.lock().map_err(|_| WasiError::IoError("Failed to lock file content".to_string()))?;
        let size = content_guard.get_ref().len() as u64;
        drop(content_guard);

        let fd = self.allocate_fd()?;
        let descriptor = FileDescriptor {
            path: path_buf,
            mode,
            file_type: FileType::Regular,
            position: if mode == OpenMode::Append { size } else { 0 },
            content,
            size,
        };

        self.descriptors.insert(fd, Some(descriptor));
        Ok(fd)
    }

    /// 读取文件内容
    ///
    /// # 参数
    /// - `fd`: 文件描述符
    /// - `buf`: 用于存储读取数据的缓冲区
    ///
    /// # 返回
    /// 成功时返回实际读取的字节数，失败时返回错误
    pub fn fd_read(&mut self, fd: Fd, buf: &mut [u8]) -> Result<usize, WasiError> {
        let descriptor = self.get_descriptor_mut(fd)?;
        descriptor.read(buf).map_err(WasiError::from)
    }

    /// 写入文件内容
    ///
    /// # 参数
    /// - `fd`: 文件描述符
    /// - `buf`: 要写入的数据
    ///
    /// # 返回
    /// 成功时返回实际写入的字节数，失败时返回错误
    pub fn fd_write(&mut self, fd: Fd, buf: &[u8]) -> Result<usize, WasiError> {
        let (path, content_arc, bytes_written) = {
            let descriptor = self.get_descriptor_mut(fd)?;
            let bytes_written = descriptor.write(buf)?;
            (descriptor.path.clone(), Arc::clone(&descriptor.content), bytes_written)
        };

        self.files.insert(path, content_arc);

        Ok(bytes_written)
    }

    /// 关闭文件描述符
    ///
    /// # 参数
    /// - `fd`: 文件描述符
    ///
    /// # 返回
    /// 成功时返回 Ok(())，失败时返回错误
    pub fn fd_close(&mut self, fd: Fd) -> Result<(), WasiError> {
        if !self.descriptors.contains_key(&fd) {
            return Err(WasiError::BadFd);
        }

        self.descriptors.insert(fd, None);
        Ok(())
    }

    /// 定位文件指针
    ///
    /// # 参数
    /// - `fd`: 文件描述符
    /// - `offset`: 偏移量
    /// - `whence`: 定位方式（0: 从文件开头，1: 从当前位置，2: 从文件末尾）
    ///
    /// # 返回
    /// 成功时返回新的文件位置，失败时返回错误
    pub fn fd_seek(&mut self, fd: Fd, offset: i64, whence: u8) -> Result<u64, WasiError> {
        let descriptor = self.get_descriptor_mut(fd)?;
        descriptor.seek(offset, whence).map_err(WasiError::from)
    }

    /// 获取文件描述符的可变引用
    ///
    /// # 参数
    /// - `fd`: 文件描述符
    ///
    /// # 返回
    /// 成功时返回文件描述符的可变引用，失败时返回错误
    fn get_descriptor_mut(&mut self, fd: Fd) -> Result<&mut FileDescriptor, WasiError> {
        match self.descriptors.get_mut(&fd) {
            Some(Some(descriptor)) => Ok(descriptor),
            Some(None) => Err(WasiError::FileClosed),
            None => Err(WasiError::BadFd),
        }
    }

    /// 获取文件描述符的不可变引用
    ///
    /// # 参数
    /// - `fd`: 文件描述符
    ///
    /// # 返回
    /// 成功时返回文件描述符的不可变引用，失败时返回错误
    pub fn get_descriptor(&self, fd: Fd) -> Result<&FileDescriptor, WasiError> {
        match self.descriptors.get(&fd) {
            Some(Some(descriptor)) => Ok(descriptor),
            Some(None) => Err(WasiError::FileClosed),
            None => Err(WasiError::BadFd),
        }
    }

    /// 创建目录
    ///
    /// # 参数
    /// - `path`: 目录路径
    ///
    /// # 返回
    /// 成功时返回 Ok(())，失败时返回错误
    pub fn mkdir(&mut self, path: &str) -> Result<(), WasiError> {
        let path_buf = PathBuf::from(path);
        if self.files.contains_key(&path_buf) {
            return Err(WasiError::FileExists);
        }

        let fd = self.allocate_fd()?;
        let descriptor = FileDescriptor::new_directory(path_buf.clone());
        self.descriptors.insert(fd, Some(descriptor));

        self.files.insert(path_buf, Arc::new(Mutex::new(Cursor::new(Vec::new()))));
        Ok(())
    }

    /// 删除文件
    ///
    /// # 参数
    /// - `path`: 文件路径
    ///
    /// # 返回
    /// 成功时返回 Ok(())，失败时返回错误
    pub fn unlink(&mut self, path: &str) -> Result<(), WasiError> {
        let path_buf = PathBuf::from(path);
        if self.files.remove(&path_buf).is_none() {
            return Err(WasiError::FileNotFound);
        }
        Ok(())
    }

    /// 检查文件是否存在
    ///
    /// # 参数
    /// - `path`: 文件路径
    ///
    /// # 返回
    /// 如果文件存在返回 true，否则返回 false
    pub fn exists(&self, path: &str) -> bool {
        self.files.contains_key(&PathBuf::from(path))
    }

    /// 获取文件大小
    ///
    /// # 参数
    /// - `fd`: 文件描述符
    ///
    /// # 返回
    /// 成功时返回文件大小，失败时返回错误
    pub fn fd_filestat_get(&self, fd: Fd) -> Result<FileStat, WasiError> {
        let descriptor = self.get_descriptor(fd)?;
        Ok(FileStat { file_type: descriptor.file_type, size: descriptor.size })
    }

    /// 同步文件数据到存储
    ///
    /// # 参数
    /// - `fd`: 文件描述符
    ///
    /// # 返回
    /// 成功时返回 Ok(())，失败时返回错误
    pub fn fd_sync(&mut self, fd: Fd) -> Result<(), WasiError> {
        let _descriptor = self.get_descriptor(fd)?;
        Ok(())
    }

    /// 截断文件到指定大小
    ///
    /// # 参数
    /// - `fd`: 文件描述符
    /// - `size`: 新的文件大小
    ///
    /// # 返回
    /// 成功时返回 Ok(())，失败时返回错误
    pub fn fd_truncate(&mut self, fd: Fd, size: u64) -> Result<(), WasiError> {
        let descriptor = self.get_descriptor_mut(fd)?;
        let mut content =
            descriptor.content.lock().map_err(|_| WasiError::IoError("Failed to lock file content".to_string()))?;

        let current_content = content.get_ref().clone();
        let new_content = if size as usize <= current_content.len() {
            current_content[..size as usize].to_vec()
        }
        else {
            let mut extended = current_content;
            extended.resize(size as usize, 0);
            extended
        };

        *content = Cursor::new(new_content);
        descriptor.size = size;
        if descriptor.position > size {
            descriptor.position = size;
        }

        Ok(())
    }
}

/// 文件状态信息
///
/// 包含文件的元数据信息，如文件类型和大小等。
#[derive(Debug, Clone, Copy)]
pub struct FileStat {
    /// 文件类型
    pub file_type: FileType,

    /// 文件大小（字节）
    pub size: u64,
}
