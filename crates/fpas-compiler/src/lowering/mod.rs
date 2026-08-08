//! AST and semantic-metadata lowering for the P7 register-development subset.

mod aggregates;
mod calls;
mod case;
mod closures;
mod concurrency;
mod context;
mod control_flow;
mod expr;
mod members;
mod routines;
mod stmt;
mod types;

use std::collections::{BTreeMap, BTreeSet};

use fpas_ir::{
    Function, FunctionId, Global, GlobalId, IntrinsicId, IntrinsicSignature, Operation, Program,
};
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
    let mut routines = Vec::new();
    routines::collect(&program.declarations, &mut routines);
    let mut type_table = types::TypeTable::from_metadata(&metadata).map_err(|error| vec![error])?;
    let enum_constants = collect_enum_constants(program);
    let mut globals = Vec::new();
    let mut global_bindings = BTreeMap::new();
    for declaration in &program.declarations {
        let (name, type_expr, span, mutable) = match declaration {
            fpas_parser::Decl::Const(definition) => (
                &definition.name,
                &definition.type_expr,
                definition.span,
                false,
            ),
            fpas_parser::Decl::Var(definition) => (
                &definition.name,
                &definition.type_expr,
                definition.span,
                false,
            ),
            fpas_parser::Decl::MutableVar(definition) => (
                &definition.name,
                &definition.type_expr,
                definition.span,
                true,
            ),
            _ => continue,
        };
        let ty = type_table
            .type_expr(type_expr)
            .map_err(|error| vec![error])?;
        let id = GlobalId::try_from_index(globals.len())
            .map_err(|_| vec![context::unsupported(span, "global identifier overflow")])?;
        globals.push(Global {
            id,
            name: name.clone(),
            ty,
            mutable,
        });
        global_bindings.insert(name.to_ascii_lowercase(), context::GlobalBinding { id, ty });
    }
    let callables = routines::callable_table(&routines, &mut type_table, &metadata)
        .map_err(|error| vec![error])?;
    let first_closure_id = u32::try_from(routines.len().saturating_add(1)).map_err(|_| {
        vec![context::unsupported(
            program.span,
            "function identifier overflow",
        )]
    })?;
    let mut closures = closures::ClosureRegistry::new(first_closure_id, callables.clone());
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
        globals: global_bindings.clone(),
        enum_constants: enum_constants.clone(),
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
    for declaration in &program.declarations {
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
        context
            .emit_effect(
                Operation::StoreGlobal {
                    global: global_bindings[&name.to_ascii_lowercase()].id,
                    value,
                },
                span,
            )
            .map_err(|error| vec![error])?;
    }
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
                &mut type_table,
                routines::LoweringInput {
                    id,
                    metadata: &metadata,
                    callables: &callables,
                    globals: &global_bindings,
                    enum_constants: &enum_constants,
                    closure_targets: closures.targets.clone(),
                    bound_method_targets: closures.bound_targets.clone(),
                    cell_names: closures.cell_names.get(&id).cloned().unwrap_or_default(),
                },
            )
            .map_err(|error| vec![error])?,
        );
    }
    for routine in &closures.routines {
        functions.push(
            closures
                .lower(
                    routine,
                    &metadata,
                    &callables,
                    &mut type_table,
                    &global_bindings,
                    &enum_constants,
                )
                .map_err(|error| vec![error])?,
        );
    }
    for routine in &closures.bound_routines {
        functions.push(
            closures
                .lower_bound(
                    routine,
                    &metadata,
                    &mut type_table,
                    &global_bindings,
                    &enum_constants,
                )
                .map_err(|error| vec![error])?,
        );
    }
    functions.sort_by_key(|function| function.id);
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
        vec![crate::error::internal_compiler_error(
            format!("Register IR failed validation: {error}"),
            "This is an internal compiler error. Re-run compilation and report the source program.",
            program.span.line,
            program.span.column,
        )]
    })?;
    Ok(ir)
}

fn collect_enum_constants(program: &AstProgram) -> BTreeMap<String, i64> {
    let mut constants = BTreeMap::new();
    let mut ambiguous = BTreeSet::new();
    for declaration in &program.declarations {
        let fpas_parser::Decl::TypeDef(definition) = declaration else {
            continue;
        };
        let fpas_parser::TypeBody::Enum(enumeration) = &definition.body else {
            continue;
        };
        if enumeration
            .members
            .iter()
            .any(|member| !member.fields.is_empty())
        {
            continue;
        }
        let mut next = Some(0_i64);
        for member in &enumeration.members {
            let Some(value) = member.value.or(next) else {
                continue;
            };
            next = value.checked_add(1);
            let short = member.name.to_ascii_lowercase();
            constants.insert(
                format!("{}.{}", definition.name, member.name).to_ascii_lowercase(),
                value,
            );
            if ambiguous.contains(&short) {
                continue;
            }
            match constants.entry(short) {
                std::collections::btree_map::Entry::Occupied(entry) => {
                    ambiguous.insert(entry.remove_entry().0);
                }
                std::collections::btree_map::Entry::Vacant(entry) => {
                    entry.insert(value);
                }
            }
        }
    }
    constants
}

fn collect_intrinsic_signatures(
    functions: &[Function],
    _types: &types::TypeTable,
) -> Vec<IntrinsicSignature> {
    let mut arities = BTreeMap::<IntrinsicId, usize>::new();
    for function in functions {
        for instruction in function.blocks.iter().flat_map(|block| &block.instructions) {
            if let Operation::Intrinsic {
                intrinsic,
                arguments,
            } = &instruction.operation
            {
                arities.entry(*intrinsic).or_insert(arguments.len());
            }
        }
    }
    arities
        .into_iter()
        .map(|(id, arity)| {
            let wire = u16::try_from(id.get()).ok();
            let variadic = wire.and_then(fpas_bytecode::Intrinsic::from_u16)
                == Some(fpas_bytecode::Intrinsic::Str(
                    fpas_bytecode::StrIntrinsic::Format,
                ));
            let parameters = if variadic {
                vec![types::DYNAMIC, types::DYNAMIC]
            } else {
                vec![types::DYNAMIC; arity]
            };
            IntrinsicSignature {
                id,
                parameters,
                variadic,
                result: types::DYNAMIC,
            }
        })
        .collect()
}
