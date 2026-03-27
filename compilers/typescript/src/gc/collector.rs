//! 垃圾收集器核心模块

use std::{
    collections::{HashSet, VecDeque},
    ptr::NonNull,
    sync::{Arc, Mutex},
    thread,
};

use typescript_types::TsValue;

use crate::memory::{AllocationStrategy, Allocator, AllocatorFactory};

use super::{
    event::GCEventHandler,
    object::GcObject,
    phase::GCPhase,
    stats::GCStats,
    work_stealing::{RememberedSetEntry, WorkStealingQueue},
};

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
        }
    }

    /// 创建使用伙伴分配器的垃圾收集器
    pub fn with_buddy_allocator() -> Self {
        Self::with_strategy(AllocationStrategy::Buddy)
    }

    /// 切换内存分配策略
    pub fn set_allocation_strategy(&mut self, strategy: AllocationStrategy) {
        let new_allocator = AllocatorFactory::create(strategy);
        self.allocator = new_allocator;
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

        self.marked_objects.clear();
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
                    self.marked_objects.clear();
                    self.remembered_set.clear();
                    return true;
                }
                false
            }
            GCPhase::ConcurrentMarking => false,
            GCPhase::ConcurrentSweeping => false,
        }
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

        for &obj in &self.young_generation {
            let ptr = obj.as_ptr() as usize;
            unsafe {
                if self.marked_objects.contains(&ptr) {
                    (*obj.as_ptr()).header.increment_age();

                    if self.should_promote(&(*obj.as_ptr())) {
                        self.old_generation.push(obj);
                        promoted_count += 1;
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

        let mut promoted_bytes = 0;
        for &obj in &self.old_generation {
            unsafe {
                promoted_bytes += (*obj.as_ptr()).total_size();
            }
        }

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

        self.marked_objects.clear();
        self.remembered_set.clear();
    }

    /// 决定对象是否应该晋升到老年代
    fn should_promote(&self, obj: &GcObject) -> bool {
        if obj.header.is_tenured() {
            return true;
        }

        if obj.header.size > 1024 {
            return true;
        }

        false
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

    /// 并行标记对象
    fn mark_parallel(&mut self) {
        self.init_mark();

        let roots_copy = self.roots.clone();

        for root in roots_copy {
            self.mark_object(root);
        }

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
        let mut processed = 0;
        let mut collected_count = 0;
        let mut collected_bytes = 0;
        let mut survivors = Vec::new();

        for &obj in &self.heap_objects {
            if processed >= BATCH_SIZE {
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

            let duration = 0;
            self.stats.record_collection(duration, self.marked_objects.len(), collected_count, collected_bytes);
        }

        processed >= self.heap_objects.len()
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
    fn compact(&mut self) {
        self.heap_objects.sort_by(|a, b| unsafe {
            let size_a = (*a.as_ptr()).total_size();
            let size_b = (*b.as_ptr()).total_size();
            size_a.cmp(&size_b)
        });

        self.free_blocks.clear();
    }

    /// 执行增量内存压缩
    fn compact_incremental(&mut self) -> bool {
        self.heap_objects.sort_by(|a, b| unsafe {
            let size_a = (*a.as_ptr()).total_size();
            let size_b = (*b.as_ptr()).total_size();
            size_a.cmp(&size_b)
        });

        self.free_blocks.clear();

        true
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
            || (self.total_memory > 0 && (self.used_memory as f64 / self.total_memory as f64) < 0.5)
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
        self.phase = GCPhase::Marking;
        self.mark_all();
        self.concurrent_mark_done = true;
    }

    /// 启动并发清除
    fn start_concurrent_sweep(&mut self) {
        self.phase = GCPhase::Sweeping;
        self.sweep();
        self.concurrent_sweep_done = true;
    }

    /// 检查并发操作是否完成
    fn check_concurrent_status(&mut self) {
        if self.phase == GCPhase::ConcurrentMarking && self.concurrent_mark_done {
            self.phase = GCPhase::Marking;
            self.concurrent_mark_thread.take();
        }

        if self.phase == GCPhase::ConcurrentSweeping && self.concurrent_sweep_done {
            self.phase = GCPhase::Sweeping;
            self.concurrent_sweep_thread.take();
        }
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
