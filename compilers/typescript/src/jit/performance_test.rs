//! JIT 编译器性能测试
//!
//! 包含不同场景的性能测试，验证优化效果

use super::*;
use crate::codegen::{BinaryOp, Instruction};
use std::time::Instant;
use typescript_types::TsValue;

/// 性能测试结果
#[derive(Debug)]
pub struct PerformanceTestResult {
    /// 测试名称
    pub test_name: String,
    /// 执行次数
    pub iterations: usize,
    /// 总执行时间（微秒）
    pub total_time: u64,
    /// 平均执行时间（微秒）
    pub avg_time: f64,
    /// 编译次数
    pub compilation_count: usize,
    /// 缓存命中率
    pub cache_hit_rate: f64,
}

/// 性能测试套件
pub struct PerformanceTestSuite {
    /// JIT 编译器
    jit_compiler: JITCompiler,
    /// 测试结果
    results: Vec<PerformanceTestResult>,
}

impl PerformanceTestSuite {
    /// 创建新的性能测试套件
    pub fn new() -> Self {
        Self { jit_compiler: JITCompiler::new(), results: Vec::new() }
    }

    /// 运行所有测试
    pub fn run_all_tests(&mut self) {
        self.test_fibonacci();
        self.test_loop();
        self.test_function_call();
        self.test_memory_access();
        self.test_branch_prediction();
    }

    /// 测试斐波那契数列计算
    pub fn test_fibonacci(&mut self) {
        // 简单的斐波那契函数指令
        let instructions = vec![
            Instruction::LoadLocal(0),
            Instruction::PushNumber(0.0),
            Instruction::BinaryOp(BinaryOp::Lte),
            Instruction::JumpIfFalse(4),
            Instruction::PushNumber(0.0),
            Instruction::Return,
            Instruction::LoadLocal(0),
            Instruction::PushNumber(1.0),
            Instruction::BinaryOp(BinaryOp::Lte),
            Instruction::JumpIfFalse(4),
            Instruction::PushNumber(1.0),
            Instruction::Return,
            Instruction::LoadLocal(0),
            Instruction::PushNumber(1.0),
            Instruction::BinaryOp(BinaryOp::Sub),
            Instruction::StoreLocal(1),
            Instruction::LoadLocal(0),
            Instruction::PushNumber(2.0),
            Instruction::BinaryOp(BinaryOp::Sub),
            Instruction::StoreLocal(2),
            Instruction::LoadLocal(1),
            Instruction::Call(1),
            Instruction::LoadLocal(2),
            Instruction::Call(1),
            Instruction::BinaryOp(BinaryOp::Add),
            Instruction::Return,
        ];

        let test_name = "fibonacci".to_string();
        let iterations = 10000;
        let start_time = Instant::now();

        for i in 0..iterations {
            let n = (i % 20) as f64;
            let args = vec![TsValue::Number(n)];
            let _ = self.jit_compiler.execute_function(&test_name, &args, &instructions);
        }

        let duration = start_time.elapsed();
        let total_time = duration.as_micros() as u64;
        let avg_time = total_time as f64 / iterations as f64;
        let stats = self.jit_compiler.get_statistics();
        let cache_hit_rate = stats.cache_hit_rate();

        self.results.push(PerformanceTestResult {
            test_name,
            iterations,
            total_time,
            avg_time,
            compilation_count: stats.total_compilations,
            cache_hit_rate,
        });
    }

    /// 测试循环性能
    pub fn test_loop(&mut self) {
        // 简单的循环指令
        let instructions = vec![
            Instruction::PushNumber(0.0),
            Instruction::StoreLocal(1),
            Instruction::LoadLocal(0),
            Instruction::StoreLocal(2),
            Instruction::Jump(5),
            Instruction::LoadLocal(1),
            Instruction::LoadLocal(2),
            Instruction::BinaryOp(BinaryOp::Add),
            Instruction::StoreLocal(1),
            Instruction::LoadLocal(2),
            Instruction::PushNumber(1.0),
            Instruction::BinaryOp(BinaryOp::Sub),
            Instruction::StoreLocal(2),
            Instruction::LoadLocal(2),
            Instruction::PushNumber(0.0),
            Instruction::BinaryOp(BinaryOp::Gt),
            Instruction::JumpIfFalse(10),
            Instruction::LoadLocal(1),
            Instruction::Return,
        ];

        let test_name = "loop".to_string();
        let iterations = 10000;
        let start_time = Instant::now();

        for i in 0..iterations {
            let n = (i % 1000) as f64;
            let args = vec![TsValue::Number(n)];
            let _ = self.jit_compiler.execute_function(&test_name, &args, &instructions);
        }

        let duration = start_time.elapsed();
        let total_time = duration.as_micros() as u64;
        let avg_time = total_time as f64 / iterations as f64;
        let stats = self.jit_compiler.get_statistics();
        let cache_hit_rate = stats.cache_hit_rate();

        self.results.push(PerformanceTestResult {
            test_name,
            iterations,
            total_time,
            avg_time,
            compilation_count: stats.total_compilations,
            cache_hit_rate,
        });
    }

