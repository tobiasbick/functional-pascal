//! AST and semantic-metadata lowering for the P3 scalar/control-flow subset.

mod case;
mod context;
mod control_flow;
mod expr;
mod stmt;
mod types;

use fpas_ir::{FunctionId, Program};
use fpas_parser::Program as AstProgram;

use crate::CompileError;

use self::context::LoweringContext;

/// Lower a semantically valid, functionless scalar/control-flow program to typed IR.
///
/// # Errors
///
/// Returns all semantic diagnostics, or one structured compiler diagnostic when the source uses a
/// construct assigned to a later register-VM phase.
pub fn lower_register_subset(program: &AstProgram) -> Result<Program, Vec<CompileError>> {
    let metadata = fpas_sema::analyze_with_types(program);
    if !metadata.errors.is_empty() {
        return Err(metadata.errors);
    }
    if !program.uses.is_empty() || !program.declarations.is_empty() {
        return Err(vec![context::unsupported(
            program.span,
            "program imports and declarations",
        )]);
    }

    let mut context = LoweringContext::new(&program.name, metadata);
    for statement in &program.body {
        context
            .lower_statement(statement)
            .map_err(|error| vec![error])?;
    }
    let function = context.finish(program.span).map_err(|error| vec![error])?;
    let ir = Program {
        types: types::scalar_type_table(),
        globals: Vec::new(),
        record_layouts: Vec::new(),
        enum_layouts: Vec::new(),
        intrinsics: Vec::new(),
        functions: vec![function],
        entry: FunctionId::new(0),
    };
    ir.validate().map_err(|error| {
        vec![crate::error::internal_compiler_error(
            format!("Register IR failed validation: {error}"),
            "This is an internal compiler error. Re-run compilation and report the source program.",
            program.span.line,
            program.span.column,
        )]
    })?;
    Ok(ir)
}
