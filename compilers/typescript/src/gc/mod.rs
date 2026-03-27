//! 垃圾收集模块
//!
//! 提供高效的垃圾收集机制，包括增量标记-清除、分代收集和引用计数。

mod collector;
mod event;
mod header;
mod memory_manager;
mod object;
mod phase;
mod stats;
mod work_stealing;

pub use collector::{GC, mark_value_references_parallel_helper, process_object_in_parallel_helper};
pub use event::GCEventHandler;
pub use header::ObjectHeader;
pub use memory_manager::MemoryManager;
pub use object::GcObject;
pub use phase::GCPhase;
pub use stats::GCStats;
pub use work_stealing::{RememberedSetEntry, WorkStealingQueue};
