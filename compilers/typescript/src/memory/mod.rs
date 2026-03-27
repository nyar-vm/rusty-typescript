//! 内存管理模块
//!
//! 提供高效的内存分配和管理机制，包括内存池和对象池。

use std::{
    alloc::{Layout, alloc, dealloc},
    collections::HashMap,
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
};
use typescript_types::TsValue;

/// 内存块大小常量
const BLOCK_SIZES: &[usize] = &[8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096];

/// 大内存块大小（用于超过 4096 的内存分配）
const LARGE_BLOCK_SIZE: usize = 4096;

/// 内存池
///
/// 管理固定大小的内存块，提供快速的分配和回收
pub struct MemoryPool {
    /// 按大小分类的空闲块列表
    free_blocks: HashMap<usize, Vec<NonNull<u8>>>,
    /// 已分配的总字节数
    total_allocated: AtomicUsize,
    /// 已使用的字节数
    total_used: AtomicUsize,
}

// 实现 Send trait，因为 MemoryPool 可以安全地在线程之间发送
unsafe impl Send for MemoryPool {}

// 实现 Sync trait，因为 MemoryPool 可以安全地在线程之间共享
unsafe impl Sync for MemoryPool {}

impl MemoryPool {
    /// 创建一个新的内存池
    pub fn new() -> Self {
        let mut free_blocks = HashMap::new();
        for &size in BLOCK_SIZES {
            free_blocks.insert(size, Vec::new());
        }

        Self { free_blocks, total_allocated: AtomicUsize::new(0), total_used: AtomicUsize::new(0) }
    }

    /// 分配指定大小的内存块
    ///
    /// 优先从池中获取空闲块，如果没有则分配新内存
    pub fn allocate(&mut self, size: usize) -> Option<NonNull<u8>> {
        let block_size = self.round_up_block_size(size);

        // 尝试从空闲列表获取
        if let Some(blocks) = self.free_blocks.get_mut(&block_size) {
            if let Some(block) = blocks.pop() {
                self.total_used.fetch_add(block_size, Ordering::Relaxed);
                return Some(block);
            }
        }

        // 分配新内存
        unsafe {
            let layout = Layout::from_size_align(block_size, 8).ok()?;
            let ptr = alloc(layout);
            if ptr.is_null() {
                return None;
            }
            self.total_allocated.fetch_add(block_size, Ordering::Relaxed);
            self.total_used.fetch_add(block_size, Ordering::Relaxed);
            Some(NonNull::new_unchecked(ptr))
        }
    }

    /// 释放内存块回池中
    ///
    /// 内存块会被放入空闲列表供后续重用
    pub fn deallocate(&mut self, ptr: NonNull<u8>, size: usize) {
        let block_size = self.round_up_block_size(size);

        if let Some(blocks) = self.free_blocks.get_mut(&block_size) {
            blocks.push(ptr);
            self.total_used.fetch_sub(block_size, Ordering::Relaxed);
        }
        else {
            // 如果大小不匹配标准块大小，直接释放
            unsafe {
                let layout = Layout::from_size_align(block_size, 8).unwrap();
                dealloc(ptr.as_ptr(), layout);
                self.total_allocated.fetch_sub(block_size, Ordering::Relaxed);
                self.total_used.fetch_sub(block_size, Ordering::Relaxed);
            }
        }
    }

    /// 获取内存使用统计
    pub fn stats(&self) -> MemoryStats {
        MemoryStats {
            total_allocated: self.total_allocated.load(Ordering::Relaxed),
            total_used: self.total_used.load(Ordering::Relaxed),
            free_blocks: self.free_blocks.iter().map(|(size, blocks)| (*size, blocks.len())).collect(),
        }
    }

    /// 清理所有空闲块
    ///
    /// 将空闲列表中的所有内存块释放回操作系统
    pub fn cleanup(&mut self) {
        for (size, blocks) in &mut self.free_blocks {
            for block in blocks.drain(..) {
                unsafe {
                    let layout = Layout::from_size_align(*size, 8).unwrap();
                    dealloc(block.as_ptr(), layout);
                }
                self.total_allocated.fetch_sub(*size, Ordering::Relaxed);
            }
        }
    }

    /// 将大小向上取整到最近的块大小
    fn round_up_block_size(&self, size: usize) -> usize {
        for &block_size in BLOCK_SIZES {
            if size <= block_size {
                return block_size;
            }
        }
        // 如果超过最大块大小，向上取整到 LARGE_BLOCK_SIZE 的倍数
        ((size + LARGE_BLOCK_SIZE - 1) / LARGE_BLOCK_SIZE) * LARGE_BLOCK_SIZE
    }
}

