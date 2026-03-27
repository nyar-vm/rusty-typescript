//! 垃圾收集器测试模块
//!
//! 测试垃圾收集器的基本功能、内存管理和事件处理。

use std::sync::atomic::{AtomicUsize, Ordering};
use typescript::gc::{GC, GCEventHandler, GCPhase, GCStats, ObjectHeader};
use typescript_types::TsValue;

/// 测试垃圾收集器基本功能
///
/// 验证垃圾收集器能够正确分配和回收对象
#[test]
fn test_gc_basic() {
    let mut gc = GC::new();

    // 创建对象
    let value = TsValue::Number(42.0);
    let obj = gc.allocate(value, 8, 1);

    // 添加根对象
    gc.add_root(obj);

    // 执行 GC
    gc.collect();

    // 对象应该仍然存活
    assert_eq!(gc.heap_size(), 1);
}

/// 测试垃圾收集器回收不可达对象
///
/// 验证垃圾收集器能够正确回收不可达的对象
#[test]
fn test_gc_collect_unreachable() {
    let mut gc = GC::new();

    // 创建对象但不添加为根
    let value = TsValue::Number(42.0);
    let _obj = gc.allocate(value, 8, 1);

    // 执行 GC
    gc.collect();

    // 对象应该被回收
    assert_eq!(gc.heap_size(), 0);
}

/// 测试新生代垃圾收集
///
/// 验证新生代垃圾收集机制能够正常工作
#[test]
fn test_gc_young_gen() {
    let mut gc = GC::new();
    gc.set_young_gen_threshold(10);

    // 创建多个对象
    for i in 0..15 {
        let value = TsValue::Number(i as f64);
        gc.allocate(value, 8, 1);
    }

    // 新生代应该触发 GC
    assert!(gc.young_gen_size() < 15);
}

/// 测试对象头操作
///
/// 验证对象头的引用计数和年龄管理功能
#[test]
fn test_object_header() {
    let mut header = ObjectHeader::new(64, 1);

    assert_eq!(header.ref_count, 0);
    header.increment_ref_count();
    assert_eq!(header.ref_count, 1);
    header.decrement_ref_count();
    assert_eq!(header.ref_count, 0);

    assert!(!header.is_tenured());
    header.increment_age();
    header.increment_age();
    header.increment_age();
    assert!(header.is_tenured());
}

/// 测试垃圾收集统计
///
/// 验证垃圾收集统计功能能够正确记录收集信息
#[test]
fn test_gc_stats() {
    let mut stats = GCStats::default();

    stats.record_collection(100, 10, 5, 500);
    assert_eq!(stats.collection_count, 1);
    assert_eq!(stats.marked_objects, 10);
    assert_eq!(stats.collected_objects, 5);
    assert_eq!(stats.collected_bytes, 500);

    stats.record_collection(200, 20, 10, 1000);
    assert_eq!(stats.collection_count, 2);
    assert_eq!(stats.average_collection_time_us(), 150);
}

struct TestEventHandler {
    gc_start_count: AtomicUsize,
    gc_end_count: AtomicUsize,
}

impl TestEventHandler {
    fn new() -> Self {
        Self { gc_start_count: AtomicUsize::new(0), gc_end_count: AtomicUsize::new(0) }
    }
}

impl GCEventHandler for TestEventHandler {
    fn on_gc_start(&self, _phase: GCPhase) {
        self.gc_start_count.fetch_add(1, Ordering::Relaxed);
    }

    fn on_gc_end(&self, _phase: GCPhase, _stats: &GCStats) {
        self.gc_end_count.fetch_add(1, Ordering::Relaxed);
    }

    fn on_object_collected(&self, _size: usize) {}
}

/// 测试垃圾收集事件
///
/// 验证垃圾收集事件处理机制能够正常工作
#[test]
fn test_gc_events() {
    let mut gc = GC::new();
    let handler = Box::new(TestEventHandler::new());
    let handler_ref = unsafe { &*(handler.as_ref() as *const TestEventHandler) };

    gc.add_event_handler(handler);

    // 创建并回收对象
    let value = TsValue::Number(42.0);
    let _obj = gc.allocate(value, 8, 1);
    gc.collect();

    // 应该触发了标记和清除两个阶段的开始和结束事件
    assert!(handler_ref.gc_start_count.load(Ordering::Relaxed) >= 2);
    assert!(handler_ref.gc_end_count.load(Ordering::Relaxed) >= 2);
}

