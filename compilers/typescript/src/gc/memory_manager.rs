//! 内存管理器模块

use std::{ptr::NonNull, time::Instant};

use typescript_types::TsValue;

use super::{collector::GC, object::GcObject, stats::GCStats};

/// 内存管理器
///
/// 整合内存分配和垃圾收集功能
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
    pub fn new(threshold: usize) -> Self {
        Self { gc: GC::new(), object_count: 0, threshold, total_allocated: 0, last_collection_time: Instant::now() }
    }

    /// 分配对象
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
    pub fn add_root(&mut self, obj: NonNull<GcObject>) {
        self.gc.add_root(obj);
    }

    /// 移除根对象
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
    pub fn collect_incremental(&mut self) -> bool {
        let result = self.gc.collect_incremental();
        if result {
            self.object_count = 0;
            self.last_collection_time = Instant::now();
        }
        result
    }

    /// 记录跨代引用
    pub fn record_cross_generation_reference(&mut self, old_obj: NonNull<GcObject>, young_obj: NonNull<GcObject>) {
        self.gc.record_cross_generation_reference(old_obj, young_obj);
    }

    /// 获取垃圾收集器引用
    pub fn gc(&self) -> &GC {
        &self.gc
    }

    /// 获取垃圾收集器可变引用
    pub fn gc_mut(&mut self) -> &mut GC {
        &mut self.gc
    }

    /// 获取统计信息
    pub fn stats(&self) -> &GCStats {
        self.gc.stats()
    }

    /// 获取总分配字节数
    pub fn total_allocated(&self) -> usize {
        self.total_allocated
    }

    /// 设置垃圾回收阈值
    pub fn set_threshold(&mut self, threshold: usize) {
        self.threshold = threshold;
    }

    /// 获取垃圾回收阈值
    pub fn threshold(&self) -> usize {
        self.threshold
    }

    /// 获取内存使用情况
    pub fn memory_usage(&self) -> (usize, usize) {
        self.gc.memory_usage()
    }

    /// 获取对象计数
    pub fn get_object_count(&self) -> usize {
        self.object_count
    }

    /// 清除内存管理器状态
    pub fn clear(&mut self) {
        self.object_count = 0;
        self.total_allocated = 0;
        self.last_collection_time = Instant::now();
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new(800)
    }
}
