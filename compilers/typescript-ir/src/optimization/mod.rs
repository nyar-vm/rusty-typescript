mod constant_propagation;
mod dead_code_elimination;
mod expression_optimization;
mod loop_optimization;
mod statement_optimization;

use crate::{ControlFlowGraph, Expression, Module, Program, Statement};
pub use constant_propagation::ConstantPropagation;
pub use dead_code_elimination::DeadCodeElimination;
pub use expression_optimization::ExpressionOptimization;
pub use loop_optimization::LoopOptimization;
use typescript_types::TsValue;

/// 性能分析数据
#[derive(Debug, Clone)]
pub struct ProfileData {
    /// 函数调用次数
    pub function_calls: std::collections::HashMap<String, usize>,
    /// 循环执行次数
    pub loop_executions: std::collections::HashMap<usize, usize>,
    /// 热点表达式
    pub hot_expressions: std::collections::HashMap<String, usize>,
}

/// 优化器
pub struct Optimizer {
    /// 优化级别
    pub optimization_level: OptimizationLevel,
    /// 性能分析数据
    pub profile_data: Option<ProfileData>,
}

/// 优化级别
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum OptimizationLevel {
    /// 无优化
    None,
    /// 基本优化
    Basic,
    /// 中级优化
    Medium,
    /// 高级优化
    High,
}

impl Optimizer {
    /// 创建一个新的优化器
    pub fn new(optimization_level: OptimizationLevel) -> Self {
        Self { optimization_level, profile_data: None }
    }

    /// 创建带有性能分析数据的优化器
    pub fn new_with_profile(optimization_level: OptimizationLevel, profile_data: ProfileData) -> Self {
        Self { optimization_level, profile_data: Some(profile_data) }
    }

    /// 优化程序
    pub fn optimize_program(&self, program: &Program) -> Program {
        if self.optimization_level == OptimizationLevel::None {
            return program.clone();
        }

        let mut optimized_statements = Vec::new();
        let mut has_return = false;

        for stmt in &program.statements {
            if has_return {
                continue;
            }

            let optimized_stmt = self.optimize_statement(stmt);
            if !DeadCodeElimination::is_dead_code(&optimized_stmt) {
                optimized_statements.push(optimized_stmt.clone());
                if matches!(optimized_stmt, Statement::Return(_)) {
                    has_return = true;
                }
            }
        }

        if self.optimization_level == OptimizationLevel::High {
            self.optimize_with_cfg(&mut optimized_statements);
        }

        Program { statements: optimized_statements }
    }

    /// 优化模块
    pub fn optimize_module(&self, module: &Module) -> Module {
        if self.optimization_level == OptimizationLevel::None {
            return module.clone();
        }

        let mut optimized_statements = Vec::new();
        let mut has_return = false;

        for stmt in &module.statements {
            if has_return {
                continue;
            }

            let optimized_stmt = self.optimize_statement(stmt);
            if !DeadCodeElimination::is_dead_code(&optimized_stmt) {
                optimized_statements.push(optimized_stmt.clone());
                if matches!(optimized_stmt, Statement::Return(_)) {
                    has_return = true;
                }
            }
        }

        Module { name: module.name.clone(), statements: optimized_statements }
    }

    /// 使用控制流图进行优化
    fn optimize_with_cfg(&self, statements: &mut Vec<Statement>) {
        let mut cfg = ControlFlowGraph::from_statements(statements);

        cfg.reaching_definitions_analysis();

        cfg.live_variables_analysis();

        self.optimize_using_analysis(statements, &cfg);
    }

    /// 基于分析结果进行优化
    fn optimize_using_analysis(&self, statements: &mut Vec<Statement>, cfg: &ControlFlowGraph) {
        ConstantPropagation::propagate(statements, cfg);

        DeadCodeElimination::eliminate(statements, cfg);

        LoopOptimization::hoist_loop_invariants(statements);

        ExpressionOptimization::eliminate_common_subexpressions(statements);

        ExpressionOptimization::strength_reduction(statements);

        if self.profile_data.is_some() {
            self.optimize_based_on_profile(statements);
        }
    }

    /// 基于性能分析数据进行优化
    fn optimize_based_on_profile(&self, statements: &mut Vec<Statement>) {
        if let Some(_profile) = &self.profile_data {
            for stmt in statements {
                match stmt {
                    Statement::Expression(_expr) => {}
                    _ => {}
                }
            }
        }
    }

    /// 优化语句
    pub fn optimize_statement(&self, stmt: &Statement) -> Statement {
        statement_optimization::optimize_statement(stmt, self.optimization_level)
    }

    /// 优化表达式
    pub fn optimize_expression(&self, expr: &Expression) -> Expression {
        ExpressionOptimization::optimize(expr, self.optimization_level)
    }
}

/// 执行基本优化
pub fn optimize_basic(program: &Program) -> Program {
    let optimizer = Optimizer::new(OptimizationLevel::Basic);
    optimizer.optimize_program(program)
}

/// 执行中级优化
pub fn optimize_medium(program: &Program) -> Program {
    let optimizer = Optimizer::new(OptimizationLevel::Medium);
    optimizer.optimize_program(program)
}

/// 执行高级优化
pub fn optimize_high(program: &Program) -> Program {
    let optimizer = Optimizer::new(OptimizationLevel::High);
    optimizer.optimize_program(program)
}

/// 执行所有优化
pub fn optimize_full(program: &Program) -> Program {
    let optimizer = Optimizer::new(OptimizationLevel::High);
    optimizer.optimize_program(program)
}
