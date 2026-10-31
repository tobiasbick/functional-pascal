//! Whole-source lowering after semantic analysis.
//! See `docs/pascal/program-structure/units.md`.

use super::context::{FunctionInput, LoweringContext};
use super::intrinsic_signatures::collect_intrinsic_signatures;
use super::{
    LoweredUnit, closures, context, debug, globals, imports, record_defaults, routines, types,
    validation,
};
use crate::CompileError;
use fpas_ir::{FunctionId, Operation, Program};
use fpas_parser::{Decl, Stmt};
use std::collections::{BTreeMap, HashMap};

/// Source declarations and executable statements of a program or unit.
pub(super) struct SourceRoot<'a> {
    /// Source-level owner name.
    pub name: &'a str,
    /// Direct source imports.
    pub uses: &'a [fpas_parser::Import],
    /// Source-level declarations.
    pub declarations: &'a [Decl],
    /// Executable program body, empty for a unit.
    pub body: &'a [Stmt],
    /// Source location for root diagnostics.
    pub span: fpas_lexer::Span,
}

/// Lowers checked source with its direct and supporting interfaces.
pub(super) fn lower_analyzed_root(
    source: SourceRoot<'_>,
    metadata: fpas_sema::AnalysisMetadata,
    interfaces: &[fpas_unit::interface::UnitInterface],
    supporting_interfaces: &[fpas_unit::interface::UnitInterface],
    options: crate::CompileOptions,
) -> Result<LoweredUnit, Vec<CompileError>> {
    let SourceRoot {
        name,
        uses,
        declarations,
        body,
        span,
    } = source;
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
        0,
    );
    let mut type_table =
        types::TypeTable::from_metadata(&metadata, name).map_err(|error| vec![error])?;
    super::errors::prepare(&mut type_table, declarations, supporting_interfaces)
        .map_err(|error| vec![error])?;
    let mut constants = BTreeMap::new();
    let (mut globals, mut global_bindings) =
        globals::collect(declarations, &mut type_table).map_err(|error| vec![error])?;
    let mut callables = routines::callable_table(
        &routines,
        &routine_owners,
        &runtime_names,
        &mut type_table,
        &metadata,
        1,
        (FunctionId::new(0), &[]),
    )
    .map_err(|error| vec![error])?;
    let first_import_id = u32::try_from(routines.len().saturating_add(1))
        .map_err(|_| vec![context::unsupported(span, "function identifier overflow")])?;
    let (imports, imported_stubs) = imports::install(
        imports::InterfaceSet {
            aliases: &metadata.import_aliases,
            uses,
            direct: interfaces,
            supporting: supporting_interfaces,
            required_releases: &metadata.resource_releases,
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
    callables = closures.callables.clone();
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
        intrinsic_targets: closures.intrinsic_targets.clone(),
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
    context
        .lower_statements(body)
        .map_err(|error| vec![error])?;
    let (mut root, updated_types) = context.finish(span).map_err(|error| vec![error])?;
    // Preserve the source scope during lowering, then isolate the generated root symbol.
    root.name = format!("$entry_{}", name.to_ascii_lowercase());
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
                contracts: options.contracts,
                id,
                runtime_name: runtime_names.get(index).map(String::as_str).unwrap_or(""),
                source_name: name,
                metadata: &metadata,
                callables: &callables,
                globals: &global_bindings,
                constants: &constants,
                closure_targets: closures.targets.clone(),
                bound_method_targets: closures.bound_targets.clone(),
                intrinsic_targets: closures.intrinsic_targets.clone(),
                cell_names: closures.cell_names.get(&id).cloned().unwrap_or_default(),
            },
        )
        .map_err(|error| vec![error])?;
        type_table = updated_types;
        functions.push(function);
    }
    functions.extend(imported_stubs);
    for routine in &closures.named_routines {
        let (function, updated_types) = routines::lower(
            &routine.routine,
            &mut type_table,
            routines::LoweringInput {
                contracts: options.contracts,
                id: routine.id,
                runtime_name: &routine.name,
                source_name: name,
                metadata: &metadata,
                callables: &callables,
                globals: &global_bindings,
                constants: &constants,
                closure_targets: closures.targets.clone(),
                bound_method_targets: closures.bound_targets.clone(),
                intrinsic_targets: closures.intrinsic_targets.clone(),
                cell_names: closures
                    .cell_names
                    .get(&routine.id)
                    .cloned()
                    .unwrap_or_default(),
            },
        )
        .map_err(|error| vec![error])?;
        type_table = updated_types;
        functions.push(function);
    }
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
    for routine in &closures.intrinsic_routines {
        let (function, updated_types) = closures
            .lower_intrinsic_routine(
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
    for routine in &closures.constructor_routines {
        let (function, updated_types) = closures
            .lower_constructor(
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
    let defaults = type_table.prepare_record_defaults(&metadata, name, declarations);
    let mut next_default_id = functions
        .iter()
        .map(|function| function.id.get())
        .max()
        .unwrap_or(0);
    for default in defaults {
        next_default_id = next_default_id.checked_add(1).ok_or_else(|| {
            vec![context::unsupported(
                span,
                "record default function identifier overflow",
            )]
        })?;
        let (function, updated_types) = record_defaults::lower(
            FunctionInput {
                name: &default.name,
                source_name: name,
                id: FunctionId::new(next_default_id),
                result: default.ty,
                parameters: &[],
                captures: &[],
                globals: global_bindings.clone(),
                constants: constants.clone(),
                metadata: &metadata,
                callables: callables.clone(),
                closure_targets: closures.targets.clone(),
                bound_method_targets: closures.bound_targets.clone(),
                intrinsic_targets: closures.intrinsic_targets.clone(),
                cell_names: Default::default(),
                type_table: type_table.clone(),
            },
            &default.expression,
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
    for routine in &closures.named_routines {
        owner_map.insert(routine.id, routine.owner);
    }
    for routine in &closures.bound_routines {
        owner_map.insert(routine.id, routine.owner);
    }
    for routine in &closures.intrinsic_routines {
        owner_map.insert(routine.id, routine.owner);
    }
    for routine in &closures.constructor_routines {
        owner_map.insert(routine.id, routine.owner);
    }
    debug::attach(&mut functions, &owner_map).map_err(|error| vec![error])?;
    super::errors::lower(
        &mut functions,
        &mut type_table,
        &mut globals,
        super::errors::LoweringInput {
            routines: &routines,
            metadata: &metadata,
            interfaces: supporting_interfaces,
            span,
            owners: &owner_map,
        },
    )
    .map_err(|error| vec![error])?;
    functions.sort_by_key(|function| function.id);
    let distinct_types = type_table
        .distinct_type_names(&metadata, declarations)
        .map_err(|error| vec![error])?;
    let ir = Program {
        types: type_table.definitions(),
        globals,
        record_layouts: type_table.record_layouts(),
        enum_layouts: type_table.enum_layouts(),
        intrinsics: collect_intrinsic_signatures(&functions, &type_table),
        distinct_types,
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
        errors: type_table.errors.clone(),
        program: ir,
        imports,
    })
}