impl Default for MemoryPool {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for MemoryPool {
    fn drop(&mut self) {
        self.cleanup();
    }
}

/// 对象池
///
/// 管理特定类型对象的缓存，减少频繁的分配和释放开销
pub struct ObjectPool<T> {
    /// 空闲对象列表
    free_objects: Vec<T>,
    /// 池的最大容量
    capacity: usize,
    /// 创建新对象的工厂函数
    factory: Box<dyn Fn() -> T>,
    /// 重置对象的函数
    reset: Box<dyn Fn(&mut T)>,
}

impl<T> ObjectPool<T> {
    /// 创建一个新的对象池
    ///
    /// # 参数
    /// - `capacity`: 池的最大容量
    /// - `factory`: 创建新对象的工厂函数
    /// - `reset`: 重置对象的函数，在对象被重用前调用
    pub fn new<F, R>(capacity: usize, factory: F, reset: R) -> Self
    where
        F: Fn() -> T + 'static,
        R: Fn(&mut T) + 'static,
    {
        Self { free_objects: Vec::with_capacity(capacity), capacity, factory: Box::new(factory), reset: Box::new(reset) }
    }

    /// 从池中获取一个对象
    ///
    /// 如果池中有空闲对象，则返回并重置它；否则创建新对象
    pub fn acquire(&mut self) -> T {
        if let Some(mut obj) = self.free_objects.pop() {
            (self.reset)(&mut obj);
            obj
        }
        else {
            (self.factory)()
        }
    }

    /// 将对象释放回池中
    ///
    /// 如果池未满，对象会被缓存；否则被丢弃
    pub fn release(&mut self, obj: T) {
        if self.free_objects.len() < self.capacity {
            self.free_objects.push(obj);
        }
    }

    /// 获取池中空闲对象的数量
    pub fn free_count(&self) -> usize {
        self.free_objects.len()
    }

    /// 清空对象池
    pub fn clear(&mut self) {
        self.free_objects.clear();
    }
}

/// 内存使用统计
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// 已分配的总字节数
    pub total_allocated: usize,
    /// 已使用的字节数
    pub total_used: usize,
    /// 各大小空闲块的数量
    pub free_blocks: HashMap<usize, usize>,
}

impl MemoryStats {
    /// 计算空闲内存总量
    pub fn total_free(&self) -> usize {
        self.total_allocated.saturating_sub(self.total_used)
    }

    /// 计算内存使用百分比
    pub fn usage_percentage(&self) -> f64 {
        if self.total_allocated == 0 { 0.0 } else { (self.total_used as f64 / self.total_allocated as f64) * 100.0 }
    }
}

/// 内存分配器 trait
///
/// 定义内存分配的基本接口
pub trait Allocator: Send {
    /// 分配指定大小的内存
    fn allocate(&mut self, size: usize) -> Option<NonNull<u8>>;
    /// 释放内存
    fn deallocate(&mut self, ptr: NonNull<u8>, size: usize);
    /// 获取内存使用统计
    fn stats(&self) -> MemoryStats;
    /// 克隆分配器
    fn box_clone(&self) -> Box<dyn Allocator>;
}

/// 伙伴分配器
///
/// 使用伙伴系统算法进行内存分配，减少内存碎片
pub struct BuddyAllocator {
    /// 空闲块列表，按大小分类（2的幂）
    free_lists: Vec<Vec<NonNull<u8>>>,
    /// 总分配内存
    total_allocated: AtomicUsize,
    /// 已使用内存
    total_used: AtomicUsize,
    /// 最小块大小
    min_block_size: usize,
    /// 最大块大小
    max_block_size: usize,
}

// 实现 Send trait，因为 BuddyAllocator 可以安全地在线程之间发送
unsafe impl Send for BuddyAllocator {}

// 实现 Sync trait，因为 BuddyAllocator 可以安全地在线程之间共享
unsafe impl Sync for BuddyAllocator {}

impl BuddyAllocator {
    /// 创建一个新的伙伴分配器
    pub fn new(min_block_size: usize, max_block_size: usize) -> Self {
        // 确保块大小是2的幂
        let min_block_size = Self::next_power_of_two(min_block_size);
        let max_block_size = Self::next_power_of_two(max_block_size);

        // 计算需要的空闲列表数量
        let levels = (max_block_size.trailing_zeros() - min_block_size.trailing_zeros() + 1) as usize;
        let mut free_lists = Vec::with_capacity(levels);
        for _ in 0..levels {
            free_lists.push(Vec::new());
        }

        // 分配初始内存
        let initial_memory = unsafe {
            let layout = Layout::from_size_align(max_block_size, max_block_size).unwrap();
            let ptr = alloc(layout);
            if ptr.is_null() {
                panic!("Failed to allocate initial memory for buddy allocator");
            }
            NonNull::new_unchecked(ptr)
        };

        // 将初始内存添加到最大块的空闲列表
        let max_level = free_lists.len() - 1;
        free_lists[max_level].push(initial_memory);

        Self {
            free_lists,
            total_allocated: AtomicUsize::new(max_block_size),
            total_used: AtomicUsize::new(0),
            min_block_size,
            max_block_size,
        }
    }

