//! WASM 内存管理优化模块
//!
//! 本模块提供了高效的 WASM 内存管理功能，包括内存池分配、碎片整理等功能。

use std::{
    alloc::{Layout, alloc, dealloc},
    collections::BTreeMap,
    ptr::NonNull,
};

/// 内存块信息
///
/// 表示一个内存块的元数据，包含地址、大小和分配状态。
#[derive(Debug, Clone)]
pub struct MemoryBlock {
    /// 内存块的起始地址
    pub address: usize,
    /// 内存块的大小（字节）
    pub size: usize,
    /// 是否已分配
    pub is_allocated: bool,
}

impl MemoryBlock {
    /// 创建新的内存块
    ///
    /// # 参数
    /// - `address`: 内存块的起始地址
    /// - `size`: 内存块的大小（字节）
    ///
    /// # 返回
    /// 新创建的内存块实例
    pub fn new(address: usize, size: usize) -> Self {
        Self { address, size, is_allocated: false }
    }

    /// 检查内存块是否可以容纳指定大小
    ///
    /// # 参数
    /// - `required_size`: 需要的大小
    ///
    /// # 返回
    /// 如果内存块未分配且大小足够，返回 true
    pub fn can_fit(&self, required_size: usize) -> bool {
        !self.is_allocated && self.size >= required_size
    }

    /// 分割内存块
    ///
    /// 如果内存块大小大于所需大小，将其分割为两个块
    ///
    /// # 参数
    /// - `required_size`: 需要的大小
    ///
    /// # 返回
    /// 如果成功分割，返回新创建的内存块；否则返回 None
    pub fn split(&mut self, required_size: usize) -> Option<MemoryBlock> {
        if self.is_allocated || self.size <= required_size {
            return None;
        }

        let remaining_size = self.size - required_size;
        if remaining_size < 16 {
            return None;
        }

        let new_block = MemoryBlock::new(self.address + required_size, remaining_size);
        self.size = required_size;
        Some(new_block)
    }
}

/// 内存统计信息
///
/// 提供内存池的使用统计，包括总大小、已用大小、碎片率等。
#[derive(Debug, Clone, Default)]
pub struct MemoryStats {
    /// 内存池总大小（字节）
    pub total_size: usize,
    /// 已使用的内存大小（字节）
    pub used_size: usize,
    /// 可用内存大小（字节）
    pub available_size: usize,
    /// 已分配的内存块数量
    pub allocated_blocks: usize,
    /// 空闲内存块数量
    pub free_blocks: usize,
    /// 碎片率（0.0 到 1.0）
    pub fragmentation_rate: f64,
    /// 内存使用峰值（字节）
    pub peak_usage: usize,
    /// 分配次数
    pub allocation_count: u64,
    /// 释放次数
    pub deallocation_count: u64,
}

impl MemoryStats {
    /// 创建新的内存统计实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 计算内存使用率
    ///
    /// # 返回
    /// 内存使用率（0.0 到 1.0）
    pub fn usage_rate(&self) -> f64 {
        if self.total_size == 0 {
            return 0.0;
        }
        self.used_size as f64 / self.total_size as f64
    }

    /// 更新碎片率
    ///
    /// # 参数
    /// - `free_blocks`: 空闲内存块列表
    pub fn update_fragmentation(&mut self, free_blocks: &[MemoryBlock]) {
        if free_blocks.is_empty() || self.available_size == 0 {
            self.fragmentation_rate = 0.0;
            return;
        }

        let max_free_block = free_blocks.iter().map(|b| b.size).max().unwrap_or(0);
        let external_fragmentation = 1.0 - (max_free_block as f64 / self.available_size as f64);
        self.fragmentation_rate = external_fragmentation.max(0.0).min(1.0);
    }
}

/// 内存池管理器
///
/// 提供高效的内存池分配和管理功能，支持碎片整理。
pub struct MemoryPool {
    /// 内存块的起始地址
    base_address: usize,
    /// 内存池总大小
    total_size: usize,
    /// 所有内存块（按地址排序）
    blocks: BTreeMap<usize, MemoryBlock>,
    /// 内存统计信息
    stats: MemoryStats,
    /// 最小分配单位（字节对齐）
    alignment: usize,
}

impl MemoryPool {
    /// 创建新的内存池
    ///
    /// # 参数
    /// - `size`: 内存池总大小（字节）
    ///
    /// # 返回
    /// 新创建的内存池实例
    pub fn new(size: usize) -> Self {
        let alignment = 16;
        let aligned_size = Self::align_size(size, alignment);
        let base_address = Self::allocate_raw(aligned_size);

        let mut blocks = BTreeMap::new();
        blocks.insert(base_address, MemoryBlock::new(base_address, aligned_size));

        let stats =
            MemoryStats { total_size: aligned_size, available_size: aligned_size, free_blocks: 1, ..MemoryStats::default() };

        Self { base_address, total_size: aligned_size, blocks, stats, alignment }
    }

