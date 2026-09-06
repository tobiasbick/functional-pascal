//! AST and semantic-metadata lowering to typed IR.

mod aggregates;
mod builtin_constants;
mod calls;
mod case;
mod closures;
mod concurrency;
mod context;
mod control_flow;
mod debug;
mod expr;
mod globals;
mod imports;
mod intrinsic_signatures;
mod members;
mod routines;
mod stmt;
mod type_names;
mod types;
mod validation;

pub(crate) use imports::ImportPlan;

use std::collections::{BTreeMap, HashMap};

use fpas_ir::{FunctionId, Operation, Program};
use fpas_parser::{Decl, Program as AstProgram, Stmt, Unit};

use crate::CompileError;

use self::context::{FunctionInput, LoweringContext};
use self::intrinsic_signatures::collect_intrinsic_signatures;

pub(crate) struct LoweredUnit {
    pub program: Program,
    pub imports: ImportPlan,
}

/// Lower a semantically valid scalar program with calls and closures to typed IR.
///
/// # Errors
///
/// Returns all semantic diagnostics, or one structured compiler diagnostic when the source uses a
/// construct that cannot be lowered.
pub fn lower(program: &AstProgram) -> Result<Program, Vec<CompileError>> {
    let metadata = fpas_sema::analyze_with_types(program);
    if !metadata.errors.is_empty() {
        return Err(metadata.errors);
    }
    lower_analyzed_root(
        &program.name,
        &program.declarations,
        &program.body,
        program.span,
        metadata,
        &[],
        &[],
    )
    .map(|lowered| lowered.program)
}

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
        return Err(analysis.metadata.errors);
    }
    lower_analyzed_root(
        &unit.name.parts.join("."),
        &unit.declarations,
        &[],
        unit.span,
        analysis.metadata,
        interfaces,
        supporting_interfaces,
    )
}

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
        return Err(metadata.errors);
    }
    lower_analyzed_root(
        &program.name,
        &program.declarations,
        &program.body,
        program.span,
        metadata,
        interfaces,
        supporting_interfaces,
    )
}

