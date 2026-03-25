//! 垃圾收集模块
//!
//! 提供高效的垃圾收集机制，包括增量标记-清除、分代收集和引用计数。

use std::{
    collections::{HashMap, HashSet, VecDeque},
    ptr::NonNull,
    sync::atomic::{AtomicUsize, Ordering},
};

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
    /// 清除阶段
    Sweeping,
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
    }

    /// 获取平均收集时间
    pub fn average_collection_time_us(&self) -> u64 {
        if self.collection_count == 0 { 0 } else { self.total_collection_time_us / self.collection_count as u64 }
    }
}

/// 垃圾收集器
///
/// 实现增量标记-清除算法，支持分代收集
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
        Self {
            roots: Vec::new(),
            heap_objects: Vec::new(),
            young_generation: Vec::new(),
            old_generation: Vec::new(),
            marked_objects: HashSet::new(),
            work_list: VecDeque::new(),
            phase: GCPhase::Idle,
            stats: GCStats::default(),
            event_handlers: Vec::new(),
            incremental_step_size: 100,
            young_gen_threshold: 1000,
            old_gen_threshold: 10000,
        }
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

    /// 分配对象
    pub fn allocate(&mut self, value: TsValue, size: usize, type_id: u8) -> NonNull<GcObject> {
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
        self.phase = GCPhase::Idle;

        // 更新统计
        let duration = start_time.elapsed().as_micros() as u64;
        self.stats.record_collection(duration, self.marked_objects.len(), collected.0, collected.1);

        // 清空标记集合
        self.marked_objects.clear();
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
                    self.phase = GCPhase::Idle;
                    true
                }
                else {
                    false
                }
            }
            GCPhase::Compacting => {
                self.phase = GCPhase::Idle;
                true
            }
        }
    }

    /// 执行新生代垃圾回收
    pub fn collect_young_gen(&mut self) {
        let start_time = std::time::Instant::now();

        // 只标记根对象和新生代对象
        self.marked_objects.clear();
        self.work_list.clear();

        // 从根对象开始标记
        let roots: Vec<_> = self.roots.iter().copied().collect();
        for root in roots {
            self.mark_object(root);
        }

        // 处理工作列表
        while let Some(obj) = self.work_list.pop_front() {
            self.process_object_references(obj);
        }

        // 清除未标记的新生代对象
        let mut collected_count = 0;
        let mut collected_bytes = 0;
        let mut survivors = Vec::new();

        for &obj in &self.young_generation {
            let ptr = obj.as_ptr() as usize;
            unsafe {
                if self.marked_objects.contains(&ptr) {
                    // 对象存活，增加年龄
                    (*obj.as_ptr()).header.increment_age();

                    // 如果年龄达到阈值，晋升到老年代
                    if (*obj.as_ptr()).header.is_tenured() {
                        self.old_generation.push(obj);
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

        // 更新统计
        let duration = start_time.elapsed().as_micros() as u64;
        self.stats.record_collection(duration, self.marked_objects.len(), collected_count, collected_bytes);

        self.marked_objects.clear();
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

    /// 标记所有对象
    fn mark_all(&mut self) {
        self.init_mark();

        // 处理工作列表
        while let Some(obj) = self.work_list.pop_front() {
            self.process_object_references(obj);
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
        const BATCH_SIZE: usize = 100;
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

    /// 清除未标记的对象
    fn sweep(&mut self) -> (usize, usize) {
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
        self.young_generation.len() >= self.young_gen_threshold || self.old_generation.len() >= self.old_gen_threshold
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
}

impl MemoryManager {
    /// 创建一个新的内存管理器
    pub fn new(threshold: usize) -> Self {
        Self { gc: GC::new(), object_count: 0, threshold, total_allocated: 0 }
    }

    /// 分配对象
    pub fn allocate(&mut self, value: TsValue, size: usize, type_id: u8) -> NonNull<GcObject> {
        self.object_count += 1;
        self.total_allocated += size;

        let ptr = self.gc.allocate(value, size, type_id);

        // 检查是否需要垃圾回收
        if self.object_count >= self.threshold {
            self.gc.collect_young_gen();
            self.object_count = 0;
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
    }

    /// 执行增量垃圾回收
    pub fn collect_incremental(&mut self) -> bool {
        self.gc.collect_incremental()
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
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self::new(1000)
    }
}


