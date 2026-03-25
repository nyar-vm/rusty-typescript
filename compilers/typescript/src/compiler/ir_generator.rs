//! 中间表示生成器模块
//!
//! 负责将抽象语法树转换为中间表示（IR）。

use crate::compiler::{CompilationResult, CompilationStage};
use std::{collections::HashMap, sync::Arc};
use typescript_ir::Program;

use typescript_types::TsError;

/// 中间表示生成器
#[derive(Debug, Clone)]
pub struct IRGenerator {
    /// 当前编译阶段
    current_stage: CompilationStage,
    /// IR生成缓存，使用AST哈希作为键
    ir_cache: HashMap<u64, Arc<Program>>,
    /// 缓存大小限制
    cache_size_limit: usize,
}

impl IRGenerator {
    /// 创建新的中间表示生成器
    pub fn new() -> Self {
        Self { current_stage: CompilationStage::IRGeneration, ir_cache: HashMap::new(), cache_size_limit: 500 }
    }

    /// 计算哈希值
    fn compute_hash(source: &str) -> u64 {
        use std::{
            collections::hash_map::DefaultHasher,
            hash::{Hash, Hasher},
        };
        let mut hasher = DefaultHasher::new();
        source.hash(&mut hasher);
        hasher.finish()
    }

    /// 生成中间表示
    pub fn generate(&mut self, source: &str) -> CompilationResult<Program> {
        self.current_stage = CompilationStage::IRGeneration;

        // 计算哈希值
        let hash = Self::compute_hash(source);

        // 检查缓存
        if let Some(ir) = self.ir_cache.get(&hash) {
            return CompilationResult::Success(ir.as_ref().clone());
        }

        // 简化的 IR 生成实现
        let ir = Program { statements: vec![] };

        // 存储到缓存
        let ir_arc = Arc::new(ir.clone());
        self.ir_cache.insert(hash, ir_arc);

        // 管理缓存大小
        self.manage_cache_size();

        CompilationResult::Success(ir)
    }

    /// 并行生成多个中间表示
    #[cfg(feature = "parallel")]
    pub fn generate_parallel(&mut self, sources: &[&str]) -> Vec<CompilationResult<Program>> {
        use rayon::prelude::*;
        sources.par_iter().map(|source| self.generate(source)).collect()
    }

    /// 管理缓存大小
    fn manage_cache_size(&mut self) {
        if self.ir_cache.len() > self.cache_size_limit {
            // 简单的 LRU 策略：移除一半的缓存项
            let remove_count = self.ir_cache.len() / 2;
            let keys_to_remove: Vec<u64> = self.ir_cache.keys().take(remove_count).cloned().collect();
            for key in keys_to_remove {
                self.ir_cache.remove(&key);
            }
        }
    }

    /// 获取当前编译阶段
    pub fn current_stage(&self) -> CompilationStage {
        self.current_stage
    }

    /// 重置中间表示生成器
    pub fn reset(&mut self) {
        self.current_stage = CompilationStage::IRGeneration;
        // 清空缓存
        self.ir_cache.clear();
    }

    /// 清除缓存
    pub fn clear_cache(&mut self) {
        self.ir_cache.clear();
    }

    /// 设置缓存大小限制
    pub fn set_cache_size_limit(&mut self, limit: usize) {
        self.cache_size_limit = limit;
        // 立即调整缓存大小
        self.manage_cache_size();
    }

    /// 获取缓存大小
    pub fn cache_size(&self) -> usize {
        self.ir_cache.len()
    }
}

impl Default for IRGenerator {
    fn default() -> Self {
        Self::new()
    }
}
