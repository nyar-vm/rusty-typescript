pub mod generator;
pub mod instruction;
pub mod optimizer;

use typescript_ir::Program;
use typescript_types::TsError;

pub use generator::{
    generate_expression_instructions, generate_expression_with_locals, generate_statement_instructions,
    generate_statement_with_locals,
};
pub use instruction::{BinaryOp, Instruction, UnaryOp};
pub use optimizer::optimize_instructions;

/// 从 AST 生成 IR
pub fn ast_to_ir(program: Program) -> Program {
    program
}

/// 从 IR 生成 VM 指令
pub fn ir_to_vm_instructions(program: &Program) -> Result<Vec<Instruction>, TsError> {
    let mut instructions = vec![];

    for statement in &program.statements {
        generate_statement_instructions(statement, &mut instructions)?;
    }

    let optimized_instructions = optimize_instructions(instructions);

    println!("Generated instructions: {:?}", optimized_instructions);

    Ok(optimized_instructions)
}
