//! GC 对象模块

use typescript_types::TsValue;

use super::header::ObjectHeader;

/// GC 对象包装
#[repr(C)]
pub struct GcObject {
    /// 对象头
    pub header: ObjectHeader,
    /// 对象值
    pub value: TsValue,
}

impl GcObject {
    /// 创建新的 GC 对象
    pub fn new(value: TsValue, size: usize, type_id: u8) -> Self {
        Self { header: ObjectHeader::new(size, type_id), value }
    }

    /// 获取对象大小（包括头部）
    pub fn total_size(&self) -> usize {
        std::mem::size_of::<ObjectHeader>() + self.header.size
    }
}
