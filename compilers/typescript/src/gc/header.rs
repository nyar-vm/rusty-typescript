//! 对象头模块

/// 对象头
///
/// 每个堆分配对象的前缀，包含垃圾收集所需的元数据
#[derive(Debug, Clone, Copy)]
pub struct ObjectHeader {
    /// 对象标记（用于标记-清除算法）
    pub mark: bool,
    /// 对象年龄（用于分代收集）
    pub age: u8,
    /// 对象大小
    pub size: usize,
    /// 引用计数
    pub ref_count: usize,
    /// 对象类型标识
    pub type_id: u8,
}

impl ObjectHeader {
    /// 创建新的对象头
    pub fn new(size: usize, type_id: u8) -> Self {
        Self { mark: false, age: 0, size, ref_count: 0, type_id }
    }

    /// 增加引用计数
    pub fn increment_ref_count(&mut self) {
        self.ref_count += 1;
    }

    /// 减少引用计数
    pub fn decrement_ref_count(&mut self) -> usize {
        if self.ref_count > 0 {
            self.ref_count -= 1;
        }
        self.ref_count
    }

    /// 增加年龄
    pub fn increment_age(&mut self) {
        if self.age < u8::MAX {
            self.age += 1;
        }
    }

    /// 检查是否为老年代对象
    pub fn is_tenured(&self) -> bool {
        self.age >= 3
    }
}
