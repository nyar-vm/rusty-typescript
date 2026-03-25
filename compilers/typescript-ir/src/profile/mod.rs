//! 性能分析模块
//! 
//! 负责收集和分析运行时性能数据，为基于 profile 的优化提供数据支持。

use std::collections::HashMap;
use std::time::Instant;

/// 函数调用信息
#[derive(Debug, Clone)]
pub struct FunctionCallInfo {
    /// 调用次数
    pub call_count: u64,
    /// 总执行时间（纳秒）
    pub total_time: u128,
    /// 平均执行时间（纳秒）
    pub avg_time: f64,
    /// 最长执行时间（纳秒）
    pub max_time: u128,
    /// 最短执行时间（纳秒）
    pub min_time: u128,
}

impl FunctionCallInfo {
    /// 创建新的函数调用信息
    pub fn new() -> Self {
        Self {
            call_count: 0,
            total_time: 0,
            avg_time: 0.0,
            max_time: 0,
            min_time: u128::MAX,
        }
    }

    /// 记录一次函数调用
    pub fn record_call(&mut self, duration: u128) {
        self.call_count += 1;
        self.total_time += duration;
        self.avg_time = self.total_time as f64 / self.call_count as f64;
        self.max_time = self.max_time.max(duration);
        self.min_time = self.min_time.min(duration);
    }
}

/// 代码块执行信息
#[derive(Debug, Clone)]
pub struct BlockExecutionInfo {
    /// 执行次数
    pub exec_count: u64,
    /// 总执行时间（纳秒）
    pub total_time: u128,
    /// 平均执行时间（纳秒）
    pub avg_time: f64,
}

impl BlockExecutionInfo {
    /// 创建新的代码块执行信息
    pub fn new() -> Self {
        Self {
            exec_count: 0,
            total_time: 0,
            avg_time: 0.0,
        }
    }

    /// 记录一次代码块执行
    pub fn record_execution(&mut self, duration: u128) {
        self.exec_count += 1;
        self.total_time += duration;
        self.avg_time = self.total_time as f64 / self.exec_count as f64;
    }
}

/// 性能分析器
#[derive(Debug, Clone)]
pub struct Profiler {
    /// 函数调用信息映射
    pub function_calls: HashMap<String, FunctionCallInfo>,
    /// 代码块执行信息映射（使用块ID作为键）
    pub block_executions: HashMap<usize, BlockExecutionInfo>,
    /// 当前正在执行的函数栈
    pub call_stack: Vec<String>,
    /// 函数开始执行的时间
    pub start_times: Vec<Instant>,
    /// 是否启用分析
    pub enabled: bool,
}

impl Profiler {
    /// 创建新的性能分析器
    pub fn new() -> Self {
        Self {
            function_calls: HashMap::new(),
            block_executions: HashMap::new(),
            call_stack: Vec::new(),
            start_times: Vec::new(),
            enabled: false,
        }
    }

    /// 启用分析
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// 禁用分析
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// 开始函数调用
    pub fn begin_function(&mut self, function_name: &str) {
        if !self.enabled {
            return;
        }

        self.call_stack.push(function_name.to_string());
        self.start_times.push(Instant::now());
    }

    /// 结束函数调用
    pub fn end_function(&mut self) {
        if !self.enabled || self.call_stack.is_empty() {
            return;
        }

        let function_name = self.call_stack.pop().unwrap();
        let start_time = self.start_times.pop().unwrap();
        let duration = start_time.elapsed().as_nanos();

        let function_info = self.function_calls.entry(function_name).or_insert(FunctionCallInfo::new());
        function_info.record_call(duration);
    }

    /// 记录代码块执行
    pub fn record_block_execution(&mut self, block_id: usize, duration: u128) {
        if !self.enabled {
            return;
        }

        let block_info = self.block_executions.entry(block_id).or_insert(BlockExecutionInfo::new());
        block_info.record_execution(duration);
    }

    /// 生成分析报告
    pub fn generate_report(&self) -> String {
        let mut report = String::from("=== Performance Profile Report ===\n\n");

        // 函数调用统计
        report.push_str("Function Call Statistics:\n");
        report.push_str("-