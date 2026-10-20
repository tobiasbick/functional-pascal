//! Deterministic named-routine discovery and scalar signature lowering.

use std::collections::BTreeMap;

use fpas_ir::{Function, FunctionId, TypeId};
use fpas_parser::{
    Decl, FormalParam, FuncBody, FunctionDecl, ProcedureDecl, RecordMethod, TypeBody, TypeDef,
};
use fpas_sema::AnalysisMetadata;

use crate::CompileError;

use super::context::{Callable, FunctionInput, LoweringContext, ParameterInput, unsupported};
use super::types;

/// A named routine and its enclosing record type parameters, when applicable.
pub(super) enum Routine<'a> {
    Function(&'a FunctionDecl),
    Procedure(&'a ProcedureDecl),
    RecordFunction(&'a TypeDef, &'a FunctionDecl),
    RecordProcedure(&'a TypeDef, &'a ProcedureDecl),
}

/// Resolved bindings and captures required to lower a named routine.
pub(super) struct LoweringInput<'a> {
    pub id: FunctionId,
    pub runtime_name: &'a str,
    pub source_name: &'a str,
    pub metadata: &'a AnalysisMetadata,
    pub callables: &'a BTreeMap<String, Callable>,
    pub globals: &'a BTreeMap<String, super::context::GlobalBinding>,
    pub constants: &'a BTreeMap<String, fpas_ir::Constant>,
    pub closure_targets: std::collections::HashMap<usize, super::context::ClosureTarget>,
    pub bound_method_targets: std::collections::HashMap<usize, super::context::BoundMethodTarget>,
    pub intrinsic_targets: std::collections::HashMap<usize, super::context::BoundMethodTarget>,
    pub cell_names: std::collections::BTreeSet<String>,
}

impl<'a> Routine<'a> {
    fn name(&self) -> &str {
        match self {
            Self::Function(function) | Self::RecordFunction(_, function) => &function.name,
            Self::Procedure(procedure) | Self::RecordProcedure(_, procedure) => &procedure.name,
        }
    }

    fn params(&self) -> &[FormalParam] {
        match self {
            Self::Function(function) | Self::RecordFunction(_, function) => &function.params,
            Self::Procedure(procedure) | Self::RecordProcedure(_, procedure) => &procedure.params,
        }
    }

    pub(super) fn type_params(&self) -> Vec<fpas_parser::TypeParam> {
        let routine_parameters = match self {
            Self::Function(function) | Self::RecordFunction(_, function) => &function.type_params,
            Self::Procedure(procedure) | Self::RecordProcedure(_, procedure) => {
                &procedure.type_params
            }
        };
        let mut parameters = match self {
            Self::RecordFunction(owner, _) | Self::RecordProcedure(owner, _) => {
                owner.type_params.clone()
            }
            _ => Vec::new(),
        };
        parameters.extend_from_slice(routine_parameters);
        parameters
    }

    fn body(&self) -> &'a FuncBody {
        match self {
            Self::Function(function) | Self::RecordFunction(_, function) => &function.body,
            Self::Procedure(procedure) | Self::RecordProcedure(_, procedure) => &procedure.body,
        }
    }

    fn requires(&self) -> &[fpas_parser::RequiresClause] {
        match self {
            Self::Function(function) | Self::RecordFunction(_, function) => &function.requires,
            Self::Procedure(procedure) | Self::RecordProcedure(_, procedure) => &procedure.requires,
        }
    }

    fn ensures(&self) -> &[fpas_parser::EnsuresClause] {
        match self {
            Self::Function(function) | Self::RecordFunction(_, function) => &function.ensures,
            Self::Procedure(procedure) | Self::RecordProcedure(_, procedure) => &procedure.ensures,
        }
    }

    /// Executable statements used by closure and capture discovery.
    pub fn statements(&self) -> &'a [fpas_parser::Stmt] {
        let FuncBody::Block { stmts, .. } = self.body();
        stmts
    }

    fn result(&self, types: &mut types::TypeTable) -> Result<TypeId, CompileError> {
        match self {
            Self::Function(function) | Self::RecordFunction(_, function) => {
                types.type_expr_with_params(&function.return_type, &self.type_params())
            }
            Self::Procedure(_) | Self::RecordProcedure(_, _) => Ok(types::UNIT),
        }
    }

    pub(super) fn span(&self) -> fpas_lexer::Span {
        match self {
            Self::Function(function) | Self::RecordFunction(_, function) => function.span,
            Self::Procedure(procedure) | Self::RecordProcedure(_, procedure) => procedure.span,
        }
    }

    fn runtime_name(&self) -> String {
        match self {
            Self::RecordFunction(owner, function) => format!("{}.{}", owner.name, function.name),
            Self::RecordProcedure(owner, procedure) => format!("{}.{}", owner.name, procedure.name),
            _ => self.name().to_string(),
        }
    }

    fn capture_key(&self) -> usize {
        match self {
            Self::Function(function) | Self::RecordFunction(_, function) => {
                fpas_sema::function_decl_lookup_key(function)
            }
            Self::Procedure(procedure) | Self::RecordProcedure(_, procedure) => {
                fpas_sema::procedure_decl_lookup_key(procedure)
            }
        }
    }
}