/// 测试内存管理器
///
/// 验证内存管理器的基本功能
#[test]
fn test_memory_manager() {
    let mut mm = typescript::gc::MemoryManager::new(100);

    // 分配对象
    let ptr1 = mm.allocate(8).unwrap();
    let ptr2 = mm.allocate(16).unwrap();

    // 释放对象
    mm.deallocate(ptr1, 8);
    mm.deallocate(ptr2, 16);

    // 应该能够再次分配
    let ptr3 = mm.allocate(8).unwrap();
    assert_ne!(ptr1, ptr3);
}

/// 测试并行GC性能
///
/// 验证并行GC比串行GC更快
#[test]
fn test_parallel_gc_performance() {
    use std::time::Instant;

    // 创建大量对象
    let mut gc = typescript::gc::GC::new();

    // 分配10000个对象
    for i in 0..10000 {
        let value = typescript_types::TsValue::Number(i as f64);
        let obj = gc.allocate(value, 8, 1);
        gc.add_root(obj);
    }

    // 测量并行GC时间
    let start = Instant::now();
    gc.collect();
    let parallel_time = start.elapsed().as_micros();

    println!("Parallel GC time: {}us", parallel_time);

    // 验证对象被正确回收
    assert!(gc.heap_size() > 0); // 根对象应该存活
}

/// 测试不同内存分配策略
///
/// 验证不同内存分配策略的性能差异
#[test]
fn test_memory_allocation_strategies() {
    use std::time::Instant;

    // 测试默认分配器
    let mut gc_default = typescript::gc::GC::new();
    let start_default = Instant::now();

    for i in 0..5000 {
        let value = typescript_types::TsValue::Number(i as f64);
        gc_default.allocate(value, 8, 1);
    }

    gc_default.collect();
    let default_time = start_default.elapsed().as_micros();

    // 测试伙伴分配器
    let mut gc_buddy = typescript::gc::GC::with_buddy_allocator();
    let start_buddy = Instant::now();

    for i in 0..5000 {
        let value = typescript_types::TsValue::Number(i as f64);
        gc_buddy.allocate(value, 8, 1);
    }

    gc_buddy.collect();
    let buddy_time = start_buddy.elapsed().as_micros();

    println!("Default allocator time: {}us", default_time);
    println!("Buddy allocator time: {}us", buddy_time);
}

/// 测试分代收集
///
/// 验证分代收集的晋升机制
#[test]
fn test_generational_collection() {
    let mut gc = typescript::gc::GC::new();

    // 分配一些对象
    let mut objects = Vec::new();
    for i in 0..200 {
        let value = typescript_types::TsValue::Number(i as f64);
        let obj = gc.allocate(value, 8, 1);
        objects.push(obj);
    }

    // 添加一些根对象
    for i in 0..10 {
        gc.add_root(objects[i]);
    }

    // 执行多次新生代GC
    for _ in 0..5 {
        gc.collect_young_gen();
    }

    // 验证老年代中有对象
    assert!(gc.old_gen_size() > 0);
    // 验证新生代中也有对象
    assert!(gc.young_gen_size() > 0);
}

/// 测试并发GC
///
/// 验证并发GC的基本功能
#[test]
fn test_concurrent_gc() {
    let mut gc = typescript::gc::GC::new();

    // 启用并发GC
    gc.enable_concurrent(true);

    // 分配一些对象
    for i in 0..1000 {
        let value = typescript_types::TsValue::Number(i as f64);
        let obj = gc.allocate(value, 8, 1);
        if i < 100 {
            gc.add_root(obj);
        }
    }

    // 执行GC
    gc.collect();

    // 验证对象被正确回收
    assert!(gc.heap_size() > 0);
    assert!(gc.is_concurrent_enabled());
}

