//! 工作窃取队列和记忆集模块
//!
//! 提供并行标记时的工作分配机制和分代收集的记忆集支持。

use std::{collections::VecDeque, ptr::NonNull, sync::Mutex};

use super::object::GcObject;

/// 记忆集条目
///
/// 记录跨代引用关系，用于分代垃圾收集。
#[derive(Debug, Clone, Copy)]
pub struct RememberedSetEntry {
    /// 老年代对象指针
    pub old_obj: NonNull<GcObject>,
    /// 新生代对象指针
    pub young_obj: NonNull<GcObject>,
}

impl RememberedSetEntry {
    /// 创建新的记忆集条目
    ///
    /// # 参数
    /// - `old_obj`: 老年代对象指针
    /// - `young_obj`: 新生代对象指针
    ///
    /// # 返回值
    /// 返回新创建的记忆集条目
    pub fn new(old_obj: NonNull<GcObject>, young_obj: NonNull<GcObject>) -> Self {
        Self { old_obj, young_obj }
    }
}

/// 工作窃取队列
///
/// 用于并行标记时的工作分配和窃取，支持多线程环境。
#[derive(Debug)]
pub struct WorkStealingQueue {
    /// 队列数据
    queue: Mutex<VecDeque<NonNull<GcObject>>>,
}

// 实现 Send trait，因为 Mutex 是 Send 的
unsafe impl Send for WorkStealingQueue {}

// 实现 Sync trait，因为 Mutex 是 Sync 的
unsafe impl Sync for WorkStealingQueue {}

impl WorkStealingQueue {
    /// 创建新的工作窃取队列
    ///
    /// # 返回值
    /// 返回新创建的工作窃取队列实例
    pub fn new() -> Self {
        Self { queue: Mutex::new(VecDeque::new()) }
    }

    /// 推入工作项
    ///
    /// # 参数
    /// - `item`: 要添加的工作项（GC 对象指针）
    pub fn push(&self, item: NonNull<GcObject>) {
        self.queue.lock().unwrap().push_back(item);
    }

    /// 弹出工作项（从队首）
    ///
    /// # 返回值
    /// 返回队首的工作项，如果队列为空则返回 None
    pub fn pop(&self) -> Option<NonNull<GcObject>> {
        self.queue.lock().unwrap().pop_front()
    }

    /// 窃取工作项（从队尾）
    ///
    /// 当队列中有多个工作项时，允许其他线程从队尾窃取工作。
    ///
    /// # 返回值
    /// 返回队尾的工作项，如果队列中少于 2 个元素则返回 None
    pub fn steal(&self) -> Option<NonNull<GcObject>> {
        let mut queue = self.queue.lock().unwrap();
        if queue.len() > 1 { queue.pop_back() } else { None }
    }

    /// 检查队列是否为空
    ///
    /// # 返回值
    /// 如果队列为空返回 true，否则返回 false
    pub fn is_empty(&self) -> bool {
        self.queue.lock().unwrap().is_empty()
    }

    /// 获取队列长度
    ///
    /// # 返回值
    /// 返回队列中的工作项数量
    pub fn len(&self) -> usize {
        self.queue.lock().unwrap().len()
    }

    /// 清空队列
    pub fn clear(&self) {
        self.queue.lock().unwrap().clear();
    }

    /// 批量推入工作项
    ///
    /// # 参数
    /// - `items`: 要添加的工作项迭代器
    pub fn extend<I: IntoIterator<Item = NonNull<GcObject>>>(&self, items: I) {
        let mut queue = self.queue.lock().unwrap();
        queue.extend(items);
    }

    /// 弹出多个工作项
    ///
    /// # 参数
    /// - `count`: 要弹出的工作项数量
    ///
    /// # 返回值
    /// 返回弹出的工作项向量
    pub fn pop_multiple(&self, count: usize) -> Vec<NonNull<GcObject>> {
        let mut queue = self.queue.lock().unwrap();
        let actual_count = count.min(queue.len());
        let mut result = Vec::with_capacity(actual_count);
        for _ in 0..actual_count {
            if let Some(item) = queue.pop_front() {
                result.push(item);
            }
        }
        result
    }
}

impl Default for WorkStealingQueue {
    fn default() -> Self {
        Self::new()
    }
}
