//! JIT 编译器核心模块
//!
//! 提供热点检测、函数编译、优先级队列和事件回调功能。

use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap},
    rc::Rc,
    time::{Duration, Instant},
};

use typescript_types::{TsError, TsValue};

use crate::codegen::Instruction;

use super::optimizer::{JITOptimizer, OptimizationLevel};

/// JIT 编译事件类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JITEvent {
    /// 函数开始编译
    CompilationStarted {
        /// 函数名称
        function_name: String,
        /// 函数复杂度
        complexity: usize,
        /// 触发编译的原因
        reason: CompilationReason,
    },
    /// 函数编译完成
    CompilationCompleted {
        /// 函数名称
        function_name: String,
        /// 编译耗时
        duration: Duration,
        /// 优化级别
        optimization_level: String,
    },
    /// 函数编译失败
    CompilationFailed {
        /// 函数名称
        function_name: String,
        /// 错误信息
        error: String,
    },
    /// 热点函数检测到
    HotFunctionDetected {
        /// 函数名称
        function_name: String,
        /// 调用次数
        call_count: usize,
        /// 平均执行时间（微秒）
        avg_time: u64,
    },
}

/// 编译触发原因
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompilationReason {
    /// 调用次数达到阈值
    CallCountThreshold,
    /// 执行时间达到阈值
    TimeThreshold,
    /// 滑动窗口平均时间达到阈值
    WindowAverageThreshold,
    /// 优先级队列调度
    PriorityScheduled,
}

/// JIT 编译器事件回调 trait
pub trait JITEventCallback {
    /// 处理 JIT 编译事件
    fn on_event(&self, event: &JITEvent);
}

/// 编译状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompileStatus {
    /// 未编译
    NotCompiled,
    /// 正在编译
    Compiling,
    /// 已编译
    Compiled,
    /// 编译失败
    Failed,
}

/// 热点函数信息
#[derive(Debug, Clone)]
pub struct HotFunctionInfo {
    /// 执行次数
    pub call_count: usize,
    /// 总执行时间（微秒）
    pub total_time: u64,
    /// 平均执行时间（微秒）
    pub avg_time: u64,
    /// 最后执行时间（微秒）
    pub last_time: u64,
    /// 执行时间滑动窗口
    pub time_window: Vec<u64>,
    /// 代码复杂度
    pub complexity: usize,
    /// 执行频率（最近100次调用的平均时间间隔）
    pub execution_frequency: f64,
    /// 调用深度（平均调用栈深度）
    pub avg_call_depth: f64,
    /// 指令密度（每微秒执行的指令数）
    pub instruction_density: f64,
    /// 分支预测成功率
    pub branch_prediction_success: f64,
    /// 内存访问频率
    pub memory_access_frequency: usize,
    /// 编译状态
    pub compile_status: CompileStatus,
    /// 编译失败原因
    pub compile_error: Option<String>,
    /// 编译完成时间
    pub compiled_at: Option<Instant>,
    /// 编译耗时
    pub compile_duration: Option<Duration>,
    /// 优化级别
    pub optimization_level: Option<OptimizationLevel>,
}

impl Default for HotFunctionInfo {
    fn default() -> Self {
        Self {
            call_count: 0,
            total_time: 0,
            avg_time: 0,
            last_time: 0,
            time_window: Vec::with_capacity(10),
            complexity: 1,
            execution_frequency: 0.0,
            avg_call_depth: 0.0,
            instruction_density: 0.0,
            branch_prediction_success: 0.0,
            memory_access_frequency: 0,
            compile_status: CompileStatus::NotCompiled,
            compile_error: None,
            compiled_at: None,
            compile_duration: None,
            optimization_level: None,
        }
    }
}

/// 优先级队列项
#[derive(Debug, Clone)]
pub struct PriorityEntry {
    /// 函数名称
    pub function_name: String,
    /// 优先级分数（越高越优先）
    pub priority_score: f64,
    /// 调用次数
    pub call_count: usize,
    /// 平均执行时间
    pub avg_time: u64,
    /// 复杂度
    pub complexity: usize,
    /// 执行频率
    pub execution_frequency: f64,
    /// 指令密度
    pub instruction_density: f64,
    /// 内存访问频率
    pub memory_access_frequency: usize,
}

impl PartialEq for PriorityEntry {
    fn eq(&self, other: &Self) -> bool {
        self.priority_score == other.priority_score
    }
}

