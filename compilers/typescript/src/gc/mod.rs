//! 垃圾收集模块
//!
//! 提供高效的垃圾收集机制，包括增量标记-清除、分代收集和引用计数。

use std::{
    collections::{HashMap, HashSet, VecDeque},
    ptr::NonNull,
    sync::{
        Arc, Mutex, MutexGuard,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
};

use crate::memory::{AllocationStrategy, Allocator, AllocatorFactory};

use typescript_types::TsValue;

/// 对象头
///
/// 每个堆分配对象的前缀，包含垃圾收集所需的元数据
#[derive(Debug, Clone, Copy)]
pub struct ObjectHeader {
    /// 对象标记（用于标记-清除算法）
    pub mark: bool,
    /// 对象年龄（用于分代收集）
    pub age: u8,
    /// 对象大小
    pub size: usize,
    /// 引用计数
    pub ref_count: usize,
    /// 对象类型标识
    pub type_id: u8,
}

impl ObjectHeader {
    /// 创建新的对象头
    pub fn new(size: usize, type_id: u8) -> Self {
        Self { mark: false, age: 0, size, ref_count: 0, type_id }
    }

    /// 增加引用计数
    pub fn increment_ref_count(&mut self) {
        self.ref_count += 1;
    }

    /// 减少引用计数
    pub fn decrement_ref_count(&mut self) -> usize {
        if self.ref_count > 0 {
            self.ref_count -= 1;
        }
        self.ref_count
    }

    /// 增加年龄
    pub fn increment_age(&mut self) {
        if self.age < u8::MAX {
            self.age += 1;
        }
    }

    /// 检查是否为老年代对象
    pub fn is_tenured(&self) -> bool {
        self.age >= 3
    }
}

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

/// 垃圾收集事件回调
pub trait GCEventHandler: Send + Sync {
    /// 垃圾收集开始
    fn on_gc_start(&self, phase: GCPhase);
    /// 垃圾收集结束
    fn on_gc_end(&self, phase: GCPhase, stats: &GCStats);
    /// 对象被回收
    fn on_object_collected(&self, size: usize);
}

/// 垃圾收集统计
#[derive(Debug, Clone, Default)]
pub struct GCStats {
    /// 总收集次数
    pub collection_count: usize,
    /// 标记的对象数量
    pub marked_objects: usize,
    /// 回收的对象数量
    pub collected_objects: usize,
    /// 回收的字节数
    pub collected_bytes: usize,
    /// 上次收集耗时（微秒）
    pub last_collection_time_us: u64,
    /// 总收集耗时（微秒）
    pub total_collection_time_us: u64,
    /// 内存碎片率
    pub fragmentation_ratio: f64,
    /// 平均对象大小
    pub average_object_size: usize,
    /// 新生代收集次数
    pub young_gen_collections: usize,
    /// 老年代收集次数
    pub old_gen_collections: usize,
    /// 晋升到老年的对象数量
    pub promoted_objects: usize,
    /// 晋升到老年的字节数
    pub promoted_bytes: usize,
    /// 并行标记耗时（微秒）
    pub parallel_mark_time_us: u64,
    /// 并行清除耗时（微秒）
    pub parallel_sweep_time_us: u64,
    /// 内存分配次数
    pub allocation_count: usize,
    /// 内存分配总字节数
    pub allocation_bytes: usize,
    /// 最大暂停时间（微秒）
    pub max_pause_time_us: u64,
    /// 最小暂停时间（微秒）
    pub min_pause_time_us: u64,
}

impl GCStats {
    /// 记录收集
    pub fn record_collection(&mut self, duration_us: u64, marked: usize, collected: usize, bytes: usize) {
        self.collection_count += 1;
        self.marked_objects += marked;
        self.collected_objects += collected;
        self.collected_bytes += bytes;
        self.last_collection_time_us = duration_us;
        self.total_collection_time_us += duration_us;

        // 更新平均对象大小
        if collected > 0 {
            self.average_object_size = bytes / collected;
        }

        // 更新最大和最小暂停时间
        if self.max_pause_time_us < duration_us {
            self.max_pause_time_us = duration_us;
        }
        if self.min_pause_time_us == 0 || self.min_pause_time_us > duration_us {
            self.min_pause_time_us = duration_us;
        }
    }

    /// 记录新生代收集
    pub fn record_young_gen_collection(
        &mut self,
        duration_us: u64,
        marked: usize,
        collected: usize,
        bytes: usize,
        promoted: usize,
        promoted_bytes: usize,
    ) {
        self.young_gen_collections += 1;
        self.record_collection(duration_us, marked, collected, bytes);
        self.promoted_objects += promoted;
        self.promoted_bytes += promoted_bytes;
    }

    /// 记录老年代收集
    pub fn record_old_gen_collection(&mut self, duration_us: u64, marked: usize, collected: usize, bytes: usize) {
        self.old_gen_collections += 1;
        self.record_collection(duration_us, marked, collected, bytes);
    }

    /// 记录内存分配
    pub fn record_allocation(&mut self, size: usize) {
        self.allocation_count += 1;
        self.allocation_bytes += size;
    }

    /// 记录并行操作时间
    pub fn record_parallel_mark_time(&mut self, duration_us: u64) {
        self.parallel_mark_time_us += duration_us;
    }

    /// 记录并行清除时间
    pub fn record_parallel_sweep_time(&mut self, duration_us: u64) {
        self.parallel_sweep_time_us += duration_us;
    }

    /// 获取平均收集时间
    pub fn average_collection_time_us(&self) -> u64 {
        if self.collection_count == 0 { 0 } else { self.total_collection_time_us / self.collection_count as u64 }
    }

    /// 获取平均新生代收集时间
    pub fn average_young_gen_time_us(&self) -> u64 {
        if self.young_gen_collections == 0 { 0 } else { self.total_collection_time_us / self.young_gen_collections as u64 }
    }

    /// 获取平均晋升率
    pub fn average_promotion_rate(&self) -> f64 {
        if self.young_gen_collections == 0 { 0.0 } else { self.promoted_objects as f64 / self.young_gen_collections as f64 }
    }

    /// 更新内存碎片率
    pub fn update_fragmentation(&mut self, used: usize, total: usize) {
        if total > 0 {
            self.fragmentation_ratio = (total - used) as f64 / total as f64;
        }
    }

    /// 重置统计信息
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

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

/// 垃圾收集器
///
/// 实现增量标记-清除算法，支持分代收集和内存压缩
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
}

/// GC 对象包装
#[repr(C)]
pub struct GcObject {
    /// 对象头
    pub header: ObjectHeader,
    /// 对象值
    pub value: TsValue,
}

impl GcObject {
    /// 创建新的 GC 对象
    pub fn new(value: TsValue, size: usize, type_id: u8) -> Self {
        Self { header: ObjectHeader::new(size, type_id), value }
    }

    /// 获取对象大小（包括头部）
    pub fn total_size(&self) -> usize {
        std::mem::size_of::<ObjectHeader>() + self.header.size
    }
}

impl GC {
    /// 创建一个新的垃圾收集器
    pub fn new() -> Self {
        Self::with_strategy(AllocationStrategy::Default)
    }

    /// 创建使用指定内存分配策略的垃圾收集器
    pub fn with_strategy(strategy: AllocationStrategy) -> Self {
        let parallelism = thread::available_parallelism().unwrap_or(std::num::NonZeroUsize::new(1).unwrap()).get();
        let mut work_queues = Vec::with_capacity(parallelism);
        for _ in 0..parallelism {
            work_queues.push(Arc::new(WorkStealingQueue::new()));
        }

        // 使用指定策略的分配器
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
            incremental_step_size: 200, // 增加步长以提高性能
            young_gen_threshold: 800,   // 调整阈值以平衡收集频率
            old_gen_threshold: 8000,    // 调整阈值以平衡收集频率
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
        }
    }

    /// 创建使用伙伴分配器的垃圾收集器
    pub fn with_buddy_allocator() -> Self {
        Self::with_strategy(AllocationStrategy::Buddy)
    }

    /// 切换内存分配策略
    pub fn set_allocation_strategy(&mut self, strategy: AllocationStrategy) {
        // 创建新的分配器
        let new_allocator = AllocatorFactory::create(strategy);
        self.allocator = new_allocator;
        // 重置内存统计
        let stats = self.allocator.stats();
        self.total_memory = stats.total_allocated;
        self.used_memory = stats.total_used;
    }

    /// 设置增量标记步长
    pub fn set_incremental_step_size(&mut self, size: usize) {
        self.incremental_step_size = size;
    }

    /// 设置新生代阈值
    pub fn set_young_gen_threshold(&mut self, threshold: usize) {
        self.young_gen_threshold = threshold;
    }

    /// 设置老年代阈值
    pub fn set_old_gen_threshold(&mut self, threshold: usize) {
        self.old_gen_threshold = threshold;
    }

    /// 添加事件处理器
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
    pub fn allocate(&mut self, value: TsValue, size: usize, type_id: u8) -> NonNull<GcObject> {
        // 计算总大小
        let total_size = std::mem::size_of::<GcObject>() + size;

        // 记录内存分配
        self.stats.record_allocation(total_size);

        // 尝试从内存分配器分配
        if let Some(ptr) = self.allocate_memory(total_size) {
            let obj_ptr = ptr as *mut GcObject;
            unsafe {
                // 初始化对象
                obj_ptr.write(GcObject::new(value, size, type_id));
                let non_null_ptr = NonNull::new(obj_ptr).unwrap();

                self.heap_objects.push(non_null_ptr);
                self.young_generation.push(non_null_ptr);

                // 检查是否需要触发新生代 GC
                if self.young_generation.len() >= self.young_gen_threshold {
                    self.collect_young_gen();
                }

                return non_null_ptr;
            }
        }

        // 回退到标准分配
        let obj = Box::new(GcObject::new(value, size, type_id));
        let ptr = NonNull::from(Box::leak(obj));

        self.heap_objects.push(ptr);
        self.young_generation.push(ptr);

        // 检查是否需要触发新生代 GC
        if self.young_generation.len() >= self.young_gen_threshold {
            self.collect_young_gen();
        }

        ptr
    }

    /// 添加根对象
    pub fn add_root(&mut self, mut obj: NonNull<GcObject>) {
        self.roots.push(obj);
        unsafe {
            obj.as_mut().header.increment_ref_count();
        }
    }

    /// 移除根对象
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

        // 标记阶段
        self.mark_all();

        self.notify_gc_end(GCPhase::Marking);
        self.notify_gc_start(GCPhase::Sweeping);
        self.phase = GCPhase::Sweeping;

        // 清除阶段
        let collected = self.sweep();

        self.notify_gc_end(GCPhase::Sweeping);

        // 压缩阶段
        self.notify_gc_start(GCPhase::Compacting);
        self.phase = GCPhase::Compacting;
        self.compact();
        self.notify_gc_end(GCPhase::Compacting);

        self.phase = GCPhase::Idle;

        // 更新统计
        let duration = start_time.elapsed().as_micros() as u64;
        self.stats.record_collection(duration, self.marked_objects.len(), collected.0, collected.1);
        self.stats.update_fragmentation(self.used_memory, self.total_memory);

        // 清空标记集合
        self.marked_objects.clear();
        // 清空记忆集
        self.remembered_set.clear();
    }

    /// 执行增量垃圾回收（一次执行一个步骤）
    pub fn collect_incremental(&mut self) -> bool {
        match self.phase {
            GCPhase::Idle => {
                self.notify_gc_start(GCPhase::Marking);
                self.phase = GCPhase::Marking;
                self.init_mark();
                false
            }
            GCPhase::Marking => {
                let done = self.mark_incremental();
                if done {
                    self.notify_gc_end(GCPhase::Marking);
                    self.notify_gc_start(GCPhase::Sweeping);
                    self.phase = GCPhase::Sweeping;
                    self.init_sweep();
                }
                false
            }
            GCPhase::Sweeping => {
                let done = self.sweep_incremental();
                if done {
                    self.notify_gc_end(GCPhase::Sweeping);
                    self.notify_gc_start(GCPhase::Compacting);
                    self.phase = GCPhase::Compacting;
                }
                false
            }
            GCPhase::Compacting => {
                let done = self.compact_incremental();
                if done {
                    self.notify_gc_end(GCPhase::Compacting);
                    self.phase = GCPhase::Idle;
                    // 清空标记集合和记忆集
                    self.marked_objects.clear();
                    self.remembered_set.clear();
                    return true;
                }
                false
            }
            GCPhase::ConcurrentMarking => {
                // 并发标记实现
                false
            }
            GCPhase::ConcurrentSweeping => {
                // 并发清除实现
                false
            }
        }
    }

    /// 执行新生代垃圾回收
    pub fn collect_young_gen(&mut self) {
        let start_time = std::time::Instant::now();

        // 只标记根对象、新生代对象和记忆集中的对象
        self.marked_objects.clear();
        self.work_list.clear();

        // 从根对象开始标记
        let roots: Vec<_> = self.roots.iter().copied().collect();
        for root in roots {
            self.mark_object(root);
        }

        // 从记忆集中的老年代对象开始标记
        let remembered_set_copy = self.remembered_set.clone();
        for entry in &remembered_set_copy {
            self.mark_object(entry.old_obj);
        }

        // 处理工作列表
        while let Some(obj) = self.work_list.pop_front() {
            self.process_object_references(obj);
        }

        // 清除未标记的新生代对象
        let mut collected_count = 0;
        let mut collected_bytes = 0;
        let mut survivors = Vec::new();
        let mut promoted_count = 0;

        for &obj in &self.young_generation {
            let ptr = obj.as_ptr() as usize;
            unsafe {
                if self.marked_objects.contains(&ptr) {
                    // 对象存活，增加年龄
                    (*obj.as_ptr()).header.increment_age();

                    // 优化晋升策略：根据对象大小和年龄决定是否晋升
                    if self.should_promote(&(*obj.as_ptr())) {
                        self.old_generation.push(obj);
                        promoted_count += 1;
                    }
                    else {
                        survivors.push(obj);
                    }

                    // 清除标记
                    (*obj.as_ptr()).header.mark = false;
                }
                else {
                    // 对象死亡，回收
                    collected_count += 1;
                    collected_bytes += (*obj.as_ptr()).total_size();
                    self.notify_object_collected((*obj.as_ptr()).total_size());

                    // 从堆对象列表中移除
                    self.heap_objects.retain(|&o| o != obj);

                    // 释放对象
                    let _ = Box::from_raw(obj.as_ptr());
                }
            }
        }

        self.young_generation = survivors;

        // 计算晋升字节数
        let mut promoted_bytes = 0;
        for &obj in &self.old_generation {
            unsafe {
                promoted_bytes += (*obj.as_ptr()).total_size();
            }
        }

        // 更新统计
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

        // 记录晋升统计
        println!(
            "Young gen GC: collected={}, promoted={}, survivors={}, time={}us",
            collected_count,
            promoted_count,
            self.young_generation.len(),
            duration
        );

        self.marked_objects.clear();
        // 清空记忆集
        self.remembered_set.clear();
    }

    /// 决定对象是否应该晋升到老年代
    fn should_promote(&self, obj: &GcObject) -> bool {
        // 基于年龄的晋升策略
        if obj.header.is_tenured() {
            return true;
        }

        // 大对象直接晋升
        if obj.header.size > 1024 {
            return true;
        }

        // 频繁访问的对象提前晋升
        // 这里可以添加更复杂的晋升策略

        false
    }

    /// 初始化标记阶段
    fn init_mark(&mut self) {
        self.marked_objects.clear();
        self.work_list.clear();

        // 将根对象加入工作列表
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
                return true; // 标记完成
            }
        }

        false // 还需要继续标记
    }

    /// 并行标记对象
    fn mark_parallel(&mut self) {
        // 初始化标记
        self.init_mark();

        // 创建根对象的副本以避免借用冲突
        let roots_copy = self.roots.clone();

        // 串行标记所有根对象
        for root in roots_copy {
            self.mark_object(root);
        }

        // 处理工作列表中的所有对象
        while let Some(obj) = self.work_list.pop_front() {
            self.mark_object(obj);
        }
    }

    /// 并行处理对象引用
    fn process_object_in_parallel(
        obj: NonNull<GcObject>,
        work_queues: &[Arc<WorkStealingQueue>],
        marked_objects: &Arc<Mutex<&mut HashSet<usize>>>,
    ) {
        let ptr = obj.as_ptr() as usize;

        // 检查是否已经标记
        {
            let mut marked = marked_objects.lock().unwrap();
            if marked.contains(&ptr) {
                return;
            }
            marked.insert(ptr);
        }

        // 标记对象
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
                TsValue::Function(_) => {
                    // 函数可能引用外部变量，需要特殊处理
                    // 暂时不实现
                }
                _ => {}
            }
        }
    }

    /// 并行标记值中的引用
    fn mark_value_references_parallel(
        value: &TsValue,
        work_queues: &[Arc<WorkStealingQueue>],
        marked_objects: &Arc<Mutex<&mut HashSet<usize>>>,
    ) {
        // 递归标记 TsValue 中的所有引用
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
            TsValue::Function(_) => {
                // 函数可能引用外部变量，需要特殊处理
                // 暂时不实现
            }
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
            _ => {
                // 其他类型不包含引用
            }
        }
    }

    /// 标记所有对象
    fn mark_all(&mut self) {
        if self.parallelism > 1 && self.heap_objects.len() > 1000 {
            // 使用并行标记
            self.mark_parallel();
        }
        else {
            // 使用串行标记
            self.init_mark();

            // 处理工作列表
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
                TsValue::Function(_) => {
                    // 函数可能引用外部变量，需要特殊处理
                    // 暂时不实现
                }
                _ => {}
            }
        }
    }

    /// 标记值中的引用
    fn mark_value_references(&mut self, value: &TsValue) {
        // 递归标记 TsValue 中的所有引用
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
            TsValue::Function(_) => {
                // 函数可能引用外部变量，需要特殊处理
                // 暂时不实现
            }
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
            _ => {
                // 其他类型不包含引用
            }
        }
    }

    /// 初始化清除阶段
    fn init_sweep(&mut self) {
        // 清除阶段不需要特殊初始化
    }

    /// 执行增量清除
    fn sweep_incremental(&mut self) -> bool {
        // 分批处理清除工作
        const BATCH_SIZE: usize = 150; // 增加批处理大小以提高性能
        let mut processed = 0;
        let mut collected_count = 0;
        let mut collected_bytes = 0;
        let mut survivors = Vec::new();

        for &obj in &self.heap_objects {
            if processed >= BATCH_SIZE {
                // 分批处理，下次继续
                break;
            }

            let ptr = obj.as_ptr() as usize;
            unsafe {
                if self.marked_objects.contains(&ptr) {
                    // 对象存活，清除标记
                    (*obj.as_ptr()).header.mark = false;
                    survivors.push(obj);
                }
                else {
                    // 对象死亡，回收
                    collected_count += 1;
                    collected_bytes += (*obj.as_ptr()).total_size();
                    self.notify_object_collected((*obj.as_ptr()).total_size());

                    // 释放对象
                    let _ = Box::from_raw(obj.as_ptr());
                }
            }
            processed += 1;
        }

        // 更新存活对象列表
        if processed > 0 {
            let remaining: Vec<_> = self.heap_objects.iter().skip(processed).copied().collect();
            self.heap_objects = survivors;
            self.heap_objects.extend(remaining);

            // 同时更新新生代和老年代
            self.young_generation.retain(|&obj| {
                let ptr = obj.as_ptr() as usize;
                self.marked_objects.contains(&ptr)
            });

            self.old_generation.retain(|&obj| {
                let ptr = obj.as_ptr() as usize;
                self.marked_objects.contains(&ptr)
            });

            // 更新统计
            let duration = 0; // 简化处理
            self.stats.record_collection(duration, self.marked_objects.len(), collected_count, collected_bytes);
        }

        // 检查是否完成
        processed >= self.heap_objects.len()
    }

    /// 并行清除未标记的对象
    fn sweep_parallel(&mut self) -> (usize, usize) {
        // 使用串行清除代替并行清除，以避免线程安全问题
        let mut collected_count = 0;
        let mut collected_bytes = 0;
        let mut survivors = Vec::new();

        for &obj in &self.heap_objects {
            let ptr = obj.as_ptr() as usize;
            unsafe {
                if self.marked_objects.contains(&ptr) {
                    // 对象存活，清除标记
                    (*obj.as_ptr()).header.mark = false;
                    survivors.push(obj);
                }
                else {
                    // 对象死亡，回收
                    collected_count += 1;
                    collected_bytes += (*obj.as_ptr()).total_size();
                    self.notify_object_collected((*obj.as_ptr()).total_size());

                    // 释放对象
                    let _ = Box::from_raw(obj.as_ptr());
                }
            }
        }

        self.heap_objects = survivors;

        // 同时更新新生代和老年代
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
            // 使用并行清除
            self.sweep_parallel()
        }
        else {
            // 使用串行清除
            let mut collected_count = 0;
            let mut collected_bytes = 0;
            let mut survivors = Vec::new();

            for &obj in &self.heap_objects {
                let ptr = obj.as_ptr() as usize;
                unsafe {
                    if self.marked_objects.contains(&ptr) {
                        // 对象存活，清除标记
                        (*obj.as_ptr()).header.mark = false;
                        survivors.push(obj);
                    }
                    else {
                        // 对象死亡，回收
                        collected_count += 1;
                        collected_bytes += (*obj.as_ptr()).total_size();
                        self.notify_object_collected((*obj.as_ptr()).total_size());

                        // 释放对象
                        let _ = Box::from_raw(obj.as_ptr());
                    }
                }
            }

            self.heap_objects = survivors;

            // 同时更新新生代和老年代
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
    fn compact(&mut self) {
        // 简化的压缩实现
        // 实际实现应该更复杂，包括对象移动和指针更新

        // 按对象大小排序，以便更好地利用内存
        self.heap_objects.sort_by(|a, b| unsafe {
            let size_a = (*a.as_ptr()).total_size();
            let size_b = (*b.as_ptr()).total_size();
            size_a.cmp(&size_b)
        });

        // 清空空闲块列表，准备重新分配
        self.free_blocks.clear();
    }

    /// 执行增量内存压缩
    fn compact_incremental(&mut self) -> bool {
        // 简化的增量压缩实现
        // 实际实现应该更复杂，包括分批处理对象移动

        // 按对象大小排序，以便更好地利用内存
        self.heap_objects.sort_by(|a, b| unsafe {
            let size_a = (*a.as_ptr()).total_size();
            let size_b = (*b.as_ptr()).total_size();
            size_a.cmp(&size_b)
        });

        // 清空空闲块列表，准备重新分配
        self.free_blocks.clear();

        true // 压缩完成
    }

    /// 记录跨代引用
    pub fn record_cross_generation_reference(&mut self, old_obj: NonNull<GcObject>, young_obj: NonNull<GcObject>) {
        self.remembered_set.push(RememberedSetEntry { old_obj, young_obj });
    }

    /// 获取统计信息
    pub fn stats(&self) -> &GCStats {
        &self.stats
    }

    /// 获取堆对象数量
    pub fn heap_size(&self) -> usize {
        self.heap_objects.len()
    }

    /// 获取新生代大小
    pub fn young_gen_size(&self) -> usize {
        self.young_generation.len()
    }

    /// 获取老年代大小
    pub fn old_gen_size(&self) -> usize {
        self.old_generation.len()
    }

    /// 获取当前阶段
    pub fn phase(&self) -> GCPhase {
        self.phase
    }

    /// 检查是否需要垃圾回收
    pub fn should_collect(&self) -> bool {
        self.young_generation.len() >= self.young_gen_threshold
            || self.old_generation.len() >= self.old_gen_threshold
            || (self.total_memory > 0 && (self.used_memory as f64 / self.total_memory as f64) < 0.5) // 内存碎片率过高
    }

    /// 获取内存使用情况
    pub fn memory_usage(&self) -> (usize, usize) {
        (self.used_memory, self.total_memory)
    }

    /// 启用并发GC
    pub fn enable_concurrent(&mut self, enabled: bool) {
        self.concurrent_enabled = enabled;
    }

    /// 检查并发GC是否启用
    pub fn is_concurrent_enabled(&self) -> bool {
        self.concurrent_enabled
    }

    /// 启动并发标记
    fn start_concurrent_mark(&mut self) {
        // 暂时禁用并发标记，因为 NonNull<GcObject> 不实现 Send trait
        // 后续会实现线程安全的对象引用
        self.phase = GCPhase::Marking;
        self.mark_all();
        self.concurrent_mark_done = true;
    }

    /// 启动并发清除
    fn start_concurrent_sweep(&mut self) {
        // 暂时禁用并发清除，因为 NonNull<GcObject> 不实现 Send trait
        // 后续会实现线程安全的对象引用
        self.phase = GCPhase::Sweeping;
        self.sweep();
        self.concurrent_sweep_done = true;
    }

    /// 检查并发操作是否完成
    fn check_concurrent_status(&mut self) {
        // 检查并发标记
        if self.phase == GCPhase::ConcurrentMarking && self.concurrent_mark_done {
            self.phase = GCPhase::Marking;
            self.concurrent_mark_thread.take();
        }

        // 检查并发清除
        if self.phase == GCPhase::ConcurrentSweeping && self.concurrent_sweep_done {
            self.phase = GCPhase::Sweeping;
            self.concurrent_sweep_thread.take();
        }
    }
}

