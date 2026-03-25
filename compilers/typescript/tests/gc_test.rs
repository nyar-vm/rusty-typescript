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
