//! perf 模块
//!
pub struct PerformanceMonitor {
    /// 指令执行计数
    pub instruction_count: usize,
    /// 函数调用计数
    pub call_count: usize,
    /// 栈最大深度
    pub max_stack_depth: usize,
    /// 执行开始时间
    pub start_time: Option<std::time::Instant>,
    /// 内存分配次数
    pub memory_allocations: usize,
    /// 内存释放次数
    pub memory_deallocations: usize,
    /// 垃圾收集次数
    pub gc_count: usize,
    /// 垃圾收集总时间（微秒）
    pub gc_total_time_us: u64,
    /// 最大内存使用量（字节）
    pub max_memory_used: usize,
    /// 是否启用监控
    pub enabled: bool,
}

impl PerformanceMonitor {
    /// 创建新的性能监控器
    pub fn new() -> Self {
        Self {
            instruction_count: 0,
            call_count: 0,
            max_stack_depth: 0,
            start_time: None,
            memory_allocations: 0,
            memory_deallocations: 0,
            gc_count: 0,
            gc_total_time_us: 0,
            max_memory_used: 0,
            enabled: true,
        }
    }

    /// 开始执行
    pub fn start(&mut self) {
        self.start_time = Some(std::time::Instant::now());
        self.instruction_count = 0;
        self.call_count = 0;
        self.max_stack_depth = 0;
        self.memory_allocations = 0;
        self.memory_deallocations = 0;
        self.gc_count = 0;
        self.gc_total_time_us = 0;
        self.max_memory_used = 0;
    }

    /// 记录指令执行
    pub fn record_instruction(&mut self) {
        if !self.enabled {
            return;
        }
        self.instruction_count += 1;
    }

    /// 记录函数调用
    pub fn record_call(&mut self, stack_depth: usize) {
        if !self.enabled {
            return;
        }
        self.call_count += 1;
        if stack_depth > self.max_stack_depth {
            self.max_stack_depth = stack_depth;
        }
    }

    /// 记录内存分配
    pub fn record_memory_allocation(&mut self, size: usize) {
        if !self.enabled {
            return;
        }
        self.memory_allocations += 1;
        // 这里简化处理，实际应该跟踪当前内存使用量
    }

    /// 记录内存释放
    pub fn record_memory_deallocation(&mut self, size: usize) {
        if !self.enabled {
            return;
        }
        self.memory_deallocations += 1;
    }

    /// 记录垃圾收集
    pub fn record_gc(&mut self, duration_us: u64) {
        if !self.enabled {
            return;
        }
        self.gc_count += 1;
        self.gc_total_time_us += duration_us;
    }

    /// 更新最大内存使用量
    pub fn update_max_memory_used(&mut self, current_used: usize) {
        if !self.enabled {
            return;
        }
        if current_used > self.max_memory_used {
            self.max_memory_used = current_used;
        }
    }

    /// 启用监控
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// 禁用监控
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// 检查是否启用监控
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// 获取执行时间（微秒）
    pub fn elapsed_us(&self) -> u64 {
        self.start_time.map(|t| t.elapsed().as_micros() as u64).unwrap_or(0)
    }

    /// 获取报告
    pub fn report(&self) -> String {
        let elapsed = self.elapsed_us();
        let instructions_per_us = if elapsed > 0 {
            self.instruction_count as f64 / elapsed as f64
        } else {
            0.0
        };
        
        format!(
            "Performance Report:\n\
             - Instructions executed: {}\n\
             - Function calls: {}\n\
             - Max stack depth: {}\n\
             - Execution time: {} us\
             - Instructions per us: {:.2}\n\
             - Memory allocations: {}\n\
             - Memory deallocations: {}\n\
             - Garbage collections: {}\n\
             - GC total time: {} us\
             - Max memory used: {} bytes",
            self.instruction_count,
            self.call_count,
            self.max_stack_depth,
            elapsed,
            instructions_per_us,
            self.memory_allocations,
            self.memory_deallocations,
            self.gc_count,
            self.gc_total_time_us,
            self.max_memory_used
        )
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}
