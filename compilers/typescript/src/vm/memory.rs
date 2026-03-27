//! 内存管理模块
//!
//! 提供对象池和垃圾回收功能，用于优化内存分配和回收。

use std::collections::{HashMap, HashSet, VecDeque};

use typescript_types::TsValue;

/// 内存管理器
///
/// 管理对象池和数组池，实现内存复用
pub struct MemoryManager {
    /// 对象池 - 用于复用对象，减少内存分配
    pub object_pool: VecDeque<HashMap<String, TsValue>>,
    /// 数组池 - 用于复用数组，减少内存分配
    pub array_pool: VecDeque<Vec<TsValue>>,
    /// 对象池大小限制
    pub object_pool_limit: usize,
    /// 数组池大小限制
    pub array_pool_limit: usize,
    /// 活跃对象跟踪 - 用于垃圾回收
    pub active_objects: HashSet<*const HashMap<String, TsValue>>,
    /// 活跃数组跟踪 - 用于垃圾回收
    pub active_arrays: HashSet<*const Vec<TsValue>>,
}

impl MemoryManager {
    /// 创建新的内存管理器
    pub fn new() -> Self {
        Self {
            object_pool: VecDeque::with_capacity(100),
            array_pool: VecDeque::with_capacity(100),
            object_pool_limit: 1000,
            array_pool_limit: 1000,
            active_objects: HashSet::new(),
            active_arrays: HashSet::new(),
        }
    }

    /// 从对象池获取对象
    pub fn acquire_object(&mut self) -> HashMap<String, TsValue> {
        if let Some(mut obj) = self.object_pool.pop_front() {
            obj.clear();
            obj
        }
        else {
            HashMap::with_capacity(8)
        }
    }

    /// 将对象归还到对象池
    pub fn release_object(&mut self, mut obj: HashMap<String, TsValue>) {
        if self.object_pool.len() < self.object_pool_limit {
            obj.clear();
            self.object_pool.push_back(obj);
        }
    }

    /// 从数组池获取数组
    pub fn acquire_array(&mut self) -> Vec<TsValue> {
        if let Some(mut arr) = self.array_pool.pop_front() {
            arr.clear();
            arr
        }
        else {
            Vec::with_capacity(8)
        }
    }

    /// 将数组归还到数组池
    pub fn release_array(&mut self, mut arr: Vec<TsValue>) {
        if self.array_pool.len() < self.array_pool_limit {
            arr.clear();
            self.array_pool.push_back(arr);
        }
    }

    /// 跟踪活跃对象
    pub fn track_object(&mut self, obj: &HashMap<String, TsValue>) {
        let obj_ptr = obj as *const HashMap<String, TsValue>;
        self.active_objects.insert(obj_ptr);
    }

    /// 跟踪活跃数组
    pub fn track_array(&mut self, arr: &Vec<TsValue>) {
        let arr_ptr = arr as *const Vec<TsValue>;
        self.active_arrays.insert(arr_ptr);
    }

    /// 回收 TsValue 中的对象和数组
    pub fn recycle_tsvalue(&mut self, value: TsValue) {
        match value {
            TsValue::Object(obj) => {
                self.release_object(obj);
            }
            TsValue::Array(arr) => {
                self.release_array(arr);
            }
            _ => {}
        }
    }

    /// 清理池中的对象和数组
    pub fn clear_pools(&mut self) {
        self.object_pool.clear();
        self.array_pool.clear();
    }

    /// 执行垃圾回收
    pub fn garbage_collect(&mut self, stack: &mut Vec<TsValue>) {
        self.active_objects.clear();
        self.active_arrays.clear();

        while self.object_pool.len() > 50 {
            self.object_pool.pop_back();
        }
        while self.array_pool.len() > 50 {
            self.array_pool.pop_back();
        }

        let mut new_stack = Vec::with_capacity(stack.len());
        while let Some(value) = stack.pop() {
            if new_stack.is_empty() {
                new_stack.push(value);
            }
            else {
                self.recycle_tsvalue(value);
            }
        }
        *stack = new_stack;
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new()
    }
}
