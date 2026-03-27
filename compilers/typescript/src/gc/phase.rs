//! 垃圾收集阶段模块

/// 垃圾收集阶段
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GCPhase {
    /// 空闲状态
    Idle,
    /// 标记阶段
    Marking,
    /// 并发标记阶段
    ConcurrentMarking,
    /// 清除阶段
    Sweeping,
    /// 并发清除阶段
    ConcurrentSweeping,
    /// 压缩阶段
    Compacting,
}
