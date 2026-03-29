/// 增量编译模块
///
/// 提供文件依赖图、变更检测和编译缓存功能。
use crate::errors::{CompileError, CompileResult};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

/// 文件节点
#[derive(Debug, Clone)]
pub struct FileNode {
    /// 文件路径
    pub path: PathBuf,
    /// 最后修改时间
    pub modified: SystemTime,
    /// 文件哈希（用于检测内容变化）
    pub hash: u64,
    /// 依赖的文件
    pub dependencies: HashSet<PathBuf>,
    /// 被哪些文件依赖
    pub dependents: HashSet<PathBuf>,
}

impl FileNode {
    /// 创建新的文件节点
    pub fn new(path: PathBuf) -> CompileResult<Self> {
        let metadata =
            fs::metadata(&path).map_err(|e| CompileError::CompilationFailed(format!("无法读取文件元数据: {}", e)))?;

        let modified = metadata.modified().unwrap_or_else(|_| SystemTime::UNIX_EPOCH);

        let content = fs::read_to_string(&path).map_err(|e| CompileError::CompilationFailed(format!("无法读取文件: {}", e)))?;

        let hash = calculate_hash(&content);

        Ok(Self { path, modified, hash, dependencies: HashSet::new(), dependents: HashSet::new() })
    }

    /// 检查文件是否发生变化
    pub fn has_changed(&self) -> CompileResult<bool> {
        let metadata =
            fs::metadata(&self.path).map_err(|e| CompileError::CompilationFailed(format!("无法读取文件元数据: {}", e)))?;

        let modified = metadata.modified().unwrap_or_else(|_| SystemTime::UNIX_EPOCH);

        if modified != self.modified {
            return Ok(true);
        }

        /// 检查内容哈希
        let content =
            fs::read_to_string(&self.path).map_err(|e| CompileError::CompilationFailed(format!("无法读取文件: {}", e)))?;

        let hash = calculate_hash(&content);
        Ok(hash != self.hash)
    }

    /// 更新文件信息
    pub fn update(&mut self) -> CompileResult<()> {
        let metadata =
            fs::metadata(&self.path).map_err(|e| CompileError::CompilationFailed(format!("无法读取文件元数据: {}", e)))?;

        self.modified = metadata.modified().unwrap_or_else(|_| SystemTime::UNIX_EPOCH);

        let content =
            fs::read_to_string(&self.path).map_err(|e| CompileError::CompilationFailed(format!("无法读取文件: {}", e)))?;

        self.hash = calculate_hash(&content);

        // 重新解析依赖
        self.dependencies = parse_dependencies(&self.path, &content)?;

        Ok(())
    }
}

/// 文件依赖图
#[derive(Debug, Default)]
pub struct DependencyGraph {
    /// 文件节点映射
    nodes: HashMap<PathBuf, FileNode>,
}

impl DependencyGraph {
    /// 创建新的依赖图
    pub fn new() -> Self {
        Self { nodes: HashMap::new() }
    }

    /// 添加文件到依赖图
    pub fn add_file(&mut self, path: &Path) -> CompileResult<()> {
        if self.nodes.contains_key(path) {
            return Ok(());
        }

        let node = FileNode::new(path.to_path_buf())?;
        self.nodes.insert(path.to_path_buf(), node);

        /// 更新依赖关系
        self.update_dependencies(path)?;

        Ok(())
    }

    /// 更新文件的依赖关系
    fn update_dependencies(&mut self, path: &Path) -> CompileResult<()> {
        let node = self.nodes.get(path).ok_or_else(|| CompileError::CompilationFailed("文件不存在".to_string()))?;

        let dependencies: Vec<PathBuf> = node.dependencies.iter().cloned().collect();

        /// 更新被依赖关系
        for dep in dependencies {
            if let Some(dep_node) = self.nodes.get_mut(&dep) {
                dep_node.dependents.insert(path.to_path_buf());
            }
        }

        Ok(())
    }

    /// 获取文件的依赖文件
    pub fn get_dependencies(&self, path: &Path) -> Vec<PathBuf> {
        self.nodes.get(path).map(|n| n.dependencies.iter().cloned().collect()).unwrap_or_default()
    }

