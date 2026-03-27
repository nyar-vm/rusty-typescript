//! 增量编译测试
//!
//! 测试增量编译的核心功能，验证文件依赖图和编译缓存的正确性。

use std::path::Path;
use typescript_tools::compiler::{CompileCache, DependencyGraph};

#[test]
fn test_calculate_hash() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn calculate_hash(content: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    let hash1 = calculate_hash("hello");
    let hash2 = calculate_hash("hello");
    let hash3 = calculate_hash("world");

    assert_eq!(hash1, hash2);
    assert_ne!(hash1, hash3);
}

#[test]
fn test_compile_cache() {
    let mut cache = CompileCache::new();

    assert!(!cache.is_cached(Path::new("/nonexistent.ts")));
}

#[test]
fn test_dependency_graph() {
    let mut graph = DependencyGraph::new();

    assert!(graph.get_all_files().is_empty());
    assert!(graph.get_dependencies(Path::new("/test.ts")).is_empty());
}
