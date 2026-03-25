//! 性能监控模块
//!
//! 提供性能监控和分析工具，用于识别性能瓶颈。

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

/// 性能监控器
///
/// 收集和分析程序执行性能指标
#[derive(Debug, Clone)]
pub struct PerformanceMonitor {
    /// 执行开始时间
    start_time: Option<Instant>,
    /// 指令执行计数
    instruction_count: usize,
    /// 函数调用计数
    function_call_count: usize,
    /// 内存分配计数
    allocation_count: usize,
    /// 垃圾回收次数
    gc_count: usize,
    /// 热点代码统计
    hot_spots: HashMap<String, HotSpot>,
    /// 函数执行时间统计
    function_times: HashMap<String, FunctionStats>,
    /// 内存使用历史
    memory_history: Vec<MemorySnapshot>,
    /// 最大栈深度
    max_stack_depth: usize,
    /// 当前栈深度
    current_stack_depth: usize,
}

/// 热点代码统计
#[derive(Debug, Clone, Default)]
pub struct HotSpot {
    /// 代码位置标识
    pub location: String,
    /// 执行次数
    pub execution_count: usize,
    /// 总执行时间
    pub total_time: Duration,
    /// 平均执行时间
    pub average_time: Duration,
}

/// 函数执行统计
#[derive(Debug, Clone, Default)]
pub struct FunctionStats {
    /// 函数名
    pub name: String,
    /// 调用次数
    pub call_count: usize,
    /// 总执行时间
    pub total_time: Duration,
    /// 最小执行时间
    pub min_time: Option<Duration>,
    /// 最大执行时间
    pub max_time: Option<Duration>,
    /// 平均执行时间
    pub average_time: Duration,
}

/// 内存快照
#[derive(Debug, Clone, Copy)]
pub struct MemorySnapshot {
    /// 时间戳
    pub timestamp: u64,
    /// 已分配内存（字节）
    pub allocated_bytes: usize,
    /// 已使用内存（字节）
    pub used_bytes: usize,
    /// 对象数量
    pub object_count: usize,
}

/// 性能报告
#[derive(Debug, Clone)]
pub struct PerformanceReport {
    /// 总执行时间
    pub total_execution_time: Duration,
    /// 指令执行计数
    pub instruction_count: usize,
    /// 函数调用计数
    pub function_call_count: usize,
    /// 内存分配计数
    pub allocation_count: usize,
    /// 垃圾回收次数
    pub gc_count: usize,
    /// 最大栈深度
    pub max_stack_depth: usize,
    /// 每秒指令数
    pub instructions_per_second: f64,
    /// 热点代码列表
    pub hot_spots: Vec<HotSpot>,
    /// 最耗时的函数
    pub slowest_functions: Vec<FunctionStats>,
    /// 内存使用峰值
    pub peak_memory_usage: usize,
}

impl PerformanceMonitor {
    /// 创建新的性能监控器
    pub fn new() -> Self {
        Self {
            start_time: None,
            instruction_count: 0,
            function_call_count: 0,
            allocation_count: 0,
            gc_count: 0,
            hot_spots: HashMap::new(),
            function_times: HashMap::new(),
            memory_history: Vec::new(),
            max_stack_depth: 0,
            current_stack_depth: 0,
        }
    }

    /// 开始监控
    pub fn start(&mut self) {
        self.start_time = Some(Instant::now());
        self.instruction_count = 0;
        self.function_call_count = 0;
        self.allocation_count = 0;
        self.gc_count = 0;
        self.hot_spots.clear();
        self.function_times.clear();
        self.memory_history.clear();
        self.max_stack_depth = 0;
        self.current_stack_depth = 0;
    }

    /// 停止监控
    pub fn stop(&mut self) {
        self.start_time = None;
    }

    /// 记录指令执行
    pub fn record_instruction(&mut self) {
        self.instruction_count += 1;
    }

    /// 记录函数调用开始
    pub fn record_function_start(&mut self, name: &str) {
        self.function_call_count += 1;
        self.current_stack_depth += 1;
        if self.current_stack_depth > self.max_stack_depth {
            self.max_stack_depth = self.current_stack_depth;
        }

        let entry = self
            .function_times
            .entry(name.to_string())
            .or_insert_with(|| FunctionStats { name: name.to_string(), ..Default::default() });
        entry.call_count += 1;
    }

    /// 记录函数调用结束
    pub fn record_function_end(&mut self, name: &str, duration: Duration) {
        self.current_stack_depth -= 1;

        if let Some(stats) = self.function_times.get_mut(name) {
            stats.total_time += duration;
            stats.average_time = stats.total_time / stats.call_count as u32;

            if let Some(min) = stats.min_time {
                if duration < min {
                    stats.min_time = Some(duration);
                }
            }
            else {
                stats.min_time = Some(duration);
            }

            if let Some(max) = stats.max_time {
                if duration > max {
                    stats.max_time = Some(duration);
                }
            }
            else {
                stats.max_time = Some(duration);
            }
        }
    }