    /// 获取依赖于指定文件的文件
    pub fn get_dependents(&self, path: &Path) -> Vec<PathBuf> {
        self.nodes.get(path).map(|n| n.dependents.iter().cloned().collect()).unwrap_or_default()
    }

    /// 获取所有需要重新编译的文件（包括传递依赖）
    pub fn get_files_to_recompile(&self, changed_files: &[PathBuf]) -> Vec<PathBuf> {
        let mut to_recompile: HashSet<PathBuf> = HashSet::new();
        let mut queue: Vec<PathBuf> = changed_files.to_vec();

        while let Some(file) = queue.pop() {
            if to_recompile.insert(file.clone()) {
                /// 添加所有依赖此文件的文件
                let dependents = self.get_dependents(&file);
                queue.extend(dependents);
            }
        }

        to_recompile.into_iter().collect()
    }

    /// 检测变更的文件
    pub fn detect_changes(&self) -> CompileResult<Vec<PathBuf>> {
        let mut changed = Vec::new();

        for (path, node) in &self.nodes {
            if node.has_changed()? {
                changed.push(path.clone());
            }
        }

        Ok(changed)
    }

    /// 更新文件信息
    pub fn update_file(&mut self, path: &Path) -> CompileResult<()> {
        if let Some(node) = self.nodes.get_mut(path) {
            node.update()?;
        }
        else {
            self.add_file(path)?;
        }

        Ok(())
    }

    /// 移除文件
    pub fn remove_file(&mut self, path: &Path) {
        /// 移除被依赖关系
        if let Some(node) = self.nodes.get(path) {
            let dependencies: Vec<PathBuf> = node.dependencies.iter().cloned().collect();
            for dep in dependencies {
                if let Some(dep_node) = self.nodes.get_mut(&dep) {
                    dep_node.dependents.remove(path);
                }
            }
        }

        self.nodes.remove(path);
    }

    /// 获取所有文件
    pub fn get_all_files(&self) -> Vec<PathBuf> {
        self.nodes.keys().cloned().collect()
    }
}

/// 编译缓存
#[derive(Debug, Default)]
pub struct CompileCache {
    /// 缓存的文件哈希
    file_hashes: HashMap<PathBuf, u64>,
    /// 缓存的编译结果路径
    cached_outputs: HashMap<PathBuf, PathBuf>,
}

impl CompileCache {
    /// 创建新的编译缓存
    pub fn new() -> Self {
        Self { file_hashes: HashMap::new(), cached_outputs: HashMap::new() }
    }

    /// 检查文件是否在缓存中且未变更
    pub fn is_cached(&self, path: &Path) -> bool {
        if let Some(cached_hash) = self.file_hashes.get(path) {
            if let Ok(content) = fs::read_to_string(path) {
                let current_hash = calculate_hash(&content);
                return *cached_hash == current_hash;
            }
        }
        false
    }

    /// 添加缓存
    pub fn add_cache(&mut self, source: &Path, output: &Path) -> CompileResult<()> {
        let content =
            fs::read_to_string(source).map_err(|e| CompileError::CompilationFailed(format!("无法读取文件: {}", e)))?;

        let hash = calculate_hash(&content);
        self.file_hashes.insert(source.to_path_buf(), hash);
        self.cached_outputs.insert(source.to_path_buf(), output.to_path_buf());

        Ok(())
    }

    /// 获取缓存的输出路径
    pub fn get_cached_output(&self, path: &Path) -> Option<&PathBuf> {
        self.cached_outputs.get(path)
    }

    /// 清除缓存
    pub fn clear(&mut self) {
        self.file_hashes.clear();
        self.cached_outputs.clear();
    }

    /// 移除缓存
    pub fn remove_cache(&mut self, path: &Path) {
        self.file_hashes.remove(path);
        self.cached_outputs.remove(path);
    }
}

/// 增量编译器
pub struct IncrementalCompiler {
    /// 依赖图
    dependency_graph: DependencyGraph,
    /// 编译缓存
    cache: CompileCache,
}

impl IncrementalCompiler {
    /// 创建新的增量编译器
    pub fn new() -> Self {
        Self { dependency_graph: DependencyGraph::new(), cache: CompileCache::new() }
    }