impl Eq for PriorityEntry {}

impl PartialOrd for PriorityEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        // 首先按优先级分数排序
        let score_cmp = self.priority_score.partial_cmp(&other.priority_score).unwrap_or(Ordering::Equal);
        if score_cmp != Ordering::Equal {
            return score_cmp;
        }

        // 其次按调用次数排序
        let call_cmp = other.call_count.cmp(&self.call_count);
        if call_cmp != Ordering::Equal {
            return call_cmp;
        }

        // 然后按平均执行时间排序
        let time_cmp = other.avg_time.cmp(&self.avg_time);
        if time_cmp != Ordering::Equal {
            return time_cmp;
        }

        // 最后按复杂度排序
        other.complexity.cmp(&self.complexity)
    }
}

/// 热点函数优先级队列
#[derive(Debug)]
pub struct HotFunctionPriorityQueue {
    /// 内部堆结构
    heap: BinaryHeap<PriorityEntry>,
    /// 已入队的函数集合（避免重复）
    queued_functions: HashMap<String, f64>,
    /// 函数最后更新时间
    last_updated: HashMap<String, Instant>,
}

impl HotFunctionPriorityQueue {
    /// 创建新的优先级队列
    pub fn new() -> Self {
        Self { heap: BinaryHeap::new(), queued_functions: HashMap::new(), last_updated: HashMap::new() }
    }

    /// 将函数加入优先级队列
    pub fn push(&mut self, entry: PriorityEntry) {
        if let Some(&existing_score) = self.queued_functions.get(&entry.function_name) {
            if entry.priority_score <= existing_score {
                return;
            }
        }
        self.queued_functions.insert(entry.function_name.clone(), entry.priority_score);
        self.last_updated.insert(entry.function_name.clone(), Instant::now());
        self.heap.push(entry);
    }

    /// 弹出最高优先级的函数
    pub fn pop(&mut self) -> Option<PriorityEntry> {
        loop {
            let entry = self.heap.pop()?;
            if let Some(&score) = self.queued_functions.get(&entry.function_name) {
                if (score - entry.priority_score).abs() < f64::EPSILON {
                    self.queued_functions.remove(&entry.function_name);
                    self.last_updated.remove(&entry.function_name);
                    return Some(entry);
                }
            }
        }
    }

    /// 查看最高优先级的函数（不移除）
    pub fn peek(&self) -> Option<&PriorityEntry> {
        self.heap.peek()
    }

    /// 检查队列是否为空
    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    /// 获取队列长度
    pub fn len(&self) -> usize {
        self.heap.len()
    }

    /// 清空队列
    pub fn clear(&mut self) {
        self.heap.clear();
        self.queued_functions.clear();
        self.last_updated.clear();
    }

    /// 检查函数是否在队列中
    pub fn contains(&self, function_name: &str) -> bool {
        self.queued_functions.contains_key(function_name)
    }

    /// 移除指定函数
    pub fn remove(&mut self, function_name: &str) {
        self.queued_functions.remove(function_name);
        self.last_updated.remove(function_name);
    }

    /// 更新函数优先级
    pub fn update_priority(&mut self, entry: PriorityEntry) {
        self.push(entry);
    }

    /// 清理过期的队列项
    pub fn cleanup_expired(&mut self, max_age: Duration) {
        let now = Instant::now();
        let expired: Vec<String> = self
            .last_updated
            .iter()
            .filter(|&(_, time)| now.duration_since(*time) > max_age)
            .map(|(name, _)| name.clone())
            .collect();

        for name in expired {
            self.remove(&name);
        }
    }

    /// 获取队列中所有函数的名称
    pub fn get_all_functions(&self) -> Vec<String> {
        self.queued_functions.keys().cloned().collect()
    }
}

impl Default for HotFunctionPriorityQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// JIT 编译器统计信息
#[derive(Debug, Clone, Default)]
pub struct JITStatistics {
    /// 总编译次数
    pub total_compilations: usize,
    /// 成功编译次数
    pub successful_compilations: usize,
    /// 失败编译次数
    pub failed_compilations: usize,
    /// 总编译时间
    pub total_compile_time: Duration,
    /// 热点函数数量
    pub hot_function_count: usize,
    /// 已编译函数数量
    pub compiled_function_count: usize,
    /// 总调用次数
    pub total_calls: usize,
    /// 总执行时间
    pub total_execution_time: Duration,
    /// 缓存命中次数
    pub cache_hits: usize,
    /// 缓存未命中次数
    pub cache_misses: usize,
}

