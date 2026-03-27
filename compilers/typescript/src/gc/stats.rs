//! 垃圾收集统计模块
//!
//! 提供 GC 执行过程中的各种统计信息收集和报告功能。

/// 垃圾收集统计
///
/// 记录垃圾收集过程中的各种统计数据，包括收集次数、时间、内存使用等。
#[derive(Debug, Clone, Default)]
pub struct GCStats {
    /// 总收集次数
    pub collection_count: usize,
    /// 标记的对象数量
    pub marked_objects: usize,
    /// 回收的对象数量
    pub collected_objects: usize,
    /// 回收的字节数
    pub collected_bytes: usize,
    /// 上次收集耗时（微秒）
    pub last_collection_time_us: u64,
    /// 总收集耗时（微秒）
    pub total_collection_time_us: u64,
    /// 内存碎片率
    pub fragmentation_ratio: f64,
    /// 平均对象大小
    pub average_object_size: usize,
    /// 新生代收集次数
    pub young_gen_collections: usize,
    /// 老年代收集次数
    pub old_gen_collections: usize,
    /// 晋升到老年的对象数量
    pub promoted_objects: usize,
    /// 晋升到老年的字节数
    pub promoted_bytes: usize,
    /// 并行标记耗时（微秒）
    pub parallel_mark_time_us: u64,
    /// 并行清除耗时（微秒）
    pub parallel_sweep_time_us: u64,
    /// 内存分配次数
    pub allocation_count: usize,
    /// 内存分配总字节数
    pub allocation_bytes: usize,
    /// 最大暂停时间（微秒）
    pub max_pause_time_us: u64,
    /// 最小暂停时间（微秒）
    pub min_pause_time_us: u64,
}

impl GCStats {
    /// 记录收集
    ///
    /// # 参数
    /// - `duration_us`: 收集耗时（微秒）
    /// - `marked`: 标记的对象数量
    /// - `collected`: 回收的对象数量
    /// - `bytes`: 回收的字节数
    pub fn record_collection(&mut self, duration_us: u64, marked: usize, collected: usize, bytes: usize) {
        self.collection_count += 1;
        self.marked_objects += marked;
        self.collected_objects += collected;
        self.collected_bytes += bytes;
        self.last_collection_time_us = duration_us;
        self.total_collection_time_us += duration_us;

        if collected > 0 {
            self.average_object_size = bytes / collected;
        }

        if self.max_pause_time_us < duration_us {
            self.max_pause_time_us = duration_us;
        }
        if self.min_pause_time_us == 0 || self.min_pause_time_us > duration_us {
            self.min_pause_time_us = duration_us;
        }
    }

    /// 记录新生代收集
    ///
    /// # 参数
    /// - `duration_us`: 收集耗时（微秒）
    /// - `marked`: 标记的对象数量
    /// - `collected`: 回收的对象数量
    /// - `bytes`: 回收的字节数
    /// - `promoted`: 晋升的对象数量
    /// - `promoted_bytes`: 晋升的字节数
    pub fn record_young_gen_collection(
        &mut self,
        duration_us: u64,
        marked: usize,
        collected: usize,
        bytes: usize,
        promoted: usize,
        promoted_bytes: usize,
    ) {
        self.young_gen_collections += 1;
        self.record_collection(duration_us, marked, collected, bytes);
        self.promoted_objects += promoted;
        self.promoted_bytes += promoted_bytes;
    }

    /// 记录老年代收集
    ///
    /// # 参数
    /// - `duration_us`: 收集耗时（微秒）
    /// - `marked`: 标记的对象数量
    /// - `collected`: 回收的对象数量
    /// - `bytes`: 回收的字节数
    pub fn record_old_gen_collection(&mut self, duration_us: u64, marked: usize, collected: usize, bytes: usize) {
        self.old_gen_collections += 1;
        self.record_collection(duration_us, marked, collected, bytes);
    }

    /// 记录内存分配
    ///
    /// # 参数
    /// - `size`: 分配的字节数
    pub fn record_allocation(&mut self, size: usize) {
        self.allocation_count += 1;
        self.allocation_bytes += size;
    }

    /// 记录并行标记时间
    ///
    /// # 参数
    /// - `duration_us`: 并行标记耗时（微秒）
    pub fn record_parallel_mark_time(&mut self, duration_us: u64) {
        self.parallel_mark_time_us += duration_us;
    }

    /// 记录并行清除时间
    ///
    /// # 参数
    /// - `duration_us`: 并行清除耗时（微秒）
    pub fn record_parallel_sweep_time(&mut self, duration_us: u64) {
        self.parallel_sweep_time_us += duration_us;
    }

    /// 获取平均收集时间
    ///
    /// # 返回值
    /// 返回平均收集时间（微秒）
    pub fn average_collection_time_us(&self) -> u64 {
        if self.collection_count == 0 { 0 } else { self.total_collection_time_us / self.collection_count as u64 }
    }

    /// 获取平均新生代收集时间
    ///
    /// # 返回值
    /// 返回平均新生代收集时间（微秒）
    pub fn average_young_gen_time_us(&self) -> u64 {
        if self.young_gen_collections == 0 { 0 } else { self.total_collection_time_us / self.young_gen_collections as u64 }
    }

    /// 获取平均晋升率
    ///
    /// # 返回值
    /// 返回平均每次新生代收集的晋升对象数量
    pub fn average_promotion_rate(&self) -> f64 {
        if self.young_gen_collections == 0 { 0.0 } else { self.promoted_objects as f64 / self.young_gen_collections as f64 }
    }

    /// 更新内存碎片率
    ///
    /// # 参数
    /// - `used`: 已使用内存
    /// - `total`: 总内存
    pub fn update_fragmentation(&mut self, used: usize, total: usize) {
        if total > 0 {
            self.fragmentation_ratio = (total - used) as f64 / total as f64;
        }
    }

    /// 重置统计信息
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    /// 生成统计报告
    ///
    /// # 返回值
    /// 返回统计报告字符串
    pub fn report(&self) -> String {
        format!(
            "GC Statistics:\n\
             - Collection count: {}\n\
             - Young gen collections: {}\n\
             - Old gen collections: {}\n\
             - Total collection time: {} us\n\
             - Average collection time: {} us\n\
             - Max pause time: {} us\n\
             - Min pause time: {} us\n\
             - Collected objects: {}\n\
             - Collected bytes: {}\n\
             - Promoted objects: {}\n\
             - Promoted bytes: {}\n\
             - Average promotion rate: {:.2}\n\
             - Fragmentation ratio: {:.2}%\n\
             - Allocation count: {}\n\
             - Allocation bytes: {}",
            self.collection_count,
            self.young_gen_collections,
            self.old_gen_collections,
            self.total_collection_time_us,
            self.average_collection_time_us(),
            self.max_pause_time_us,
            self.min_pause_time_us,
            self.collected_objects,
            self.collected_bytes,
            self.promoted_objects,
            self.promoted_bytes,
            self.average_promotion_rate(),
            self.fragmentation_ratio * 100.0,
            self.allocation_count,
            self.allocation_bytes,
        )
    }
}
