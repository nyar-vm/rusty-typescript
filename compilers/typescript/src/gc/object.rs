//! GC 对象模块
//!
//! 定义垃圾收集器管理的对象结构。

use typescript_types::TsValue;

use super::header::ObjectHeader;

/// GC 对象包装
///
/// 表示堆上分配的对象，包含对象头和实际值。
#[repr(C)]
pub struct GcObject {
    /// 对象头
    pub header: ObjectHeader,
    /// 对象值
    pub value: TsValue,
}

// 实现 Send trait，因为 GcObject 可以安全地在线程之间发送
unsafe impl Send for GcObject {}

// 实现 Sync trait，因为 GcObject 可以安全地在线程之间共享
unsafe impl Sync for GcObject {}

impl GcObject {
    /// 创建新的 GC 对象
    ///
    /// # 参数
    /// - `value`: TypeScript 值
    /// - `size`: 对象大小（字节）
    /// - `type_id`: 类型标识
    ///
    /// # 返回值
    /// 返回新创建的 GC 对象实例
    pub fn new(value: TsValue, size: usize, type_id: u8) -> Self {
        Self { header: ObjectHeader::new(size, type_id), value }
    }

    /// 获取对象大小（包括头部）
    ///
    /// # 返回值
    /// 返回对象的总大小（字节）
    pub fn total_size(&self) -> usize {
        std::mem::size_of::<ObjectHeader>() + self.header.size
    }

    /// 获取对象数据大小
    ///
    /// # 返回值
    /// 返回对象数据部分的大小（字节）
    pub fn data_size(&self) -> usize {
        self.header.size
    }

    /// 检查对象是否已标记
    ///
    /// # 返回值
    /// 如果对象已标记返回 true，否则返回 false
    pub fn is_marked(&self) -> bool {
        self.header.is_marked()
    }

    /// 标记对象
    pub fn mark(&mut self) {
        self.header.set_mark();
    }

    /// 取消标记对象
    pub fn unmark(&mut self) {
        self.header.clear_mark();
    }

    /// 获取对象年龄
    ///
    /// # 返回值
    /// 返回对象的年龄值
    pub fn age(&self) -> u8 {
        self.header.get_age()
    }

    /// 增加对象年龄
    pub fn increment_age(&mut self) {
        self.header.increment_age();
    }

    /// 检查对象是否为老年代对象
    ///
    /// # 返回值
    /// 如果对象是老年代对象返回 true，否则返回 false
    pub fn is_tenured(&self) -> bool {
        self.header.is_tenured()
    }

    /// 获取对象类型标识
    ///
    /// # 返回值
    /// 返回对象的类型标识
    pub fn type_id(&self) -> u8 {
        self.header.type_id
    }

    /// 获取引用计数
    ///
    /// # 返回值
    /// 返回对象的引用计数
    pub fn ref_count(&self) -> usize {
        self.header.ref_count
    }

    /// 增加引用计数
    pub fn increment_ref_count(&mut self) {
        self.header.increment_ref_count();
    }

    /// 减少引用计数
    ///
    /// # 返回值
    /// 返回减少后的引用计数
    pub fn decrement_ref_count(&mut self) -> usize {
        self.header.decrement_ref_count()
    }
}