    /// 记录内存分配
    pub fn record_allocation(&mut self, bytes: usize) {
        self.allocation_count += 1;
        self.record_memory_snapshot(bytes);
    }

    /// 记录垃圾回收
    pub fn record_gc(&mut self) {
        self.gc_count += 1;
    }

    /// 记录热点代码
    pub fn record_hot_spot(&mut self, location: &str, duration: Duration) {
        let hot_spot = self
            .hot_spots
            .entry(location.to_string())
            .or_insert_with(|| HotSpot { location: location.to_string(), ..Default::default() });

        hot_spot.execution_count += 1;
        hot_spot.total_time += duration;
        hot_spot.average_time = hot_spot.total_time / hot_spot.execution_count as u32;
    }

    /// 记录内存快照
    fn record_memory_snapshot(&mut self, used_bytes: usize) {
        let timestamp = self.elapsed_micros();
        self.memory_history.push(MemorySnapshot { timestamp, allocated_bytes: used_bytes, used_bytes, object_count: 0 });
    }

    /// 获取已执行的微秒数
    fn elapsed_micros(&self) -> u64 {
        self.start_time.map(|t| t.elapsed().as_micros() as u64).unwrap_or(0)
    }

    /// 获取总执行时间
    pub fn elapsed(&self) -> Duration {
        self.start_time.map(|t| t.elapsed()).unwrap_or(Duration::ZERO)
    }

    /// 生成性能报告
    pub fn generate_report(&self) -> PerformanceReport {
        let total_time = self.elapsed();
        let total_secs = total_time.as_secs_f64();

        // 计算每秒指令数
        let instructions_per_second = if total_secs > 0.0 { self.instruction_count as f64 / total_secs } else { 0.0 };

        // 获取热点代码（按执行次数排序）
        let mut hot_spots: Vec<HotSpot> = self.hot_spots.values().cloned().collect();
        hot_spots.sort_by(|a, b| b.execution_count.cmp(&a.execution_count));
        hot_spots.truncate(10);

        // 获取最耗时的函数
        let mut slowest_functions: Vec<FunctionStats> = self.function_times.values().cloned().collect();
        slowest_functions.sort_by(|a, b| b.total_time.cmp(&a.total_time));
        slowest_functions.truncate(10);

        // 计算内存使用峰值
        let peak_memory = self.memory_history.iter().map(|s| s.used_bytes).max().unwrap_or(0);

        PerformanceReport {
            total_execution_time: total_time,
            instruction_count: self.instruction_count,
            function_call_count: self.function_call_count,
            allocation_count: self.allocation_count,
            gc_count: self.gc_count,
            max_stack_depth: self.max_stack_depth,
            instructions_per_second,
            hot_spots,
            slowest_functions,
            peak_memory_usage: peak_memory,
        }
    }

    /// 获取当前指标
    pub fn current_metrics(&self) -> CurrentMetrics {
        CurrentMetrics {
            instruction_count: self.instruction_count,
            function_call_count: self.function_call_count,
            allocation_count: self.allocation_count,
            gc_count: self.gc_count,
            current_stack_depth: self.current_stack_depth,
            elapsed_micros: self.elapsed_micros(),
        }
    }

