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

mod bytecode;
mod compiler;
mod error;
mod lowering;
mod unit_object;

pub use error::CompileError;
pub use lowering::lower_register_subset;
pub use unit_object::{
    CompiledUnitObject, compile_program_object, compile_program_object_with_support,
    compile_unit_object, compile_unit_object_with_support,
};

/// Compile the functionless scalar/control-flow subset through the inactive register pipeline.
///
/// This development API keeps the production CLI and stack VM unchanged until the complete
/// register runtime reaches its cutover phase.
///
/// # Errors
///
/// Returns semantic diagnostics, a structured subset-lowering diagnostic, or a register-bytecode
/// construction/verifier diagnostic represented as an internal compiler failure.
pub fn compile_register_subset(
    program: &Program,
) -> Result<fpas_bytecode::VerifiedExecutable, Vec<CompileError>> {
    let ir = lower_register_subset(program)?;
    bytecode::compile_program(&ir).map_err(|error| vec![error])
}

use compiler::Compiler;
use fpas_bytecode::{Chunk, ChunkError};
use fpas_parser::Program;

use error::internal_compiler_error;

/// Compile a parsed program into bytecode.
///
/// Returns the first error encountered (sema or codegen). Prefer [`compile_all`] when you need
/// every semantic error at once (for example CLI or IDE integration).
///
/// **Documentation:** `docs/pascal/program-structure/projects.md` (from the repository root).
pub fn compile(program: &Program) -> Result<Chunk, CompileError> {
    compile_all(program).map_err(|mut errors| errors.remove(0))
}

/// Like [`compile`], but returns **all** semantic-analysis errors when sema fails, or a single
/// element when codegen fails after successful sema.
///
/// **Documentation:** `docs/pascal/program-structure/projects.md` (from the repository root).
pub fn compile_all(program: &Program) -> Result<Chunk, Vec<CompileError>> {
    let metadata = fpas_sema::analyze_with_types(program);
    if !metadata.errors.is_empty() {
        return Err(metadata.errors);
    }
    let mut compiler = Compiler::new(metadata);
    match compiler.compile_program(program) {
        Ok(()) => validated_chunk(compiler).map_err(|error| vec![error]),
        Err(e) => Err(vec![e]),
    }
}

fn validated_chunk(compiler: Compiler) -> Result<Chunk, CompileError> {
    let chunk = compiler.finish();
    chunk.validate_invariants().map_err(|error| match error {
        ChunkError::CodeLocationLengthMismatch {
            code_len,
            locations_len,
        } => internal_compiler_error(
            format!(
                "Compiled chunk has {code_len} instructions but {locations_len} source locations."
            ),
            "This is an internal compiler error. Re-run compilation and report the source program.",
            1,
            1,
        ),
        other => internal_compiler_error(
            format!("Compiled chunk failed invariant check: {other}"),
            "This is an internal compiler error. Re-run compilation and report the source program.",
            1,
            1,
        ),
    })?;
    Ok(chunk)
}

#[cfg(test)]
mod tests;
