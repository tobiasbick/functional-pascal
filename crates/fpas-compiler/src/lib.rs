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
/// Returns the first error encountered (sema or codegen). Prefer [`compile_all`] when you need
/// every semantic error at once (for example CLI or IDE integration).
///
/// **Documentation:** `docs/pascal/10-projects.md` (from the repository root).
pub fn compile(program: &Program) -> Result<Chunk, CompileError> {
    match compile_all(program) {
        Ok(chunk) => Ok(chunk),
        Err(mut errors) => Err(errors.remove(0)),
    }
}

/// Like [`compile`], but returns **all** semantic-analysis errors when sema fails, or a single
/// element when codegen fails after successful sema.
///
/// **Documentation:** `docs/pascal/10-projects.md` (from the repository root).
pub fn compile_all(program: &Program) -> Result<Chunk, Vec<CompileError>> {
    let (sema_errors, expr_types, method_calls, record_defaults, scalar_case_bindings) =
        fpas_sema::analyze_with_types(program);
    if !sema_errors.is_empty() {
        return Err(sema_errors);
    }
    let mut compiler = Compiler::new(
        expr_types,
        method_calls,
        record_defaults,
        scalar_case_bindings,
    );
    match compiler.compile_program(program) {
        Ok(()) => Ok(compiler.finish()),
        Err(e) => Err(vec![e]),
    }
}

#[cfg(test)]
mod tests;