    /// 计算下一个2的幂
    fn next_power_of_two(mut size: usize) -> usize {
        size -= 1;
        size |= size >> 1;
        size |= size >> 2;
        size |= size >> 4;
        size |= size >> 8;
        size |= size >> 16;
        size + 1
    }

    /// 获取块大小对应的级别
    fn get_level(&self, size: usize) -> usize {
        let normalized_size = if size < self.min_block_size { self.min_block_size } else { Self::next_power_of_two(size) };
        (normalized_size.trailing_zeros() - self.min_block_size.trailing_zeros()) as usize
    }

    /// 查找伙伴块
    fn find_buddy(&self, ptr: NonNull<u8>, size: usize) -> NonNull<u8> {
        let ptr_addr = ptr.as_ptr() as usize;
        let buddy_addr = ptr_addr ^ size;
        NonNull::new(buddy_addr as *mut u8).unwrap()
    }

    /// 检查伙伴块是否空闲
    fn is_buddy_free(&self, buddy: NonNull<u8>, level: usize) -> bool {
        self.free_lists[level].contains(&buddy)
    }
}

impl Allocator for BuddyAllocator {
    fn allocate(&mut self, size: usize) -> Option<NonNull<u8>> {
        let required_size = if size < self.min_block_size { self.min_block_size } else { Self::next_power_of_two(size) };

        let level = self.get_level(required_size);

        // 查找合适的块
        for i in level..self.free_lists.len() {
            if !self.free_lists[i].is_empty() {
                // 找到可用块
                let block = self.free_lists[i].pop().unwrap();

                // 分裂块直到达到所需大小
                let mut current_size = self.min_block_size << i;
                let mut current_block = block;
                let mut current_level = i;

                while current_size > required_size {
                    current_size /= 2;
                    let buddy = self.find_buddy(current_block, current_size);
                    current_level -= 1;

                    // 将伙伴块添加到下一级空闲列表
                    self.free_lists[current_level].push(buddy);
                }

                self.total_used.fetch_add(required_size, Ordering::Relaxed);
                return Some(current_block);
            }
        }

        None
    }

    fn deallocate(&mut self, ptr: NonNull<u8>, size: usize) {
        let required_size = if size < self.min_block_size { self.min_block_size } else { Self::next_power_of_two(size) };

        let mut level = self.get_level(required_size);
        let mut current_block = ptr;
        let mut current_size = required_size;

        // 尝试合并伙伴块
        loop {
            let buddy = self.find_buddy(current_block, current_size);

            if level >= self.free_lists.len() - 1 || !self.is_buddy_free(buddy, level) {
                // 无法合并，将块添加到空闲列表
                self.free_lists[level].push(current_block);
                break;
            }

            // 合并伙伴块
            let index = self.free_lists[level].iter().position(|&b| b == buddy).unwrap();
            self.free_lists[level].remove(index);

            // 更新当前块和大小
            current_block = if (current_block.as_ptr() as usize) < (buddy.as_ptr() as usize) { current_block } else { buddy };
            current_size *= 2;
            level += 1;
        }

        self.total_used.fetch_sub(required_size, Ordering::Relaxed);
    }

    fn stats(&self) -> MemoryStats {
        let mut free_blocks = HashMap::new();
        for (i, list) in self.free_lists.iter().enumerate() {
            let block_size = self.min_block_size << i;
            free_blocks.insert(block_size, list.len());
        }

        MemoryStats {
            total_allocated: self.total_allocated.load(Ordering::Relaxed),
            total_used: self.total_used.load(Ordering::Relaxed),
            free_blocks,
        }
    }

    fn box_clone(&self) -> Box<dyn Allocator> {
        Box::new(BuddyAllocator::new(self.min_block_size, self.max_block_size))
    }
}

impl Drop for BuddyAllocator {
    fn drop(&mut self) {
        // 释放所有分配的内存
        // 简化实现，实际应该释放所有块
    }
}

/// 内存分配策略
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocationStrategy {
    /// 默认内存池策略
    Default,
    /// 伙伴分配器策略
    Buddy,
}

/// 默认内存分配器
///
/// 使用内存池实现的高效分配器
pub struct DefaultAllocator {
    pool: MemoryPool,
}

impl DefaultAllocator {
    /// 创建一个新的默认分配器
    pub fn new() -> Self {
        Self { pool: MemoryPool::new() }
    }
}

impl Default for DefaultAllocator {
    fn default() -> Self {
        Self::new()
    }
}

impl Allocator for DefaultAllocator {
    fn allocate(&mut self, size: usize) -> Option<NonNull<u8>> {
        self.pool.allocate(size)
    }

