//! 对象头模块
//!
//! 定义堆分配对象的元数据头部结构。

/// 对象头
///
/// 每个堆分配对象的前缀，包含垃圾收集所需的元数据。
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
    /// 访问计数（用于基于访问模式的晋升策略）
    pub access_count: u32,
}

impl ObjectHeader {
    /// 创建新的对象头
    ///
    /// # 参数
    /// - `size`: 对象大小（字节）
    /// - `type_id`: 对象类型标识
    ///
    /// # 返回值
    /// 返回新创建的对象头实例
    pub fn new(size: usize, type_id: u8) -> Self {
        Self { mark: false, age: 0, size, ref_count: 0, type_id, access_count: 0 }
    }

    /// 增加引用计数
    pub fn increment_ref_count(&mut self) {
        self.ref_count += 1;
    }

    /// 减少引用计数
    ///
    /// # 返回值
    /// 返回减少后的引用计数
    pub fn decrement_ref_count(&mut self) -> usize {
        if self.ref_count > 0 {
            self.ref_count -= 1;
        }
        self.ref_count
    }

    /// 增加年龄
    ///
    /// 对象年龄用于分代收集中的晋升决策。
    /// 年龄最大为 255（u8::MAX）。
    pub fn increment_age(&mut self) {
        if self.age < u8::MAX {
            self.age += 1;
        }
    }

    /// 检查是否为老年代对象
    ///
    /// # 返回值
    /// 如果对象年龄达到阈值（3），返回 true，否则返回 false
    pub fn is_tenured(&self) -> bool {
        self.age >= 3
    }

    /// 重置标记状态
    pub fn clear_mark(&mut self) {
        self.mark = false;
    }

    /// 设置标记状态
    pub fn set_mark(&mut self) {
        self.mark = true;
    }

    /// 检查是否已标记
    ///
    /// # 返回值
    /// 如果对象已被标记返回 true，否则返回 false
    pub fn is_marked(&self) -> bool {
        self.mark
    }

    /// 获取对象年龄
    ///
    /// # 返回值
    /// 返回对象的年龄值
    pub fn get_age(&self) -> u8 {
        self.age
    }

    /// 重置对象年龄
    pub fn reset_age(&mut self) {
        self.age = 0;
    }

    /// 增加访问计数
    ///
    /// 用于基于访问模式的晋升策略，记录对象被访问的次数。
    pub fn increment_access_count(&mut self) {
        if self.access_count < u32::MAX {
            self.access_count += 1;
        }
    }

    /// 重置访问计数
    ///
    /// 在对象被晋升或垃圾收集时重置访问计数。
    pub fn reset_access_count(&mut self) {
        self.access_count = 0;
    }

    /// 获取访问计数
    ///
    /// # 返回值
    /// 返回对象的访问计数
    pub fn get_access_count(&self) -> u32 {
        self.access_count
    }
}
