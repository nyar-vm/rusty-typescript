//! 工作窃取队列和记忆集模块

use std::{collections::VecDeque, ptr::NonNull, sync::Mutex};

use super::object::GcObject;

/// 记忆集条目
#[derive(Debug, Clone, Copy)]
pub struct RememberedSetEntry {
    /// 老年代对象指针
    pub old_obj: NonNull<GcObject>,
    /// 新生代对象指针
    pub young_obj: NonNull<GcObject>,
}

/// 工作窃取队列
///
/// 用于并行标记时的工作分配和窃取
#[derive(Debug)]
pub struct WorkStealingQueue {
    /// 队列数据
    queue: Mutex<VecDeque<NonNull<GcObject>>>,
}

impl WorkStealingQueue {
    /// 创建新的工作窃取队列
    pub fn new() -> Self {
        Self { queue: Mutex::new(VecDeque::new()) }
    }

    /// 推入工作项
    pub fn push(&self, item: NonNull<GcObject>) {
        self.queue.lock().unwrap().push_back(item);
    }

    /// 弹出工作项（从队首）
    pub fn pop(&self) -> Option<NonNull<GcObject>> {
        self.queue.lock().unwrap().pop_front()
    }

    /// 窃取工作项（从队尾）
    pub fn steal(&self) -> Option<NonNull<GcObject>> {
        let mut queue = self.queue.lock().unwrap();
        if queue.len() > 1 { queue.pop_back() } else { None }
    }

    /// 检查队列是否为空
    pub fn is_empty(&self) -> bool {
        self.queue.lock().unwrap().is_empty()
    }

    /// 获取队列长度
    pub fn len(&self) -> usize {
        self.queue.lock().unwrap().len()
    }
}

impl Default for WorkStealingQueue {
    fn default() -> Self {
        Self::new()
    }
}
