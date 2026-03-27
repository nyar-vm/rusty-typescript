//! WASI 网络接口基础实现
//!
//! 该模块提供了 WASI 网络接口的虚拟实现，用于模拟网络操作。
//! 所有网络操作都在虚拟环境中进行，不会影响真实网络。

use std::{
    collections::HashMap,
    io::{self, Read, Write},
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    sync::{Arc, Mutex},
    time::Duration,
};

/// 套接字类型枚举
///
/// 定义支持的套接字类型，包括 TCP 和 UDP。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SocketType {
    /// TCP 套接字，提供可靠的面向连接的服务
    Tcp,
    /// UDP 套接字，提供无连接的数据报服务
    Udp,
}

/// 套接字状态枚举
///
/// 定义套接字在生命周期中可能处于的各种状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SocketState {
    /// 已创建，但未绑定地址
    Created,
    /// 已绑定到本地地址
    Bound,
    /// 正在监听连接请求（仅 TCP）
    Listening,
    /// 已连接到远程地址（仅 TCP）
    Connected,
    /// 已关闭
    Closed,
}

/// 套接字地址结构体
///
/// 表示网络套接字的地址，包含 IP 地址和端口号。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SocketAddress {
    /// IP 地址，可以是 IPv4 或 IPv6
    pub ip: IpAddr,
    /// 端口号，范围 0-65535
    pub port: u16,
}

impl SocketAddress {
    /// 创建新的套接字地址
    ///
    /// # 参数
    /// - `ip`: IP 地址
    /// - `port`: 端口号
    ///
    /// # 返回
    /// 新的 SocketAddress 实例
    pub fn new(ip: IpAddr, port: u16) -> Self {
        Self { ip, port }
    }

    /// 创建 IPv4 套接字地址
    ///
    /// # 参数
    /// - `a`, `b`, `c`, `d`: IPv4 地址的四个字节
    /// - `port`: 端口号
    ///
    /// # 返回
    /// 新的 SocketAddress 实例
    pub fn ipv4(a: u8, b: u8, c: u8, d: u8, port: u16) -> Self {
        Self { ip: IpAddr::V4(Ipv4Addr::new(a, b, c, d)), port }
    }

    /// 创建 IPv6 套接字地址
    ///
    /// # 参数
    /// - `segments`: IPv6 地址的 8 个 16 位段
    /// - `port`: 端口号
    ///
    /// # 返回
    /// 新的 SocketAddress 实例
    pub fn ipv6(segments: [u16; 8], port: u16) -> Self {
        Self {
            ip: IpAddr::V6(Ipv6Addr::new(
                segments[0],
                segments[1],
                segments[2],
                segments[3],
                segments[4],
                segments[5],
                segments[6],
                segments[7],
            )),
            port,
        }
    }

    /// 创建本地回环地址
    ///
    /// # 参数
    /// - `port`: 端口号
    ///
    /// # 返回
    /// 指向本地回环地址的 SocketAddress 实例
    pub fn localhost(port: u16) -> Self {
        Self { ip: IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port }
    }

    /// 创建任意地址（0.0.0.0）
    ///
    /// # 参数
    /// - `port`: 端口号
    ///
    /// # 返回
    /// 绑定到所有网络接口的 SocketAddress 实例
    pub fn any(port: u16) -> Self {
        Self { ip: IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port }
    }
}

impl std::fmt::Display for SocketAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.ip {
            IpAddr::V4(ip) => write!(f, "{}:{}", ip, self.port),
            IpAddr::V6(ip) => write!(f, "[{}]:{}", ip, self.port),
        }
    }
}

/// 虚拟连接数据
///
/// 存储虚拟网络连接中的数据缓冲区。
#[derive(Debug, Clone)]
struct VirtualConnection {
    /// 远程地址
    remote_addr: SocketAddress,
    /// 接收缓冲区
    recv_buffer: Vec<u8>,
    /// 发送缓冲区
    send_buffer: Vec<u8>,
}