/// Discover named routines in declaration order, retaining record ownership.
pub(super) fn collect<'a>(
    declarations: &'a [Decl],
    routines: &mut Vec<Routine<'a>>,
    owners: &mut Vec<FunctionId>,
    names: &mut Vec<String>,
    parent: FunctionId,
    parent_name: &str,
    id_offset: usize,
) {
    for declaration in declarations {
        match declaration {
            Decl::Function(function) => {
                let id = FunctionId::new(
                    u32::try_from(id_offset.saturating_add(routines.len()).saturating_add(1))
                        .unwrap_or(u32::MAX),
                );
                let name = nested_runtime_name(parent_name, &function.name);
                routines.push(Routine::Function(function));
                owners.push(parent);
                names.push(name.clone());
                let FuncBody::Block { nested, .. } = &function.body;
                collect(nested, routines, owners, names, id, &name, id_offset);
            }
            Decl::Procedure(procedure) => {
                let id = FunctionId::new(
                    u32::try_from(id_offset.saturating_add(routines.len()).saturating_add(1))
                        .unwrap_or(u32::MAX),
                );
                let name = nested_runtime_name(parent_name, &procedure.name);
                routines.push(Routine::Procedure(procedure));
                owners.push(parent);
                names.push(name.clone());
                let FuncBody::Block { nested, .. } = &procedure.body;
                collect(nested, routines, owners, names, id, &name, id_offset);
            }
            Decl::TypeDef(definition) => {
                if let TypeBody::Record(record) = &definition.body {
                    for method in &record.methods {
                        match method {
                            RecordMethod::Function(function)
                            | RecordMethod::StaticFunction(function) => {
                                let id = FunctionId::new(
                                    u32::try_from(
                                        id_offset.saturating_add(routines.len()).saturating_add(1),
                                    )
                                    .unwrap_or(u32::MAX),
                                );
                                let name = format!("{}.{}", definition.name, function.name);
                                routines.push(Routine::RecordFunction(definition, function));
                                owners.push(parent);
                                names.push(name.clone());
                                let FuncBody::Block { nested, .. } = &function.body;
                                collect(nested, routines, owners, names, id, &name, id_offset);
                            }
                            RecordMethod::Procedure(procedure)
                            | RecordMethod::StaticProcedure(procedure) => {
                                let id = FunctionId::new(
                                    u32::try_from(
                                        id_offset.saturating_add(routines.len()).saturating_add(1),
                                    )
                                    .unwrap_or(u32::MAX),
                                );
                                let name = format!("{}.{}", definition.name, procedure.name);
                                routines.push(Routine::RecordProcedure(definition, procedure));
                                owners.push(parent);
                                names.push(name.clone());
                                let FuncBody::Block { nested, .. } = &procedure.body;
                                collect(nested, routines, owners, names, id, &name, id_offset);
                            }
                        }
                    }
                }
            }
            Decl::Const(_) | Decl::Var(_) => {}
        }
    }
}

fn nested_runtime_name(parent_name: &str, name: &str) -> String {
    if parent_name.is_empty() {
        name.to_string()
    } else {
        format!("{parent_name}.{name}")
    }
}

