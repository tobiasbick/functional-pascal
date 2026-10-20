//! AST and semantic-metadata lowering to typed IR.
//!
//! **Documentation:** `docs/pascal/getting-started/first-program.md`

mod aggregates;
mod builtin_constants;
mod calls;
mod case;
mod closures;
mod concurrency;
mod context;
mod contracts;
mod control_flow;
mod debug;
mod errors;
mod expr;
mod globals;
mod imports;
mod intrinsic_signatures;
mod members;
mod record_defaults;
mod root;
mod routines;
mod stmt;
mod task_scopes;
mod type_names;
mod types;
mod validation;

pub(crate) use imports::ImportPlan;

use fpas_ir::Program;
use fpas_parser::{Program as AstProgram, Unit};

use crate::CompileError;

use self::root::{SourceRoot, lower_analyzed_root};

/// Typed IR and import bindings produced from one source root.
pub(crate) struct LoweredUnit {
    /// Essential native-error layouts, independent of optional debugger types.
    pub errors: crate::errors::ErrorMetadata,
    /// Verified lowered program.
    pub program: Program,
    /// Import bindings to resolve during linking.
    pub imports: ImportPlan,
}

/// Lower a semantically valid program to typed IR.
///
/// # Errors
///
/// Returns all semantic diagnostics, or one structured compiler diagnostic when the source uses a
/// construct that cannot be lowered.
pub fn lower(program: &AstProgram) -> Result<Program, Vec<CompileError>> {
    lower_with_metadata(program).map(|lowered| lowered.program)
}

pub(crate) fn lower_with_metadata(program: &AstProgram) -> Result<LoweredUnit, Vec<CompileError>> {
    let metadata = fpas_sema::analyze_with_types(program);
    if !metadata.errors.is_empty() {
        return Err(metadata.errors.into_iter().map(Box::new).collect());
    }
    lower_analyzed_root(
        SourceRoot {
            name: &program.name,
            uses: &program.uses,
            declarations: &program.declarations,
            body: &program.body,
            span: program.span,
        },
        metadata,
        &[],
        &[],
    )
}

/// Lowers a unit with its direct imports and transitive type support.
/// See `docs/pascal/program-structure/units.md`.
pub(crate) fn lower_unit(
    unit: &Unit,
    interfaces: &[fpas_unit::interface::UnitInterface],
    supporting_interfaces: &[fpas_unit::interface::UnitInterface],
) -> Result<LoweredUnit, Vec<CompileError>> {
    let analysis =
        fpas_sema::analyze_unit_with_interface_support(unit, interfaces, supporting_interfaces)
            .map_err(|error| {
                vec![crate::error::internal_compiler_error(
                    error.to_string(),
                    "Rebuild the dependency sidecar; its semantic interface is invalid.",
                    unit.span.line,
                    unit.span.column,
                )]
            })?;
    if !analysis.metadata.errors.is_empty() {
        return Err(analysis.metadata.errors.into_iter().map(Box::new).collect());
    }
    lower_analyzed_root(
        SourceRoot {
            name: &unit.name.parts.join("."),
            uses: &unit.uses,
            declarations: &unit.declarations,
            body: &[],
            span: unit.span,
        },
        analysis.metadata,
        interfaces,
        supporting_interfaces,
    )
}

/// Lowers a program with its source imports and available dependency interfaces.
/// See `docs/pascal/program-structure/units.md`.
pub(crate) fn lower_program_with_support(
    program: &AstProgram,
    interfaces: &[fpas_unit::interface::UnitInterface],
    supporting_interfaces: &[fpas_unit::interface::UnitInterface],
) -> Result<LoweredUnit, Vec<CompileError>> {
    let metadata = fpas_sema::analyze_program_with_interface_support(
        program,
        interfaces,
        supporting_interfaces,
    )
    .map_err(|error| {
        vec![crate::error::internal_compiler_error(
            error.to_string(),
            "Rebuild the dependency sidecar; its semantic interface is invalid.",
            program.span.line,
            program.span.column,
        )]
    })?;
    if !metadata.errors.is_empty() {
        return Err(metadata.errors.into_iter().map(Box::new).collect());
    }
    lower_analyzed_root(
        SourceRoot {
            name: &program.name,
            uses: &program.uses,
            declarations: &program.declarations,
            body: &program.body,
            span: program.span,
        },
        metadata,
        interfaces,
        supporting_interfaces,
    )
}