    /// 检查是否超过阈值
    pub fn is_threshold_exceeded(&self, threshold: &PerformanceThreshold) -> bool {
        if self.instruction_count > threshold.max_instructions {
            return true;
        }
        if self.function_call_count > threshold.max_function_calls {
            return true;
        }
        if self.allocation_count > threshold.max_allocations {
            return true;
        }
        if self.gc_count > threshold.max_gc_count {
            return true;
        }
        if self.current_stack_depth > threshold.max_stack_depth {
            return true;
        }
        if self.elapsed() > threshold.max_execution_time {
            return true;
        }
        false
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// 当前性能指标
#[derive(Debug, Clone, Copy)]
pub struct CurrentMetrics {
    /// 指令执行计数
    pub instruction_count: usize,
    /// 函数调用计数
    pub function_call_count: usize,
    /// 内存分配计数
    pub allocation_count: usize,
    /// 垃圾回收次数
    pub gc_count: usize,
    /// 当前栈深度
    pub current_stack_depth: usize,
    /// 已执行微秒数
    pub elapsed_micros: u64,
}

/// 性能阈值
#[derive(Debug, Clone)]
pub struct PerformanceThreshold {
    /// 最大指令数
    pub max_instructions: usize,
    /// 最大函数调用次数
    pub max_function_calls: usize,
    /// 最大内存分配次数
    pub max_allocations: usize,
    /// 最大垃圾回收次数
    pub max_gc_count: usize,
    /// 最大栈深度
    pub max_stack_depth: usize,
    /// 最大执行时间
    pub max_execution_time: Duration,
}

impl PerformanceThreshold {
    /// 创建宽松的阈值
    pub fn permissive() -> Self {
        Self {
            max_instructions: usize::MAX,
            max_function_calls: usize::MAX,
            max_allocations: usize::MAX,
            max_gc_count: usize::MAX,
            max_stack_depth: 10000,
            max_execution_time: Duration::from_secs(60),
        }
    }

    /// 创建严格的阈值
    pub fn strict() -> Self {
        Self {
            max_instructions: 1_000_000,
            max_function_calls: 100_000,
            max_allocations: 100_000,
            max_gc_count: 1000,
            max_stack_depth: 1000,
            max_execution_time: Duration::from_secs(5),
        }
    }

    /// 创建自定义阈值
    pub fn custom(
        max_instructions: usize,
        max_function_calls: usize,
        max_allocations: usize,
        max_gc_count: usize,
        max_stack_depth: usize,
        max_execution_time_secs: u64,
    ) -> Self {
        Self {
            max_instructions,
            max_function_calls,
            max_allocations,
            max_gc_count,
            max_stack_depth,
            max_execution_time: Duration::from_secs(max_execution_time_secs),
        }
    }
}

impl Default for PerformanceThreshold {
    fn default() -> Self {
        Self::permissive()
    }
}

impl PerformanceReport {
    /// 格式化报告为字符串
    pub fn format(&self) -> String {
        let mut output = String::new();

        output.push_str("=== Performance Report ===\n\n");
        output.push_str(&format!("Total Execution Time: {:?}\n", self.total_execution_time));
        output.push_str(&format!("Instructions Executed: {}\n", self.instruction_count));
        output.push_str(&format!("Instructions/Second: {:.2}\n", self.instructions_per_second));
        output.push_str(&format!("Function Calls: {}\n", self.function_call_count));
        output.push_str(&format!("Memory Allocations: {}\n", self.allocation_count));
        output.push_str(&format!("Garbage Collections: {}\n", self.gc_count));
        output.push_str(&format!("Max Stack Depth: {}\n", self.max_stack_depth));
        output.push_str(&format!("Peak Memory Usage: {} bytes\n", self.peak_memory_usage));

        if !self.hot_spots.is_empty() {
            output.push_str("\n--- Hot Spots (Top 10) ---\n");
            for (i, spot) in self.hot_spots.iter().enumerate() {
                output.push_str(&format!(
                    "{}. {}: {} calls, avg {:?}\n",
                    i + 1,
                    spot.location,
                    spot.execution_count,
                    spot.average_time
                ));
            }
        }

        if !self.slowest_functions.is_empty() {
            output.push_str("\n--- Slowest Functions (Top 10) ---\n");
            for (i, func) in self.slowest_functions.iter().enumerate() {
                output.push_str(&format!(
                    "{}. {}: {} calls, total {:?}, avg {:?}\n",
                    i + 1,
                    func.name,
                    func.call_count,
                    func.total_time,
                    func.average_time
                ));
            }
        }

        output
    }
}

/// 性能分析器
///
/// 用于分析代码性能的辅助工具
pub struct Profiler {
    /// 监控器
    monitor: PerformanceMonitor,
    /// 当前函数调用栈
    call_stack: Vec<(String, Instant)>,
    /// 是否启用
    enabled: bool,
}

impl Profiler {
    /// 创建新的分析器
    pub fn new() -> Self {
        Self { monitor: PerformanceMonitor::new(), call_stack: Vec::new(), enabled: true }
    }

    /// 启用分析
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// 禁用分析
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// 开始分析会话
    pub fn start(&mut self) {
        if self.enabled {
            self.monitor.start();
        }
    }

    /// 进入函数
    pub fn enter_function(&mut self, name: &str) {
        if self.enabled {
            self.monitor.record_function_start(name);
            self.call_stack.push((name.to_string(), Instant::now()));
        }
    }

    /// 退出函数
    pub fn exit_function(&mut self) {
        if self.enabled {
            if let Some((name, start_time)) = self.call_stack.pop() {
                let duration = start_time.elapsed();
                self.monitor.record_function_end(&name, duration);
            }
        }
    }

    /// 记录指令
    pub fn record_instruction(&mut self) {
        if self.enabled {
            self.monitor.record_instruction();
        }
    }

    /// 生成报告
    pub fn report(&self) -> PerformanceReport {
        self.monitor.generate_report()
    }

    /// 获取监控器引用
    pub fn monitor(&self) -> &PerformanceMonitor {
        &self.monitor
    }
}

impl Default for Profiler {
    fn default() -> Self {
        Self::new()
    }
}