/// 并行处理对象引用（辅助函数）
fn process_object_in_parallel(
    obj: NonNull<GcObject>,
    work_queues: &[Arc<WorkStealingQueue>],
    marked_objects: &Arc<Mutex<&mut HashSet<usize>>>,
) {
    let ptr = obj.as_ptr() as usize;

    // 检查是否已经标记
    {
        let mut marked = marked_objects.lock().unwrap();
        if marked.contains(&ptr) {
            return;
        }
        marked.insert(ptr);
    }

    // 标记对象
    unsafe {
        (*obj.as_ptr()).header.mark = true;
    }

    // 处理对象引用
    unsafe {
        match &(*obj.as_ptr()).value {
            TsValue::Object(props) => {
                for (_, value) in props {
                    mark_value_references_parallel(value, work_queues, marked_objects);
                }
            }
            TsValue::Array(elements) => {
                for elem in elements {
                    mark_value_references_parallel(elem, work_queues, marked_objects);
                }
            }
            TsValue::Function(_) => {
                // 函数可能引用外部变量，需要特殊处理
                // 暂时不实现
            }
            _ => {}
        }
    }
}

/// 并行标记值中的引用（辅助函数）
fn mark_value_references_parallel(
    value: &TsValue,
    work_queues: &[Arc<WorkStealingQueue>],
    marked_objects: &Arc<Mutex<&mut HashSet<usize>>>,
) {
    // 递归标记 TsValue 中的所有引用
    match value {
        TsValue::Object(props) => {
            for (_, val) in props {
                mark_value_references_parallel(val, work_queues, marked_objects);
            }
        }
        TsValue::Array(elements) => {
            for elem in elements {
                mark_value_references_parallel(elem, work_queues, marked_objects);
            }
        }
        TsValue::Function(_) => {
            // 函数可能引用外部变量，需要特殊处理
            // 暂时不实现
        }
        TsValue::Union(values) => {
            for val in values {
                mark_value_references_parallel(val, work_queues, marked_objects);
            }
        }
        TsValue::Generic(_, args) => {
            for arg in args {
                mark_value_references_parallel(arg, work_queues, marked_objects);
            }
        }
        TsValue::Map(entries) => {
            for (key, val) in entries {
                mark_value_references_parallel(key, work_queues, marked_objects);
                mark_value_references_parallel(val, work_queues, marked_objects);
            }
        }
        TsValue::Set(values) => {
            for val in values {
                mark_value_references_parallel(val, work_queues, marked_objects);
            }
        }
        TsValue::Promise(value) => {
            mark_value_references_parallel(value, work_queues, marked_objects);
        }
        _ => {
            // 其他类型不包含引用
        }
    }
}

