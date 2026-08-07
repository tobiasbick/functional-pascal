//! AST and semantic-metadata lowering for the P4 register-development subset.

mod calls;
mod case;
mod closures;
mod context;
mod control_flow;
mod expr;
mod routines;
mod stmt;
mod types;

use fpas_ir::{FunctionId, Program};
use fpas_parser::Program as AstProgram;

use crate::CompileError;

use self::context::{FunctionInput, LoweringContext};

/// Lower a semantically valid scalar program with calls and closures to typed IR.
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
    if !program.uses.is_empty()
        || program.declarations.iter().any(|declaration| {
            !matches!(
                declaration,
                fpas_parser::Decl::Function(_) | fpas_parser::Decl::Procedure(_)
            )
        })
    {
        return Err(vec![context::unsupported(
            program.span,
            "program imports and non-routine declarations",
        )]);
    }
    let mut routines = Vec::new();
    routines::collect(&program.declarations, &mut routines);
    let mut type_table = types::TypeTable::from_metadata(&metadata).map_err(|error| vec![error])?;
    let callables = routines::callable_table(&routines, &mut type_table, &metadata)
        .map_err(|error| vec![error])?;
    let first_closure_id = u32::try_from(routines.len().saturating_add(1)).map_err(|_| {
        vec![context::unsupported(
            program.span,
            "function identifier overflow",
        )]
    })?;
    let mut closures = closures::ClosureRegistry::new(first_closure_id);
    closures
        .discover_statements(
            &program.body,
            FunctionId::new(0),
            &metadata,
            &mut type_table,
        )
        .map_err(|error| vec![error])?;
    for (index, routine) in routines.iter().enumerate() {
        let id = FunctionId::new(u32::try_from(index + 1).map_err(|_| {
            vec![context::unsupported(
                program.span,
                "function identifier overflow",
            )]
        })?);
        closures
            .discover_statements(routine.statements(), id, &metadata, &mut type_table)
            .map_err(|error| vec![error])?;
    }
    let mut context = LoweringContext::new(FunctionInput {
        name: &program.name,
        id: FunctionId::new(0),
        result: types::UNIT,
        parameters: &[],
        captures: &[],
        metadata: &metadata,
        callables: callables.clone(),
        closure_targets: closures.targets.clone(),
        cell_names: closures
            .cell_names
            .get(&FunctionId::new(0))
            .cloned()
            .unwrap_or_default(),
        type_table: type_table.clone(),
    })
    .map_err(|error| vec![error])?;
    for statement in &program.body {
        context
            .lower_statement(statement)
            .map_err(|error| vec![error])?;
    }
    let root = context.finish(program.span).map_err(|error| vec![error])?;
    let mut functions = vec![root];
    for (index, routine) in routines.iter().enumerate() {
        let id = FunctionId::new(u32::try_from(index + 1).map_err(|_| {
            vec![context::unsupported(
                program.span,
                "function identifier overflow",
            )]
        })?);
        functions.push(
            routines::lower(
                routine,
                id,
                &metadata,
                &callables,
                &mut type_table,
                closures.targets.clone(),
                closures.cell_names.get(&id).cloned().unwrap_or_default(),
            )
            .map_err(|error| vec![error])?,
        );
    }
    for routine in &closures.routines {
        functions.push(
            closures
                .lower(routine, &metadata, &callables, &mut type_table)
                .map_err(|error| vec![error])?,
        );
    }
    let ir = Program {
        types: type_table.definitions(),
        globals: Vec::new(),
        record_layouts: Vec::new(),
        enum_layouts: Vec::new(),
        intrinsics: Vec::new(),
        functions,
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