    /// 测试函数调用性能
    pub fn test_function_call(&mut self) {
        // 简单的函数调用指令
        let instructions = vec![
            Instruction::LoadLocal(0),
            Instruction::LoadLocal(1),
            Instruction::BinaryOp(BinaryOp::Add),
            Instruction::Return,
        ];

        let test_name = "function_call".to_string();
        let iterations = 100000;
        let start_time = Instant::now();

        for i in 0..iterations {
            let a = (i % 1000) as f64;
            let b = ((i + 1) % 1000) as f64;
            let args = vec![TsValue::Number(a), TsValue::Number(b)];
            let _ = self.jit_compiler.execute_function(&test_name, &args, &instructions);
        }

        let duration = start_time.elapsed();
        let total_time = duration.as_micros() as u64;
        let avg_time = total_time as f64 / iterations as f64;
        let stats = self.jit_compiler.get_statistics();
        let cache_hit_rate = stats.cache_hit_rate();

        self.results.push(PerformanceTestResult {
            test_name,
            iterations,
            total_time,
            avg_time,
            compilation_count: stats.total_compilations,
            cache_hit_rate,
        });
    }

    /// 测试内存访问性能
    pub fn test_memory_access(&mut self) {
        // 简单的内存访问指令
        let instructions = vec![
            Instruction::CreateObject,
            Instruction::StoreLocal(1),
            Instruction::LoadLocal(1),
            Instruction::PushString("value".to_string()),
            Instruction::LoadLocal(0),
            Instruction::SetProperty,
            Instruction::LoadLocal(1),
            Instruction::PushString("value".to_string()),
            Instruction::GetProperty,
            Instruction::Return,
        ];

        let test_name = "memory_access".to_string();
        let iterations = 10000;
        let start_time = Instant::now();

        for i in 0..iterations {
            let value = (i % 1000) as f64;
            let args = vec![TsValue::Number(value)];
            let _ = self.jit_compiler.execute_function(&test_name, &args, &instructions);
        }

        let duration = start_time.elapsed();
        let total_time = duration.as_micros() as u64;
        let avg_time = total_time as f64 / iterations as f64;
        let stats = self.jit_compiler.get_statistics();
        let cache_hit_rate = stats.cache_hit_rate();

        self.results.push(PerformanceTestResult {
            test_name,
            iterations,
            total_time,
            avg_time,
            compilation_count: stats.total_compilations,
            cache_hit_rate,
        });
    }

    /// 测试分支预测性能
    pub fn test_branch_prediction(&mut self) {
        // 简单的分支预测指令
        let instructions = vec![
            Instruction::LoadLocal(0),
            Instruction::PushNumber(500.0),
            Instruction::BinaryOp(BinaryOp::Gt),
            Instruction::JumpIfFalse(4),
            Instruction::PushNumber(1.0),
            Instruction::Return,
            Instruction::PushNumber(0.0),
            Instruction::Return,
        ];

        let test_name = "branch_prediction".to_string();
        let iterations = 100000;
        let start_time = Instant::now();

        for i in 0..iterations {
            let value = (i % 1000) as f64;
            let args = vec![TsValue::Number(value)];
            let _ = self.jit_compiler.execute_function(&test_name, &args, &instructions);
        }

        let duration = start_time.elapsed();
        let total_time = duration.as_micros() as u64;
        let avg_time = total_time as f64 / iterations as f64;
        let stats = self.jit_compiler.get_statistics();
        let cache_hit_rate = stats.cache_hit_rate();

        self.results.push(PerformanceTestResult {
            test_name,
            iterations,
            total_time,
            avg_time,
            compilation_count: stats.total_compilations,
            cache_hit_rate,
        });
    }

    /// 打印测试结果
    pub fn print_results(&self) {
        println!("=== JIT 编译器性能测试结果 ===");
        println!(
            "{:<20} {:<10} {:<15} {:<15} {:<15} {:<15}",
            "测试名称", "迭代次数", "总时间(μs)", "平均时间(μs)", "编译次数", "缓存命中率"
        );
        println!("{:-<90}", "");

        for result in &self.results {
            println!(
                "{:<20} {:<10} {:<15} {:<15.2} {:<15} {:<15.2}%",
                result.test_name,
                result.iterations,
                result.total_time,
                result.avg_time,
                result.compilation_count,
                result.cache_hit_rate * 100.0
            );
        }

        println!("{:-<90}", "");
    }

    /// 获取测试结果
    pub fn get_results(&self) -> &Vec<PerformanceTestResult> {
        &self.results
    }
}