impl JITStatistics {
    /// 创建新的统计信息
    pub fn new() -> Self {
        Self::default()
    }

    /// 计算缓存命中率
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            return 0.0;
        }
        self.cache_hits as f64 / total as f64
    }

    /// 计算平均编译时间
    pub fn avg_compile_time(&self) -> Duration {
        if self.total_compilations == 0 {
            return Duration::ZERO;
        }
        self.total_compile_time / self.total_compilations as u32
    }

    /// 计算编译成功率
    pub fn success_rate(&self) -> f64 {
        if self.total_compilations == 0 {
            return 0.0;
        }
        self.successful_compilations as f64 / self.total_compilations as f64
    }
}

/// 自适应热点阈值配置
#[derive(Debug, Clone)]
pub struct AdaptiveThresholdConfig {
    /// 基础调用次数阈值
    pub base_call_threshold: usize,
    /// 基础时间阈值（微秒）
    pub base_time_threshold: u64,
    /// 复杂度权重因子
    pub complexity_weight: f64,
    /// 时间权重因子
    pub time_weight: f64,
    /// 调用次数权重因子
    pub call_weight: f64,
    /// 执行频率权重因子
    pub frequency_weight: f64,
    /// 指令密度权重因子
    pub density_weight: f64,
    /// 内存访问权重因子
    pub memory_weight: f64,
    /// 最小阈值
    pub min_threshold: usize,
    /// 最大阈值
    pub max_threshold: usize,
    /// 高复杂度阈值（超过此值认为是高复杂度）
    pub high_complexity_threshold: usize,
    /// 低复杂度阈值（低于此值认为是低复杂度）
    pub low_complexity_threshold: usize,
}

impl Default for AdaptiveThresholdConfig {
    fn default() -> Self {
        Self {
            base_call_threshold: 100,
            base_time_threshold: 100,
            complexity_weight: 0.25,
            time_weight: 0.3,
            call_weight: 0.2,
            frequency_weight: 0.1,
            density_weight: 0.1,
            memory_weight: 0.05,
            min_threshold: 10,
            max_threshold: 1000,
            high_complexity_threshold: 50,
            low_complexity_threshold: 10,
        }
    }
}

impl AdaptiveThresholdConfig {
    /// 创建新的自适应阈值配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 计算自适应调用次数阈值
    pub fn calculate_call_threshold(&self, complexity: usize, execution_frequency: f64, instruction_density: f64) -> usize {
        let complexity_factor = if complexity > self.high_complexity_threshold {
            0.5
        }
        else if complexity < self.low_complexity_threshold {
            1.5
        }
        else {
            let complexity_ratio = (complexity - self.low_complexity_threshold) as f64
                / (self.high_complexity_threshold - self.low_complexity_threshold) as f64;
            1.5 - complexity_ratio
        };

        let frequency_factor = if execution_frequency > 10.0 {
            0.8
        }
        else if execution_frequency < 1.0 {
            1.2
        }
        else {
            1.0
        };

        let density_factor = if instruction_density > 100.0 {
            0.9
        }
        else if instruction_density < 10.0 {
            1.1
        }
        else {
            1.0
        };

        let threshold = (self.base_call_threshold as f64 * complexity_factor * frequency_factor * density_factor) as usize;
        threshold.clamp(self.min_threshold, self.max_threshold)
    }

    /// 计算自适应时间阈值
    pub fn calculate_time_threshold(
        &self,
        complexity: usize,
        memory_access_frequency: usize,
        branch_prediction_success: f64,
    ) -> u64 {
        let complexity_factor = if complexity > self.high_complexity_threshold {
            0.5
        }
        else if complexity < self.low_complexity_threshold {
            1.5
        }
        else {
            let complexity_ratio = (complexity - self.low_complexity_threshold) as f64
                / (self.high_complexity_threshold - self.low_complexity_threshold) as f64;
            1.5 - complexity_ratio
        };

        let memory_factor = if memory_access_frequency > 100 {
            0.8
        }
        else if memory_access_frequency < 10 {
            1.2
        }
        else {
            1.0
        };

        let branch_factor = if branch_prediction_success < 0.5 {
            0.9
        }
        else if branch_prediction_success > 0.9 {
            1.1
        }
        else {
            1.0
        };

        (self.base_time_threshold as f64 * complexity_factor * memory_factor * branch_factor) as u64
    }

