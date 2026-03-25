//! 编译缓存模块
//!
//! 提供编译过程中的缓存功能，支持增量编译和中间结果复用。

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    time::SystemTime,
};
use typescript_ir::{Program, TypedProgram};
use typescript_types::TsValue;

/// 缓存项元数据
#[derive(Debug, Clone)]
pub struct CacheMetadata {
    /// 缓存项的哈希值
    pub hash: String,
    /// 缓存项的创建时间
    pub created_at: SystemTime,
    /// 缓存项的依赖项
    pub dependencies: HashSet<String>,
    /// 缓存项的编译阶段
    pub stage: crate::compiler::CompilationStage,
}

/// 缓存项
#[derive(Debug, Clone)]
pub enum CacheItem {
    /// 词法分析结果
    LexicalResult(Vec<u8>),
    /// 语法分析结果（使用序列化后的数据）
    SyntaxResult(Vec<u8>),
    /// 语义分析结果
    SemanticResult(Box<HashMap<String, crate::compiler::context::Symbol>>),
    /// 中间表示结果
    IRResult(Box<Program>),
    /// 优化结果
    OptimizationResult(Box<TypedProgram>),
    /// 代码生成结果
    CodegenResult(Box<Vec<u8>>),
    /// 执行结果
    ExecutionResult(Box<TsValue>),
}

/// 编译缓存
#[derive(Debug, Clone)]
pub struct CompilationCache {
    /// 内存缓存
    memory_cache: HashMap<String, (CacheMetadata, CacheItem)>,
    /// 磁盘缓存路径
    disk_cache_path: Option<PathBuf>,
    /// 是否启用磁盘缓存
    enable_disk_cache: bool,
}

impl CompilationCache {
    /// 创建新的编译缓存
    pub fn new(disk_cache_path: Option<PathBuf>, enable_disk_cache: bool) -> Self {
        Self { memory_cache: HashMap::new(), disk_cache_path, enable_disk_cache }
    }

    /// 计算源内容的哈希值
    pub fn compute_hash(source: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(source.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// 检查缓存是否有效
    pub fn is_cache_valid(&self, hash: &str, dependencies: &HashSet<String>) -> bool {
        if let Some((metadata, _)) = self.memory_cache.get(hash) {
            // 检查依赖项是否发生变化
            if metadata.dependencies == *dependencies {
                return true;
            }
        }

        // 暂时禁用磁盘缓存，因为缺少 serde 实现
        // if self.enable_disk_cache && self.disk_cache_path.is_some() {
        //     let cache_file = self.disk_cache_path.as_ref().unwrap().join(format!("{}.json", hash));
        //     if cache_file.exists() {
        //         // 读取磁盘缓存并检查依赖项
        //         if let Ok(mut file) = File::open(cache_file) {
        //             let mut content = String::new();
        //             if file.read_to_string(&mut content).is_ok() {
        //                 if let Ok((metadata, _)) = serde_json::from_str::<(CacheMetadata, CacheItem)>(&content) {
        //                     return metadata.dependencies == *dependencies;
        //                 }
        //             }
        //         }
        //     }
        // }

        false
    }

    /// 从缓存中获取项
    pub fn get_cache(&mut self, hash: &str) -> Option<&(CacheMetadata, CacheItem)> {
        // 首先检查内存缓存
        if self.memory_cache.contains_key(hash) {
            return self.memory_cache.get(hash);
        }

        // 暂时禁用磁盘缓存，因为缺少 serde 实现
        // if self.enable_disk_cache && self.disk_cache_path.is_some() {
        //     let cache_file = self.disk_cache_path.as_ref().unwrap().join(format!("{}.json", hash));
        //     if cache_file.exists() {
        //         if let Ok(mut file) = File::open(cache_file) {
        //             let mut content = String::new();
        //             if file.read_to_string(&mut content).is_ok() {
        //                 if let Ok(item) = serde_json::from_str::<(CacheMetadata, CacheItem)>(&content) {
        //                     // 将磁盘缓存加载到内存
        //                     self.memory_cache.insert(hash.to_string(), item);
        //                     // 再次获取引用
        //                     return self.memory_cache.get(hash);
        //                 }
        //             }
        //         }
        //     }
        // }

        None
    }

    /// 存储缓存项
    pub fn store_cache(
        &mut self,
        hash: &str,
        item: CacheItem,
        dependencies: HashSet<String>,
        stage: crate::compiler::CompilationStage,
    ) {
        let metadata = CacheMetadata { hash: hash.to_string(), created_at: SystemTime::now(), dependencies, stage };

        // 存储到内存缓存
        self.memory_cache.insert(hash.to_string(), (metadata.clone(), item.clone()));

        // 暂时禁用磁盘缓存，因为缺少 serde 实现
        // if self.enable_disk_cache && self.disk_cache_path.is_some() {
        //     let cache_dir = self.disk_cache_path.as_ref().unwrap();
        //     if !cache_dir.exists() {
        //         let _ = fs::create_dir_all(cache_dir);
        //     }
        //
        //     let cache_file = cache_dir.join(format!("{}.json", hash));
        //     if let Ok(mut file) = File::create(cache_file) {
        //         let content = serde_json::to_string(&(metadata, item)).unwrap();
        //         let _ = file.write_all(content.as_bytes());
        //     }
        // }
    }

    /// 清除缓存
    pub fn clear_cache(&mut self) {
        // 清除内存缓存
        self.memory_cache.clear();

        // 暂时禁用磁盘缓存，因为缺少 serde 实现
        // if self.enable_disk_cache && self.disk_cache_path.is_some() {
        //     let cache_dir = self.disk_cache_path.as_ref().unwrap();
        //     if cache_dir.exists() {
        //         let _ = fs::remove_dir_all(cache_dir);
        //         let _ = fs::create_dir_all(cache_dir);
        //     }
        // }
    }

    /// 移除特定缓存项
    pub fn remove_cache(&mut self, hash: &str) {
        // 从内存缓存中移除
        self.memory_cache.remove(hash);

        // 暂时禁用磁盘缓存，因为缺少 serde 实现
        // if self.enable_disk_cache && self.disk_cache_path.is_some() {
        //     let cache_file = self.disk_cache_path.as_ref().unwrap().join(format!("{}.json", hash));
        //     if cache_file.exists() {
        //         let _ = fs::remove_file(cache_file);
        //     }
        // }
    }

    /// 获取缓存大小
    pub fn cache_size(&self) -> usize {
        self.memory_cache.len()
    }
}

impl Default for CompilationCache {
    fn default() -> Self {
        Self::new(None, false)
    }
}
