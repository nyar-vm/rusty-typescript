//! TypeScript 测试套件
//!
//! 本测试套件按功能模块组织，包含解析器、运行时、性能、内存管理、虚拟机、FFI、平台和 WebIDL 等测试。

mod ffi;
mod memory;
mod parser;
mod performance;
mod platform;
mod runtime;
mod vm;
mod webidl;