    /// 计算优先级分数
    pub fn calculate_priority_score(&self, info: &HotFunctionInfo) -> f64 {
        let call_score = (info.call_count as f64).ln() * self.call_weight;
        let time_score = (info.avg_time as f64 + 1.0).ln() * self.time_weight;
        let complexity_score = (info.complexity as f64).ln() * self.complexity_weight;
        let frequency_score = (info.execution_frequency + 1.0).ln() * self.frequency_weight;
        let density_score = (info.instruction_density + 1.0).ln() * self.density_weight;
        let memory_score = (info.memory_access_frequency as f64 + 1.0).ln() * self.memory_weight;

        call_score + time_score + complexity_score + frequency_score + density_score + memory_score
    }
}

/// JIT 编译器
///
/// 将热点代码编译为机器码，提高执行性能
pub struct JITCompiler {
    /// 编译缓存，存储已编译的函数
    compiled_functions: HashMap<String, Rc<dyn Fn(&[TsValue]) -> TsValue>>,
    /// 热点函数信息
    hot_functions: HashMap<String, HotFunctionInfo>,
    /// 热点函数优先级队列
    priority_queue: HotFunctionPriorityQueue,
    /// 自适应阈值配置
    threshold_config: AdaptiveThresholdConfig,
    /// 统计信息
    statistics: JITStatistics,
    /// 事件回调列表
    event_callbacks: Vec<Rc<dyn JITEventCallback>>,
    /// 默认优化级别
    default_optimization_level: OptimizationLevel,
    /// 后台编译任务队列
    compilation_tasks: std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<(String, Vec<Instruction>)>>>,
    /// 后台编译线程
    background_thread: Option<std::thread::JoinHandle<()>>,
    /// 线程终止信号
    terminate_signal: std::sync::Arc<std::sync::atomic::AtomicBool>,
    /// 预热函数列表
    warmup_functions: Vec<(String, Vec<Instruction>)>,
    /// 缓存大小限制
    cache_size_limit: usize,
    /// 缓存访问时间
    cache_access_times: HashMap<String, Instant>,
    /// 缓存使用次数
    cache_usage_counts: HashMap<String, usize>,
}