/// 套接字结构体
///
/// 表示一个网络套接字，包含类型、状态、地址等信息。
#[derive(Debug)]
pub struct Socket {
    /// 套接字类型
    pub socket_type: SocketType,
    /// 套接字当前状态
    pub state: SocketState,
    /// 本地绑定的地址
    pub local_addr: Option<SocketAddress>,
    /// 远程连接的地址
    pub remote_addr: Option<SocketAddress>,
    /// 接收缓冲区
    recv_buffer: Vec<u8>,
    /// 发送缓冲区
    send_buffer: Vec<u8>,
    /// 待接受的连接队列（仅用于 TCP 监听套接字）
    pending_connections: Vec<VirtualConnection>,
    /// 接收超时时间
    recv_timeout: Option<Duration>,
    /// 发送超时时间
    send_timeout: Option<Duration>,
    /// 是否启用广播（仅用于 UDP）
    broadcast: bool,
    /// 是否启用地址重用
    reuse_addr: bool,
}

impl Socket {
    /// 创建新的套接字
    ///
    /// # 参数
    /// - `socket_type`: 套接字类型
    ///
    /// # 返回
    /// 新的 Socket 实例，初始状态为 Created
    pub fn new(socket_type: SocketType) -> Self {
        Self {
            socket_type,
            state: SocketState::Created,
            local_addr: None,
            remote_addr: None,
            recv_buffer: Vec::new(),
            send_buffer: Vec::new(),
            pending_connections: Vec::new(),
            recv_timeout: None,
            send_timeout: None,
            broadcast: false,
            reuse_addr: false,
        }
    }

    /// 绑定到本地地址
    ///
    /// # 参数
    /// - `addr`: 要绑定的本地地址
    ///
    /// # 返回
    /// 成功返回 Ok(())，失败返回错误
    pub fn bind(&mut self, addr: SocketAddress) -> io::Result<()> {
        if self.state != SocketState::Created {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Socket already bound or in invalid state"));
        }
        self.local_addr = Some(addr);
        self.state = SocketState::Bound;
        Ok(())
    }

    /// 开始监听连接
    ///
    /// # 参数
    /// - `backlog`: 待处理连接队列的最大长度
    ///
    /// # 返回
    /// 成功返回 Ok(())，失败返回错误
    pub fn listen(&mut self, backlog: usize) -> io::Result<()> {
        if self.socket_type != SocketType::Tcp {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Only TCP sockets can listen"));
        }
        if self.state != SocketState::Bound {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Socket must be bound before listening"));
        }
        self.pending_connections = Vec::with_capacity(backlog);
        self.state = SocketState::Listening;
        Ok(())
    }

    /// 接受新连接
    ///
    /// # 返回
    /// 成功返回新连接的套接字，失败返回错误
    pub fn accept(&mut self) -> io::Result<Socket> {
        if self.state != SocketState::Listening {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Socket is not listening"));
        }
        if self.pending_connections.is_empty() {
            return Err(io::Error::new(io::ErrorKind::WouldBlock, "No pending connections"));
        }
        let conn = self.pending_connections.remove(0);
        let mut new_socket = Socket::new(SocketType::Tcp);
        new_socket.local_addr = self.local_addr.clone();
        new_socket.remote_addr = Some(conn.remote_addr);
        new_socket.recv_buffer = conn.recv_buffer;
        new_socket.state = SocketState::Connected;
        Ok(new_socket)
    }

    /// 连接到远程地址
    ///
    /// # 参数
    /// - `addr`: 远程地址
    ///
    /// # 返回
    /// 成功返回 Ok(())，失败返回错误
    pub fn connect(&mut self, addr: SocketAddress) -> io::Result<()> {
        if self.socket_type != SocketType::Tcp {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Only TCP sockets can connect"));
        }
        if self.state == SocketState::Closed {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Socket is closed"));
        }
        self.remote_addr = Some(addr);
        self.state = SocketState::Connected;
        Ok(())
    }