impl Default for GC {
    fn default() -> Self {
        Self::new()
    }
}

/// 内存管理器
///
/// 整合内存分配和垃圾收集功能
pub struct MemoryManager {
    /// 垃圾收集器
    gc: GC,
    /// 分配的对象计数
    object_count: usize,
    /// 触发垃圾回收的阈值
    threshold: usize,
    /// 总分配字节数
    total_allocated: usize,
    /// 上次垃圾回收时间
    last_collection_time: std::time::Instant,
}

impl MemoryManager {
    /// 创建一个新的内存管理器
    pub fn new(threshold: usize) -> Self {
        Self { gc: GC::new(), object_count: 0, threshold, total_allocated: 0, last_collection_time: std::time::Instant::now() }
    }

    /// 分配对象
    pub fn allocate(&mut self, value: TsValue, size: usize, type_id: u8) -> NonNull<GcObject> {
        self.object_count += 1;
        self.total_allocated += size;

        let ptr = self.gc.allocate(value, size, type_id);

        // 检查是否需要垃圾回收
        if self.object_count >= self.threshold
            || self.gc.should_collect()
            || self.last_collection_time.elapsed().as_millis() > 1000
        {
            // 定期触发 GC
            self.gc.collect_young_gen();
            self.object_count = 0;
            self.last_collection_time = std::time::Instant::now();
        }

        ptr
    }