    /// 对齐大小
    fn align_size(size: usize, alignment: usize) -> usize {
        (size + alignment - 1) & !(alignment - 1)
    }

    /// 分配原始内存
    fn allocate_raw(size: usize) -> usize {
        let layout = Layout::from_size_align(size, 16).expect("Invalid layout");
        unsafe {
            let ptr = alloc(layout);
            if ptr.is_null() {
                panic!("Failed to allocate memory pool");
            }
            ptr as usize
        }
    }

    /// 释放原始内存
    unsafe fn deallocate_raw(address: usize, size: usize) {
        let layout = Layout::from_size_align(size, 16).expect("Invalid layout");
        unsafe { dealloc(address as *mut u8, layout) };
    }

    /// 分配内存
    ///
    /// 使用改进的最佳适配算法分配指定大小的内存块
    ///
    /// # 参数
    /// - `size`: 需要分配的内存大小
    ///
    /// # 返回
    /// 分配的内存地址，如果分配失败返回 None
    pub fn allocate(&mut self, size: usize) -> Option<usize> {
        let aligned_size = Self::align_size(size, self.alignment);
        let min_size = aligned_size.max(16);

        // 快速路径：检查是否有刚好匹配的块
        let mut matching_addr: Option<usize> = None;
        for (addr, block) in &self.blocks {
            if !block.is_allocated && block.size == min_size {
                matching_addr = Some(*addr);
                break;
            }
        }

        if let Some(addr) = matching_addr {
            let block = self.blocks.get_mut(&addr).unwrap();
            block.is_allocated = true;
            self.update_stats_on_allocate(min_size);
            return Some(addr);
        }

        // 最佳适配算法
        let best_fit = self
            .blocks
            .iter()
            .filter(|(_, block)| block.can_fit(min_size))
            .min_by_key(|(_, block)| block.size)
            .map(|(addr, _)| *addr);

        let block_address = best_fit?;

        let new_block = {
            let block = self.blocks.get_mut(&block_address)?;
            block.is_allocated = true;

            if block.size > min_size + 16 { block.split(min_size) } else { None }
        };

        if let Some(new_block) = new_block {
            self.blocks.insert(new_block.address, new_block);
        }

        let allocated_size = self.blocks.get(&block_address)?.size;
        self.update_stats_on_allocate(allocated_size);

        Some(block_address)
    }

    /// 释放内存
    ///
    /// 释放指定地址的内存块，并尝试合并相邻的空闲块
    ///
    /// # 参数
    /// - `address`: 要释放的内存地址
    ///
    /// # 返回
    /// 如果成功释放返回 true，否则返回 false
    pub fn deallocate(&mut self, address: usize) -> bool {
        let block = match self.blocks.get_mut(&address) {
            Some(b) => b,
            None => return false,
        };

        if !block.is_allocated {
            return false;
        }

        let freed_size = block.size;
        block.is_allocated = false;

        self.merge_adjacent_free_blocks(address);
        self.update_stats_on_deallocate(freed_size);

        true
    }

    /// 合并相邻的空闲块
    fn merge_adjacent_free_blocks(&mut self, address: usize) {
        let current_block = match self.blocks.get(&address) {
            Some(b) if !b.is_allocated => b.clone(),
            _ => return,
        };

        // 合并下一个块
        let next_address = address + current_block.size;
        if let Some(next_block) = self.blocks.get(&next_address) {
            if !next_block.is_allocated {
                let merged_size = current_block.size + next_block.size;
                self.blocks.remove(&next_address);
                if let Some(current) = self.blocks.get_mut(&address) {
                    current.size = merged_size;
                }
            }
        }

        // 合并前一个块
        if let Some((prev_addr, prev_block)) = self.blocks.range(..address).next_back() {
            if !prev_block.is_allocated && prev_block.address + prev_block.size == address {
                let merged_size = prev_block.size + current_block.size;
                let prev_addr = *prev_addr;
                self.blocks.remove(&address);
                if let Some(prev) = self.blocks.get_mut(&prev_addr) {
                    prev.size = merged_size;
                }
            }
        }
    }