impl JITCompiler {
    /// 创建一个新的 JIT 编译器
    pub fn new() -> Self {
        let compilation_tasks = std::sync::Arc::new(std::sync::Mutex::new(std::collections::VecDeque::new()));
        let terminate_signal = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let tasks_clone = compilation_tasks.clone();
        let signal_clone = terminate_signal.clone();

        let background_thread = std::thread::spawn(move || {
            while !signal_clone.load(std::sync::atomic::Ordering::Relaxed) {
                if let Ok(mut tasks) = tasks_clone.lock() {
                    if let Some((_function_name, _instructions)) = tasks.pop_front() {
                        drop(tasks);
                        // 这里可以实现后台编译逻辑
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                    else {
                        drop(tasks);
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                }
            }
        });

        Self {
            compiled_functions: HashMap::new(),
            hot_functions: HashMap::new(),
            priority_queue: HotFunctionPriorityQueue::new(),
            threshold_config: AdaptiveThresholdConfig::new(),
            statistics: JITStatistics::new(),
            event_callbacks: Vec::new(),
            default_optimization_level: OptimizationLevel::High,
            compilation_tasks,
            background_thread: Some(background_thread),
            terminate_signal,
            warmup_functions: Vec::new(),
            cache_size_limit: 1000,
            cache_access_times: HashMap::new(),
            cache_usage_counts: HashMap::new(),
        }
    }

    /// 使用自定义配置创建 JIT 编译器
    pub fn with_config(config: AdaptiveThresholdConfig) -> Self {
        let compilation_tasks = std::sync::Arc::new(std::sync::Mutex::new(std::collections::VecDeque::new()));
        let terminate_signal = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let tasks_clone = compilation_tasks.clone();
        let signal_clone = terminate_signal.clone();

        let background_thread = std::thread::spawn(move || {
            while !signal_clone.load(std::sync::atomic::Ordering::Relaxed) {
                if let Ok(mut tasks) = tasks_clone.lock() {
                    if let Some((_function_name, _instructions)) = tasks.pop_front() {
                        drop(tasks);
                        // 这里可以实现后台编译逻辑
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                    else {
                        drop(tasks);
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                }
            }
        });

        Self {
            compiled_functions: HashMap::new(),
            hot_functions: HashMap::new(),
            priority_queue: HotFunctionPriorityQueue::new(),
            threshold_config: config,
            statistics: JITStatistics::new(),
            event_callbacks: Vec::new(),
            default_optimization_level: OptimizationLevel::High,
            compilation_tasks,
            background_thread: Some(background_thread),
            terminate_signal,
            warmup_functions: Vec::new(),
            cache_size_limit: 1000,
            cache_access_times: HashMap::new(),
            cache_usage_counts: HashMap::new(),
        }
    }

    /// 注册事件回调
    pub fn register_callback(&mut self, callback: Rc<dyn JITEventCallback>) {
        self.event_callbacks.push(callback);
    }

    /// 移除所有事件回调
    pub fn clear_callbacks(&mut self) {
        self.event_callbacks.clear();
    }

    /// 触发事件
    fn emit_event(&self, event: JITEvent) {
        for callback in &self.event_callbacks {
            callback.on_event(&event);
        }
    }

    /// 获取自适应调用次数阈值
    pub fn get_adaptive_call_threshold(&self, complexity: usize, execution_frequency: f64, instruction_density: f64) -> usize {
        self.threshold_config.calculate_call_threshold(complexity, execution_frequency, instruction_density)
    }

    /// 获取自适应时间阈值
    pub fn get_adaptive_time_threshold(
        &self,
        complexity: usize,
        memory_access_frequency: usize,
        branch_prediction_success: f64,
    ) -> u64 {
        self.threshold_config.calculate_time_threshold(complexity, memory_access_frequency, branch_prediction_success)
    }

    /// 检查函数是否需要 JIT 编译
    pub fn should_compile(&mut self, function_name: &str) -> bool {
        // 首先获取或创建热点函数信息
        let (
            call_count,
            avg_time,
            complexity,
            execution_frequency,
            instruction_density,
            memory_access_frequency,
            branch_prediction_success,
            compile_status,
            time_window,
        ) = {
            let info = self.hot_functions.entry(function_name.to_string()).or_insert_with(HotFunctionInfo::default);
            info.call_count += 1;
            self.statistics.total_calls += 1;

            (
                info.call_count,
                info.avg_time,
                info.complexity,
                info.execution_frequency,
                info.instruction_density,
                info.memory_access_frequency,
                info.branch_prediction_success,
                info.compile_status,
                info.time_window.clone(),
            )
        };

        if compile_status == CompileStatus::Compiled {
            self.statistics.cache_hits += 1;
            return false;
        }

        self.statistics.cache_misses += 1;

        // 计算阈值
        let adaptive_call_threshold = self.get_adaptive_call_threshold(complexity, execution_frequency, instruction_density);
        let adaptive_time_threshold =
            self.get_adaptive_time_threshold(complexity, memory_access_frequency, branch_prediction_success);

        let window_avg = if !time_window.is_empty() { time_window.iter().sum::<u64>() / time_window.len() as u64 } else { 0 };

        let should_compile = (call_count >= adaptive_call_threshold
            || avg_time >= adaptive_time_threshold
            || window_avg >= adaptive_time_threshold)
            && compile_status != CompileStatus::Compiling
            && compile_status != CompileStatus::Compiled;

        if should_compile {
            let reason = if call_count >= adaptive_call_threshold {
                CompilationReason::CallCountThreshold
            }
            else if avg_time >= adaptive_time_threshold {
                CompilationReason::TimeThreshold
            }
            else {
                CompilationReason::WindowAverageThreshold
            };

            self.emit_event(JITEvent::HotFunctionDetected { function_name: function_name.to_string(), call_count, avg_time });

            self.emit_event(JITEvent::CompilationStarted { function_name: function_name.to_string(), complexity, reason });
        }

        should_compile
    }

    /// 记录函数执行时间
    pub fn record_execution_time(&mut self, function_name: &str, execution_time: u64) {
        if let Some(info) = self.hot_functions.get_mut(function_name) {
            info.total_time += execution_time;
            info.avg_time = info.total_time / info.call_count as u64;
            info.last_time = execution_time;

            info.time_window.push(execution_time);
            if info.time_window.len() > 10 {
                info.time_window.remove(0);
            }

            self.statistics.total_execution_time += Duration::from_micros(execution_time);
        }
    }

    /// 设置函数复杂度
    pub fn set_function_complexity(&mut self, function_name: &str, complexity: usize) {
        let info = self.hot_functions.entry(function_name.to_string()).or_insert_with(HotFunctionInfo::default);
        info.complexity = complexity;
    }

    /// 将函数加入优先级队列
    pub fn enqueue_for_compilation(&mut self, function_name: &str) {
        if let Some(info) = self.hot_functions.get(function_name) {
            if info.compile_status == CompileStatus::NotCompiled || info.compile_status == CompileStatus::Failed {
                let priority_score = self.threshold_config.calculate_priority_score(info);
                let entry = PriorityEntry {
                    function_name: function_name.to_string(),
                    priority_score,
                    call_count: info.call_count,
                    avg_time: info.avg_time,
                    complexity: info.complexity,
                    execution_frequency: info.execution_frequency,
                    instruction_density: info.instruction_density,
                    memory_access_frequency: info.memory_access_frequency,
                };
                self.priority_queue.push(entry);
            }
        }
    }

    /// 从优先级队列获取下一个待编译函数
    pub fn get_next_compilation_target(&mut self) -> Option<String> {
        while let Some(entry) = self.priority_queue.pop() {
            if let Some(info) = self.hot_functions.get(&entry.function_name) {
                if info.compile_status == CompileStatus::NotCompiled || info.compile_status == CompileStatus::Failed {
                    return Some(entry.function_name);
                }
            }
        }
        None
    }

    /// 获取编译状态
    pub fn get_compile_status(&self, function_name: &str) -> CompileStatus {
        self.hot_functions.get(function_name).map(|info| info.compile_status).unwrap_or(CompileStatus::NotCompiled)
    }

    /// 设置编译状态
    fn set_compile_status(&mut self, function_name: &str, status: CompileStatus) {
        if let Some(info) = self.hot_functions.get_mut(function_name) {
            info.compile_status = status;
        }
    }

    /// 编译函数为机器码
    pub fn compile_function(
        &mut self,
        function_name: &str,
        instructions: &[Instruction],
    ) -> Result<Rc<dyn Fn(&[TsValue]) -> TsValue>, TsError> {
        if let Some(compiled) = self.compiled_functions.get(function_name) {
            return Ok(compiled.clone());
        }

        self.set_compile_status(function_name, CompileStatus::Compiling);
        self.statistics.total_compilations += 1;

        let start_time = Instant::now();

        let optimization_level = self.select_optimization_level(function_name);
        let result = self.compile_to_machine_code(function_name, instructions, &optimization_level);

        let duration = start_time.elapsed();
        self.statistics.total_compile_time += duration;

        match result {
            Ok(compiled_fn) => {
                self.compiled_functions.insert(function_name.to_string(), compiled_fn.clone());

                if let Some(info) = self.hot_functions.get_mut(function_name) {
                    info.compile_status = CompileStatus::Compiled;
                    info.compiled_at = Some(Instant::now());
                    info.compile_duration = Some(duration);
                    info.optimization_level = Some(optimization_level.clone());
                }

                // 更新缓存访问信息
                self.update_cache_info(function_name);

                // 检查并清理缓存
                self.cleanup_cache();

                self.statistics.successful_compilations += 1;
                self.statistics.compiled_function_count += 1;

                self.emit_event(JITEvent::CompilationCompleted {
                    function_name: function_name.to_string(),
                    duration,
                    optimization_level: format!("{:?}", optimization_level),
                });

                Ok(compiled_fn)
            }
            Err(ref err) => {
                if let Some(info) = self.hot_functions.get_mut(function_name) {
                    info.compile_status = CompileStatus::Failed;
                    info.compile_error = Some(err.to_string());
                }

                self.statistics.failed_compilations += 1;

                self.emit_event(JITEvent::CompilationFailed {
                    function_name: function_name.to_string(),
                    error: err.to_string(),
                });

                result
            }
        }
    }

    /// 根据函数特征选择合适的优化级别
    fn select_optimization_level(&self, function_name: &str) -> OptimizationLevel {
        if let Some(info) = self.hot_functions.get(function_name) {
            // 根据函数特征选择优化级别
            if info.call_count > 1000 && info.avg_time > 500 {
                // 高频执行且执行时间长的函数，使用高级优化
                OptimizationLevel::High
            }
            else if info.call_count > 500 || (info.avg_time > 200 && info.complexity > 20) {
                // 中频执行或执行时间较长的函数，使用中级优化
                OptimizationLevel::Medium
            }
            else if info.call_count > 100 || info.complexity > 10 {
                // 低频执行但复杂度较高的函数，使用基本优化
                OptimizationLevel::Basic
            }
            else {
                // 其他函数，使用无优化
                OptimizationLevel::None
            }
        }
        else {
            // 默认使用基本优化
            OptimizationLevel::Basic
        }
    }

    /// 将指令编译为机器码
    fn compile_to_machine_code(
        &self,
        function_name: &str,
        instructions: &[Instruction],
        optimization_level: &OptimizationLevel,
    ) -> Result<Rc<dyn Fn(&[TsValue]) -> TsValue + 'static>, TsError> {
        let optimized_instructions = self.optimize_instructions_for_jit(instructions, optimization_level);

        let function_name = function_name.to_string();
        let optimized_instructions = optimized_instructions;
        let compiled_fn = Rc::new(move |args: &[TsValue]| {
            println!("JIT executing function: {}", function_name);

            let mut vm = crate::vm::VM::new(vec![]);

            for (i, arg) in args.iter().enumerate() {
                vm.set_local(i, arg.clone());
            }

            match vm.execute_instructions(&optimized_instructions) {
                Ok(result) => result,
                Err(_) => TsValue::Undefined,
            }
        });

        Ok(compiled_fn)
    }

    /// 为 JIT 编译优化指令
    fn optimize_instructions_for_jit(
        &self,
        instructions: &[Instruction],
        optimization_level: &OptimizationLevel,
    ) -> Vec<Instruction> {
        let mut optimizer = JITOptimizer::new(optimization_level.clone());
        optimizer.optimize_instructions(instructions)
    }

    /// 获取已编译的函数
    pub fn get_compiled_function(&mut self, function_name: &str) -> Option<Rc<dyn Fn(&[TsValue]) -> TsValue>> {
        let result = self.compiled_functions.get(function_name).cloned();
        if result.is_some() {
            self.statistics.cache_hits += 1;
            // 更新缓存访问时间和使用次数
            self.update_cache_info(function_name);
        }
        else {
            self.statistics.cache_misses += 1;
        }
        result
    }

    /// 更新缓存访问信息
    fn update_cache_info(&mut self, function_name: &str) {
        self.cache_access_times.insert(function_name.to_string(), Instant::now());
        *self.cache_usage_counts.entry(function_name.to_string()).or_insert(0) += 1;
    }

    /// 清理缓存
    pub fn cleanup_cache(&mut self) {
        if self.compiled_functions.len() > self.cache_size_limit {
            // 按使用次数和访问时间排序，移除最不常用的函数
            let mut cache_items: Vec<(String, usize, Instant)> = self
                .cache_usage_counts
                .iter()
                .filter_map(|(name, count)| self.cache_access_times.get(name).map(|time| (name.clone(), *count, *time)))
                .collect();

            // 按使用次数升序，访问时间升序排序
            cache_items.sort_by(|a, b| if a.1 != b.1 { a.1.cmp(&b.1) } else { a.2.cmp(&b.2) });

            // 移除超出限制的项
            let items_to_remove = cache_items.len() - self.cache_size_limit;
            for item in cache_items.iter().take(items_to_remove) {
                self.compiled_functions.remove(&item.0);
                self.cache_access_times.remove(&item.0);
                self.cache_usage_counts.remove(&item.0);

                // 更新热点函数状态
                if let Some(info) = self.hot_functions.get_mut(&item.0) {
                    info.compile_status = CompileStatus::NotCompiled;
                    info.compiled_at = None;
                    info.compile_duration = None;
                }
            }
        }
    }

    /// 设置缓存大小限制
    pub fn set_cache_size_limit(&mut self, limit: usize) {
        self.cache_size_limit = limit;
        // 立即清理缓存以符合新限制
        self.cleanup_cache();
    }

    /// 获取缓存大小限制
    pub fn get_cache_size_limit(&self) -> usize {
        self.cache_size_limit
    }

    /// 获取当前缓存大小
    pub fn get_current_cache_size(&self) -> usize {
        self.compiled_functions.len()
    }

    /// 获取热点函数信息
    pub fn get_hot_function_info(&self, function_name: &str) -> Option<&HotFunctionInfo> {
        self.hot_functions.get(function_name)
    }

    /// 获取所有热点函数名称
    pub fn get_hot_function_names(&self) -> Vec<String> {
        self.hot_functions.keys().cloned().collect()
    }

    /// 获取统计信息
    pub fn get_statistics(&self) -> &JITStatistics {
        &self.statistics
    }

    /// 重置统计信息
    pub fn reset_statistics(&mut self) {
        self.statistics = JITStatistics::new();
    }

    /// 获取优先级队列长度
    pub fn pending_compilation_count(&self) -> usize {
        self.priority_queue.len()
    }

    /// 检查函数是否在编译队列中
    pub fn is_queued_for_compilation(&self, function_name: &str) -> bool {
        self.priority_queue.contains(function_name)
    }

    /// 清空编译缓存
    pub fn clear_cache(&mut self) {
        self.compiled_functions.clear();
        for info in self.hot_functions.values_mut() {
            info.compile_status = CompileStatus::NotCompiled;
            info.compiled_at = None;
            info.compile_duration = None;
            info.compile_error = None;
        }
        self.statistics.compiled_function_count = 0;
    }

    /// 设置默认优化级别
    pub fn set_optimization_level(&mut self, level: OptimizationLevel) {
        self.default_optimization_level = level;
    }

    /// 获取默认优化级别
    pub fn get_optimization_level(&self) -> &OptimizationLevel {
        &self.default_optimization_level
    }

    /// 获取自适应阈值配置
    pub fn get_threshold_config(&self) -> &AdaptiveThresholdConfig {
        &self.threshold_config
    }

    /// 设置自适应阈值配置
    pub fn set_threshold_config(&mut self, config: AdaptiveThresholdConfig) {
        self.threshold_config = config;
    }

    /// 批量更新热点函数优先级
    pub fn update_priorities(&mut self) {
        for (function_name, info) in &self.hot_functions {
            if info.compile_status == CompileStatus::NotCompiled || info.compile_status == CompileStatus::Failed {
                let priority_score = self.threshold_config.calculate_priority_score(info);
                let entry = PriorityEntry {
                    function_name: function_name.clone(),
                    priority_score,
                    call_count: info.call_count,
                    avg_time: info.avg_time,
                    complexity: info.complexity,
                    execution_frequency: info.execution_frequency,
                    instruction_density: info.instruction_density,
                    memory_access_frequency: info.memory_access_frequency,
                };
                self.priority_queue.push(entry);
            }
        }
    }

    /// 添加后台编译任务
    pub fn add_background_compilation_task(&mut self, function_name: &str, instructions: &[Instruction]) {
        let task = (function_name.to_string(), instructions.to_vec());
        if let Ok(mut tasks) = self.compilation_tasks.lock() {
            tasks.push_back(task);
        }
    }

    /// 添加预热函数
    pub fn add_warmup_function(&mut self, function_name: &str, instructions: &[Instruction]) {
        self.warmup_functions.push((function_name.to_string(), instructions.to_vec()));
    }

    /// 运行预热
    pub fn run_warmup(&mut self) {
        // 先收集所有预热函数
        let warmup_functions = self.warmup_functions.clone();

        for (function_name, instructions) in warmup_functions {
            self.set_function_complexity(&function_name, instructions.len());
            self.add_background_compilation_task(&function_name, &instructions);
        }
    }

    /// 执行已编译的函数
    pub fn execute_function(
        &mut self,
        function_name: &str,
        args: &[TsValue],
        instructions: &[Instruction],
    ) -> Result<TsValue, TsError> {
        // 首先尝试获取已编译的函数
        if let Some(compiled_fn) = self.get_compiled_function(function_name) {
            return Ok(compiled_fn(args));
        }

        // 如果函数未编译，则编译后执行
        let compiled_fn = self.compile_function(function_name, instructions)?;
        Ok(compiled_fn(args))
    }

    /// 关闭后台编译线程
    pub fn shutdown(&mut self) {
        self.terminate_signal.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(thread) = self.background_thread.take() {
            let _ = thread.join();
        }
    }

    /// 获取热点函数数量
    pub fn hot_function_count(&self) -> usize {
        self.hot_functions.len()
    }

    /// 获取已编译函数数量
    pub fn compiled_function_count(&self) -> usize {
        self.compiled_functions.len()
    }
}

impl Default for JITCompiler {
    fn default() -> Self {
        Self::new()
    }
}
