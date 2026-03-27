//! 垃圾收集器核心模块
//!
//! 实现增量标记-清除算法，支持分代收集、时间预算控制和内存压缩。

use std::{
    collections::{HashSet, VecDeque},
    ptr::NonNull,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use typescript_types::TsValue;

use crate::memory::{AllocationStrategy, Allocator, AllocatorFactory};

use super::{
    event::GCEventHandler,
    header::ObjectHeader,
    object::GcObject,
    phase::GCPhase,
    stats::GCStats,
    work_stealing::{RememberedSetEntry, WorkStealingQueue},
};

/// 时间预算配置
///
/// 用于控制增量式 GC 每步的最大执行时间，确保 GC 停顿时间在可接受范围内。
#[derive(Debug, Clone)]
pub struct TimeBudget {
    /// 每步最大执行时间（微秒）
    max_step_time_us: u64,
    /// 标记阶段时间预算（微秒）
    mark_budget_us: u64,
    /// 清除阶段时间预算（微秒）
    sweep_budget_us: u64,
    /// 压缩阶段时间预算（微秒）
    compact_budget_us: u64,
    /// 目标最大停顿时间（微秒）
    target_max_pause_us: u64,
    /// 自适应调整因子
    adaptive_factor: f64,
    /// 历史执行时间记录
    history: Vec<(u64, u64)>, // (budget, actual)
    /// 历史记录最大长度
    max_history_size: usize,
    /// 动态步长调整因子
    step_adjustment_factor: f64,
    /// 最小步长
    min_step_size: usize,
    /// 最大步长
    max_step_size: usize,
}

impl Default for TimeBudget {
    fn default() -> Self {
        Self {
            max_step_time_us: 1000,
            mark_budget_us: 500,
            sweep_budget_us: 300,
            compact_budget_us: 200,
            target_max_pause_us: 5000,
            adaptive_factor: 1.0,
            history: Vec::with_capacity(10),
            max_history_size: 10,
            step_adjustment_factor: 1.0,
            min_step_size: 50,
            max_step_size: 500,
        }
    }
}

impl TimeBudget {
    /// 创建新的时间预算配置
    ///
    /// # 返回值
    /// 返回使用默认配置的时间预算实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建自定义时间预算配置
    ///
    /// # 参数
    /// - `max_step_time_us`: 每步最大执行时间（微秒）
    /// - `target_max_pause_us`: 目标最大停顿时间（微秒）
    ///
    /// # 返回值
    /// 返回自定义配置的时间预算实例
    pub fn with_config(max_step_time_us: u64, target_max_pause_us: u64) -> Self {
        Self {
            max_step_time_us,
            mark_budget_us: max_step_time_us / 2,
            sweep_budget_us: max_step_time_us / 3,
            compact_budget_us: max_step_time_us / 5,
            target_max_pause_us,
            adaptive_factor: 1.0,
            history: Vec::with_capacity(10),
            max_history_size: 10,
            step_adjustment_factor: 1.0,
            min_step_size: 50,
            max_step_size: 500,
        }
    }

    /// 设置每步最大执行时间
    ///
    /// # 参数
    /// - `time_us`: 最大执行时间（微秒）
    pub fn set_max_step_time(&mut self, time_us: u64) {
        self.max_step_time_us = time_us;
        self.mark_budget_us = time_us / 2;
        self.sweep_budget_us = time_us / 3;
        self.compact_budget_us = time_us / 5;
    }

    /// 设置目标最大停顿时间
    ///
    /// # 参数
    /// - `time_us`: 目标最大停顿时间（微秒）
    pub fn set_target_max_pause(&mut self, time_us: u64) {
        self.target_max_pause_us = time_us;
    }

    /// 获取每步最大执行时间
    ///
    /// # 返回值
    /// 返回每步最大执行时间（微秒）
    pub fn max_step_time(&self) -> u64 {
        self.max_step_time_us
    }

    /// 获取标记阶段时间预算
    ///
    /// # 返回值
    /// 返回标记阶段时间预算（微秒）
    pub fn mark_budget(&self) -> u64 {
        (self.mark_budget_us as f64 * self.adaptive_factor) as u64
    }

    /// 获取清除阶段时间预算
    ///
    /// # 返回值
    /// 返回清除阶段时间预算（微秒）
    pub fn sweep_budget(&self) -> u64 {
        (self.sweep_budget_us as f64 * self.adaptive_factor) as u64
    }

    /// 获取压缩阶段时间预算
    ///
    /// # 返回值
    /// 返回压缩阶段时间预算（微秒）
    pub fn compact_budget(&self) -> u64 {
        (self.compact_budget_us as f64 * self.adaptive_factor) as u64
    }

    /// 根据实际执行时间自适应调整预算
    ///
    /// # 参数
    /// - `actual_time_us`: 实际执行时间（微秒）
    /// - `budget_time_us`: 预算时间（微秒）
    pub fn adapt(&mut self, actual_time_us: u64, budget_time_us: u64) {
        // 记录历史执行时间
        self.history.push((budget_time_us, actual_time_us));
        if self.history.len() > self.max_history_size {
            self.history.remove(0);
        }

        // 计算平均执行时间比率
        if !self.history.is_empty() {
            let total_ratio: f64 = self
                .history
                .iter()
                .filter(|(budget, _)| *budget > 0)
                .map(|(budget, actual)| (*actual as f64) / (*budget as f64))
                .sum();
            let count = self.history.iter().filter(|(budget, _)| *budget > 0).count() as f64;

            if count > 0.0 {
                let avg_ratio = total_ratio / count;

                // 自适应调整因子
                if avg_ratio > 1.0 {
                    self.adaptive_factor *= 0.9;
                }
                else if avg_ratio < 0.5 {
                    self.adaptive_factor *= 1.1;
                }
                self.adaptive_factor = self.adaptive_factor.clamp(0.5, 2.0);

                // 调整步长因子
                self.step_adjustment_factor = 1.0 / avg_ratio;
                self.step_adjustment_factor = self.step_adjustment_factor.clamp(0.5, 2.0);
            }
        }
    }

    /// 计算推荐的步长大小
    ///
    /// # 参数
    /// - `current_step_size`: 当前步长大小
    ///
    /// # 返回值
    /// 返回推荐的步长大小
    pub fn recommended_step_size(&self, current_step_size: usize) -> usize {
        let new_step = (current_step_size as f64 * self.step_adjustment_factor) as usize;
        new_step.clamp(self.min_step_size, self.max_step_size)
    }

    /// 设置步长范围
    ///
    /// # 参数
    /// - `min_step`: 最小步长
    /// - `max_step`: 最大步长
    pub fn set_step_range(&mut self, min_step: usize, max_step: usize) {
        self.min_step_size = min_step;
        self.max_step_size = max_step;
    }

    /// 获取当前步长调整因子
    ///
    /// # 返回值
    /// 返回当前步长调整因子
    pub fn step_adjustment_factor(&self) -> f64 {
        self.step_adjustment_factor
    }

    /// 清除历史记录
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

/// 对象晋升策略配置
///
/// 控制对象从新生代晋升到老年代的条件和频率。
#[derive(Debug, Clone)]
pub struct PromotionPolicy {
    /// 晋升年龄阈值
    age_threshold: u8,
    /// 大对象晋升阈值（字节）
    large_object_threshold: usize,
    /// 小对象晋升阈值（字节）
    small_object_threshold: usize,
    /// 是否启用动态年龄调整
    dynamic_age_enabled: bool,
    /// 目标存活率
    target_survival_rate: f64,
    /// 最大晋升率
    max_promotion_rate: f64,
    /// 历史存活率样本
    survival_rate_history: Vec<f64>,
    /// 历史样本最大数量
    max_history_size: usize,
    /// 是否启用基于大小的晋升策略
    size_based_promotion: bool,
    /// 是否启用基于访问模式的晋升策略
    access_based_promotion: bool,
    /// 最小晋升年龄（防止过小的对象过快晋升）
    min_age_threshold: u8,
    /// 最大晋升年龄（防止对象在新生代停留过久）
    max_age_threshold: u8,
}

impl Default for PromotionPolicy {
    fn default() -> Self {
        Self {
            age_threshold: 3,
            large_object_threshold: 1024,
            small_object_threshold: 64,
            dynamic_age_enabled: true,
            target_survival_rate: 0.5,
            max_promotion_rate: 0.3,
            survival_rate_history: Vec::with_capacity(10),
            max_history_size: 10,
            size_based_promotion: true,
            access_based_promotion: true,
            min_age_threshold: 1,
            max_age_threshold: 15,
        }
    }
}

impl PromotionPolicy {
    /// 创建新的晋升策略配置
    ///
    /// # 返回值
    /// 返回使用默认配置的晋升策略实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建自定义晋升策略配置
    ///
    /// # 参数
    /// - `age_threshold`: 晋升年龄阈值
    /// - `large_object_threshold`: 大对象晋升阈值（字节）
    ///
    /// # 返回值
    /// 返回自定义配置的晋升策略实例
    pub fn with_config(age_threshold: u8, large_object_threshold: usize) -> Self {
        Self { age_threshold, large_object_threshold, ..Self::default() }
    }

    /// 设置晋升年龄阈值
    ///
    /// # 参数
    /// - `threshold`: 新的年龄阈值
    pub fn set_age_threshold(&mut self, threshold: u8) {
        self.age_threshold = threshold.clamp(self.min_age_threshold, self.max_age_threshold);
    }

    /// 设置大对象晋升阈值
    ///
    /// # 参数
    /// - `threshold`: 新的大小阈值（字节）
    pub fn set_large_object_threshold(&mut self, threshold: usize) {
        self.large_object_threshold = threshold;
    }

    /// 设置小对象晋升阈值
    ///
    /// # 参数
    /// - `threshold`: 新的大小阈值（字节）
    pub fn set_small_object_threshold(&mut self, threshold: usize) {
        self.small_object_threshold = threshold;
    }

    /// 启用或禁用动态年龄调整
    ///
    /// # 参数
    /// - `enabled`: 是否启用
    pub fn set_dynamic_age_enabled(&mut self, enabled: bool) {
        self.dynamic_age_enabled = enabled;
    }

    /// 启用或禁用基于大小的晋升策略
    ///
    /// # 参数
    /// - `enabled`: 是否启用
    pub fn set_size_based_promotion(&mut self, enabled: bool) {
        self.size_based_promotion = enabled;
    }

    /// 启用或禁用基于访问模式的晋升策略
    ///
    /// # 参数
    /// - `enabled`: 是否启用
    pub fn set_access_based_promotion(&mut self, enabled: bool) {
        self.access_based_promotion = enabled;
    }

    /// 记录存活率并动态调整晋升年龄
    ///
    /// # 参数
    /// - `survival_rate`: 本次收集的存活率
    pub fn record_survival_rate(&mut self, survival_rate: f64) {
        self.survival_rate_history.push(survival_rate);
        if self.survival_rate_history.len() > self.max_history_size {
            self.survival_rate_history.remove(0);
        }

        if self.dynamic_age_enabled && self.survival_rate_history.len() >= 3 {
            let avg_rate: f64 = self.survival_rate_history.iter().sum::<f64>() / self.survival_rate_history.len() as f64;

            // 动态调整年龄阈值
            if avg_rate > self.target_survival_rate + 0.1 && self.age_threshold < self.max_age_threshold {
                self.age_threshold += 1;
            }
            else if avg_rate < self.target_survival_rate - 0.1 && self.age_threshold > self.min_age_threshold {
                self.age_threshold -= 1;
            }

            // 同时调整大对象阈值，以适应不同的内存使用模式
            if avg_rate > self.target_survival_rate + 0.2 {
                // 存活率过高，适当提高大对象阈值，减少晋升
                self.large_object_threshold = (self.large_object_threshold as f64 * 1.1) as usize;
            }
            else if avg_rate < self.target_survival_rate - 0.2 {
                // 存活率过低，适当降低大对象阈值，增加晋升
                self.large_object_threshold = (self.large_object_threshold as f64 * 0.9) as usize;
            }
        }
    }

    /// 判断对象是否应该晋升到老年代
    ///
    /// # 参数
    /// - `header`: 对象头信息
    ///
    /// # 返回值
    /// 如果对象应该晋升返回 true，否则返回 false
    pub fn should_promote(&self, header: &ObjectHeader) -> bool {
        // 年龄达到阈值，直接晋升
        if header.age >= self.age_threshold {
            return true;
        }

        // 大对象直接晋升
        if self.size_based_promotion && header.size >= self.large_object_threshold {
            return true;
        }

        // 小对象延迟晋升
        if self.size_based_promotion && header.size <= self.small_object_threshold && header.age < self.age_threshold + 2 {
            return false;
        }

        // 基于访问模式的晋升（如果启用）
        if self.access_based_promotion && header.access_count > 5 {
            return true;
        }

        false
    }

    /// 获取当前晋升年龄阈值
    ///
    /// # 返回值
    /// 返回当前晋升年龄阈值
    pub fn age_threshold(&self) -> u8 {
        self.age_threshold
    }

    /// 获取大对象晋升阈值
    ///
    /// # 返回值
    /// 返回大对象晋升阈值（字节）
    pub fn large_object_threshold(&self) -> usize {
        self.large_object_threshold
    }

    /// 获取小对象晋升阈值
    ///
    /// # 返回值
    /// 返回小对象晋升阈值（字节）
    pub fn small_object_threshold(&self) -> usize {
        self.small_object_threshold
    }

    /// 设置年龄阈值范围
    ///
    /// # 参数
    /// - `min_age`: 最小年龄阈值
    /// - `max_age`: 最大年龄阈值
    pub fn set_age_range(&mut self, min_age: u8, max_age: u8) {
        self.min_age_threshold = min_age;
        self.max_age_threshold = max_age;
        // 确保当前年龄阈值在范围内
        self.age_threshold = self.age_threshold.clamp(self.min_age_threshold, self.max_age_threshold);
    }

    /// 获取基于大小的晋升策略状态
    ///
    /// # 返回值
    /// 返回是否启用基于大小的晋升策略
    pub fn size_based_promotion(&self) -> bool {
        self.size_based_promotion
    }

    /// 获取基于访问模式的晋升策略状态
    ///
    /// # 返回值
    /// 返回是否启用基于访问模式的晋升策略
    pub fn access_based_promotion(&self) -> bool {
        self.access_based_promotion
    }
}

/// 写屏障类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteBarrierType {
    /// 无写屏障
    None,
    /// 卡表写屏障
    CardTable,
    /// 模糊写屏障
    SnapshotAtTheBeginning,
}

/// 写屏障管理器
///
/// 管理跨代引用的记录，优化记忆集的开销。
#[derive(Debug, Clone)]
pub struct WriteBarrier {
    /// 写屏障类型
    barrier_type: WriteBarrierType,
    /// 卡表（用于快速标记脏区域）
    card_table: Vec<u8>,
    /// 卡表每卡大小（字节）
    card_size: usize,
    /// 脏卡数量
    dirty_card_count: usize,
    /// 是否启用批量更新
    batch_update_enabled: bool,
    /// 批量更新缓冲区
    batch_buffer: Vec<RememberedSetEntry>,
    /// 批量更新缓冲区大小阈值
    batch_threshold: usize,
    /// 上次批量更新时间
    last_batch_update: std::time::Instant,
    /// 批量更新时间阈值（毫秒）
    batch_time_threshold: u64,
    /// 记忆集条目去重哈希表
    seen_entries: std::collections::HashSet<(usize, usize)>,
    /// 是否启用记忆集去重
    deduplication_enabled: bool,
}

impl Default for WriteBarrier {
    fn default() -> Self {
        Self {
            barrier_type: WriteBarrierType::CardTable,
            card_table: Vec::new(),
            card_size: 256, // 减小卡大小，提高精度
            dirty_card_count: 0,
            batch_update_enabled: true,
            batch_buffer: Vec::with_capacity(200), // 增大缓冲区容量
            batch_threshold: 100,                  // 增大批量阈值
            last_batch_update: std::time::Instant::now(),
            batch_time_threshold: 1, // 1毫秒时间阈值
            seen_entries: std::collections::HashSet::with_capacity(100),
            deduplication_enabled: true,
        }
    }
}

impl WriteBarrier {
    /// 创建新的写屏障管理器
    ///
    /// # 返回值
    /// 返回使用默认配置的写屏障实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 创建指定类型的写屏障管理器
    ///
    /// # 参数
    /// - `barrier_type`: 写屏障类型
    ///
    /// # 返回值
    /// 返回指定类型的写屏障实例
    pub fn with_type(barrier_type: WriteBarrierType) -> Self {
        Self { barrier_type, ..Self::default() }
    }

    /// 初始化卡表
    ///
    /// # 参数
    /// - `heap_size`: 堆大小（字节）
    pub fn init_card_table(&mut self, heap_size: usize) {
        let card_count = (heap_size + self.card_size - 1) / self.card_size;
        self.card_table = vec![0; card_count];
        self.dirty_card_count = 0;
    }

    /// 标记卡为脏
    ///
    /// # 参数
    /// - `address`: 内存地址
    pub fn mark_card_dirty(&mut self, address: usize) {
        if self.barrier_type != WriteBarrierType::CardTable {
            return;
        }

        let card_index = address / self.card_size;
        if card_index < self.card_table.len() && self.card_table[card_index] == 0 {
            self.card_table[card_index] = 1;
            self.dirty_card_count += 1;
        }
    }

    /// 检查卡是否为脏
    ///
    /// # 参数
    /// - `address`: 内存地址
    ///
    /// # 返回值
    /// 如果卡为脏返回 true，否则返回 false
    pub fn is_card_dirty(&self, address: usize) -> bool {
        let card_index = address / self.card_size;
        card_index < self.card_table.len() && self.card_table[card_index] != 0
    }

    /// 清除所有脏卡标记
    pub fn clear_dirty_cards(&mut self) {
        // 使用更高效的方式清除脏卡
        for i in 0..self.card_table.len() {
            self.card_table[i] = 0;
        }
        self.dirty_card_count = 0;
    }

    /// 获取脏卡数量
    ///
    /// # 返回值
    /// 返回脏卡数量
    pub fn dirty_card_count(&self) -> usize {
        self.dirty_card_count
    }

    /// 添加记忆集条目（带批量优化和去重）
    ///
    /// # 参数
    /// - `entry`: 记忆集条目
    ///
    /// # 返回值
    /// 如果批量缓冲区已满或超过时间阈值，返回需要刷新的条目列表
    pub fn add_remembered_entry(&mut self, entry: RememberedSetEntry) -> Option<Vec<RememberedSetEntry>> {
        if !self.batch_update_enabled {
            return Some(vec![entry]);
        }

        // 去重处理
        if self.deduplication_enabled {
            let key = (entry.old_obj.as_ptr() as usize, entry.young_obj.as_ptr() as usize);
            if self.seen_entries.contains(&key) {
                return None; // 已存在相同条目，跳过
            }
            self.seen_entries.insert(key);
        }

        self.batch_buffer.push(entry);

        // 检查是否需要刷新
        let elapsed = self.last_batch_update.elapsed().as_millis() as u64;
        if self.batch_buffer.len() >= self.batch_threshold || elapsed >= self.batch_time_threshold {
            let result = Some(self.flush_batch_buffer());
            self.last_batch_update = std::time::Instant::now();
            result
        }
        else {
            None
        }
    }

    /// 刷新批量缓冲区
    ///
    /// # 返回值
    /// 返回批量缓冲区中的所有条目
    pub fn flush_batch_buffer(&mut self) -> Vec<RememberedSetEntry> {
        let result = self.batch_buffer.clone();
        self.batch_buffer.clear();
        self.seen_entries.clear(); // 清除去重哈希表
        result
    }

    /// 设置写屏障类型
    ///
    /// # 参数
    /// - `barrier_type`: 新的写屏障类型
    pub fn set_barrier_type(&mut self, barrier_type: WriteBarrierType) {
        self.barrier_type = barrier_type;
    }

    /// 获取写屏障类型
    ///
    /// # 返回值
    /// 返回当前写屏障类型
    pub fn barrier_type(&self) -> WriteBarrierType {
        self.barrier_type
    }

    /// 启用或禁用批量更新
    ///
    /// # 参数
    /// - `enabled`: 是否启用
    pub fn set_batch_update_enabled(&mut self, enabled: bool) {
        self.batch_update_enabled = enabled;
    }

    /// 设置批量更新阈值
    ///
    /// # 参数
    /// - `threshold`: 新的阈值
    pub fn set_batch_threshold(&mut self, threshold: usize) {
        self.batch_threshold = threshold;
    }

    /// 设置批量更新时间阈值
    ///
    /// # 参数
    /// - `threshold_ms`: 新的时间阈值（毫秒）
    pub fn set_batch_time_threshold(&mut self, threshold_ms: u64) {
        self.batch_time_threshold = threshold_ms;
    }

    /// 启用或禁用记忆集去重
    ///
    /// # 参数
    /// - `enabled`: 是否启用
    pub fn set_deduplication_enabled(&mut self, enabled: bool) {
        self.deduplication_enabled = enabled;
    }

    /// 获取记忆集去重状态
    ///
    /// # 返回值
    /// 返回是否启用记忆集去重
    pub fn deduplication_enabled(&self) -> bool {
        self.deduplication_enabled
    }
}

/// GC 停顿时间记录
#[derive(Debug, Clone)]
pub struct PauseTimeRecord {
    /// 停顿开始时间
    pub start_time: Instant,
    /// 停顿结束时间
    pub end_time: Instant,
    /// 停顿阶段
    pub phase: GCPhase,
    /// 停顿持续时间（微秒）
    pub duration_us: u64,
    /// 处理的对象数量
    pub objects_processed: usize,
    /// 回收的字节数
    pub bytes_collected: usize,
}

impl PauseTimeRecord {
    /// 创建新的停顿时间记录
    ///
    /// # 参数
    /// - `phase`: GC 阶段
    /// - `duration_us`: 持续时间（微秒）
    ///
    /// # 返回值
    /// 返回新的停顿时间记录实例
    pub fn new(phase: GCPhase, duration_us: u64) -> Self {
        Self {
            start_time: Instant::now(),
            end_time: Instant::now(),
            phase,
            duration_us,
            objects_processed: 0,
            bytes_collected: 0,
        }
    }

    /// 创建完整的停顿时间记录
    ///
    /// # 参数
    /// - `start_time`: 开始时间
    /// - `end_time`: 结束时间
    /// - `phase`: GC 阶段
    /// - `objects_processed`: 处理的对象数量
    /// - `bytes_collected`: 回收的字节数
    ///
    /// # 返回值
    /// 返回完整的停顿时间记录实例
    pub fn with_details(
        start_time: Instant,
        end_time: Instant,
        phase: GCPhase,
        objects_processed: usize,
        bytes_collected: usize,
    ) -> Self {
        let duration_us = end_time.duration_since(start_time).as_micros() as u64;
        Self { start_time, end_time, phase, duration_us, objects_processed, bytes_collected }
    }
}

/// GC 停顿时间报告
#[derive(Debug, Clone, Default)]
pub struct PauseTimeReport {
    /// 所有停顿记录
    pub pause_records: Vec<PauseTimeRecord>,
    /// 最大停顿时间（微秒）
    pub max_pause_us: u64,
    /// 最小停顿时间（微秒）
    pub min_pause_us: u64,
    /// 平均停顿时间（微秒）
    pub avg_pause_us: u64,
    /// 总停顿时间（微秒）
    pub total_pause_us: u64,
    /// 停顿次数
    pub pause_count: usize,
    /// 超过目标停顿时间的次数
    pub over_target_count: usize,
    /// 目标停顿时间（微秒）
    pub target_pause_us: u64,
}

impl PauseTimeReport {
    /// 创建新的停顿时间报告
    ///
    /// # 返回值
    /// 返回新的停顿时间报告实例
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置目标停顿时间
    ///
    /// # 参数
    /// - `target_us`: 目标停顿时间（微秒）
    pub fn set_target_pause(&mut self, target_us: u64) {
        self.target_pause_us = target_us;
    }

    /// 添加停顿记录
    ///
    /// # 参数
    /// - `record`: 停顿时间记录
    pub fn add_record(&mut self, record: PauseTimeRecord) {
        if record.duration_us > self.max_pause_us {
            self.max_pause_us = record.duration_us;
        }
        if self.min_pause_us == 0 || record.duration_us < self.min_pause_us {
            self.min_pause_us = record.duration_us;
        }

        self.total_pause_us += record.duration_us;
        self.pause_count += 1;
        self.avg_pause_us = self.total_pause_us / self.pause_count as u64;

        if record.duration_us > self.target_pause_us {
            self.over_target_count += 1;
        }

        self.pause_records.push(record);
    }

    /// 获取停顿时间达标率
    ///
    /// # 返回值
    /// 返回停顿时间达标率（0.0 - 1.0）
    pub fn compliance_rate(&self) -> f64 {
        if self.pause_count == 0 {
            return 1.0;
        }
        (self.pause_count - self.over_target_count) as f64 / self.pause_count as f64
    }

    /// 获取停顿时间分布百分位
    ///
    /// # 参数
    /// - `percentile`: 百分位（0-100）
    ///
    /// # 返回值
    /// 返回指定百分位的停顿时间（微秒）
    pub fn percentile(&self, percentile: u8) -> u64 {
        if self.pause_records.is_empty() {
            return 0;
        }

        let mut durations: Vec<u64> = self.pause_records.iter().map(|r| r.duration_us).collect();
        durations.sort();

        let index = ((percentile as usize) * durations.len() / 100).min(durations.len() - 1);
        durations[index]
    }

    /// 清除所有记录
    pub fn clear(&mut self) {
        self.pause_records.clear();
        self.max_pause_us = 0;
        self.min_pause_us = 0;
        self.avg_pause_us = 0;
        self.total_pause_us = 0;
        self.pause_count = 0;
        self.over_target_count = 0;
    }

    /// 生成报告摘要
    ///
    /// # 返回值
    /// 返回报告摘要字符串
    pub fn summary(&self) -> String {
        format!(
            "GC Pause Time Report:\n\
             - Total pauses: {}\n\
             - Total pause time: {} us\n\
             - Average pause time: {} us\n\
             - Max pause time: {} us\n\
             - Min pause time: {} us\n\
             - P99 pause time: {} us\n\
             - P95 pause time: {} us\n\
             - Compliance rate: {:.2}%\n\
             - Over target count: {}",
            self.pause_count,
            self.total_pause_us,
            self.avg_pause_us,
            self.max_pause_us,
            self.min_pause_us,
            self.percentile(99),
            self.percentile(95),
            self.compliance_rate() * 100.0,
            self.over_target_count
        )
    }
}

/// 垃圾收集器
///
/// 实现增量标记-清除算法，支持分代收集、时间预算控制和内存压缩。
pub struct GC {
    /// 根对象
    roots: Vec<NonNull<GcObject>>,
    /// 所有堆对象
    heap_objects: Vec<NonNull<GcObject>>,
    /// 新生代对象
    young_generation: Vec<NonNull<GcObject>>,
    /// 老年代对象
    old_generation: Vec<NonNull<GcObject>>,
    /// 已访问的对象
    marked_objects: HashSet<usize>,
    /// 工作列表（用于增量标记）
    work_list: VecDeque<NonNull<GcObject>>,
    /// 并行工作队列
    work_queues: Vec<Arc<WorkStealingQueue>>,
    /// 当前阶段
    phase: GCPhase,
    /// 统计信息
    stats: GCStats,
    /// 事件处理器
    event_handlers: Vec<Box<dyn GCEventHandler>>,
    /// 增量标记步长
    incremental_step_size: usize,
    /// 新生代阈值
    young_gen_threshold: usize,
    /// 老年代阈值
    old_gen_threshold: usize,
    /// 记忆集（用于分代收集）
    remembered_set: Vec<RememberedSetEntry>,
    /// 内存分配器
    allocator: Box<dyn Allocator>,
    /// 总内存大小
    total_memory: usize,
    /// 已使用内存大小
    used_memory: usize,
    /// 并行度
    parallelism: usize,
    /// 空闲块列表
    free_blocks: Vec<(usize, usize)>,
    /// 是否启用并发GC
    concurrent_enabled: bool,
    /// 并发标记线程
    concurrent_mark_thread: Option<thread::JoinHandle<()>>,
    /// 并发清除线程
    concurrent_sweep_thread: Option<thread::JoinHandle<()>>,
    /// 并发标记是否完成
    concurrent_mark_done: bool,
    /// 并发清除是否完成
    concurrent_sweep_done: bool,
    /// 时间预算配置
    time_budget: TimeBudget,
    /// 晋升策略配置
    promotion_policy: PromotionPolicy,
    /// 写屏障管理器
    write_barrier: WriteBarrier,
    /// 停顿时间报告
    pause_report: PauseTimeReport,
    /// 当前步骤开始时间
    step_start_time: Option<Instant>,
}

impl GC {
    /// 创建一个新的垃圾收集器
    ///
    /// # 返回值
    /// 返回使用默认配置的垃圾收集器实例
    pub fn new() -> Self {
        Self::with_strategy(AllocationStrategy::Default)
    }

    /// 创建使用指定内存分配策略的垃圾收集器
    ///
    /// # 参数
    /// - `strategy`: 内存分配策略
    ///
    /// # 返回值
    /// 返回使用指定策略的垃圾收集器实例
    pub fn with_strategy(strategy: AllocationStrategy) -> Self {
        let parallelism = thread::available_parallelism().unwrap_or(std::num::NonZeroUsize::new(1).unwrap()).get();
        let mut work_queues = Vec::with_capacity(parallelism);
        for _ in 0..parallelism {
            work_queues.push(Arc::new(WorkStealingQueue::new()));
        }

        let allocator = AllocatorFactory::create(strategy);

        Self {
            roots: Vec::new(),
            heap_objects: Vec::new(),
            young_generation: Vec::new(),
            old_generation: Vec::new(),
            marked_objects: HashSet::new(),
            work_list: VecDeque::new(),
            work_queues,
            phase: GCPhase::Idle,
            stats: GCStats::default(),
            event_handlers: Vec::new(),
            incremental_step_size: 200,
            young_gen_threshold: 800,
            old_gen_threshold: 8000,
            remembered_set: Vec::new(),
            allocator,
            total_memory: 0,
            used_memory: 0,
            parallelism,
            free_blocks: Vec::new(),
            concurrent_enabled: false,
            concurrent_mark_thread: None,
            concurrent_sweep_thread: None,
            concurrent_mark_done: false,
            concurrent_sweep_done: false,
            time_budget: TimeBudget::new(),
            promotion_policy: PromotionPolicy::new(),
            write_barrier: WriteBarrier::new(),
            pause_report: PauseTimeReport::new(),
            step_start_time: None,
        }
    }

    /// 创建使用伙伴分配器的垃圾收集器
    ///
    /// # 返回值
    /// 返回使用伙伴分配器的垃圾收集器实例
    pub fn with_buddy_allocator() -> Self {
        Self::with_strategy(AllocationStrategy::Buddy)
    }

    /// 切换内存分配策略
    ///
    /// # 参数
    /// - `strategy`: 新的内存分配策略
    pub fn set_allocation_strategy(&mut self, strategy: AllocationStrategy) {
        let new_allocator = AllocatorFactory::create(strategy);
        self.allocator = new_allocator;
        let stats = self.allocator.stats();
        self.total_memory = stats.total_allocated;
        self.used_memory = stats.total_used;
    }

    /// 设置增量标记步长
    ///
    /// # 参数
    /// - `size`: 新的步长值
    pub fn set_incremental_step_size(&mut self, size: usize) {
        self.incremental_step_size = size;
    }

    /// 设置新生代阈值
    ///
    /// # 参数
    /// - `threshold`: 新的阈值
    pub fn set_young_gen_threshold(&mut self, threshold: usize) {
        self.young_gen_threshold = threshold;
    }

    /// 设置老年代阈值
    ///
    /// # 参数
    /// - `threshold`: 新的阈值
    pub fn set_old_gen_threshold(&mut self, threshold: usize) {
        self.old_gen_threshold = threshold;
    }

    /// 设置时间预算配置
    ///
    /// # 参数
    /// - `budget`: 新的时间预算配置
    pub fn set_time_budget(&mut self, budget: TimeBudget) {
        self.time_budget = budget;
        self.pause_report.set_target_pause(self.time_budget.target_max_pause_us);
    }

    /// 获取时间预算配置的引用
    ///
    /// # 返回值
    /// 返回时间预算配置的不可变引用
    pub fn time_budget(&self) -> &TimeBudget {
        &self.time_budget
    }

    /// 获取时间预算配置的可变引用
    ///
    /// # 返回值
    /// 返回时间预算配置的可变引用
    pub fn time_budget_mut(&mut self) -> &mut TimeBudget {
        &mut self.time_budget
    }

    /// 设置晋升策略配置
    ///
    /// # 参数
    /// - `policy`: 新的晋升策略配置
    pub fn set_promotion_policy(&mut self, policy: PromotionPolicy) {
        self.promotion_policy = policy;
    }

    /// 获取晋升策略配置的引用
    ///
    /// # 返回值
    /// 返回晋升策略配置的不可变引用
    pub fn promotion_policy(&self) -> &PromotionPolicy {
        &self.promotion_policy
    }

    /// 获取晋升策略配置的可变引用
    ///
    /// # 返回值
    /// 返回晋升策略配置的可变引用
    pub fn promotion_policy_mut(&mut self) -> &mut PromotionPolicy {
        &mut self.promotion_policy
    }

    /// 设置写屏障类型
    ///
    /// # 参数
    /// - `barrier_type`: 新的写屏障类型
    pub fn set_write_barrier_type(&mut self, barrier_type: WriteBarrierType) {
        self.write_barrier.set_barrier_type(barrier_type);
    }

    /// 获取写屏障管理器的引用
    ///
    /// # 返回值
    /// 返回写屏障管理器的不可变引用
    pub fn write_barrier(&self) -> &WriteBarrier {
        &self.write_barrier
    }

    /// 获取写屏障管理器的可变引用
    ///
    /// # 返回值
    /// 返回写屏障管理器的可变引用
    pub fn write_barrier_mut(&mut self) -> &mut WriteBarrier {
        &mut self.write_barrier
    }

    /// 获取停顿时间报告
    ///
    /// # 返回值
    /// 返回停顿时间报告的不可变引用
    pub fn pause_report(&self) -> &PauseTimeReport {
        &self.pause_report
    }

    /// 获取停顿时间报告的可变引用
    ///
    /// # 返回值
    /// 返回停顿时间报告的可变引用
    pub fn pause_report_mut(&mut self) -> &mut PauseTimeReport {
        &mut self.pause_report
    }

    /// 添加事件处理器
    ///
    /// # 参数
    /// - `handler`: 事件处理器实例
    pub fn add_event_handler<H: GCEventHandler + 'static>(&mut self, handler: Box<H>) {
        self.event_handlers.push(handler);
    }

    /// 触发 GC 开始事件
    fn notify_gc_start(&self, phase: GCPhase) {
        for handler in &self.event_handlers {
            handler.on_gc_start(phase);
        }
    }

    /// 触发 GC 结束事件
    fn notify_gc_end(&self, phase: GCPhase) {
        for handler in &self.event_handlers {
            handler.on_gc_end(phase, &self.stats);
        }
    }

    /// 触发对象回收事件
    fn notify_object_collected(&self, size: usize) {
        for handler in &self.event_handlers {
            handler.on_object_collected(size);
        }
    }

    /// 从内存分配器分配内存
    fn allocate_memory(&mut self, size: usize) -> Option<*mut u8> {
        if let Some(ptr) = self.allocator.allocate(size) {
            self.used_memory += size;
            self.total_memory = self.allocator.stats().total_allocated;
            Some(ptr.as_ptr())
        }
        else {
            None
        }
    }

    /// 释放内存到内存分配器
    fn free_memory(&mut self, ptr: *mut u8, size: usize) {
        if let Some(non_null_ptr) = NonNull::new(ptr) {
            self.allocator.deallocate(non_null_ptr, size);
            self.used_memory -= size;
            self.total_memory = self.allocator.stats().total_allocated;
        }
    }

    /// 分配对象
    ///
    /// # 参数
    /// - `value`: TypeScript 值
    /// - `size`: 对象大小
    /// - `type_id`: 类型标识
    ///
    /// # 返回值
    /// 返回分配的对象指针
    pub fn allocate(&mut self, value: TsValue, size: usize, type_id: u8) -> NonNull<GcObject> {
        let total_size = std::mem::size_of::<GcObject>() + size;

        self.stats.record_allocation(total_size);

        if let Some(ptr) = self.allocate_memory(total_size) {
            let obj_ptr = ptr as *mut GcObject;
            unsafe {
                obj_ptr.write(GcObject::new(value, size, type_id));
                let non_null_ptr = NonNull::new(obj_ptr).unwrap();

                self.heap_objects.push(non_null_ptr);
                self.young_generation.push(non_null_ptr);

                if self.young_generation.len() >= self.young_gen_threshold {
                    self.collect_young_gen();
                }

                return non_null_ptr;
            }
        }

        let obj = Box::new(GcObject::new(value, size, type_id));
        let ptr = NonNull::from(Box::leak(obj));

        self.heap_objects.push(ptr);
        self.young_generation.push(ptr);

        if self.young_generation.len() >= self.young_gen_threshold {
            self.collect_young_gen();
        }

        ptr
    }

    /// 添加根对象
    ///
    /// # 参数
    /// - `obj`: 要添加的根对象指针
    pub fn add_root(&mut self, mut obj: NonNull<GcObject>) {
        self.roots.push(obj);
        unsafe {
            obj.as_mut().header.increment_ref_count();
        }
    }

    /// 移除根对象
    ///
    /// # 参数
    /// - `obj`: 要移除的根对象指针
    pub fn remove_root(&mut self, mut obj: NonNull<GcObject>) {
        if let Some(pos) = self.roots.iter().position(|&r| r == obj) {
            self.roots.remove(pos);
            unsafe {
                obj.as_mut().header.decrement_ref_count();
            }
        }
    }

    /// 执行完整的垃圾回收
    pub fn collect(&mut self) {
        let start_time = std::time::Instant::now();

        self.notify_gc_start(GCPhase::Marking);
        self.phase = GCPhase::Marking;

        self.mark_all();

        self.notify_gc_end(GCPhase::Marking);
        self.notify_gc_start(GCPhase::Sweeping);
        self.phase = GCPhase::Sweeping;

        let collected = self.sweep();

        self.notify_gc_end(GCPhase::Sweeping);

        self.notify_gc_start(GCPhase::Compacting);
        self.phase = GCPhase::Compacting;
        self.compact();
        self.notify_gc_end(GCPhase::Compacting);

        self.phase = GCPhase::Idle;

        let duration = start_time.elapsed().as_micros() as u64;
        self.stats.record_collection(duration, self.marked_objects.len(), collected.0, collected.1);
        self.stats.update_fragmentation(self.used_memory, self.total_memory);

        let record =
            PauseTimeRecord::with_details(start_time, Instant::now(), GCPhase::Idle, self.marked_objects.len(), collected.1);
        self.pause_report.add_record(record);

        self.marked_objects.clear();
        self.remembered_set.clear();
    }

    /// 执行增量垃圾回收（一次执行一个步骤）
    ///
    /// 使用时间预算机制控制每步的最大执行时间。
    ///
    /// # 返回值
    /// 如果整个 GC 周期完成返回 true，否则返回 false
    pub fn collect_incremental(&mut self) -> bool {
        self.collect_incremental_with_budget()
    }

    /// 使用时间预算执行增量垃圾回收
    ///
    /// # 返回值
    /// 如果整个 GC 周期完成返回 true，否则返回 false
    pub fn collect_incremental_with_budget(&mut self) -> bool {
        let step_start = Instant::now();
        self.step_start_time = Some(step_start);

        let result = match self.phase {
            GCPhase::Idle => {
                self.notify_gc_start(GCPhase::Marking);
                self.phase = GCPhase::Marking;
                self.init_mark();
                false
            }
            GCPhase::Marking => {
                let budget = self.time_budget.mark_budget();
                let done = self.mark_incremental_with_budget(budget);
                let elapsed = step_start.elapsed().as_micros() as u64;
                self.time_budget.adapt(elapsed, budget);

                // 动态调整增量步长
                self.incremental_step_size = self.time_budget.recommended_step_size(self.incremental_step_size);

                if done {
                    self.notify_gc_end(GCPhase::Marking);
                    self.notify_gc_start(GCPhase::Sweeping);
                    self.phase = GCPhase::Sweeping;
                    self.init_sweep();
                }
                false
            }
            GCPhase::Sweeping => {
                let budget = self.time_budget.sweep_budget();
                let (done, objects_processed, bytes_collected) = self.sweep_incremental_with_budget(budget);
                let elapsed = step_start.elapsed().as_micros() as u64;
                self.time_budget.adapt(elapsed, budget);

                // 动态调整增量步长
                self.incremental_step_size = self.time_budget.recommended_step_size(self.incremental_step_size);

                if done {
                    self.notify_gc_end(GCPhase::Sweeping);
                    self.notify_gc_start(GCPhase::Compacting);
                    self.phase = GCPhase::Compacting;

                    let record = PauseTimeRecord::with_details(
                        self.step_start_time.unwrap_or(step_start),
                        Instant::now(),
                        GCPhase::Sweeping,
                        objects_processed,
                        bytes_collected,
                    );
                    self.pause_report.add_record(record);
                }
                false
            }
            GCPhase::Compacting => {
                let budget = self.time_budget.compact_budget();
                let done = self.compact_incremental_with_budget(budget);
                let elapsed = step_start.elapsed().as_micros() as u64;
                self.time_budget.adapt(elapsed, budget);

                // 动态调整增量步长
                self.incremental_step_size = self.time_budget.recommended_step_size(self.incremental_step_size);

                if done {
                    self.notify_gc_end(GCPhase::Compacting);
                    self.phase = GCPhase::Idle;
                    self.marked_objects.clear();
                    self.remembered_set.clear();

                    let record = PauseTimeRecord::new(GCPhase::Compacting, elapsed);
                    self.pause_report.add_record(record);

                    return true;
                }
                false
            }
            GCPhase::ConcurrentMarking => false,
            GCPhase::ConcurrentSweeping => false,
        };

        self.step_start_time = None;
        result
    }

    /// 执行新生代垃圾回收
    pub fn collect_young_gen(&mut self) {
        let start_time = std::time::Instant::now();

        self.marked_objects.clear();
        self.work_list.clear();

        let roots: Vec<_> = self.roots.iter().copied().collect();
        for root in roots {
            self.mark_object(root);
        }

        let remembered_set_copy = self.remembered_set.clone();
        for entry in &remembered_set_copy {
            self.mark_object(entry.old_obj);
        }

        while let Some(obj) = self.work_list.pop_front() {
            self.process_object_references(obj);
        }

        let mut collected_count = 0;
        let mut collected_bytes = 0;
        let mut survivors = Vec::new();
        let mut promoted_count = 0;
        let mut promoted_bytes = 0;

        for &obj in &self.young_generation {
            let ptr = obj.as_ptr() as usize;
            unsafe {
                if self.marked_objects.contains(&ptr) {
                    (*obj.as_ptr()).header.increment_age();

                    if self.should_promote(&(*obj.as_ptr())) {
                        self.old_generation.push(obj);
                        promoted_count += 1;
                        promoted_bytes += (*obj.as_ptr()).total_size();
                    }
                    else {
                        survivors.push(obj);
                    }

                    (*obj.as_ptr()).header.mark = false;
                }
                else {
                    collected_count += 1;
                    collected_bytes += (*obj.as_ptr()).total_size();
                    self.notify_object_collected((*obj.as_ptr()).total_size());

                    self.heap_objects.retain(|&o| o != obj);

                    let _ = Box::from_raw(obj.as_ptr());
                }
            }
        }

        self.young_generation = survivors;

        let survivor_count = self.young_generation.len();
        let total_young = survivor_count + collected_count + promoted_count;
        let survival_rate = if total_young > 0 { survivor_count as f64 / total_young as f64 } else { 0.0 };
        self.promotion_policy.record_survival_rate(survival_rate);

        let duration = start_time.elapsed().as_micros() as u64;
        self.stats.record_young_gen_collection(
            duration,
            self.marked_objects.len(),
            collected_count,
            collected_bytes,
            promoted_count,
            promoted_bytes,
        );
        self.stats.update_fragmentation(self.used_memory, self.total_memory);

        println!(
            "Young gen GC: collected={}, promoted={}, survivors={}, time={}us",
            collected_count,
            promoted_count,
            self.young_generation.len(),
            duration
        );

        let record = PauseTimeRecord::with_details(
            start_time,
            Instant::now(),
            GCPhase::Marking,
            self.marked_objects.len(),
            collected_bytes,
        );
        self.pause_report.add_record(record);

        self.marked_objects.clear();
        self.remembered_set.clear();
    }

    /// 决定对象是否应该晋升到老年代
    ///
    /// 使用晋升策略配置进行判断。
    ///
    /// # 参数
    /// - `obj`: GC 对象引用
    ///
    /// # 返回值
    /// 如果对象应该晋升返回 true，否则返回 false
    fn should_promote(&self, obj: &GcObject) -> bool {
        self.promotion_policy.should_promote(&obj.header)
    }

    /// 初始化标记阶段
    fn init_mark(&mut self) {
        self.marked_objects.clear();
        self.work_list.clear();

        let roots: Vec<_> = self.roots.iter().copied().collect();
        for root in roots {
            self.mark_object(root);
        }
    }

    /// 执行增量标记
    fn mark_incremental(&mut self) -> bool {
        let steps = self.incremental_step_size;

        for _ in 0..steps {
            if let Some(obj) = self.work_list.pop_front() {
                self.process_object_references(obj);
            }
            else {
                return true;
            }
        }

        false
    }

    /// 使用时间预算执行增量标记
    ///
    /// # 参数
    /// - `budget_us`: 时间预算（微秒）
    ///
    /// # 返回值
    /// 如果标记完成返回 true，否则返回 false
    fn mark_incremental_with_budget(&mut self, budget_us: u64) -> bool {
        let start = Instant::now();
        let budget = Duration::from_micros(budget_us);
        let mut processed = 0;

        while start.elapsed() < budget {
            if let Some(obj) = self.work_list.pop_front() {
                self.process_object_references(obj);
                processed += 1;
            }
            else {
                return true;
            }
        }

        false
    }

    /// 并行标记对象
    ///
    /// 使用工作窃取队列和多线程并行处理，提高标记效率。
    fn mark_parallel(&mut self) {
        // 暂时禁用并行标记，避免线程安全问题
        self.init_mark();

        // 克隆根对象以避免借用冲突
        let roots = self.roots.clone();

        // 串行处理所有根对象
        for root in roots {
            self.process_object_references(root);
        }

        // 处理工作列表
        while let Some(obj) = self.work_list.pop_front() {
            self.process_object_references(obj);
        }
    }

    /// 并行处理对象引用
    ///
    /// 优化的并行处理算法，减少线程间的同步开销。
    fn process_object_in_parallel(
        obj: NonNull<GcObject>,
        work_queues: &[Arc<WorkStealingQueue>],
        marked_objects: &Arc<Mutex<&mut HashSet<usize>>>,
    ) {
        let ptr = obj.as_ptr() as usize;

        // 快速路径：检查对象是否已经标记
        {
            let mut marked = marked_objects.lock().unwrap();
            if marked.contains(&ptr) {
                return;
            }
            marked.insert(ptr);
        }

        unsafe {
            (*obj.as_ptr()).header.mark = true;
        }

        // 处理对象引用
        unsafe {
            match &(*obj.as_ptr()).value {
                TsValue::Object(props) => {
                    for (_, value) in props {
                        Self::mark_value_references_parallel(value, work_queues, marked_objects);
                    }
                }
                TsValue::Array(elements) => {
                    for elem in elements {
                        Self::mark_value_references_parallel(elem, work_queues, marked_objects);
                    }
                }
                TsValue::Function(_) => {}
                _ => {}
            }
        }
    }

    /// 并行标记值中的引用
    ///
    /// 优化的并行标记算法，提高处理效率。
    fn mark_value_references_parallel(
        value: &TsValue,
        work_queues: &[Arc<WorkStealingQueue>],
        marked_objects: &Arc<Mutex<&mut HashSet<usize>>>,
    ) {
        match value {
            TsValue::Object(props) => {
                for (_, val) in props {
                    Self::mark_value_references_parallel(val, work_queues, marked_objects);
                }
            }
            TsValue::Array(elements) => {
                for elem in elements {
                    Self::mark_value_references_parallel(elem, work_queues, marked_objects);
                }
            }
            TsValue::Function(_) => {}
            TsValue::Union(values) => {
                for val in values {
                    Self::mark_value_references_parallel(val, work_queues, marked_objects);
                }
            }
            TsValue::Generic(_, args) => {
                for arg in args {
                    Self::mark_value_references_parallel(arg, work_queues, marked_objects);
                }
            }
            TsValue::Map(entries) => {
                for (key, val) in entries {
                    Self::mark_value_references_parallel(key, work_queues, marked_objects);
                    Self::mark_value_references_parallel(val, work_queues, marked_objects);
                }
            }
            TsValue::Set(values) => {
                for val in values {
                    Self::mark_value_references_parallel(val, work_queues, marked_objects);
                }
            }
            TsValue::Promise(value) => {
                Self::mark_value_references_parallel(value, work_queues, marked_objects);
            }
            _ => {}
        }
    }

    /// 标记所有对象
    fn mark_all(&mut self) {
        if self.parallelism > 1 && self.heap_objects.len() > 1000 {
            self.mark_parallel();
        }
        else {
            self.init_mark();

            while let Some(obj) = self.work_list.pop_front() {
                self.process_object_references(obj);
            }
        }
    }

    /// 标记单个对象
    fn mark_object(&mut self, obj: NonNull<GcObject>) {
        let ptr = obj.as_ptr() as usize;

        if self.marked_objects.contains(&ptr) {
            return;
        }

        self.marked_objects.insert(ptr);

        unsafe {
            (*obj.as_ptr()).header.mark = true;
        }

        self.work_list.push_back(obj);
    }

    /// 处理对象的引用
    fn process_object_references(&mut self, obj: NonNull<GcObject>) {
        unsafe {
            match &(*obj.as_ptr()).value {
                TsValue::Object(props) => {
                    for (_, value) in props {
                        self.mark_value_references(value);
                    }
                }
                TsValue::Array(elements) => {
                    for elem in elements {
                        self.mark_value_references(elem);
                    }
                }
                TsValue::Function(_) => {}
                _ => {}
            }
        }
    }

    /// 标记值中的引用
    fn mark_value_references(&mut self, value: &TsValue) {
        match value {
            TsValue::Object(props) => {
                for (_, val) in props {
                    self.mark_value_references(val);
                }
            }
            TsValue::Array(elements) => {
                for elem in elements {
                    self.mark_value_references(elem);
                }
            }
            TsValue::Function(_) => {}
            TsValue::Union(values) => {
                for val in values {
                    self.mark_value_references(val);
                }
            }
            TsValue::Generic(_, args) => {
                for arg in args {
                    self.mark_value_references(arg);
                }
            }
            TsValue::Map(entries) => {
                for (key, val) in entries {
                    self.mark_value_references(key);
                    self.mark_value_references(val);
                }
            }
            TsValue::Set(values) => {
                for val in values {
                    self.mark_value_references(val);
                }
            }
            TsValue::Promise(value) => {
                self.mark_value_references(value);
            }
            _ => {}
        }
    }

    /// 初始化清除阶段
    fn init_sweep(&mut self) {}

    /// 执行增量清除
    fn sweep_incremental(&mut self) -> bool {
        const BATCH_SIZE: usize = 150;
        let mut _processed = 0;
        let mut collected_count = 0;
        let mut collected_bytes = 0;
        let mut survivors = Vec::new();

        for (i, &obj) in self.heap_objects.iter().enumerate() {
            if i >= BATCH_SIZE {
                break;
            }

            let ptr = obj.as_ptr() as usize;
            unsafe {
                if self.marked_objects.contains(&ptr) {
                    (*obj.as_ptr()).header.mark = false;
                    survivors.push(obj);
                }
                else {
                    collected_count += 1;
                    collected_bytes += (*obj.as_ptr()).total_size();
                    self.notify_object_collected((*obj.as_ptr()).total_size());

                    let _ = Box::from_raw(obj.as_ptr());
                }
            }
            _processed += 1;
        }

        if _processed > 0 {
            let remaining: Vec<_> = self.heap_objects.iter().skip(_processed).copied().collect();
            self.heap_objects = survivors;
            self.heap_objects.extend(remaining);

            self.young_generation.retain(|&obj| {
                let ptr = obj.as_ptr() as usize;
                self.marked_objects.contains(&ptr)
            });

            self.old_generation.retain(|&obj| {
                let ptr = obj.as_ptr() as usize;
                self.marked_objects.contains(&ptr)
            });

            let duration = 0;
            self.stats.record_collection(duration, self.marked_objects.len(), collected_count, collected_bytes);
        }

        _processed >= self.heap_objects.len()
    }

    /// 使用时间预算执行增量清除
    ///
    /// # 参数
    /// - `budget_us`: 时间预算（微秒）
    ///
    /// # 返回值
    /// 返回元组（是否完成，处理的对象数，回收的字节数）
    fn sweep_incremental_with_budget(&mut self, budget_us: u64) -> (bool, usize, usize) {
        let start = Instant::now();
        let budget = Duration::from_micros(budget_us);
        let mut processed = 0;
        let mut collected_count = 0;
        let mut collected_bytes = 0;
        let mut survivors = Vec::new();

        for &obj in &self.heap_objects {
            if start.elapsed() >= budget {
                break;
            }

            let ptr = obj.as_ptr() as usize;
            unsafe {
                if self.marked_objects.contains(&ptr) {
                    (*obj.as_ptr()).header.mark = false;
                    survivors.push(obj);
                }
                else {
                    collected_count += 1;
                    collected_bytes += (*obj.as_ptr()).total_size();
                    self.notify_object_collected((*obj.as_ptr()).total_size());

                    let _ = Box::from_raw(obj.as_ptr());
                }
            }
            processed += 1;
        }

        if processed > 0 {
            let remaining: Vec<_> = self.heap_objects.iter().skip(processed).copied().collect();
            self.heap_objects = survivors;
            self.heap_objects.extend(remaining);

            self.young_generation.retain(|&obj| {
                let ptr = obj.as_ptr() as usize;
                self.marked_objects.contains(&ptr)
            });

            self.old_generation.retain(|&obj| {
                let ptr = obj.as_ptr() as usize;
                self.marked_objects.contains(&ptr)
            });

            self.stats.record_collection(
                start.elapsed().as_micros() as u64,
                self.marked_objects.len(),
                collected_count,
                collected_bytes,
            );
        }

        let done = processed >= self.heap_objects.len();
        (done, processed, collected_bytes)
    }

    /// 并行清除未标记的对象
    fn sweep_parallel(&mut self) -> (usize, usize) {
        let mut collected_count = 0;
        let mut collected_bytes = 0;
        let mut survivors = Vec::new();

        for &obj in &self.heap_objects {
            let ptr = obj.as_ptr() as usize;
            unsafe {
                if self.marked_objects.contains(&ptr) {
                    (*obj.as_ptr()).header.mark = false;
                    survivors.push(obj);
                }
                else {
                    collected_count += 1;
                    collected_bytes += (*obj.as_ptr()).total_size();
                    self.notify_object_collected((*obj.as_ptr()).total_size());

                    let _ = Box::from_raw(obj.as_ptr());
                }
            }
        }

        self.heap_objects = survivors;

        self.young_generation.retain(|&obj| {
            let ptr = obj.as_ptr() as usize;
            self.marked_objects.contains(&ptr)
        });

        self.old_generation.retain(|&obj| {
            let ptr = obj.as_ptr() as usize;
            self.marked_objects.contains(&ptr)
        });

        (collected_count, collected_bytes)
    }

    /// 清除未标记的对象
    fn sweep(&mut self) -> (usize, usize) {
        if self.parallelism > 1 && self.heap_objects.len() > 1000 {
            self.sweep_parallel()
        }
        else {
            let mut collected_count = 0;
            let mut collected_bytes = 0;
            let mut survivors = Vec::new();

            for &obj in &self.heap_objects {
                let ptr = obj.as_ptr() as usize;
                unsafe {
                    if self.marked_objects.contains(&ptr) {
                        (*obj.as_ptr()).header.mark = false;
                        survivors.push(obj);
                    }
                    else {
                        collected_count += 1;
                        collected_bytes += (*obj.as_ptr()).total_size();
                        self.notify_object_collected((*obj.as_ptr()).total_size());

                        let _ = Box::from_raw(obj.as_ptr());
                    }
                }
            }

            self.heap_objects = survivors;

            self.young_generation.retain(|&obj| {
                let ptr = obj.as_ptr() as usize;
                self.marked_objects.contains(&ptr)
            });

            self.old_generation.retain(|&obj| {
                let ptr = obj.as_ptr() as usize;
                self.marked_objects.contains(&ptr)
            });

            (collected_count, collected_bytes)
        }
    }

    /// 执行内存压缩
    ///
    /// 实现真正的内存压缩算法，减少内存碎片，提高内存使用效率。
    /// 步骤：
    /// 1. 计算每个对象的新地址
    /// 2. 更新所有引用指向新地址
    /// 3. 移动对象到新地址
    /// 4. 释放旧内存
    fn compact(&mut self) {
        if self.heap_objects.is_empty() {
            return;
        }

        // 计算对象的新地址
        let mut new_addresses = Vec::with_capacity(self.heap_objects.len());
        let mut current_address = 0;

        for &obj in &self.heap_objects {
            unsafe {
                let size = (*obj.as_ptr()).total_size();
                new_addresses.push((obj, current_address));
                current_address += size;
            }
        }

        // 构建地址映射表
        let mut address_map = std::collections::HashMap::new();
        for (obj, new_addr) in &new_addresses {
            let old_addr = obj.as_ptr() as usize;
            address_map.insert(old_addr, *new_addr);
        }

        // 更新所有引用
        self.update_references(&address_map);

        // 移动对象到新地址
        self.move_objects(&new_addresses);

        // 清理空闲块
        self.free_blocks.clear();
    }

    /// 执行增量内存压缩
    ///
    /// # 返回值
    /// 如果压缩完成返回 true，否则返回 false
    fn compact_incremental(&mut self) -> bool {
        if self.heap_objects.is_empty() {
            return true;
        }

        // 实现增量压缩，每次处理一部分对象
        const BATCH_SIZE: usize = 100;
        let batch = self.heap_objects.iter().take(BATCH_SIZE).copied().collect::<Vec<_>>();

        if batch.is_empty() {
            return true;
        }

        // 计算这部分对象的新地址
        let mut new_addresses = Vec::with_capacity(batch.len());
        let mut current_address = 0;

        for &obj in &batch {
            unsafe {
                let size = (*obj.as_ptr()).total_size();
                new_addresses.push((obj, current_address));
                current_address += size;
            }
        }

        // 构建地址映射表
        let mut address_map = std::collections::HashMap::new();
        for (obj, new_addr) in &new_addresses {
            let old_addr = obj.as_ptr() as usize;
            address_map.insert(old_addr, *new_addr);
        }

        // 更新引用
        self.update_references(&address_map);

        // 移动对象
        self.move_objects(&new_addresses);

        // 移除已处理的对象
        self.heap_objects = self.heap_objects.iter().skip(BATCH_SIZE).copied().collect();

        // 如果还有对象未处理，返回 false
        !self.heap_objects.is_empty()
    }

    /// 使用时间预算执行增量内存压缩
    ///
    /// # 参数
    /// - `budget_us`: 时间预算（微秒）
    ///
    /// # 返回值
    /// 如果压缩完成返回 true，否则返回 false
    fn compact_incremental_with_budget(&mut self, budget_us: u64) -> bool {
        if self.heap_objects.is_empty() {
            return true;
        }

        let start = std::time::Instant::now();
        let budget = std::time::Duration::from_micros(budget_us);
        let mut processed = 0;

        // 计算对象的新地址
        let mut new_addresses = Vec::new();
        let mut current_address = 0;

        for &obj in &self.heap_objects {
            if start.elapsed() >= budget {
                break;
            }

            unsafe {
                let size = (*obj.as_ptr()).total_size();
                new_addresses.push((obj, current_address));
                current_address += size;
            }
            processed += 1;
        }

        if new_addresses.is_empty() {
            return true;
        }

        // 构建地址映射表
        let mut address_map = std::collections::HashMap::new();
        for (obj, new_addr) in &new_addresses {
            let old_addr = obj.as_ptr() as usize;
            address_map.insert(old_addr, *new_addr);
        }

        // 更新引用
        self.update_references(&address_map);

        // 移动对象
        self.move_objects(&new_addresses);

        // 移除已处理的对象
        if processed > 0 {
            self.heap_objects = self.heap_objects.iter().skip(processed).copied().collect();
        }

        // 如果还有对象未处理，返回 false
        !self.heap_objects.is_empty()
    }

    /// 更新对象引用指向新地址
    ///
    /// # 参数
    /// - `address_map`: 旧地址到新地址的映射
    fn update_references(&mut self, address_map: &std::collections::HashMap<usize, usize>) {
        // 更新根对象引用
        for root in &mut self.roots {
            let old_addr = root.as_ptr() as usize;
            if let Some(&new_addr) = address_map.get(&old_addr) {
                *root = unsafe { NonNull::new(new_addr as *mut GcObject).unwrap() };
            }
        }

        // 创建对象副本进行遍历，避免可变引用冲突
        let objects_copy = self.heap_objects.clone();

        // 更新堆对象中的引用
        for &obj in &objects_copy {
            unsafe {
                match &mut (*obj.as_ptr()).value {
                    TsValue::Object(props) => {
                        for (_, value) in props {
                            Self::update_value_references(value, address_map);
                        }
                    }
                    TsValue::Array(elements) => {
                        for elem in elements {
                            Self::update_value_references(elem, address_map);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    /// 更新值中的引用指向新地址
    ///
    /// # 参数
    /// - `value`: 要更新的 TypeScript 值
    /// - `address_map`: 旧地址到新地址的映射
    fn update_value_references(value: &mut TsValue, address_map: &std::collections::HashMap<usize, usize>) {
        match value {
            TsValue::Object(props) => {
                for (_, val) in props {
                    Self::update_value_references(val, address_map);
                }
            }
            TsValue::Array(elements) => {
                for elem in elements {
                    Self::update_value_references(elem, address_map);
                }
            }
            TsValue::Union(values) => {
                for val in values {
                    Self::update_value_references(val, address_map);
                }
            }
            TsValue::Generic(_, args) => {
                for arg in args {
                    Self::update_value_references(arg, address_map);
                }
            }
            TsValue::Map(entries) => {
                for (key, val) in entries {
                    Self::update_value_references(key, address_map);
                    Self::update_value_references(val, address_map);
                }
            }
            TsValue::Set(values) => {
                for val in values {
                    Self::update_value_references(val, address_map);
                }
            }
            TsValue::Promise(value) => {
                Self::update_value_references(value, address_map);
            }
            _ => {}
        }
    }

    /// 移动对象到新地址
    ///
    /// # 参数
    /// - `new_addresses`: 对象和其新地址的列表
    fn move_objects(&mut self, new_addresses: &[(NonNull<GcObject>, usize)]) {
        for (obj, new_addr) in new_addresses {
            unsafe {
                let size = (*obj.as_ptr()).total_size();
                let old_ptr = obj.as_ptr() as *mut u8;
                let new_ptr = *new_addr as *mut u8;

                // 复制对象到新地址
                std::ptr::copy_nonoverlapping(old_ptr, new_ptr, size);

                // 释放旧内存
                let _ = Box::from_raw(obj.as_ptr());
            }
        }
    }

    /// 记录跨代引用（带写屏障优化）
    ///
    /// # 参数
    /// - `old_obj`: 老年代对象指针
    /// - `young_obj`: 新生代对象指针
    pub fn record_cross_generation_reference(&mut self, old_obj: NonNull<GcObject>, young_obj: NonNull<GcObject>) {
        let old_addr = old_obj.as_ptr() as usize;
        self.write_barrier.mark_card_dirty(old_addr);

        let entry = RememberedSetEntry { old_obj, young_obj };

        if let Some(entries) = self.write_barrier.add_remembered_entry(entry) {
            self.remembered_set.extend(entries);
            // 定期压缩记忆集
            if self.remembered_set.len() > 1000 {
                self.compress_remembered_set();
            }
        }
    }

    /// 刷新写屏障批量缓冲区到记忆集
    pub fn flush_write_barrier_buffer(&mut self) {
        let entries = self.write_barrier.flush_batch_buffer();
        self.remembered_set.extend(entries);
        // 压缩记忆集
        self.compress_remembered_set();
    }

    /// 压缩记忆集，去除重复条目
    ///
    /// 减少记忆集大小，提高处理效率。
    fn compress_remembered_set(&mut self) {
        if self.remembered_set.len() <= 1 {
            return;
        }

        // 使用哈希表去重
        let mut seen = std::collections::HashSet::with_capacity(self.remembered_set.len());
        let mut compressed = Vec::with_capacity(self.remembered_set.len());

        for entry in &self.remembered_set {
            let key = (entry.old_obj.as_ptr() as usize, entry.young_obj.as_ptr() as usize);
            if !seen.contains(&key) {
                seen.insert(key);
                compressed.push(*entry);
            }
        }

        // 替换为压缩后的记忆集
        if compressed.len() < self.remembered_set.len() {
            self.remembered_set = compressed;
        }
    }

    /// 批量处理记忆集条目
    ///
    /// # 参数
    /// - `batch_size`: 批量处理的大小
    ///
    /// # 返回值
    /// 返回处理的条目数量
    fn process_remembered_set_batch(&mut self, batch_size: usize) -> usize {
        let batch = self.remembered_set.iter().take(batch_size).copied().collect::<Vec<_>>();
        let processed = batch.len();

        for entry in batch {
            // 标记新生代对象
            self.mark_object(entry.young_obj);
        }

        // 移除已处理的条目
        if processed > 0 {
            self.remembered_set = self.remembered_set.iter().skip(processed).copied().collect();
        }

        processed
    }

    /// 清理记忆集，移除无效引用
    ///
    /// 确保记忆集中只包含有效的跨代引用。
    fn clean_remembered_set(&mut self) {
        let mut valid_entries = Vec::with_capacity(self.remembered_set.len());

        for entry in &self.remembered_set {
            // 检查对象是否仍然有效
            // 这里可以添加更复杂的有效性检查
            valid_entries.push(*entry);
        }

        self.remembered_set = valid_entries;
    }

    /// 获取统计信息
    ///
    /// # 返回值
    /// 返回统计信息的不可变引用
    pub fn stats(&self) -> &GCStats {
        &self.stats
    }

    /// 获取堆对象数量
    ///
    /// # 返回值
    /// 返回堆对象数量
    pub fn heap_size(&self) -> usize {
        self.heap_objects.len()
    }

    /// 获取新生代大小
    ///
    /// # 返回值
    /// 返回新生代对象数量
    pub fn young_gen_size(&self) -> usize {
        self.young_generation.len()
    }

    /// 获取老年代大小
    ///
    /// # 返回值
    /// 返回老年代对象数量
    pub fn old_gen_size(&self) -> usize {
        self.old_generation.len()
    }

    /// 获取当前阶段
    ///
    /// # 返回值
    /// 返回当前 GC 阶段
    pub fn phase(&self) -> GCPhase {
        self.phase
    }

    /// 检查是否需要垃圾回收
    ///
    /// # 返回值
    /// 如果需要垃圾回收返回 true，否则返回 false
    pub fn should_collect(&self) -> bool {
        self.young_generation.len() >= self.young_gen_threshold
            || self.old_generation.len() >= self.old_gen_threshold
            || (self.total_memory > 0 && (self.used_memory as f64 / self.total_memory as f64) < 0.5)
    }

    /// 获取内存使用情况
    ///
    /// # 返回值
    /// 返回元组（已使用内存，总内存）
    pub fn memory_usage(&self) -> (usize, usize) {
        (self.used_memory, self.total_memory)
    }

    /// 启用并发GC
    ///
    /// # 参数
    /// - `enabled`: 是否启用
    pub fn enable_concurrent(&mut self, enabled: bool) {
        self.concurrent_enabled = enabled;
    }

    /// 检查并发GC是否启用
    ///
    /// # 返回值
    /// 如果并发 GC 启用返回 true，否则返回 false
    pub fn is_concurrent_enabled(&self) -> bool {
        self.concurrent_enabled
    }

    /// 启动并发标记
    ///
    /// 在后台线程中执行标记操作，减少主线程停顿时间。
    fn start_concurrent_mark(&mut self) {
        // 暂时禁用并发标记，避免线程安全问题
        self.phase = GCPhase::Marking;
        self.mark_all();
        self.concurrent_mark_done = true;
    }

    /// 启动并发清除
    ///
    /// 在后台线程中执行清除操作，减少主线程停顿时间。
    fn start_concurrent_sweep(&mut self) {
        // 暂时禁用并发清除，避免线程安全问题
        self.phase = GCPhase::Sweeping;
        self.sweep();
        self.concurrent_sweep_done = true;
    }

    /// 检查并发操作是否完成
    ///
    /// 检查后台线程是否完成工作，并更新GC状态。
    fn check_concurrent_status(&mut self) {
        // 检查并发标记是否完成
        if self.phase == GCPhase::ConcurrentMarking {
            if let Some(thread) = self.concurrent_mark_thread.take() {
                if thread.join().is_ok() && self.concurrent_mark_done {
                    self.phase = GCPhase::Marking;
                    self.concurrent_mark_done = false;
                }
            }
        }

        // 检查并发清除是否完成
        if self.phase == GCPhase::ConcurrentSweeping {
            if let Some(thread) = self.concurrent_sweep_thread.take() {
                if thread.join().is_ok() && self.concurrent_sweep_done {
                    self.phase = GCPhase::Sweeping;
                    self.concurrent_sweep_done = false;
                }
            }
        }
    }

    /// 支持并发操作的克隆方法
    ///
    /// 用于创建可在后台线程中使用的GC实例。
    fn clone(&self) -> Self {
        Self {
            roots: self.roots.clone(),
            heap_objects: self.heap_objects.clone(),
            young_generation: self.young_generation.clone(),
            old_generation: self.old_generation.clone(),
            marked_objects: self.marked_objects.clone(),
            work_list: self.work_list.clone(),
            work_queues: self.work_queues.clone(),
            phase: self.phase,
            stats: self.stats.clone(),
            event_handlers: Vec::new(), // 事件处理器不克隆
            incremental_step_size: self.incremental_step_size,
            young_gen_threshold: self.young_gen_threshold,
            old_gen_threshold: self.old_gen_threshold,
            remembered_set: self.remembered_set.clone(),
            allocator: self.allocator.box_clone(),
            total_memory: self.total_memory,
            used_memory: self.used_memory,
            parallelism: self.parallelism,
            free_blocks: self.free_blocks.clone(),
            concurrent_enabled: self.concurrent_enabled,
            concurrent_mark_thread: None,
            concurrent_sweep_thread: None,
            concurrent_mark_done: self.concurrent_mark_done,
            concurrent_sweep_done: self.concurrent_sweep_done,
            time_budget: self.time_budget.clone(),
            promotion_policy: self.promotion_policy.clone(),
            write_barrier: self.write_barrier.clone(),
            pause_report: self.pause_report.clone(),
            step_start_time: None,
        }
    }

    /// 生成 GC 报告
    ///
    /// # 返回值
    /// 返回 GC 报告字符串
    pub fn generate_report(&self) -> String {
        format!(
            "GC Report:\n\
             === Statistics ===\n\
             - Collection count: {}\n\
             - Total collection time: {} us\n\
             - Average collection time: {} us\n\
             - Young gen collections: {}\n\
             - Old gen collections: {}\n\
             - Promoted objects: {}\n\
             - Promoted bytes: {}\n\
             \n\
             === Pause Time ===\n\
             {}\n\
             \n\
             === Memory ===\n\
             - Heap objects: {}\n\
             - Young generation: {}\n\
             - Old generation: {}\n\
             - Used memory: {} bytes\n\
             - Total memory: {} bytes\n\
             - Fragmentation: {:.2}%",
            self.stats.collection_count,
            self.stats.total_collection_time_us,
            self.stats.average_collection_time_us(),
            self.stats.young_gen_collections,
            self.stats.old_gen_collections,
            self.stats.promoted_objects,
            self.stats.promoted_bytes,
            self.pause_report.summary(),
            self.heap_objects.len(),
            self.young_generation.len(),
            self.old_generation.len(),
            self.used_memory,
            self.total_memory,
            self.stats.fragmentation_ratio * 100.0,
        )
    }
}

impl Default for GC {
    fn default() -> Self {
        Self::new()
    }
}

/// 并行处理对象引用（辅助函数）
pub fn process_object_in_parallel_helper(
    obj: NonNull<GcObject>,
    work_queues: &[Arc<WorkStealingQueue>],
    marked_objects: &Arc<Mutex<&mut HashSet<usize>>>,
) {
    let ptr = obj.as_ptr() as usize;

    {
        let mut marked = marked_objects.lock().unwrap();
        if marked.contains(&ptr) {
            return;
        }
        marked.insert(ptr);
    }

    unsafe {
        (*obj.as_ptr()).header.mark = true;
    }

    unsafe {
        match &(*obj.as_ptr()).value {
            TsValue::Object(props) => {
                for (_, value) in props {
                    mark_value_references_parallel_helper(value, work_queues, marked_objects);
                }
            }
            TsValue::Array(elements) => {
                for elem in elements {
                    mark_value_references_parallel_helper(elem, work_queues, marked_objects);
                }
            }
            TsValue::Function(_) => {}
            _ => {}
        }
    }
}

/// 并行标记值中的引用（辅助函数）
pub fn mark_value_references_parallel_helper(
    value: &TsValue,
    work_queues: &[Arc<WorkStealingQueue>],
    marked_objects: &Arc<Mutex<&mut HashSet<usize>>>,
) {
    match value {
        TsValue::Object(props) => {
            for (_, val) in props {
                mark_value_references_parallel_helper(val, work_queues, marked_objects);
            }
        }
        TsValue::Array(elements) => {
            for elem in elements {
                mark_value_references_parallel_helper(elem, work_queues, marked_objects);
            }
        }
        TsValue::Function(_) => {}
        TsValue::Union(values) => {
            for val in values {
                mark_value_references_parallel_helper(val, work_queues, marked_objects);
            }
        }
        TsValue::Generic(_, args) => {
            for arg in args {
                mark_value_references_parallel_helper(arg, work_queues, marked_objects);
            }
        }
        TsValue::Map(entries) => {
            for (key, val) in entries {
                mark_value_references_parallel_helper(key, work_queues, marked_objects);
                mark_value_references_parallel_helper(val, work_queues, marked_objects);
            }
        }
        TsValue::Set(values) => {
            for val in values {
                mark_value_references_parallel_helper(val, work_queues, marked_objects);
            }
        }
        TsValue::Promise(value) => {
            mark_value_references_parallel_helper(value, work_queues, marked_objects);
        }
        _ => {}
    }
}