/// Lower routine signatures and captures, seeded by the inherited owner and its captures.
pub(super) fn callable_table(
    routines: &[Routine<'_>],
    owners: &[FunctionId],
    names: &[String],
    types: &mut types::TypeTable,
    metadata: &AnalysisMetadata,
    first_id: u32,
    (inherited_owner, inherited_captures): (FunctionId, &[super::context::CaptureInput]),
) -> Result<BTreeMap<String, Callable>, CompileError> {
    let mut table = BTreeMap::new();
    let mut captures_by_function = BTreeMap::<FunctionId, Vec<super::context::CaptureInput>>::new();
    captures_by_function.insert(inherited_owner, inherited_captures.to_vec());
    for (index, routine) in routines.iter().enumerate() {
        let function = FunctionId::new(
            u32::try_from(index.saturating_add(first_id as usize))
                .map_err(|_| unsupported(routine.span(), "function identifier overflow"))?,
        );
        let parent = owners.get(index).copied().unwrap_or(FunctionId::new(0));
        let type_parameters = routine.type_params();
        let parameters = routine
            .params()
            .iter()
            .map(|parameter| types.formal_param_type(parameter, &type_parameters))
            .collect::<Result<Vec<_>, _>>()?;
        let result = routine.result(types)?;
        let value_type = types.function_type(parameters.clone(), result, routine.span())?;
        let parent_captures = captures_by_function
            .get(&parent)
            .cloned()
            .unwrap_or_default();
        let captures = metadata
            .nested_routine_captures
            .get(&routine.capture_key())
            .map(|info| {
                info.captures
                    .iter()
                    .map(|capture| {
                        let ty = types.intern(
                            &capture.ty,
                            routine.span().line,
                            routine.span().column,
                        )?;
                        let reuses_cell = parent_captures.iter().any(|outer| {
                            outer.name.eq_ignore_ascii_case(&capture.name)
                                && outer.declaration
                                    == Some(capture.declaration.diagnostic_span_or_synthetic())
                                && outer.kind != fpas_ir::CaptureKind::Value
                        });
                        super::context::CaptureInput::from_binding(
                            capture,
                            ty,
                            reuses_cell,
                            types,
                            routine.span(),
                        )
                    })
                    .collect::<Result<Vec<_>, CompileError>>()
            })
            .transpose()?
            .unwrap_or_default();
        captures_by_function.insert(function, captures.clone());
        let runtime_name = names
            .get(index)
            .cloned()
            .unwrap_or_else(|| routine.runtime_name());
        table.insert(
            runtime_name.to_ascii_lowercase(),
            Callable {
                declaration: Some((routine.span().source_id, routine.span().offset)),
                function,
                parameters,
                result,
                value_type,
                captures,
            },
        );
    }
    Ok(table)
}

/// Lower the body of one named routine with its resolved runtime bindings.
pub(super) fn lower(
    routine: &Routine<'_>,
    types: &mut types::TypeTable,
    input: LoweringInput<'_>,
) -> Result<(Function, types::TypeTable), CompileError> {
    let LoweringInput {
        id,
        runtime_name,
        source_name,
        metadata,
        callables,
        globals,
        constants,
        closure_targets,
        bound_method_targets,
        intrinsic_targets,
        cell_names,
    } = input;
    let runtime_name = runtime_name.to_string();
    let captures = callables
        .get(&runtime_name.to_ascii_lowercase())
        .map(|callable| callable.captures.clone())
        .unwrap_or_default();
    let type_parameters = routine.type_params();
    let parameters = routine
        .params()
        .iter()
        .map(|parameter| {
            types
                .formal_param_type(parameter, &type_parameters)
                .map(|ty| ParameterInput {
                    name: parameter.name.clone(),
                    type_name: types.declared_scalar_name(&parameter.type_expr),
                    ty,
                    declaration: Some(parameter.span.diagnostic_span_or_synthetic()),
                    reference: parameter.mode == fpas_parser::ParamMode::Var,
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut context = LoweringContext::new(FunctionInput {
        name: &runtime_name,
        source_name,
        id,
        result: routine.result(types)?,
        parameters: &parameters,
        captures: &captures,
        globals: globals.clone(),
        constants: constants.clone(),
        metadata,
        callables: callables.clone(),
        closure_targets,
        bound_method_targets,
        intrinsic_targets,
        cell_names,
        type_table: types.clone(),
    })?;
    let FuncBody::Block { stmts, .. } = routine.body();
    context.lower_preconditions(
        &format!("{source_name}.{runtime_name}"),
        routine.params(),
        routine.requires(),
    )?;
    context.prepare_postconditions(
        &format!("{source_name}.{runtime_name}"),
        routine.params(),
        routine.ensures(),
    )?;
    context.lower_statements(stmts)?;
    context.finish(routine.span())
}