    /// 发送数据
    ///
    /// # 参数
    /// - `data`: 要发送的数据
    ///
    /// # 返回
    /// 成功返回发送的字节数，失败返回错误
    pub fn send(&mut self, data: &[u8]) -> io::Result<usize> {
        if self.state == SocketState::Closed {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "Socket is closed"));
        }
        if self.socket_type == SocketType::Tcp && self.state != SocketState::Connected {
            return Err(io::Error::new(io::ErrorKind::NotConnected, "TCP socket not connected"));
        }
        self.send_buffer.extend_from_slice(data);
        Ok(data.len())
    }

    /// 发送数据到指定地址（用于 UDP）
    ///
    /// # 参数
    /// - `data`: 要发送的数据
    /// - `addr`: 目标地址
    ///
    /// # 返回
    /// 成功返回发送的字节数，失败返回错误
    pub fn send_to(&mut self, data: &[u8], _addr: SocketAddress) -> io::Result<usize> {
        if self.state == SocketState::Closed {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "Socket is closed"));
        }
        self.send_buffer.extend_from_slice(data);
        Ok(data.len())
    }

    /// 接收数据
    ///
    /// # 参数
    /// - `buf`: 接收数据的缓冲区
    ///
    /// # 返回
    /// 成功返回接收的字节数，失败返回错误
    pub fn recv(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.state == SocketState::Closed {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "Socket is closed"));
        }
        if self.recv_buffer.is_empty() {
            return Err(io::Error::new(io::ErrorKind::WouldBlock, "No data available"));
        }
        let len = std::cmp::min(buf.len(), self.recv_buffer.len());
        buf[..len].copy_from_slice(&self.recv_buffer[..len]);
        self.recv_buffer.drain(..len);
        Ok(len)
    }

    /// 接收数据并返回发送者地址（用于 UDP）
    ///
    /// # 参数
    /// - `buf`: 接收数据的缓冲区
    ///
    /// # 返回
    /// 成功返回 (字节数, 发送者地址)，失败返回错误
    pub fn recv_from(&mut self, buf: &mut [u8]) -> io::Result<(usize, SocketAddress)> {
        if self.state == SocketState::Closed {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "Socket is closed"));
        }
        if self.recv_buffer.is_empty() {
            return Err(io::Error::new(io::ErrorKind::WouldBlock, "No data available"));
        }
        let len = std::cmp::min(buf.len(), self.recv_buffer.len());
        buf[..len].copy_from_slice(&self.recv_buffer[..len]);
        self.recv_buffer.drain(..len);
        let addr = self.remote_addr.clone().unwrap_or_else(|| SocketAddress::any(0));
        Ok((len, addr))
    }

    /// 关闭套接字
    ///
    /// # 返回
    /// 成功返回 Ok(())，失败返回错误
    pub fn close(&mut self) -> io::Result<()> {
        self.state = SocketState::Closed;
        self.recv_buffer.clear();
        self.send_buffer.clear();
        self.pending_connections.clear();
        Ok(())
    }

    /// 设置接收超时时间
    ///
    /// # 参数
    /// - `timeout`: 超时时间
    pub fn set_recv_timeout(&mut self, timeout: Option<Duration>) {
        self.recv_timeout = timeout;
    }

    /// 设置发送超时时间
    ///
    /// # 参数
    /// - `timeout`: 超时时间
    pub fn set_send_timeout(&mut self, timeout: Option<Duration>) {
        self.send_timeout = timeout;
    }

    /// 设置是否启用广播
    ///
    /// # 参数
    /// - `broadcast`: 是否启用广播
    pub fn set_broadcast(&mut self, broadcast: bool) {
        self.broadcast = broadcast;
    }

    /// 设置是否启用地址重用
    ///
    /// # 参数
    /// - `reuse`: 是否启用地址重用
    pub fn set_reuse_addr(&mut self, reuse: bool) {
        self.reuse_addr = reuse;
    }

    /// 向接收缓冲区添加数据（用于虚拟网络模拟）
    ///
    /// # 参数
    /// - `data`: 要添加的数据
    /// - `from`: 数据来源地址
    pub fn inject_data(&mut self, data: &[u8], from: SocketAddress) {
        self.recv_buffer.extend_from_slice(data);
        if self.remote_addr.is_none() {
            self.remote_addr = Some(from);
        }
    }

    /// 向待处理连接队列添加虚拟连接（用于虚拟网络模拟）
    ///
    /// # 参数
    /// - `remote_addr`: 远程地址
    pub fn inject_connection(&mut self, remote_addr: SocketAddress) {
        if self.state == SocketState::Listening {
            self.pending_connections.push(VirtualConnection { remote_addr, recv_buffer: Vec::new(), send_buffer: Vec::new() });
        }
    }
}

impl Read for Socket {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.recv(buf)
    }
}