    /// 添加根对象
    pub fn add_root(&mut self, obj: NonNull<GcObject>) {
        self.gc.add_root(obj);
    }

    /// 移除根对象
    pub fn remove_root(&mut self, obj: NonNull<GcObject>) {
        self.gc.remove_root(obj);
    }

    /// 手动触发垃圾回收
    pub fn collect(&mut self) {
        self.gc.collect();
        self.object_count = 0;
        self.last_collection_time = std::time::Instant::now();
    }

    /// 执行增量垃圾回收
    pub fn collect_incremental(&mut self) -> bool {
        let result = self.gc.collect_incremental();
        if result {
            self.object_count = 0;
            self.last_collection_time = std::time::Instant::now();
        }
        result
    }

    /// 记录跨代引用
    pub fn record_cross_generation_reference(&mut self, old_obj: NonNull<GcObject>, young_obj: NonNull<GcObject>) {
        self.gc.record_cross_generation_reference(old_obj, young_obj);
    }

    /// 获取垃圾收集器引用
    pub fn gc(&self) -> &GC {
        &self.gc
    }

    /// 获取垃圾收集器可变引用
    pub fn gc_mut(&mut self) -> &mut GC {
        &mut self.gc
    }

    /// 获取统计信息
    pub fn stats(&self) -> &GCStats {
        self.gc.stats()
    }

    /// 获取总分配字节数
    pub fn total_allocated(&self) -> usize {
        self.total_allocated
    }

    /// 设置垃圾回收阈值
    pub fn set_threshold(&mut self, threshold: usize) {
        self.threshold = threshold;
    }

    /// 获取垃圾回收阈值
    pub fn threshold(&self) -> usize {
        self.threshold
    }

    /// 获取内存使用情况
    pub fn memory_usage(&self) -> (usize, usize) {
        self.gc.memory_usage()
    }

    /// 获取对象计数
    pub fn get_object_count(&self) -> usize {
        self.object_count
    }

    /// 清除内存管理器状态
    pub fn clear(&mut self) {
        self.object_count = 0;
        self.total_allocated = 0;
        self.last_collection_time = std::time::Instant::now();
    }
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new(800) // 调整默认阈值以提高性能
    }
}