    /// 碎片整理
    ///
    /// 通过移动已分配的块来减少内存碎片
    ///
    /// # 返回
    /// 整理后的碎片率
    pub fn defragment(&mut self) -> f64 {
        // 快速路径：如果碎片率不高，直接返回
        let current_stats = self.get_stats();
        if current_stats.fragmentation_rate < 0.2 {
            return current_stats.fragmentation_rate;
        }

        // 收集所有已分配的块
        let mut allocated_blocks: Vec<(usize, usize)> =
            self.blocks.iter().filter(|(_, b)| b.is_allocated).map(|(addr, b)| (*addr, b.size)).collect();

        if allocated_blocks.is_empty() {
            // 如果没有已分配的块，直接清空并创建一个大的空闲块
            self.blocks.clear();
            self.blocks.insert(self.base_address, MemoryBlock::new(self.base_address, self.total_size));
            self.update_stats();
            return 0.0;
        }

        // 按地址排序
        allocated_blocks.sort_by_key(|(addr, _)| *addr);

        // 重建内存块
        self.blocks.clear();

        let mut current_address = self.base_address;
        for (_, size) in allocated_blocks {
            let mut block = MemoryBlock::new(current_address, size);
            block.is_allocated = true;
            self.blocks.insert(current_address, block);
            current_address += size;
        }

        // 添加剩余的空闲块
        if current_address < self.base_address + self.total_size {
            let free_size = self.base_address + self.total_size - current_address;
            let free_block = MemoryBlock::new(current_address, free_size);
            self.blocks.insert(current_address, free_block);
        }

        // 更新统计信息
        self.update_stats();
        self.stats.fragmentation_rate
    }

    /// 更新统计信息
    fn update_stats(&mut self) {
        let free_blocks: Vec<&MemoryBlock> = self.blocks.values().filter(|b| !b.is_allocated).collect();

        self.stats.free_blocks = free_blocks.len();
        self.stats.available_size = free_blocks.iter().map(|b| b.size).sum();
        self.stats.used_size = self.total_size - self.stats.available_size;
        self.stats.allocated_blocks = self.blocks.values().filter(|b| b.is_allocated).count();
        self.stats.update_fragmentation(&free_blocks.iter().map(|&b| b.clone()).collect::<Vec<_>>());
    }

    /// 分配时更新统计
    fn update_stats_on_allocate(&mut self, size: usize) {
        self.stats.used_size += size;
        self.stats.available_size -= size;
        self.stats.allocated_blocks += 1;
        self.stats.allocation_count += 1;

        if self.stats.used_size > self.stats.peak_usage {
            self.stats.peak_usage = self.stats.used_size;
        }

        // 减少碎片率更新频率，每10次分配更新一次
        if self.stats.allocation_count % 10 == 0 {
            self.update_stats();
        }
    }

    /// 释放时更新统计
    fn update_stats_on_deallocate(&mut self, size: usize) {
        self.stats.used_size -= size;
        self.stats.available_size += size;
        self.stats.allocated_blocks -= 1;
        self.stats.deallocation_count += 1;

        // 减少碎片率更新频率，每10次释放更新一次
        if self.stats.deallocation_count % 10 == 0 {
            self.update_stats();
        }
    }

    /// 获取内存统计信息
    ///
    /// # 返回
    /// 当前内存池的统计信息快照
    pub fn get_stats(&self) -> MemoryStats {
        self.stats.clone()
    }

    /// 获取内存池总大小
    pub fn total_size(&self) -> usize {
        self.total_size
    }

    /// 获取可用内存大小
    pub fn available_size(&self) -> usize {
        self.stats.available_size
    }

    /// 获取已用内存大小
    pub fn used_size(&self) -> usize {
        self.stats.used_size
    }

    /// 检查地址是否在内存池范围内
    pub fn contains(&self, address: usize) -> bool {
        address >= self.base_address && address < self.base_address + self.total_size
    }
}

impl Drop for MemoryPool {
    fn drop(&mut self) {
        unsafe {
            Self::deallocate_raw(self.base_address, self.total_size);
        }
    }
}

/// WASI 内存管理器
///
/// 提供 WASM 环境下的高级内存管理功能。
pub struct WasiMemory {
    /// 主内存池
    main_pool: MemoryPool,
    /// 临时内存池（用于短期分配）
    temp_pool: Option<MemoryPool>,
    /// 是否启用碎片整理
    auto_defrag: bool,
    /// 碎片整理阈值
    defrag_threshold: f64,
}

impl WasiMemory {
    /// 创建新的 WASI 内存管理器
    ///
    /// # 参数
    /// - `main_size`: 主内存池大小（字节）
    ///
    /// # 返回
    /// 新创建的 WASI 内存管理器实例
    pub fn new(main_size: usize) -> Self {
        Self { main_pool: MemoryPool::new(main_size), temp_pool: None, auto_defrag: true, defrag_threshold: 0.3 }
    }