impl Write for Socket {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.send(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// 套接字描述符类型
type SocketFd = u32;

/// WASI 网络管理器
///
/// 管理所有虚拟套接字，提供网络操作的统一接口。
#[derive(Debug, Default)]
pub struct WasiNet {
    /// 套接字描述符计数器
    next_fd: SocketFd,
    /// 套接字映射表
    sockets: HashMap<SocketFd, Socket>,
    /// 地址到套接字描述符的映射
    addr_to_fd: HashMap<SocketAddress, SocketFd>,
}

impl WasiNet {
    /// 创建新的 WASI 网络管理器
    ///
    /// # 返回
    /// 新的 WasiNet 实例
    pub fn new() -> Self {
        Self { next_fd: 1, sockets: HashMap::new(), addr_to_fd: HashMap::new() }
    }

    /// 分配新的套接字描述符
    fn allocate_fd(&mut self) -> SocketFd {
        let fd = self.next_fd;
        self.next_fd += 1;
        fd
    }

    /// 获取套接字引用
    ///
    /// # 参数
    /// - `fd`: 套接字描述符
    ///
    /// # 返回
    /// 成功返回套接字引用，失败返回错误
    pub fn get_socket(&self, fd: SocketFd) -> io::Result<&Socket> {
        self.sockets.get(&fd).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid socket descriptor"))
    }

    /// 获取套接字可变引用
    ///
    /// # 参数
    /// - `fd`: 套接字描述符
    ///
    /// # 返回
    /// 成功返回套接字可变引用，失败返回错误
    pub fn get_socket_mut(&mut self, fd: SocketFd) -> io::Result<&mut Socket> {
        self.sockets.get_mut(&fd).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid socket descriptor"))
    }

    /// 检查套接字是否存在
    ///
    /// # 参数
    /// - `fd`: 套接字描述符
    ///
    /// # 返回
    /// 如果套接字存在返回 true，否则返回 false
    pub fn socket_exists(&self, fd: SocketFd) -> bool {
        self.sockets.contains_key(&fd)
    }

    /// 获取所有活动套接字描述符
    ///
    /// # 返回
    /// 套接字描述符列表
    pub fn get_all_fds(&self) -> Vec<SocketFd> {
        self.sockets.keys().copied().collect()
    }

    /// 获取套接字数量
    ///
    /// # 返回
    /// 当前管理的套接字数量
    pub fn socket_count(&self) -> usize {
        self.sockets.len()
    }

    /// 向指定套接字注入数据（用于虚拟网络模拟）
    ///
    /// # 参数
    /// - `fd`: 目标套接字描述符
    /// - `data`: 要注入的数据
    /// - `from`: 数据来源地址
    ///
    /// # 返回
    /// 成功返回 Ok(())，失败返回错误
    pub fn inject_data_to_socket(&mut self, fd: SocketFd, data: &[u8], from: SocketAddress) -> io::Result<()> {
        let socket = self.get_socket_mut(fd)?;
        socket.inject_data(data, from);
        Ok(())
    }

    /// 向监听套接字注入虚拟连接（用于虚拟网络模拟）
    ///
    /// # 参数
    /// - `fd`: 目标套接字描述符
    /// - `remote_addr`: 远程地址
    ///
    /// # 返回
    /// 成功返回 Ok(())，失败返回错误
    pub fn inject_connection_to_socket(&mut self, fd: SocketFd, remote_addr: SocketAddress) -> io::Result<()> {
        let socket = self.get_socket_mut(fd)?;
        socket.inject_connection(remote_addr);
        Ok(())
    }
}

/// 创建套接字
///
/// # 参数
/// - `net`: WASI 网络管理器可变引用
/// - `socket_type`: 套接字类型
///
/// # 返回
/// 成功返回套接字描述符，失败返回错误
pub fn sock_open(net: &mut WasiNet, socket_type: SocketType) -> io::Result<SocketFd> {
    let fd = net.allocate_fd();
    let socket = Socket::new(socket_type);
    net.sockets.insert(fd, socket);
    Ok(fd)
}

/// 绑定套接字到地址
///
/// # 参数
/// - `net`: WASI 网络管理器可变引用
/// - `fd`: 套接字描述符
/// - `addr`: 要绑定的地址
///
/// # 返回
/// 成功返回 Ok(())，失败返回错误
pub fn sock_bind(net: &mut WasiNet, fd: SocketFd, addr: SocketAddress) -> io::Result<()> {
    if net.addr_to_fd.contains_key(&addr) {
        let existing_socket = net.get_socket(fd)?;
        if !existing_socket.reuse_addr {
            return Err(io::Error::new(io::ErrorKind::AddrInUse, "Address already in use"));
        }
    }
    let socket = net.get_socket_mut(fd)?;
    socket.bind(addr.clone())?;
    net.addr_to_fd.insert(addr, fd);
    Ok(())
}

/// 开始监听连接
///
/// # 参数
/// - `net`: WASI 网络管理器可变引用
/// - `fd`: 套接字描述符
/// - `backlog`: 待处理连接队列的最大长度
///
/// # 返回
/// 成功返回 Ok(())，失败返回错误
pub fn sock_listen(net: &mut WasiNet, fd: SocketFd, backlog: usize) -> io::Result<()> {
    let socket = net.get_socket_mut(fd)?;
    socket.listen(backlog)
}

/// 接受新连接
///
/// # 参数
/// - `net`: WASI 网络管理器可变引用
/// - `fd`: 监听套接字描述符
///
/// # 返回
/// 成功返回新连接的套接字描述符，失败返回错误
pub fn sock_accept(net: &mut WasiNet, fd: SocketFd) -> io::Result<SocketFd> {
    let new_socket = {
        let socket = net.get_socket_mut(fd)?;
        socket.accept()?
    };
    let new_fd = net.allocate_fd();
    net.sockets.insert(new_fd, new_socket);
    Ok(new_fd)
}

/// 连接到远程地址
///
/// # 参数
/// - `net`: WASI 网络管理器可变引用
/// - `fd`: 套接字描述符
/// - `addr`: 远程地址
///
/// # 返回
/// 成功返回 Ok(())，失败返回错误
pub fn sock_connect(net: &mut WasiNet, fd: SocketFd, addr: SocketAddress) -> io::Result<()> {
    let socket = net.get_socket_mut(fd)?;
    socket.connect(addr)
}

/// 发送数据
///
/// # 参数
/// - `net`: WASI 网络管理器可变引用
/// - `fd`: 套接字描述符
/// - `data`: 要发送的数据
///
/// # 返回
/// 成功返回发送的字节数，失败返回错误
pub fn sock_send(net: &mut WasiNet, fd: SocketFd, data: &[u8]) -> io::Result<usize> {
    let socket = net.get_socket_mut(fd)?;
    socket.send(data)
}

/// 发送数据到指定地址
///
/// # 参数
/// - `net`: WASI 网络管理器可变引用
/// - `fd`: 套接字描述符
/// - `data`: 要发送的数据
/// - `addr`: 目标地址
///
/// # 返回
/// 成功返回发送的字节数，失败返回错误
pub fn sock_send_to(net: &mut WasiNet, fd: SocketFd, data: &[u8], addr: SocketAddress) -> io::Result<usize> {
    let socket = net.get_socket_mut(fd)?;
    socket.send_to(data, addr)
}

/// 接收数据
///
/// # 参数
/// - `net`: WASI 网络管理器可变引用
/// - `fd`: 套接字描述符
/// - `buf`: 接收数据的缓冲区
///
/// # 返回
/// 成功返回接收的字节数，失败返回错误
pub fn sock_recv(net: &mut WasiNet, fd: SocketFd, buf: &mut [u8]) -> io::Result<usize> {
    let socket = net.get_socket_mut(fd)?;
    socket.recv(buf)
}

/// 接收数据并返回发送者地址
///
/// # 参数
/// - `net`: WASI 网络管理器可变引用
/// - `fd`: 套接字描述符
/// - `buf`: 接收数据的缓冲区
///
/// # 返回
/// 成功返回 (字节数, 发送者地址)，失败返回错误
pub fn sock_recv_from(net: &mut WasiNet, fd: SocketFd, buf: &mut [u8]) -> io::Result<(usize, SocketAddress)> {
    let socket = net.get_socket_mut(fd)?;
    socket.recv_from(buf)
}

/// 关闭套接字
///
/// # 参数
/// - `net`: WASI 网络管理器可变引用
/// - `fd`: 套接字描述符
///
/// # 返回
/// 成功返回 Ok(())，失败返回错误
pub fn sock_close(net: &mut WasiNet, fd: SocketFd) -> io::Result<()> {
    let socket = net.sockets.get(&fd);
    if let Some(sock) = socket {
        if let Some(addr) = &sock.local_addr {
            net.addr_to_fd.remove(addr);
        }
    }
    let socket = net.get_socket_mut(fd)?;
    socket.close()?;
    net.sockets.remove(&fd);
    Ok(())
}

/// 线程安全的 WASI 网络管理器
///
/// 提供线程安全的网络操作接口，使用 Arc<Mutex> 包装。
#[derive(Debug, Clone)]
pub struct ThreadSafeWasiNet {
    /// 内部网络管理器
    inner: Arc<Mutex<WasiNet>>,
}

impl ThreadSafeWasiNet {
    /// 创建新的线程安全 WASI 网络管理器
    ///
    /// # 返回
    /// 新的 ThreadSafeWasiNet 实例
    pub fn new() -> Self {
        Self { inner: Arc::new(Mutex::new(WasiNet::new())) }
    }

    /// 创建套接字
    ///
    /// # 参数
    /// - `socket_type`: 套接字类型
    ///
    /// # 返回
    /// 成功返回套接字描述符，失败返回错误
    pub fn sock_open(&self, socket_type: SocketType) -> io::Result<SocketFd> {
        let mut net = self.inner.lock().map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to lock network manager"))?;
        sock_open(&mut net, socket_type)
    }

    /// 绑定套接字到地址
    ///
    /// # 参数
    /// - `fd`: 套接字描述符
    /// - `addr`: 要绑定的地址
    ///
    /// # 返回
    /// 成功返回 Ok(())，失败返回错误
    pub fn sock_bind(&self, fd: SocketFd, addr: SocketAddress) -> io::Result<()> {
        let mut net = self.inner.lock().map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to lock network manager"))?;
        sock_bind(&mut net, fd, addr)
    }

    /// 开始监听连接
    ///
    /// # 参数
    /// - `fd`: 套接字描述符
    /// - `backlog`: 待处理连接队列的最大长度
    ///
    /// # 返回
    /// 成功返回 Ok(())，失败返回错误
    pub fn sock_listen(&self, fd: SocketFd, backlog: usize) -> io::Result<()> {
        let mut net = self.inner.lock().map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to lock network manager"))?;
        sock_listen(&mut net, fd, backlog)
    }

    /// 接受新连接
    ///
    /// # 参数
    /// - `fd`: 监听套接字描述符
    ///
    /// # 返回
    /// 成功返回新连接的套接字描述符，失败返回错误
    pub fn sock_accept(&self, fd: SocketFd) -> io::Result<SocketFd> {
        let mut net = self.inner.lock().map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to lock network manager"))?;
        sock_accept(&mut net, fd)
    }

    /// 连接到远程地址
    ///
    /// # 参数
    /// - `fd`: 套接字描述符
    /// - `addr`: 远程地址
    ///
    /// # 返回
    /// 成功返回 Ok(())，失败返回错误
    pub fn sock_connect(&self, fd: SocketFd, addr: SocketAddress) -> io::Result<()> {
        let mut net = self.inner.lock().map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to lock network manager"))?;
        sock_connect(&mut net, fd, addr)
    }

    /// 发送数据
    ///
    /// # 参数
    /// - `fd`: 套接字描述符
    /// - `data`: 要发送的数据
    ///
    /// # 返回
    /// 成功返回发送的字节数，失败返回错误
    pub fn sock_send(&self, fd: SocketFd, data: &[u8]) -> io::Result<usize> {
        let mut net = self.inner.lock().map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to lock network manager"))?;
        sock_send(&mut net, fd, data)
    }

    /// 接收数据
    ///
    /// # 参数
    /// - `fd`: 套接字描述符
    /// - `buf`: 接收数据的缓冲区
    ///
    /// # 返回
    /// 成功返回接收的字节数，失败返回错误
    pub fn sock_recv(&self, fd: SocketFd, buf: &mut [u8]) -> io::Result<usize> {
        let mut net = self.inner.lock().map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to lock network manager"))?;
        sock_recv(&mut net, fd, buf)
    }

    /// 关闭套接字
    ///
    /// # 参数
    /// - `fd`: 套接字描述符
    ///
    /// # 返回
    /// 成功返回 Ok(())，失败返回错误
    pub fn sock_close(&self, fd: SocketFd) -> io::Result<()> {
        let mut net = self.inner.lock().map_err(|_| io::Error::new(io::ErrorKind::Other, "Failed to lock network manager"))?;
        sock_close(&mut net, fd)
    }
}

impl Default for ThreadSafeWasiNet {
    fn default() -> Self {
        Self::new()
    }
}