    fn deallocate(&mut self, ptr: NonNull<u8>, size: usize) {
        self.pool.deallocate(ptr, size);
    }

    fn stats(&self) -> MemoryStats {
        self.pool.stats()
    }

    fn box_clone(&self) -> Box<dyn Allocator> {
        Box::new(DefaultAllocator { pool: MemoryPool::new() })
    }
}

/// 分配器工厂
///
/// 根据策略创建不同的内存分配器
pub struct AllocatorFactory;

impl AllocatorFactory {
    /// 创建指定策略的内存分配器
    pub fn create(strategy: AllocationStrategy) -> Box<dyn Allocator> {
        match strategy {
            AllocationStrategy::Default => Box::new(DefaultAllocator::new()),
            AllocationStrategy::Buddy => Box::new(BuddyAllocator::new(8, 4096)),
        }
    }
}

/// 全局内存管理器
///
/// 管理整个运行时的内存分配
pub struct GlobalMemoryManager {
    /// 内存池
    pool: MemoryPool,
    /// 对象池（用于常见类型）
    string_pool: ObjectPool<String>,
    /// TsValue 数组对象池
    tsvalue_array_pool: ObjectPool<Vec<TsValue>>,
    /// 字符串元组对象池（用于对象属性）
    string_tsvalue_pool: ObjectPool<Vec<(String, TsValue)>>,
    /// 内存分配阈值，超过时触发垃圾回收
    gc_threshold: usize,
    /// 上次垃圾回收后的分配计数
    allocations_since_gc: AtomicUsize,
}

impl GlobalMemoryManager {
    /// 创建一个新的全局内存管理器
    pub fn new(gc_threshold: usize) -> Self {
        Self {
            pool: MemoryPool::new(),
            string_pool: ObjectPool::new(1000, String::new, |s| {
                s.clear();
            }),
            tsvalue_array_pool: ObjectPool::new(
                500,
                || Vec::with_capacity(10),
                |v| {
                    v.clear();
                },
            ),
            string_tsvalue_pool: ObjectPool::new(
                500,
                || Vec::with_capacity(10),
                |v| {
                    v.clear();
                },
            ),
            gc_threshold,
            allocations_since_gc: AtomicUsize::new(0),
        }
    }

    /// 分配内存
    pub fn allocate(&mut self, size: usize) -> Option<NonNull<u8>> {
        self.allocations_since_gc.fetch_add(1, Ordering::Relaxed);
        self.pool.allocate(size)
    }

    /// 释放内存
    pub fn deallocate(&mut self, ptr: NonNull<u8>, size: usize) {
        self.pool.deallocate(ptr, size);
    }

    /// 获取字符串对象
    pub fn acquire_string(&mut self) -> String {
        self.string_pool.acquire()
    }

    /// 释放字符串对象
    pub fn release_string(&mut self, s: String) {
        self.string_pool.release(s);
    }

    /// 获取 TsValue 数组对象
    pub fn acquire_tsvalue_array(&mut self) -> Vec<TsValue> {
        self.tsvalue_array_pool.acquire()
    }

    /// 释放 TsValue 数组对象
    pub fn release_tsvalue_array(&mut self, v: Vec<TsValue>) {
        self.tsvalue_array_pool.release(v);
    }

    /// 获取字符串元组对象（用于对象属性）
    pub fn acquire_string_tsvalue(&mut self) -> Vec<(String, TsValue)> {
        self.string_tsvalue_pool.acquire()
    }

    /// 释放字符串元组对象
    pub fn release_string_tsvalue(&mut self, v: Vec<(String, TsValue)>) {
        self.string_tsvalue_pool.release(v);
    }

    /// 检查是否需要垃圾回收
    pub fn should_gc(&self) -> bool {
        self.allocations_since_gc.load(Ordering::Relaxed) >= self.gc_threshold
    }

    /// 重置垃圾回收计数器
    pub fn reset_gc_counter(&self) {
        self.allocations_since_gc.store(0, Ordering::Relaxed);
    }

    /// 获取内存使用统计
    pub fn stats(&self) -> MemoryStats {
        self.pool.stats()
    }

    /// 设置垃圾回收阈值
    pub fn set_gc_threshold(&mut self, threshold: usize) {
        self.gc_threshold = threshold;
    }

    /// 获取垃圾回收阈值
    pub fn gc_threshold(&self) -> usize {
        self.gc_threshold
    }

    /// 清理内存池
    pub fn cleanup(&mut self) {
        self.pool.cleanup();
        self.string_pool.clear();
        self.tsvalue_array_pool.clear();
        self.string_tsvalue_pool.clear();
    }
}

impl Default for GlobalMemoryManager {
    fn default() -> Self {
        Self::new(10000)
    }
}