    /// 创建带有临时内存池的 WASI 内存管理器
    ///
    /// # 参数
    /// - `main_size`: 主内存池大小（字节）
    /// - `temp_size`: 临时内存池大小（字节）
    ///
    /// # 返回
    /// 新创建的 WASI 内存管理器实例
    pub fn with_temp_pool(main_size: usize, temp_size: usize) -> Self {
        Self {
            main_pool: MemoryPool::new(main_size),
            temp_pool: Some(MemoryPool::new(temp_size)),
            auto_defrag: true,
            defrag_threshold: 0.3,
        }
    }

    /// 设置自动碎片整理
    ///
    /// # 参数
    /// - `enabled`: 是否启用
    /// - `threshold`: 碎片率阈值（0.0 到 1.0）
    pub fn set_auto_defrag(&mut self, enabled: bool, threshold: f64) {
        self.auto_defrag = enabled;
        self.defrag_threshold = threshold.clamp(0.0, 1.0);
    }

    /// 分配内存
    ///
    /// # 参数
    /// - `size`: 需要分配的内存大小
    /// - `is_temp`: 是否为临时分配
    ///
    /// # 返回
    /// 分配的内存指针，如果分配失败返回 None
    pub fn allocate(&mut self, size: usize, is_temp: bool) -> Option<NonNull<u8>> {
        let pool = if is_temp { self.temp_pool.as_mut()? } else { &mut self.main_pool };

        let address = pool.allocate(size)?;
        NonNull::new(address as *mut u8)
    }

    /// 释放内存
    ///
    /// # 参数
    /// - `ptr`: 要释放的内存指针
    /// - `is_temp`: 是否来自临时内存池
    ///
    /// # 返回
    /// 如果成功释放返回 true
    pub fn deallocate(&mut self, ptr: NonNull<u8>, is_temp: bool) -> bool {
        let address = ptr.as_ptr() as usize;

        let pool = if is_temp {
            match &mut self.temp_pool {
                Some(p) => p,
                None => return false,
            }
        }
        else {
            &mut self.main_pool
        };

        pool.deallocate(address)
    }

    /// 执行碎片整理
    ///
    /// 如果碎片率超过阈值，自动执行碎片整理
    ///
    /// # 返回
    /// 整理后的碎片率
    pub fn defragment(&mut self) -> f64 {
        self.main_pool.defragment()
    }

    /// 检查并执行自动碎片整理
    fn check_auto_defrag(&mut self) {
        if !self.auto_defrag {
            return;
        }

        let stats = self.main_pool.get_stats();
        if stats.fragmentation_rate > self.defrag_threshold {
            self.defragment();
        }
    }

    /// 获取主内存池统计信息
    ///
    /// # 返回
    /// 主内存池的统计信息
    pub fn get_stats(&self) -> MemoryStats {
        self.main_pool.get_stats()
    }

    /// 获取临时内存池统计信息
    ///
    /// # 返回
    /// 临时内存池的统计信息，如果没有临时池返回 None
    pub fn get_temp_stats(&self) -> Option<MemoryStats> {
        self.temp_pool.as_ref().map(|p| p.get_stats())
    }

    /// 获取总内存使用情况
    ///
    /// # 返回
    /// 所有内存池的总统计信息
    pub fn get_total_stats(&self) -> MemoryStats {
        let main_stats = self.main_pool.get_stats();

        if let Some(temp_pool) = &self.temp_pool {
            let temp_stats = temp_pool.get_stats();
            MemoryStats {
                total_size: main_stats.total_size + temp_stats.total_size,
                used_size: main_stats.used_size + temp_stats.used_size,
                available_size: main_stats.available_size + temp_stats.available_size,
                allocated_blocks: main_stats.allocated_blocks + temp_stats.allocated_blocks,
                free_blocks: main_stats.free_blocks + temp_stats.free_blocks,
                fragmentation_rate: (main_stats.fragmentation_rate + temp_stats.fragmentation_rate) / 2.0,
                peak_usage: main_stats.peak_usage.max(temp_stats.peak_usage),
                allocation_count: main_stats.allocation_count + temp_stats.allocation_count,
                deallocation_count: main_stats.deallocation_count + temp_stats.deallocation_count,
            }
        }
        else {
            main_stats
        }
    }

    /// 重置临时内存池
    ///
    /// 清空临时内存池中的所有分配
    pub fn reset_temp_pool(&mut self) {
        if let Some(temp_pool) = &mut self.temp_pool {
            let size = temp_pool.total_size();
            *temp_pool = MemoryPool::new(size);
        }
    }

    /// 获取主内存池大小
    pub fn main_pool_size(&self) -> usize {
        self.main_pool.total_size()
    }

    /// 获取可用内存大小
    pub fn available_memory(&self) -> usize {
        self.main_pool.available_size()
    }
}

unsafe impl Send for WasiMemory {}

unsafe impl Sync for WasiMemory {}
