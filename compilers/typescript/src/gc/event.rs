//! 垃圾收集事件处理模块

use super::{phase::GCPhase, stats::GCStats};

/// 垃圾收集事件回调
pub trait GCEventHandler: Send + Sync {
    /// 垃圾收集开始
    fn on_gc_start(&self, phase: GCPhase);
    /// 垃圾收集结束
    fn on_gc_end(&self, phase: GCPhase, stats: &GCStats);
    /// 对象被回收
    fn on_object_collected(&self, size: usize);
}