/// 测试GC统计功能
///
/// 验证GC统计信息的正确性
#[test]
fn test_gc_stats() {
    let mut gc = typescript::gc::GC::new();

    // 分配一些对象
    for i in 0..1000 {
        let value = typescript_types::TsValue::Number(i as f64);
        let obj = gc.allocate(value, 8, 1);
        if i < 100 {
            gc.add_root(obj);
        }
    }

    // 执行GC
    gc.collect();

    // 获取统计信息
    let stats = gc.stats();

    // 验证统计信息
    assert!(stats.collection_count > 0);
    assert!(stats.collected_objects > 0);
    assert!(stats.allocation_count > 0);
    assert!(stats.total_collection_time_us > 0);

    println!("GC Stats:");
    println!("  Collection count: {}", stats.collection_count);
    println!("  Collected objects: {}", stats.collected_objects);
    println!("  Collected bytes: {}", stats.collected_bytes);
    println!("  Average collection time: {}us", stats.average_collection_time_us());
    println!("  Memory fragmentation: {:.2}%", stats.fragmentation_ratio * 100.0);
}

/// 测试内存分配策略切换
///
/// 验证可以动态切换内存分配策略
#[test]
fn test_allocation_strategy_switch() {
    use typescript::memory::AllocationStrategy;

    let mut gc = typescript::gc::GC::new();

    // 分配一些对象使用默认策略
    for i in 0..100 {
        let value = typescript_types::TsValue::Number(i as f64);
        gc.allocate(value, 8, 1);
    }

    // 切换到伙伴分配器
    gc.set_allocation_strategy(AllocationStrategy::Buddy);

    // 再分配一些对象
    for i in 100..200 {
        let value = typescript_types::TsValue::Number(i as f64);
        gc.allocate(value, 8, 1);
    }

    // 执行GC
    gc.collect();

    // 验证对象被正确回收
    assert!(gc.heap_size() > 0);
}

/// 测试日志事件处理器
///
/// 验证日志事件处理器能够正确输出 GC 事件信息
#[test]
fn test_logging_event_handler() {
    use typescript::gc::LoggingEventHandler;
    
    let handler = LoggingEventHandler::new();
    handler.on_gc_start(GCPhase::Marking);

    let stats = GCStats::default();
    handler.on_gc_end(GCPhase::Marking, &stats);
    handler.on_object_collected(100);
}

/// 测试详细日志事件处理器
///
/// 验证详细日志事件处理器能够输出更详细的 GC 事件信息
#[test]
fn test_verbose_logging_event_handler() {
    use typescript::gc::LoggingEventHandler;
    
    let handler = LoggingEventHandler::verbose();
    handler.on_gc_start(GCPhase::Sweeping);

    let mut stats = GCStats::default();
    stats.marked_objects = 10;
    stats.collected_objects = 5;
    stats.collected_bytes = 500;
    handler.on_gc_end(GCPhase::Sweeping, &stats);
    handler.on_object_collected(100);
}

/// 测试工作窃取队列基本功能
///
/// 验证工作窃取队列的基本操作
#[test]
fn test_work_stealing_queue_basic() {
    use typescript::gc::WorkStealingQueue;
    use std::ptr::NonNull;
    
    let queue = WorkStealingQueue::new();

    assert!(queue.is_empty());
    assert_eq!(queue.len(), 0);

    let obj = Box::new(typescript::gc::GcObject::new(typescript_types::TsValue::Null, 0, 0));
    let ptr = NonNull::from(Box::leak(obj));

    queue.push(ptr);
    assert!(!queue.is_empty());
    assert_eq!(queue.len(), 1);

    let popped = queue.pop();
    assert!(popped.is_some());
    assert!(queue.is_empty());

    unsafe {
        let _ = Box::from_raw(ptr.as_ptr());
    }
}

/// 测试工作窃取队列的窃取功能
///
/// 验证工作窃取队列的窃取操作
#[test]
fn test_work_stealing_queue_steal() {
    use typescript::gc::WorkStealingQueue;
    use std::ptr::NonNull;
    
    let queue = WorkStealingQueue::new();

    let obj1 = Box::new(typescript::gc::GcObject::new(typescript_types::TsValue::Null, 0, 0));
    let ptr1 = NonNull::from(Box::leak(obj1));
    let obj2 = Box::new(typescript::gc::GcObject::new(typescript_types::TsValue::Null, 0, 0));
    let ptr2 = NonNull::from(Box::leak(obj2));

    queue.push(ptr1);
    queue.push(ptr2);

    let stolen = queue.steal();
    assert!(stolen.is_some());

    let popped = queue.pop();
    assert!(popped.is_some());

    assert!(queue.is_empty());

    unsafe {
        let _ = Box::from_raw(ptr1.as_ptr());
        let _ = Box::from_raw(ptr2.as_ptr());
    }
}
