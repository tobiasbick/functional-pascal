#![cfg_attr(
    test,
    expect(
        clippy::expect_used,
        reason = "compiler tests use expect to keep bytecode assertions focused on behavior"
    )
)]
#![cfg_attr(
    test,
    expect(
        clippy::panic,
        reason = "compiler tests use explicit panic for structural mismatches"
    )
)]

mod compiler;
mod error;

pub use error::CompileError;

use compiler::Compiler;
use fpas_bytecode::Chunk;
use fpas_parser::Program;

/// Compile a parsed program into bytecode.
///
/// Returns the first error encountered (sema or codegen). All sema errors are
/// collected but only the first is surfaced through the current single-error
/// return type.
pub fn compile(program: &Program) -> Result<Chunk, CompileError> {
    let (sema_errors, expr_types, method_calls, record_defaults, scalar_case_bindings) =
        fpas_sema::analyze_with_types(program);
    if !sema_errors.is_empty() {
        let Some(first_error) = sema_errors.into_iter().next() else {
            unreachable!("non-empty semantic error list expected after emptiness check");
        };
        return Err(first_error);
    }
    let mut compiler = Compiler::new(
        expr_types,
        method_calls,
        record_defaults,
        scalar_case_bindings,
    );
    compiler.compile_program(program)?;
    Ok(compiler.finish())
}

#[cfg(test)]
mod tests;
