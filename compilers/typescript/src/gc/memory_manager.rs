//! 内存管理器模块
//!
//! 整合内存分配和垃圾收集功能，提供统一的内存管理接口。

use std::{ptr::NonNull, time::Instant};

use typescript_types::TsValue;

use super::{collector::GC, object::GcObject, stats::GCStats};

/// 内存管理器
///
/// 整合内存分配和垃圾收集功能，提供统一的内存管理接口。
/// 负责自动触发垃圾回收，并跟踪内存使用情况。
pub struct MemoryManager {
    /// 垃圾收集器
    gc: GC,
    /// 分配的对象计数
    object_count: usize,
    /// 触发垃圾回收的阈值
    threshold: usize,
    /// 总分配字节数
    total_allocated: usize,
    /// 上次垃圾回收时间
    last_collection_time: Instant,
}

impl MemoryManager {
    /// 创建一个新的内存管理器
    ///
    /// # 参数
    /// - `threshold`: 触发垃圾回收的对象数量阈值
    ///
    /// # 返回值
    /// 返回新创建的内存管理器实例
    pub fn new(threshold: usize) -> Self {
        Self { gc: GC::new(), object_count: 0, threshold, total_allocated: 0, last_collection_time: Instant::now() }
    }

    /// 分配对象
    ///
    /// # 参数
    /// - `value`: TypeScript 值
    /// - `size`: 对象大小（字节）
    /// - `type_id`: 类型标识
    ///
    /// # 返回值
    /// 返回分配的对象指针
    pub fn allocate(&mut self, value: TsValue, size: usize, type_id: u8) -> NonNull<GcObject> {
        self.object_count += 1;
        self.total_allocated += size;

        let ptr = self.gc.allocate(value, size, type_id);

        if self.object_count >= self.threshold
            || self.gc.should_collect()
            || self.last_collection_time.elapsed().as_millis() > 1000
        {
            self.gc.collect_young_gen();
            self.object_count = 0;
            self.last_collection_time = Instant::now();
        }

        ptr
    }

    /// 添加根对象
    ///
    /// # 参数
    /// - `obj`: 要添加的根对象指针
    pub fn add_root(&mut self, obj: NonNull<GcObject>) {
        self.gc.add_root(obj);
    }

    /// 移除根对象
    ///
    /// # 参数
    /// - `obj`: 要移除的根对象指针
    pub fn remove_root(&mut self, obj: NonNull<GcObject>) {
        self.gc.remove_root(obj);
    }

    /// 手动触发垃圾回收
    pub fn collect(&mut self) {
        self.gc.collect();
        self.object_count = 0;
        self.last_collection_time = Instant::now();
    }

    /// 执行增量垃圾回收
    ///
    /// # 返回值
    /// 如果整个 GC 周期完成返回 true，否则返回 false
    pub fn collect_incremental(&mut self) -> bool {
        let result = self.gc.collect_incremental();
        if result {
            self.object_count = 0;
            self.last_collection_time = Instant::now();
        }
        result
    }

    /// 记录跨代引用
    ///
    /// # 参数
    /// - `old_obj`: 老年代对象指针
    /// - `young_obj`: 新生代对象指针
    pub fn record_cross_generation_reference(&mut self, old_obj: NonNull<GcObject>, young_obj: NonNull<GcObject>) {
        self.gc.record_cross_generation_reference(old_obj, young_obj);
    }

    /// 获取垃圾收集器引用
    ///
    /// # 返回值
    /// 返回垃圾收集器的不可变引用
    pub fn gc(&self) -> &GC {
        &self.gc
    }

    /// 获取垃圾收集器可变引用
    ///
    /// # 返回值
    /// 返回垃圾收集器的可变引用
    pub fn gc_mut(&mut self) -> &mut GC {
        &mut self.gc
    }

    /// 获取统计信息
    ///
    /// # 返回值
    /// 返回统计信息的不可变引用
    pub fn stats(&self) -> &GCStats {
        self.gc.stats()
    }

    /// 获取总分配字节数
    ///
    /// # 返回值
    /// 返回总共分配的字节数
    pub fn total_allocated(&self) -> usize {
        self.total_allocated
    }

    /// 设置垃圾回收阈值
    ///
    /// # 参数
    /// - `threshold`: 新的对象数量阈值
    pub fn set_threshold(&mut self, threshold: usize) {
        self.threshold = threshold;
    }

    /// 获取垃圾回收阈值
    ///
    /// # 返回值
    /// 返回当前的对象数量阈值
    pub fn threshold(&self) -> usize {
        self.threshold
    }

    /// 获取内存使用情况
    ///
    /// # 返回值
    /// 返回元组（已使用内存，总内存）
    pub fn memory_usage(&self) -> (usize, usize) {
        self.gc.memory_usage()
    }

    /// 获取对象计数
    ///
    /// # 返回值
    /// 返回当前分配的对象数量
    pub fn get_object_count(&self) -> usize {
        self.object_count
    }

    /// 清除内存管理器状态
    pub fn clear(&mut self) {
        self.object_count = 0;
        self.total_allocated = 0;
        self.last_collection_time = Instant::now();
    }

    /// 生成内存使用报告
    ///
    /// # 返回值
    /// 返回内存使用报告字符串
    pub fn report(&self) -> String {
        let (used, total) = self.memory_usage();
        format!(
            "Memory Manager Report:\n\
             - Object count: {}\n\
             - Total allocated: {} bytes\n\
             - Memory usage: {}/{} bytes\n\
             - Threshold: {} objects\n\
             - Last collection: {:?} ago",
            self.object_count,
            self.total_allocated,
            used,
            total,
            self.threshold,
            self.last_collection_time.elapsed()
        )
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new(800)
    }
}
