//! 并行编译模块
//!
//! 提供并行编译功能，管理编译任务的并行执行，提高编译效率。

use crate::compiler::{CompilationResult, CompilationTask, CompilationUnit};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, RwLock},
    thread,
};
use typescript_types::{TsError, TsValue};

// 导入 num_cpus 库用于获取系统 CPU 核心数
use num_cpus;

/// 任务队列
#[derive(Debug, Clone)]
pub struct TaskQueue {
    /// 任务队列
    queue: Arc<Mutex<VecDeque<CompilationTask>>>,
    /// 完成的任务数
    completed_tasks: Arc<Mutex<usize>>,
    /// 总任务数
    total_tasks: Arc<Mutex<usize>>,
}

impl TaskQueue {
    /// 创建新的任务队列
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            completed_tasks: Arc::new(Mutex::new(0)),
            total_tasks: Arc::new(Mutex::new(0)),
        }
    }

    /// 添加任务
    pub fn add_task(&self, task: CompilationTask) {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(task);
        let mut total_tasks = self.total_tasks.lock().unwrap();
        *total_tasks += 1;
    }

    /// 获取任务
    pub fn get_task(&self) -> Option<CompilationTask> {
        let mut queue = self.queue.lock().unwrap();
        queue.pop_front()
    }

    /// 标记任务完成
    pub fn mark_task_completed(&self) {
        let mut completed_tasks = self.completed_tasks.lock().unwrap();
        *completed_tasks += 1;
    }

    /// 获取完成的任务数
    pub fn get_completed_tasks(&self) -> usize {
        *self.completed_tasks.lock().unwrap()
    }

    /// 获取总任务数
    pub fn get_total_tasks(&self) -> usize {
        *self.total_tasks.lock().unwrap()
    }

    /// 检查是否所有任务都已完成
    pub fn is_all_tasks_completed(&self) -> bool {
        *self.completed_tasks.lock().unwrap() == *self.total_tasks.lock().unwrap()
    }

    /// 检查队列是否为空
    pub fn is_empty(&self) -> bool {
        self.queue.lock().unwrap().is_empty()
    }
}

/// 编译结果收集器
#[derive(Debug, Clone)]
pub struct ResultCollector {
    /// 编译结果
    results: Arc<RwLock<Vec<(String, CompilationResult<TsValue>)>>>,
}

impl ResultCollector {
    /// 创建新的结果收集器
    pub fn new() -> Self {
        Self { results: Arc::new(RwLock::new(Vec::new())) }
    }

    /// 添加结果
    pub fn add_result(&self, task_id: String, result: CompilationResult<TsValue>) {
        let mut results = self.results.write().unwrap();
        results.push((task_id, result));
    }

    /// 获取所有结果
    pub fn get_results(&self) -> Vec<(String, CompilationResult<TsValue>)> {
        self.results.read().unwrap().clone()
    }

    /// 获取结果数量
    pub fn get_result_count(&self) -> usize {
        self.results.read().unwrap().len()
    }
}

/// 并行编译器
#[derive(Clone)]
pub struct ParallelCompiler {
    /// 线程数
    thread_count: usize,
    /// 任务队列
    task_queue: TaskQueue,
    /// 结果收集器
    result_collector: ResultCollector,
    /// 编译函数
    compile_fn: Arc<dyn Fn(&CompilationUnit) -> CompilationResult<TsValue> + Send + Sync>,
}

impl std::fmt::Debug for ParallelCompiler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParallelCompiler")
            .field("thread_count", &self.thread_count)
            .field("task_queue", &self.task_queue)
            .field("result_collector", &self.result_collector)
            .field("compile_fn", &"Fn(&CompilationUnit) -> CompilationResult<TsValue>")
            .finish()
    }
}

impl ParallelCompiler {
    /// 创建新的并行编译器
    pub fn new(
        thread_count: usize,
        compile_fn: Arc<dyn Fn(&CompilationUnit) -> CompilationResult<TsValue> + Send + Sync>,
    ) -> Self {
        Self { thread_count, task_queue: TaskQueue::new(), result_collector: ResultCollector::new(), compile_fn }
    }

    /// 添加编译任务
    pub fn add_task(&self, task: CompilationTask) {
        self.task_queue.add_task(task);
    }

    /// 执行所有任务
    pub fn execute(&self) -> Vec<(String, CompilationResult<TsValue>)> {
        let mut handles = Vec::new();

        // 创建工作线程
        for _ in 0..self.thread_count {
            let task_queue = self.task_queue.clone();
            let result_collector = self.result_collector.clone();
            let compile_fn = self.compile_fn.clone();

            let handle = thread::spawn(move || {
                // 持续从队列中获取任务并执行
                while !task_queue.is_all_tasks_completed() {
                    if let Some(task) = task_queue.get_task() {
                        // 执行编译任务
                        let result = (compile_fn)(&task.unit);
                        // 收集结果
                        result_collector.add_result(task.id, result);
                        // 标记任务完成
                        task_queue.mark_task_completed();
                    }
                }
            });

            handles.push(handle);
        }

        // 等待所有线程完成
        for handle in handles {
            handle.join().unwrap();
        }

        // 返回所有结果
        self.result_collector.get_results()
    }

    /// 获取任务队列
    pub fn get_task_queue(&self) -> &TaskQueue {
        &self.task_queue
    }

    /// 获取结果收集器
    pub fn get_result_collector(&self) -> &ResultCollector {
        &self.result_collector
    }
}

impl Default for ParallelCompiler {
    fn default() -> Self {
        Self::new(
            num_cpus::get(),
            Arc::new(|_unit| CompilationResult::Error(TsError::Other("Default compile function".to_string()))),
        )
    }
}
