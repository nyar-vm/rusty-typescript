//! 内存管理测试模块
//!
//! 测试内存池、对象池和内存统计功能。

use typescript::memory::{GlobalMemoryManager, MemoryPool, ObjectPool};

/// 测试内存池的分配和释放
///
/// 验证内存池能够正确分配和释放内存
#[test]
fn test_memory_pool_allocate_deallocate() {
    let mut pool = MemoryPool::new();

    // 分配内存
    let ptr = pool.allocate(64).unwrap();
    assert!(!ptr.as_ptr().is_null());

    // 释放内存
    pool.deallocate(ptr, 64);

    // 检查统计
    let stats = pool.stats();
    assert_eq!(stats.total_used, 0);
}

/// 测试对象池的获取和释放
///
/// 验证对象池能够正确管理对象的生命周期
#[test]
fn test_object_pool() {
    let mut pool = ObjectPool::new(10, || Vec::<i32>::with_capacity(100), |v| v.clear());

    // 获取对象
    let mut obj = pool.acquire();
    obj.push(1);
    obj.push(2);

    // 释放对象
    pool.release(obj);
    assert_eq!(pool.free_count(), 1);

    // 再次获取，应该得到重置后的对象
    let obj2 = pool.acquire();
    assert!(obj2.is_empty());
}

/// 测试内存统计功能
///
/// 验证内存统计能够正确记录内存使用情况
#[test]
fn test_memory_stats() {
    let mut pool = MemoryPool::new();

    // 分配一些内存
    let ptr1 = pool.allocate(64).unwrap();
    let ptr2 = pool.allocate(128).unwrap();

    let stats = pool.stats();
    assert!(stats.total_used > 0);
    assert!(stats.usage_percentage() > 0.0);

    // 释放内存
    pool.deallocate(ptr1, 64);
    pool.deallocate(ptr2, 128);

    let stats = pool.stats();
    assert_eq!(stats.total_used, 0);
}

/// 测试全局内存管理器
///
/// 验证全局内存管理器能够正确管理内存分配
#[test]
fn test_global_memory_manager() {
    let mut manager = GlobalMemoryManager::new(100);

    // 分配内存
    let ptr1 = manager.allocate(8).unwrap();
    let ptr2 = manager.allocate(16).unwrap();

    // 释放内存
    manager.deallocate(ptr1, 8);
    manager.deallocate(ptr2, 16);

    // 应该能够再次分配
    let ptr3 = manager.allocate(8).unwrap();
    assert_ne!(ptr1, ptr3);
}