fn lower_analyzed_root(
    name: &str,
    declarations: &[Decl],
    body: &[Stmt],
    span: fpas_lexer::Span,
    metadata: fpas_sema::AnalysisMetadata,
    interfaces: &[fpas_unit::interface::UnitInterface],
    supporting_interfaces: &[fpas_unit::interface::UnitInterface],
) -> Result<LoweredUnit, Vec<CompileError>> {
    let mut routines = Vec::new();
    let mut routine_owners = Vec::new();
    let mut runtime_names = Vec::new();
    routines::collect(
        declarations,
        &mut routines,
        &mut routine_owners,
        &mut runtime_names,
        FunctionId::new(0),
        "",
    );
    let mut type_table = types::TypeTable::from_metadata(&metadata).map_err(|error| vec![error])?;
    let mut constants = BTreeMap::new();
    let (mut globals, mut global_bindings) =
        globals::collect(declarations, &metadata, &mut type_table).map_err(|error| vec![error])?;
    let mut callables = routines::callable_table(
        &routines,
        &routine_owners,
        &runtime_names,
        &mut type_table,
        &metadata,
    )
    .map_err(|error| vec![error])?;
    let first_import_id = u32::try_from(routines.len().saturating_add(1))
        .map_err(|_| vec![context::unsupported(span, "function identifier overflow")])?;
    let (imports, imported_stubs) = imports::install(
        imports::InterfaceSet {
            direct: interfaces,
            supporting: supporting_interfaces,
        },
        imports::BindingTables {
            types: &mut type_table,
            callables: &mut callables,
            globals: &mut globals,
            global_bindings: &mut global_bindings,
            constants: &mut constants,
        },
        first_import_id,
        span,
    )
    .map_err(|error| vec![error])?;
    let first_closure_id = first_import_id
        .checked_add(
            u32::try_from(imported_stubs.len())
                .map_err(|_| vec![context::unsupported(span, "function identifier overflow")])?,
        )
        .ok_or_else(|| vec![context::unsupported(span, "function identifier overflow")])?;
    let mut closures = closures::ClosureRegistry::new(first_closure_id, callables.clone(), name);
    closures.seed_named_nested_cells(&routine_owners, &runtime_names, &callables);
    closures
        .discover_declaration_initializers(
            declarations,
            FunctionId::new(0),
            &metadata,
            &mut type_table,
        )
        .map_err(|error| vec![error])?;
    closures
        .discover_statements(body, FunctionId::new(0), &metadata, &mut type_table)
        .map_err(|error| vec![error])?;
    for (index, routine) in routines.iter().enumerate() {
        let id = FunctionId::new(
            u32::try_from(index + 1)
                .map_err(|_| vec![context::unsupported(span, "function identifier overflow")])?,
        );
        closures
            .discover_statements(routine.statements(), id, &metadata, &mut type_table)
            .map_err(|error| vec![error])?;
    }
    let mut context = LoweringContext::new(FunctionInput {
        name,
        source_name: name,
        id: FunctionId::new(0),
        result: types::UNIT,
        parameters: &[],
        captures: &[],
        globals: global_bindings.clone(),
        constants: constants.clone(),
        metadata: &metadata,
        callables: callables.clone(),
        closure_targets: closures.targets.clone(),
        bound_method_targets: closures.bound_targets.clone(),
        cell_names: closures
            .cell_names
            .get(&FunctionId::new(0))
            .cloned()
            .unwrap_or_default(),
        type_table: type_table.clone(),
    })
    .map_err(|error| vec![error])?;
    let mut global_initializers = Vec::new();
    for declaration in declarations {
        let (name, value, span) = match declaration {
            fpas_parser::Decl::Const(definition) => {
                (&definition.name, &definition.value, definition.span)
            }
            fpas_parser::Decl::Var(definition) => {
                (&definition.name, &definition.value, definition.span)
            }
            fpas_parser::Decl::MutableVar(definition) => {
                (&definition.name, &definition.value, definition.span)
            }
            _ => continue,
        };
        let value = context
            .lower_expression(value)
            .map_err(|error| vec![error])?;
        let global = global_bindings[&name.to_ascii_lowercase()].id;
        let location = context
            .emit_effect_with_location(Operation::StoreGlobal { global, value }, span)
            .map_err(|error| vec![error])?;
        global_initializers.push((global, location));
    }
    for statement in body {
        context
            .lower_statement(statement)
            .map_err(|error| vec![error])?;
    }
    let (root, updated_types) = context.finish(span).map_err(|error| vec![error])?;
    for (global, location) in global_initializers {
        let declaration = globals
            .get_mut(global.get() as usize)
            .ok_or_else(|| vec![context::unsupported(span, "global initializer identifier")])?;
        declaration.initializer = Some(fpas_ir::GlobalInitializer {
            function: FunctionId::new(0),
            location,
        });
    }
    type_table = updated_types;
    let mut functions = vec![root];
    for (index, routine) in routines.iter().enumerate() {
        let id = FunctionId::new(
            u32::try_from(index + 1)
                .map_err(|_| vec![context::unsupported(span, "function identifier overflow")])?,
        );
        let (function, updated_types) = routines::lower(
            routine,
            &mut type_table,
            routines::LoweringInput {
                id,
                runtime_name: runtime_names.get(index).map(String::as_str).unwrap_or(""),
                source_name: name,
                metadata: &metadata,
                callables: &callables,
                globals: &global_bindings,
                constants: &constants,
                closure_targets: closures.targets.clone(),
                bound_method_targets: closures.bound_targets.clone(),
                cell_names: closures.cell_names.get(&id).cloned().unwrap_or_default(),
            },
        )
        .map_err(|error| vec![error])?;
        type_table = updated_types;
        functions.push(function);
    }
    functions.extend(imported_stubs);
    for routine in &closures.routines {
        let (function, updated_types) = closures
            .lower(
                routine,
                &metadata,
                &callables,
                &mut type_table,
                &global_bindings,
                &constants,
            )
            .map_err(|error| vec![error])?;
        type_table = updated_types;
        functions.push(function);
    }
    for routine in &closures.bound_routines {
        let (function, updated_types) = closures
            .lower_bound(
                routine,
                &metadata,
                &mut type_table,
                &global_bindings,
                &constants,
            )
            .map_err(|error| vec![error])?;
        type_table = updated_types;
        functions.push(function);
    }
    functions.sort_by_key(|function| function.id);
    let mut owner_map = HashMap::new();
    for (index, owner) in routine_owners.iter().copied().enumerate() {
        let id = FunctionId::new(
            u32::try_from(index.saturating_add(1))
                .map_err(|_| vec![context::unsupported(span, "function identifier overflow")])?,
        );
        owner_map.insert(id, owner);
    }
    for routine in &closures.routines {
        owner_map.insert(routine.id, routine.owner);
    }
    for routine in &closures.bound_routines {
        owner_map.insert(routine.id, routine.owner);
    }
    debug::attach(&mut functions, &owner_map).map_err(|error| vec![error])?;
    let ir = Program {
        types: type_table.definitions(),
        globals,
        record_layouts: type_table.record_layouts(),
        enum_layouts: type_table.enum_layouts(),
        intrinsics: collect_intrinsic_signatures(&functions, &type_table),
        functions,
        entry: FunctionId::new(0),
    };
    ir.validate().map_err(|error| {
        let context = validation::context(&ir, &error);
        let source = validation::source(&ir, &error).unwrap_or(span);
        vec![crate::error::internal_compiler_error(
            format!("Register IR failed validation: {error}{context}"),
            "This is an internal compiler error. Re-run compilation and report the source program.",
            source.line,
            source.column,
        )]
    })?;
    Ok(LoweredUnit {
        program: ir,
        imports,
    })
}