    /// 初始化项目
    pub fn init_project(&mut self, files: &[PathBuf]) -> CompileResult<()> {
        for file in files {
            self.dependency_graph.add_file(file)?;
        }
        Ok(())
    }

    /// 获取需要编译的文件
    pub fn get_files_to_compile(&self) -> CompileResult<Vec<PathBuf>> {
        let changed = self.dependency_graph.detect_changes()?;
        let to_compile = self.dependency_graph.get_files_to_recompile(&changed);

        /// 过滤掉缓存中未变更的文件
        let result: Vec<PathBuf> = to_compile.into_iter().filter(|f| !self.cache.is_cached(f)).collect();

        Ok(result)
    }

    /// 标记文件已编译
    pub fn mark_compiled(&mut self, source: &Path, output: &Path) -> CompileResult<()> {
        self.dependency_graph.update_file(source)?;
        self.cache.add_cache(source, output)?;
        Ok(())
    }

    /// 获取依赖图
    pub fn dependency_graph(&self) -> &DependencyGraph {
        &self.dependency_graph
    }

    /// 获取缓存
    pub fn cache(&self) -> &CompileCache {
        &self.cache
    }

    /// 清除缓存
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}

/// 计算字符串哈希
fn calculate_hash(content: &str) -> u64 {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    let mut hasher = DefaultHasher::new();
    content.hash(&mut hasher);
    hasher.finish()
}

/// 解析文件依赖
fn parse_dependencies(path: &Path, content: &str) -> CompileResult<HashSet<PathBuf>> {
    let mut dependencies = HashSet::new();
    let base_dir = path.parent().ok_or_else(|| CompileError::CompilationFailed("无法获取父目录".to_string()))?;

    /// 解析 import 语句
    for line in content.lines() {
        let trimmed = line.trim();

        /// 匹配 import ... from '...' 或 import ... from "..."
        if trimmed.starts_with("import ") {
            if let Some(from_pos) = trimmed.find(" from ") {
                let after_from = &trimmed[from_pos + 6..];
                if let Some(quote_start) = after_from.find(|c| c == '\'' || c == '"') {
                    let after_quote = &after_from[quote_start + 1..];
                    if let Some(quote_end) = after_quote.find(|c| c == '\'' || c == '"') {
                        let module_path = &after_quote[..quote_end];

                        /// 解析相对路径
                        if module_path.starts_with("./") || module_path.starts_with("../") {
                            let resolved = resolve_module_path(base_dir, module_path)?;
                            dependencies.insert(resolved);
                        }
                    }
                }
            }
        }

        /// 匹配 require('...') 或 require("...")
        if trimmed.contains("require(") {
            if let Some(start) = trimmed.find("require(") {
                let after_require = &trimmed[start + 8..];
                if let Some(quote_start) = after_require.find(|c| c == '\'' || c == '"') {
                    let after_quote = &after_require[quote_start + 1..];
                    if let Some(quote_end) = after_quote.find(|c| c == '\'' || c == '"') {
                        let module_path = &after_quote[..quote_end];

                        /// 解析相对路径
                        if module_path.starts_with("./") || module_path.starts_with("../") {
                            let resolved = resolve_module_path(base_dir, module_path)?;
                            dependencies.insert(resolved);
                        }
                    }
                }
            }
        }
    }

    Ok(dependencies)
}

/// 解析模块路径
fn resolve_module_path(base_dir: &Path, module_path: &str) -> CompileResult<PathBuf> {
    let resolved = base_dir.join(module_path);

    /// 尝试添加扩展名
    let extensions = [".ts", ".tsx", ".js", ".jsx", "/index.ts", "/index.tsx", "/index.js", "/index.jsx"];

    for ext in &extensions {
        let with_ext =
            if module_path.ends_with(ext) { resolved.clone() } else { PathBuf::from(format!("{}{}", resolved.display(), ext)) };

        if with_ext.exists() {
            return Ok(with_ext.canonicalize().unwrap_or(with_ext));
        }
    }

    /// 返回原始路径
    Ok(resolved.canonicalize().unwrap_or(resolved))
}
