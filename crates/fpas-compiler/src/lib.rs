#![cfg_attr(
    test,
    expect(
        clippy::expect_used,
        reason = "compiler tests use expect to keep bytecode assertions focused on behavior"
    )
)]
mod bytecode;
mod error;
mod intrinsic_catalog;
mod lowering;
mod object;

pub use error::CompileError;
pub use lowering::lower;
pub use object::{
    CompiledUnitObject, compile_program_object_with_support, compile_unit_object,
    compile_unit_object_with_support,
};

/// Compile one program directly to a verified executable.
///
/// Production commands use the object/linker path; this direct API remains available for
/// focused compiler tests and callers that do not need separately compiled units.
///
/// # Errors
///
/// Returns semantic diagnostics, a structured lowering diagnostic, or a bytecode
/// construction/verifier diagnostic represented as an internal compiler failure.
pub fn compile(program: &Program) -> Result<fpas_bytecode::VerifiedExecutable, Vec<CompileError>> {
    let ir = lower(program)?;
    bytecode::compile_program(&ir).map_err(|error| vec![error])
}

/// Compile one root program into a relocatable object.
///
/// The object keeps functions independently encoded, converts numeric table references to
/// object-local relocations, and is the production compiler input to the register linker.
///
/// # Errors
///
/// Returns semantic/lowering diagnostics or an internal object-construction diagnostic.
pub fn compile_object(
    program: &Program,
) -> Result<fpas_unit::object::RelocatableObject, Vec<CompileError>> {
    let executable = compile(program)?;
    fpas_unit::object::RelocatableObject::from_executable(&program.name, executable).map_err(
        |error| {
            vec![error::internal_compiler_error(
                format!("Object construction failed: {error}."),
                "This is an internal compiler error. Re-run compilation and report the source program.",
                program.span.line,
                program.span.column,
            )]
        },
    )
}

use fpas_parser::Program;

#[cfg(test)]
mod tests;
