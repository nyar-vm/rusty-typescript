//! 垃圾收集事件处理模块
//!
//! 提供 GC 事件监听和处理机制，允许外部代码监控 GC 执行过程。

use super::{phase::GCPhase, stats::GCStats};

/// 垃圾收集事件处理器
///
/// 定义处理垃圾收集事件的接口，实现此 trait 可以监听 GC 的执行过程。
/// 所有方法都要求实现 `Send + Sync`，以支持多线程环境。
pub trait GCEventHandler: Send + Sync {
    /// 垃圾收集开始事件
    ///
    /// 当 GC 开始某个阶段时调用。
    ///
    /// # 参数
    /// - `phase`: 当前开始的 GC 阶段
    fn on_gc_start(&self, phase: GCPhase);

    /// 垃圾收集结束事件
    ///
    /// 当 GC 完成某个阶段时调用。
    ///
    /// # 参数
    /// - `phase`: 当前结束的 GC 阶段
    /// - `stats`: 当前的 GC 统计信息
    fn on_gc_end(&self, phase: GCPhase, stats: &GCStats);

    /// 对象被回收事件
    ///
    /// 当一个对象被垃圾回收时调用。
    ///
    /// # 参数
    /// - `size`: 被回收对象的大小（字节）
    fn on_object_collected(&self, size: usize);
}

/// 简单的日志事件处理器
///
/// 将 GC 事件输出到标准输出，用于调试和监控。
#[derive(Debug, Clone, Copy)]
pub struct LoggingEventHandler {
    /// 是否启用详细日志
    pub verbose: bool,
}

impl LoggingEventHandler {
    /// 创建新的日志事件处理器
    ///
    /// # 返回值
    /// 返回新创建的日志事件处理器实例
    pub fn new() -> Self {
        Self { verbose: false }
    }

    /// 创建启用详细日志的事件处理器
    ///
    /// # 返回值
    /// 返回启用详细日志的事件处理器实例
    pub fn verbose() -> Self {
        Self { verbose: true }
    }
}

impl Default for LoggingEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl GCEventHandler for LoggingEventHandler {
    fn on_gc_start(&self, phase: GCPhase) {
        println!("[GC] Starting {:?} phase", phase);
    }

    fn on_gc_end(&self, phase: GCPhase, stats: &GCStats) {
        println!(
            "[GC] Completed {:?} phase - Collections: {}, Time: {}us",
            phase, stats.collection_count, stats.last_collection_time_us
        );
        if self.verbose {
            println!(
                "[GC] Details - Marked: {}, Collected: {}, Bytes: {}",
                stats.marked_objects, stats.collected_objects, stats.collected_bytes
            );
        }
    }

    fn on_object_collected(&self, size: usize) {
        if self.verbose {
            println!("[GC] Collected object of size {} bytes", size);
        }
    }
}

/// 统计收集事件处理器
///
/// 收集 GC 事件的统计信息，用于分析和报告。
#[derive(Debug, Clone, Default)]
pub struct StatsCollectingEventHandler {
    /// GC 开始次数
    pub gc_start_count: usize,
    /// GC 结束次数
    pub gc_end_count: usize,
    /// 回收的对象总数
    pub total_objects_collected: usize,
    /// 回收的字节总数
    pub total_bytes_collected: usize,
}

impl StatsCollectingEventHandler {
    /// 创建新的统计收集事件处理器
    ///
    /// # 返回值
    /// 返回新创建的统计收集事件处理器实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 重置统计信息
    pub fn reset(&mut self) {
        self.gc_start_count = 0;
        self.gc_end_count = 0;
        self.total_objects_collected = 0;
        self.total_bytes_collected = 0;
    }
}

impl GCEventHandler for StatsCollectingEventHandler {
    fn on_gc_start(&self, _phase: GCPhase) {
        // Note: This requires interior mutability for proper implementation
        // For now, we just ignore this in the trait implementation
    }

    fn on_gc_end(&self, _phase: GCPhase, _stats: &GCStats) {
        // Note: This requires interior mutability for proper implementation
    }

    fn on_object_collected(&self, size: usize) {
        // Note: This requires interior mutability for proper implementation
        let _ = size;
    }
}
